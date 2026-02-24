use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;
use crate::ui;

pub struct GhInstaller;

const GH_VERSION_DEFAULT: &str = "2.87.3";

#[async_trait]
impl Installer for GhInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "gh",
            name: "GitHub CLI",
            description: "GitHub 官方命令行工具",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // gh zip 解压后 gh.exe 可能在根目录或 bin/ 下，两处都检查
        let root = ctx.config.tools_dir().join("gh");
        let candidates = [
            root.join("bin").join("gh.exe"),
            root.join("gh.exe"),
        ];
        for gh_exe in &candidates {
            if gh_exe.exists() {
                if let Ok(out) = std::process::Command::new(gh_exe).arg("--version").output() {
                    if out.status.success() {
                        let version =
                            parse_gh_version(&String::from_utf8_lossy(&out.stdout));
                        return Ok(DetectResult::InstalledByHudo(version));
                    }
                }
            }
        }

        // 系统 PATH
        if let Ok(out) = std::process::Command::new("gh").arg("--version").output() {
            if out.status.success() {
                let version = parse_gh_version(&String::from_utf8_lossy(&out.stdout));
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.gh.as_deref().unwrap_or(GH_VERSION_DEFAULT);
        let filename = format!("gh_{}_windows_amd64.zip", version);
        let url = format!(
            "https://github.com/cli/cli/releases/download/v{}/{}",
            version, filename
        );
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("gh");

        let version = match &config.versions.gh {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 GitHub CLI 最新版本...");
                crate::version::gh_latest()
                    .await
                    .unwrap_or_else(|| GH_VERSION_DEFAULT.to_string())
            }
        };

        let filename = format!("gh_{}_windows_amd64.zip", version);
        let url = format!(
            "https://github.com/cli/cli/releases/download/v{}/{}",
            version, filename
        );

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 GitHub CLI...");
        let tmp_dir = config.cache_dir().join("gh-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有形如 gh_{version}_windows_amd64/ 的子目录
        let inner = download::find_single_subdir(&tmp_dir).unwrap_or(tmp_dir.clone());
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        std::fs::rename(&inner, &install_dir).ok();
        std::fs::remove_dir_all(&tmp_dir).ok();

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        // gh zip 解压后 gh.exe 可能在 bin/ 下或根目录，动态判断
        let bin_dir = install_path.join("bin");
        let path = if bin_dir.join("gh.exe").exists() {
            bin_dir.to_string_lossy().to_string()
        } else {
            install_path.to_string_lossy().to_string()
        };
        vec![EnvAction::AppendPath { path }]
    }

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let gh = find_gh(ctx.config);

        ui::print_title("配置 GitHub CLI");

        // 检查是否已登录
        if let Ok(out) = std::process::Command::new(&gh).arg("auth").arg("status").output() {
            if out.status.success() {
                let info = String::from_utf8_lossy(&out.stderr);
                for line in info.lines() {
                    ui::print_info(line.trim());
                }
                ui::print_success("GitHub CLI 已登录");
                return Ok(());
            }
        }

        // 未登录，提示并运行 gh auth login
        ui::print_info("尚未登录 GitHub，即将打开浏览器进行授权...");
        ui::print_info("如需跳过，按 Ctrl+C 取消");
        println!();

        let status = std::process::Command::new(&gh)
            .args(["auth", "login"])
            .status()
            .context("启动 gh auth login 失败")?;

        if status.success() {
            ui::print_success("GitHub CLI 登录成功");
        } else {
            ui::print_warning("登录未完成，可稍后手动运行: gh auth login");
        }

        Ok(())
    }
}

/// "gh version 2.87.3 (2025-01-15)" → "2.87.3"
fn parse_gh_version(output: &str) -> String {
    output
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("gh version "))
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("已安装")
        .to_string()
}

/// 找到 gh 可执行文件路径（优先 hudo 安装，其次系统 PATH）
fn find_gh(config: &HudoConfig) -> PathBuf {
    let root = config.tools_dir().join("gh");
    let bin = root.join("bin").join("gh.exe");
    if bin.exists() {
        return bin;
    }
    let direct = root.join("gh.exe");
    if direct.exists() {
        return direct;
    }
    PathBuf::from("gh")
}
