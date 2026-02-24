use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

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
