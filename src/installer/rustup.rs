use anyhow::{Context, Result};
use async_trait::async_trait;
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use super::mingw::MingwInstaller;
use crate::config::HudoConfig;
use crate::download;

pub struct RustupInstaller;

#[async_trait]
impl Installer for RustupInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "rust",
            name: "Rust",
            description: "Rust 编程语言 (via rustup)",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let rustup_home = ctx.config.tools_dir().join("rustup");
        let cargo_home = ctx.config.lang_dir().join("cargo");
        let rustc = cargo_home.join("bin").join("rustc.exe");
        if rustc.exists() && rustup_home.exists() {
            if let Ok(out) = std::process::Command::new(&rustc).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH
        if let Ok(out) = std::process::Command::new("rustc").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        (
            "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
                .to_string(),
            "rustup-init.exe".to_string(),
        )
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let rustup_home = config.tools_dir().join("rustup");
        let cargo_home = config.lang_dir().join("cargo");

        // 检测 gcc（MinGW-w64）是否可用
        ensure_gcc(ctx).await?;

        let (url, filename) = self.resolve_download(config);

        std::fs::create_dir_all(&rustup_home).ok();
        std::fs::create_dir_all(&cargo_home).ok();

        // 下载 rustup-init.exe
        let exe_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 使用 GNU 工具链（依赖 MinGW-w64 的 gcc，无需 MSVC）
        crate::ui::print_action("安装 Rust (GNU 工具链)...");
        let status = std::process::Command::new(&exe_path)
            .args([
                "-y",
                "--no-modify-path",
                "--default-host",
                "x86_64-pc-windows-gnu",
                "--default-toolchain",
                "stable",
            ])
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .status()
            .context("启动 rustup-init 失败")?;

        if !status.success() {
            anyhow::bail!(
                "rustup-init 失败，退出码: {}",
                status.code().unwrap_or(-1)
            );
        }

        let version = get_rustc_version(&cargo_home).unwrap_or_else(|| "stable".to_string());

        Ok(InstallResult {
            install_path: cargo_home,
            version,
        })
    }

    fn env_actions(&self, _install_path: &PathBuf, config: &HudoConfig) -> Vec<EnvAction> {
        let rustup_home = config.tools_dir().join("rustup");
        let cargo_home = config.lang_dir().join("cargo");
        vec![
            EnvAction::Set {
                name: "RUSTUP_HOME".to_string(),
                value: rustup_home.to_string_lossy().to_string(),
            },
            EnvAction::Set {
                name: "CARGO_HOME".to_string(),
                value: cargo_home.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: cargo_home.join("bin").to_string_lossy().to_string(),
            },
        ]
    }
}

/// 检测 gcc 是否可用；若不可用则提示用户选择安装 MinGW-w64 或取消
async fn ensure_gcc(ctx: &InstallContext<'_>) -> Result<()> {
    if detect_gcc(ctx.config) {
        return Ok(());
    }

    crate::ui::print_warning("未检测到 gcc（MinGW-w64），Rust GNU 工具链需要它作为链接器");

    let install_now = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("  是否现在安装 MinGW-w64 (gcc)？")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !install_now {
        anyhow::bail!("请先安装 MinGW-w64：hudo install c");
    }

    // 安装 MinGW-w64
    MingwInstaller.install(ctx).await?;

    // 将 mingw64/bin 加入当前进程 PATH，让 rustup-init 能找到 gcc
    let mingw_bin = ctx.config.tools_dir().join("mingw64").join("bin");
    if let Ok(old_path) = std::env::var("PATH") {
        std::env::set_var(
            "PATH",
            format!("{};{}", mingw_bin.display(), old_path),
        );
    }

    // 设置环境变量（持久化到注册表）
    let mingw_inst = MingwInstaller;
    let actions = mingw_inst.env_actions(&mingw_bin.parent().unwrap().to_path_buf(), ctx.config);
    for action in actions {
        match action {
            super::EnvAction::AppendPath { path } => {
                crate::env::EnvManager::append_to_path(&path)?;
            }
            super::EnvAction::Set { name, value } => {
                crate::env::EnvManager::set_var(&name, &value)?;
            }
        }
    }
    crate::env::EnvManager::broadcast_change();

    Ok(())
}

/// 检测 gcc 是否可用（hudo MinGW 路径优先，然后系统 PATH）
fn detect_gcc(config: &HudoConfig) -> bool {
    // hudo 管理的 MinGW
    let gcc_hudo = config.tools_dir().join("mingw64").join("bin").join("gcc.exe");
    if gcc_hudo.exists() {
        return true;
    }

    // 系统 PATH 中的 gcc
    std::process::Command::new("gcc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_rustc_version(cargo_home: &PathBuf) -> Option<String> {
    let rustc = cargo_home.join("bin").join("rustc.exe");
    std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
