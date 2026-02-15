use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct GoInstaller;

const GO_VERSION_DEFAULT: &str = "1.24.0";

#[async_trait]
impl Installer for GoInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "go",
            name: "Go",
            description: "Go 编程语言",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let go_exe = ctx.config.lang_dir().join("go").join("bin").join("go.exe");
        if go_exe.exists() {
            if let Ok(out) = std::process::Command::new(&go_exe).arg("version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        // 检查系统 PATH
        if let Ok(out) = std::process::Command::new("go").arg("version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = match config.go.version.as_str() {
            "latest" | "" => GO_VERSION_DEFAULT,
            v => v,
        };
        let filename = format!("go{}.windows-amd64.zip", version);
        let base = config.mirrors.go.as_deref().unwrap_or("https://go.dev/dl");
        let url = format!("{}/{}", base.trim_end_matches('/'), filename);
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.lang_dir().join("go");
        let (url, filename) = self.resolve_download(config);

        // 下载 zip
        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 解压到 lang/ 目录（zip 内有 go/ 顶层目录，解压后即为 lang/go/）
        crate::ui::print_action("解压 Go...");
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        download::extract_zip(&zip_path, &config.lang_dir())?;

        // 创建 GOPATH 目录
        let gopath = config.lang_dir().join("gopath");
        std::fs::create_dir_all(&gopath).ok();

        let version = get_go_version(&install_dir).unwrap_or_else(|| {
            match config.go.version.as_str() {
                "latest" | "" => GO_VERSION_DEFAULT,
                v => v,
            }.to_string()
        });

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, config: &HudoConfig) -> Vec<EnvAction> {
        let gopath = config.lang_dir().join("gopath");
        vec![
            EnvAction::Set {
                name: "GOROOT".to_string(),
                value: install_path.to_string_lossy().to_string(),
            },
            EnvAction::Set {
                name: "GOPATH".to_string(),
                value: gopath.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: install_path.join("bin").to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: gopath.join("bin").to_string_lossy().to_string(),
            },
        ]
    }
}

fn get_go_version(install_dir: &PathBuf) -> Option<String> {
    let go_exe = install_dir.join("bin").join("go.exe");
    std::process::Command::new(go_exe)
        .arg("version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
