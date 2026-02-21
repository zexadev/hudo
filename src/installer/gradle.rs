use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct GradleInstaller;

const GRADLE_VERSION_DEFAULT: &str = "8.12.1";

#[async_trait]
impl Installer for GradleInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "gradle",
            name: "Gradle",
            description: "Gradle 构建工具 (Java/Android)",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let gradle_bat = ctx.config.tools_dir().join("gradle").join("bin").join("gradle.bat");
        if gradle_bat.exists() {
            if let Ok(out) = std::process::Command::new(&gradle_bat).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .find(|l| l.starts_with("Gradle "))
                        .unwrap_or("已安装")
                        .to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH
        if let Ok(out) = std::process::Command::new("gradle").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .find(|l| l.starts_with("Gradle "))
                    .unwrap_or("已安装")
                    .to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.gradle.as_deref().unwrap_or(GRADLE_VERSION_DEFAULT);
        build_url(config, version)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("gradle");

        // 检测 JDK 是否可用
        super::jdk::ensure_jdk(ctx, "Gradle").await?;

        let version = match &config.versions.gradle {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 Gradle 最新版本...");
                crate::version::gradle_latest()
                    .await
                    .unwrap_or_else(|| GRADLE_VERSION_DEFAULT.to_string())
            }
        };

        let (url, filename) = build_url(config, &version);
        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 Gradle...");
        let tmp_dir = config.cache_dir().join("gradle-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有 gradle-{version}/ 子目录
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
        vec![
            EnvAction::Set {
                name: "GRADLE_HOME".to_string(),
                value: install_path.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: install_path.join("bin").to_string_lossy().to_string(),
            },
        ]
    }
}

fn build_url(config: &HudoConfig, version: &str) -> (String, String) {
    let base = config
        .mirrors
        .gradle
        .as_deref()
        .unwrap_or("https://services.gradle.org/distributions");
    let url = format!(
        "{}/gradle-{}-bin.zip",
        base.trim_end_matches('/'),
        version
    );
    let filename = format!("gradle-{}-bin.zip", version);
    (url, filename)
}
