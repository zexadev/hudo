use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct VscodeInstaller;

#[async_trait]
impl Installer for VscodeInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "vscode",
            name: "VS Code",
            description: "Visual Studio Code 编辑器",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let code_exe = ctx.config.ide_dir().join("vscode").join("Code.exe");
        if code_exe.exists() {
            if let Ok(out) = std::process::Command::new(&code_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("unknown")
                        .to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH 上的 code 命令
        if let Ok(out) = std::process::Command::new("code").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let url = config.mirrors.vscode.as_deref()
            .unwrap_or("https://update.code.visualstudio.com/latest/win32-x64-archive/stable")
            .to_string();
        (url, "vscode-win32-x64.zip".to_string())
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.ide_dir().join("vscode");
        let (url, filename) = self.resolve_download(config);

        // 每次下载最新版
        let cached = config.cache_dir().join(&filename);
        if cached.exists() {
            std::fs::remove_file(&cached).ok();
        }

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 VS Code...");
        if install_dir.exists() {
            // 保留 data/ 目录（用户配置和扩展）
            let data_dir = install_dir.join("data");
            let has_data = data_dir.exists();
            let tmp_data = config.cache_dir().join("vscode-data-backup");
            if has_data {
                if tmp_data.exists() {
                    std::fs::remove_dir_all(&tmp_data).ok();
                }
                std::fs::rename(&data_dir, &tmp_data).ok();
            }
            std::fs::remove_dir_all(&install_dir).ok();
            download::extract_zip(&zip_path, &install_dir)?;
            if has_data {
                std::fs::rename(&tmp_data, &install_dir.join("data")).ok();
            }
        } else {
            download::extract_zip(&zip_path, &install_dir)?;
        }

        // 创建 data/ 目录使其成为 portable 模式
        let data_dir = install_dir.join("data");
        std::fs::create_dir_all(&data_dir).ok();

        let version = get_vscode_version(&install_dir).unwrap_or_else(|| "unknown".to_string());

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![
            EnvAction::AppendPath {
                path: install_path.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: install_path.join("bin").to_string_lossy().to_string(),
            },
        ]
    }
}

fn get_vscode_version(install_dir: &PathBuf) -> Option<String> {
    let code_exe = install_dir.join("Code.exe");
    std::process::Command::new(code_exe)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .map(|s| s.to_string())
        })
}
