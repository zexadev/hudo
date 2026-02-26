use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{run_as_admin, DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;
use crate::registry::InstallRegistry;

pub struct ChromeInstaller;

/// Chrome 不支持自定义安装路径：
/// - 企业 MSI → %ProgramFiles%\Google\Chrome\Application\（需管理员）
/// - 标准安装程序 → %LOCALAPPDATA%\Google\Chrome\Application\（用户级）
#[async_trait]
impl Installer for ChromeInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "chrome",
            name: "Google Chrome",
            description: "Google Chrome 浏览器（路径由 Google 安装程序决定）",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let Some(exe) = find_chrome_exe() else {
            return Ok(DetectResult::NotInstalled);
        };
        let version = get_chrome_version(&exe).unwrap_or_else(|| "已安装".to_string());
        // 通过 state.json 判断是否由 hudo 安装
        let reg = InstallRegistry::load(&ctx.config.state_path()).unwrap_or_default();
        if reg.get("chrome").is_some() {
            Ok(DetectResult::InstalledByHudo(version))
        } else {
            Ok(DetectResult::InstalledExternal(version))
        }
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        (
            "https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi"
                .to_string(),
            "chrome-enterprise-64.msi".to_string(),
        )
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let (url, filename) = self.resolve_download(config);

        let msi_path = download::download(&url, &config.cache_dir(), &filename).await?;
        let msi_str = msi_path.to_string_lossy().to_string();

        crate::ui::print_action("安装 Google Chrome（需要管理员权限）...");

        // 先直接尝试（hudo 以管理员运行时直接成功）
        let direct_ok = std::process::Command::new("msiexec")
            .args(["/i", &msi_str, "/quiet", "/norestart"])
            .status()
            .map(|s| matches!(s.code(), Some(0) | Some(3010)))
            .unwrap_or(false);

        if !direct_ok {
            crate::ui::print_info("需要管理员权限，请在弹出的 UAC 窗口中点击\"是\"...");
            run_as_admin("msiexec", &["/i", &msi_str, "/quiet", "/norestart"])
                .context("Chrome 安装失败")?;
        }

        let install_dir = find_chrome_app_dir()
            .ok_or_else(|| anyhow::anyhow!("Chrome 安装后未找到，请重启终端后重试"))?;
        let version = get_chrome_version(&install_dir.join("chrome.exe"))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, _install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![] // Chrome 不是命令行工具，不需要添加到 PATH
    }

    async fn pre_uninstall(&self, _ctx: &InstallContext<'_>) -> Result<()> {
        // Chrome 自带卸载程序位于 Application/<version>/Installer/setup.exe
        if let Some(app_dir) = find_chrome_app_dir() {
            if let Ok(entries) = std::fs::read_dir(&app_dir) {
                for entry in entries.flatten() {
                    let versioned = entry.path();
                    let setup = versioned.join("Installer").join("setup.exe");
                    if versioned.is_dir() && setup.exists() {
                        crate::ui::print_action("运行 Chrome 卸载程序...");
                        let _ = std::process::Command::new(&setup)
                            .args(["--uninstall", "--force-uninstall"])
                            .status();
                        return Ok(());
                    }
                }
            }
        }
        crate::ui::print_warning("未找到 Chrome 内置卸载程序，请通过「控制面板」手动卸载");
        Ok(())
    }
}

fn find_chrome_exe() -> Option<PathBuf> {
    find_chrome_app_dir().map(|d| d.join("chrome.exe"))
}

fn find_chrome_app_dir() -> Option<PathBuf> {
    // 系统级（企业 MSI 安装，需管理员）
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let path = PathBuf::from(pf)
            .join("Google")
            .join("Chrome")
            .join("Application");
        if path.join("chrome.exe").exists() {
            return Some(path);
        }
    }
    // 用户级（标准安装程序，无需管理员）
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let path = PathBuf::from(local)
            .join("Google")
            .join("Chrome")
            .join("Application");
        if path.join("chrome.exe").exists() {
            return Some(path);
        }
    }
    None
}

fn get_chrome_version(chrome_exe: &PathBuf) -> Option<String> {
    let path_str = chrome_exe.to_string_lossy();
    let ps_cmd = format!(
        "(Get-Item '{}').VersionInfo.FileVersion",
        path_str.replace('\'', "''")
    );
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if v.is_empty() { None } else { Some(v) }
        })
}
