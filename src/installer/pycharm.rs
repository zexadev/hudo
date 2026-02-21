use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct PycharmInstaller;

const PYCHARM_VERSION_DEFAULT: &str = "2024.3.5";

#[async_trait]
impl Installer for PycharmInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "pycharm",
            name: "PyCharm",
            description: "PyCharm Community IDE",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let pycharm_exe = ctx.config.ide_dir().join("pycharm").join("bin").join("pycharm64.exe");
        if pycharm_exe.exists() {
            // PyCharm 没有简单的 --version，从 product-info.json 读
            let info_file = ctx.config.ide_dir().join("pycharm").join("product-info.json");
            if let Ok(content) = std::fs::read_to_string(&info_file) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ver) = val.get("version").and_then(|v| v.as_str()) {
                        return Ok(DetectResult::InstalledByHudo(format!("PyCharm CE {}", ver)));
                    }
                }
            }
            return Ok(DetectResult::InstalledByHudo("已安装".to_string()));
        }

        // 检查系统中是否有 pycharm
        if let Ok(out) = std::process::Command::new("where").arg("pycharm64").output() {
            if out.status.success() {
                return Ok(DetectResult::InstalledExternal("已安装".to_string()));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.pycharm.as_deref().unwrap_or(PYCHARM_VERSION_DEFAULT);
        let base = config.mirrors.pycharm.as_deref()
            .unwrap_or("https://download.jetbrains.com");
        let url = format!(
            "{}/python/pycharm-community-{}.win.zip",
            base.trim_end_matches('/'),
            version
        );
        (url, "pycharm-community.zip".to_string())
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.ide_dir().join("pycharm");

        // 解析版本: config > API > hardcoded
        let version = match &config.versions.pycharm {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 PyCharm 最新版本...");
                crate::version::pycharm_latest()
                    .await
                    .unwrap_or_else(|| PYCHARM_VERSION_DEFAULT.to_string())
            }
        };

        let base = config
            .mirrors
            .pycharm
            .as_deref()
            .unwrap_or("https://download.jetbrains.com");
        let url = format!(
            "{}/python/pycharm-community-{}.win.zip",
            base.trim_end_matches('/'),
            version
        );
        let filename = "pycharm-community.zip".to_string();

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 PyCharm Community...");
        // zip 内有版本号子目录如 pycharm-community-2024.3.5/
        let tmp_dir = config.cache_dir().join("pycharm-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // 找到解压出的子目录
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
        vec![EnvAction::AppendPath {
            path: install_path.join("bin").to_string_lossy().to_string(),
        }]
    }
}

