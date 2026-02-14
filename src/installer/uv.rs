use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct UvInstaller;

#[async_trait]
impl Installer for UvInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "uv",
            name: "uv",
            description: "Python 包管理器与项目管理工具",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let uv_exe = ctx.config.tools_dir().join("uv").join("uv.exe");
        if uv_exe.exists() {
            if let Ok(out) = std::process::Command::new(&uv_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH
        if let Ok(out) = std::process::Command::new("uv").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        (
            "https://astral.sh/uv/install.ps1".to_string(),
            "uv-installer.ps1".to_string(),
        )
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("uv");
        let (url, filename) = self.resolve_download(config);

        // 安装脚本不缓存，总是下载最新版以获取最新 uv
        let cached = config.cache_dir().join(&filename);
        if cached.exists() {
            std::fs::remove_file(&cached).ok();
        }

        // 下载安装脚本
        let ps1_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 用 PowerShell 执行官方安装脚本
        println!("  正在安装 uv...");
        let status = std::process::Command::new("powershell")
            .args([
                "-ExecutionPolicy",
                "ByPass",
                "-File",
                &ps1_path.to_string_lossy(),
            ])
            .env("UV_INSTALL_DIR", &install_dir)
            .env("UV_NO_MODIFY_PATH", "1")
            .status()
            .context("启动 PowerShell 安装脚本失败")?;

        if !status.success() {
            anyhow::bail!(
                "uv 安装脚本执行失败，退出码: {}",
                status.code().unwrap_or(-1)
            );
        }

        let version = get_uv_version(&install_dir).unwrap_or_else(|| "unknown".to_string());

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        let dir = install_path.to_string_lossy();
        vec![
            EnvAction::AppendPath {
                path: dir.to_string(),
            },
            EnvAction::Set {
                name: "UV_PYTHON_INSTALL_DIR".to_string(),
                value: install_path.join("python").to_string_lossy().to_string(),
            },
            EnvAction::Set {
                name: "UV_TOOL_DIR".to_string(),
                value: install_path.join("tools").to_string_lossy().to_string(),
            },
            EnvAction::Set {
                name: "UV_CACHE_DIR".to_string(),
                value: install_path.join("cache").to_string_lossy().to_string(),
            },
        ]
    }
}

/// 获取已安装的 uv 版本
fn get_uv_version(install_dir: &PathBuf) -> Option<String> {
    let uv_exe = install_dir.join("uv.exe");
    std::process::Command::new(uv_exe)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
