use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct DotnetInstaller;

const DOTNET_SDK_DEFAULT: &str = "10.0.302";

#[async_trait]
impl Installer for DotnetInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "dotnet",
            name: ".NET SDK",
            description: ".NET SDK（C#/F# 开发工具链）",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let dotnet_exe = ctx.config.lang_dir().join("dotnet").join("dotnet.exe");
        if dotnet_exe.exists() {
            if let Some(v) = dotnet_version(&dotnet_exe) {
                return Ok(DetectResult::InstalledByHudo(v));
            }
            return Ok(DetectResult::InstalledByHudo("已安装".to_string()));
        }

        if let Ok(out) = std::process::Command::new("dotnet").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !version.is_empty() {
                    return Ok(DetectResult::InstalledExternal(version));
                }
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        (
            "https://dot.net/v1/dotnet-install.ps1".to_string(),
            "dotnet-install.ps1".to_string(),
        )
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.lang_dir().join("dotnet");
        let (url, filename) = self.resolve_download(config);

        // 安装脚本会更新，不用缓存
        let cached = config.cache_dir().join(&filename);
        if cached.exists() {
            std::fs::remove_file(&cached).ok();
        }
        let ps1_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 版本 = 锁定 > releases-index 最新活跃 LTS SDK > 内置默认（均传精确 -Version 保证可复现）
        let version = match &config.versions.dotnet {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 .NET SDK 最新 LTS 版本...");
                match crate::version::dotnet_latest().await {
                    Some(v) => {
                        crate::ui::print_info(&format!("最新版本: {}", v));
                        v
                    }
                    None => {
                        crate::ui::print_warning(&format!(
                            "获取最新版本失败，使用内置默认版本 {}",
                            DOTNET_SDK_DEFAULT
                        ));
                        DOTNET_SDK_DEFAULT.to_string()
                    }
                }
            }
        };

        // 官方脚本自带下载与解压逻辑（Azure CDN），-NoPath 由 hudo 统一管理环境变量
        let sp = crate::ui::spinner(&format!("安装 .NET SDK {}（脚本下载约数百 MB）...", version));
        let output = crate::installer::winps_command()
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &ps1_path.to_string_lossy(),
                "-InstallDir",
                &install_dir.to_string_lossy(),
                "-Version",
                &version,
                "-NoPath",
            ])
            .output();
        sp.finish_and_clear();
        let output = output.context("启动 dotnet-install.ps1 失败")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
            let skip = tail.len().saturating_sub(5);
            anyhow::bail!(
                ".NET SDK 安装脚本失败:\n{}",
                tail.into_iter().skip(skip).collect::<Vec<_>>().join("\n")
            );
        }

        let version = dotnet_version(&install_dir.join("dotnet.exe")).unwrap_or(version);

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![
            EnvAction::Set {
                name: "DOTNET_ROOT".to_string(),
                value: install_path.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: install_path.to_string_lossy().to_string(),
            },
        ]
    }
}

fn dotnet_version(exe: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new(exe).arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}
