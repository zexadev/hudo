use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct NodejsInstaller;

const FNM_VERSION: &str = "1.38.1";

#[async_trait]
impl Installer for NodejsInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "nodejs",
            name: "Node.js",
            description: "Node.js 运行时 (via fnm)",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 的 fnm
        let fnm_exe = ctx.config.tools_dir().join("fnm").join("fnm.exe");
        if fnm_exe.exists() {
            if let Ok(out) = std::process::Command::new(&fnm_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH 上的 fnm 或 node
        if let Ok(out) = std::process::Command::new("fnm").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }
        if let Ok(out) = std::process::Command::new("node").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(format!("node {}", version)));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let filename = "fnm-windows.zip".to_string();
        let default_base = format!(
            "https://github.com/Schniz/fnm/releases/download/v{}",
            FNM_VERSION
        );
        let base = config.mirrors.fnm.as_deref().unwrap_or(&default_base);
        let url = format!("{}/{}", base.trim_end_matches('/'), filename);
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let fnm_dir = config.tools_dir().join("fnm");
        let node_dir = config.lang_dir().join("node");
        let (url, filename) = self.resolve_download(config);

        // 下载 fnm zip
        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 解压 fnm.exe 到 tools/fnm/
        crate::ui::print_action("解压 fnm...");
        std::fs::create_dir_all(&fnm_dir).ok();
        download::extract_zip(&zip_path, &fnm_dir)?;

        // 创建 FNM_DIR
        std::fs::create_dir_all(&node_dir).ok();

        // 用 fnm 安装最新 LTS 版 Node.js
        crate::ui::print_action("通过 fnm 安装 Node.js LTS...");
        let fnm_exe = fnm_dir.join("fnm.exe");
        let status = std::process::Command::new(&fnm_exe)
            .args(["install", "--lts"])
            .env("FNM_DIR", &node_dir)
            .status()
            .context("fnm install --lts 失败")?;

        if !status.success() {
            anyhow::bail!(
                "fnm install 失败，退出码: {}",
                status.code().unwrap_or(-1)
            );
        }

        // 设置默认版本
        std::process::Command::new(&fnm_exe)
            .args(["default", "lts-latest"])
            .env("FNM_DIR", &node_dir)
            .status()
            .ok();

        let version = get_fnm_version(&fnm_dir).unwrap_or_else(|| FNM_VERSION.to_string());

        Ok(InstallResult {
            install_path: fnm_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, config: &HudoConfig) -> Vec<EnvAction> {
        let node_dir = config.lang_dir().join("node");
        vec![
            EnvAction::Set {
                name: "FNM_DIR".to_string(),
                value: node_dir.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: install_path.to_string_lossy().to_string(),
            },
        ]
    }
}

fn get_fnm_version(fnm_dir: &PathBuf) -> Option<String> {
    let fnm_exe = fnm_dir.join("fnm.exe");
    std::process::Command::new(fnm_exe)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
