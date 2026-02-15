use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct BunInstaller;

#[async_trait]
impl Installer for BunInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "bun",
            name: "Bun",
            description: "JavaScript/TypeScript 运行时与包管理器",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let bun_exe = ctx.config.tools_dir().join("bun").join("bun.exe");
        if bun_exe.exists() {
            if let Ok(out) = std::process::Command::new(&bun_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        if let Ok(out) = std::process::Command::new("bun").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        // Bun 官方提供 Windows x64 zip
        (
            "https://github.com/oven-sh/bun/releases/latest/download/bun-windows-x64.zip"
                .to_string(),
            "bun-windows-x64.zip".to_string(),
        )
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("bun");
        let (url, filename) = self.resolve_download(config);

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 解压到临时目录，再把内容移到 tools/bun/
        crate::ui::print_action("解压 Bun...");
        let tmp_dir = config.cache_dir().join("bun-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有 bun-windows-x64/ 子目录，把内容移到 install_dir
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        let inner = tmp_dir.join("bun-windows-x64");
        if inner.exists() {
            std::fs::rename(&inner, &install_dir)
                .context("移动 Bun 文件失败")?;
        } else {
            // 如果没有子目录，直接重命名 tmp
            std::fs::rename(&tmp_dir, &install_dir)
                .context("移动 Bun 文件失败")?;
        }
        std::fs::remove_dir_all(&tmp_dir).ok();

        let version = get_bun_version(&install_dir).unwrap_or_else(|| "unknown".to_string());

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![EnvAction::AppendPath {
            path: install_path.to_string_lossy().to_string(),
        }]
    }
}

fn get_bun_version(install_dir: &PathBuf) -> Option<String> {
    let bun_exe = install_dir.join("bun.exe");
    std::process::Command::new(bun_exe)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
