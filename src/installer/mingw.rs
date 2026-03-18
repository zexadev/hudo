use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct MingwInstaller;

/// MinGW-w64 via winlibs (GCC + MinGW-w64, standalone build)
const MINGW_GCC_VERSION: &str = "15.2.0";
const MINGW_W64_VERSION: &str = "13.0.0";
const MINGW_REVISION: &str = "r6";

#[async_trait]
impl Installer for MingwInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "c",
            name: "C/C++",
            description: "GCC 编译器 (MinGW-w64)",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let gcc_exe = ctx.config.tools_dir().join("mingw64").join("bin").join("gcc.exe");
        if gcc_exe.exists() {
            if let Ok(out) = std::process::Command::new(&gcc_exe).arg("--version").output() {
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

        // 检查系统 PATH
        if let Ok(out) = std::process::Command::new("gcc").arg("--version").output() {
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

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        // 实际下载 URL 在 install() 中动态获取，此处仅作 trait 占位
        // 回退到硬编码版本（与 install() 中的 unwrap_or_else 一致）
        let tag = format!("{}posix-{}-ucrt-{}", MINGW_GCC_VERSION, MINGW_W64_VERSION, MINGW_REVISION);
        let filename = format!("winlibs-x86_64-posix-seh-gcc-{}-mingw-w64ucrt-{}-{}.zip", MINGW_GCC_VERSION, MINGW_W64_VERSION, MINGW_REVISION);
        let url = format!("https://github.com/brechtsanders/winlibs_mingw/releases/download/{}/{}", tag, filename);
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("mingw64");

        crate::ui::print_action("查询 MinGW-w64 最新版本...");
        let (url, filename, gcc_version) = match crate::version::mingw_latest().await {
            Some((tag, filename, gcc_version)) => {
                let url = format!(
                    "https://github.com/brechtsanders/winlibs_mingw/releases/download/{}/{}",
                    tag, filename
                );
                (url, filename, gcc_version)
            }
            None => {
                let (url, filename) = self.resolve_download(config);
                (url, filename, MINGW_GCC_VERSION.to_string())
            }
        };

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 解压（zip 内有 mingw64/ 顶层目录）
        crate::ui::print_action("解压 MinGW-w64...");
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        download::extract_zip(&zip_path, &config.tools_dir())?;

        // 验证
        let gcc = install_dir.join("bin").join("gcc.exe");
        if !gcc.exists() {
            anyhow::bail!("解压后未找到 gcc.exe，安装可能失败");
        }

        let version = get_gcc_version(&install_dir).unwrap_or(gcc_version);

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

fn get_gcc_version(install_dir: &PathBuf) -> Option<String> {
    let gcc = install_dir.join("bin").join("gcc.exe");
    std::process::Command::new(gcc)
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
