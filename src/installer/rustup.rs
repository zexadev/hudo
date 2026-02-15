use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
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
        let (url, filename) = self.resolve_download(config);

        std::fs::create_dir_all(&rustup_home).ok();
        std::fs::create_dir_all(&cargo_home).ok();

        // 下载 rustup-init.exe
        let exe_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 通过环境变量指定安装路径，静默安装默认 stable 工具链
        crate::ui::print_action("安装 Rust（通过 rustup）...");
        let status = std::process::Command::new(&exe_path)
            .args(["-y", "--no-modify-path", "--default-toolchain", "stable"])
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

fn get_rustc_version(cargo_home: &PathBuf) -> Option<String> {
    let rustc = cargo_home.join("bin").join("rustc.exe");
    std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
