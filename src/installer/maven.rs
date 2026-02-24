use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct MavenInstaller;

const MAVEN_VERSION_DEFAULT: &str = "3.9.9";

#[async_trait]
impl Installer for MavenInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "maven",
            name: "Maven",
            description: "Apache Maven 构建工具 (Java)",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录（mvn.cmd 需通过 cmd /c 执行）
        let mvn_cmd = ctx.config.tools_dir().join("maven").join("bin").join("mvn.cmd");
        if mvn_cmd.exists() {
            if let Ok(out) = std::process::Command::new("cmd")
                .args(["/c", &mvn_cmd.to_string_lossy(), "--version"])
                .output()
            {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("已安装")
                        .to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH（mvn 是 .cmd，通过 cmd /c 调用）
        if let Ok(out) = std::process::Command::new("cmd")
            .args(["/c", "mvn", "--version"])
            .output()
        {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("已安装")
                    .to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.maven.as_deref().unwrap_or(MAVEN_VERSION_DEFAULT);
        let (url, filename) = build_url(config, version);
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("maven");

        // 检测 JDK 是否可用
        super::jdk::ensure_jdk(ctx, "Maven").await?;

        let version = match &config.versions.maven {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 Maven 最新版本...");
                crate::version::maven_latest()
                    .await
                    .unwrap_or_else(|| MAVEN_VERSION_DEFAULT.to_string())
            }
        };

        let (url, filename) = build_url(config, &version);
        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 Maven...");
        let tmp_dir = config.cache_dir().join("maven-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有 apache-maven-{version}/ 子目录
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
                name: "MAVEN_HOME".to_string(),
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
        .maven
        .as_deref()
        .unwrap_or("https://downloads.apache.org/maven/maven-3");
    let url = format!(
        "{}/{}/binaries/apache-maven-{}-bin.zip",
        base.trim_end_matches('/'),
        version,
        version
    );
    let filename = format!("apache-maven-{}-bin.zip", version);
    (url, filename)
}
