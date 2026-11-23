use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;
use crate::registry::InstallRegistry;

pub struct PowertoysInstaller;

const POWERTOYS_VERSION_DEFAULT: &str = "0.100.2";

/// PowerToys 用户级安装器固定装到 %LOCALAPPDATA%\PowerToys（路径由微软安装器决定），
/// hudo 归属靠 state.json 记录判断（与 chrome 同模式）
#[async_trait]
impl Installer for PowertoysInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "powertoys",
            name: "PowerToys",
            description: "微软官方效率工具集（路径由安装程序决定）",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let Some(exe) = find_powertoys_exe() else {
            return Ok(DetectResult::NotInstalled);
        };
        let version = file_version(&exe).unwrap_or_else(|| "已安装".to_string());
        let reg = InstallRegistry::load(&ctx.config.state_path()).unwrap_or_default();
        if reg.get("powertoys").is_some() {
            Ok(DetectResult::InstalledByHudo(version))
        } else {
            Ok(DetectResult::InstalledExternal(version))
        }
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config
            .versions
            .powertoys
            .as_deref()
            .unwrap_or(POWERTOYS_VERSION_DEFAULT);
        (download_url(version), setup_filename(version))
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;

        let version = match &config.versions.powertoys {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 PowerToys 最新版本...");
                match crate::version::powertoys_latest().await {
                    Some(v) => {
                        crate::ui::print_info(&format!("最新版本: {}", v));
                        v
                    }
                    None => {
                        crate::ui::print_warning(&format!(
                            "获取最新版本失败，使用内置默认版本 {}",
                            POWERTOYS_VERSION_DEFAULT
                        ));
                        POWERTOYS_VERSION_DEFAULT.to_string()
                    }
                }
            }
        };

        let filename = setup_filename(&version);
        let exe_path = download::download(&download_url(&version), &config.cache_dir(), &filename).await?;

        // UserSetup 为 per-user 安装（免 UAC）；WiX Burn 引导器支持 /quiet /norestart 静默
        let sp = crate::ui::spinner("正在运行 PowerToys 安装程序（可能需要几分钟）...");
        let output = std::process::Command::new(&exe_path)
            .args(["/quiet", "/norestart"])
            .output();
        sp.finish_and_clear();
        let output = output.context("启动 PowerToys 安装程序失败")?;

        if !output.status.success() {
            anyhow::bail!(
                "PowerToys 安装程序退出码: {}",
                output.status.code().unwrap_or(-1)
            );
        }

        let install_dir = find_powertoys_dir()
            .ok_or_else(|| anyhow::anyhow!("PowerToys 安装后未找到，请重启终端后重试"))?;
        let version = file_version(&install_dir.join("PowerToys.exe")).unwrap_or(version);

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, _install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![] // GUI 工具集，不进 PATH
    }

    async fn pre_uninstall(&self, _ctx: &InstallContext<'_>) -> Result<()> {
        // 先停掉常驻进程，否则卸载器可能等待或失败
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", "PowerToys.exe", "/F", "/T"])
            .output();

        // Burn 引导器在 Uninstall 注册表记录 QuietUninstallString，per-user 在 HKCU。
        // 不能经 cmd /c 执行：std 会把内嵌引号转义成 \"，cmd 不识别，含空格路径必断；
        // 改为解析出 exe 与参数直接执行
        if let Some(cmd) = quiet_uninstall_string() {
            let Some((exe, args)) = split_command_line(&cmd) else {
                crate::ui::print_warning(&format!("无法解析卸载命令: {}", cmd));
                return Ok(());
            };
            crate::ui::print_action("运行 PowerToys 卸载程序...");
            let sp = crate::ui::spinner("等待 PowerToys 卸载完成...");
            let output = std::process::Command::new(&exe).args(&args).output();
            sp.finish_and_clear();
            match output {
                Ok(o) if o.status.success() => return Ok(()),
                Ok(o) => crate::ui::print_warning(&format!(
                    "PowerToys 卸载程序退出码: {}",
                    o.status.code().unwrap_or(-1)
                )),
                Err(e) => crate::ui::print_warning(&format!("PowerToys 卸载程序启动失败: {}", e)),
            }
        } else {
            crate::ui::print_warning("未找到 PowerToys 卸载入口，请通过「设置 → 应用」手动卸载");
        }
        Ok(())
    }
}

fn download_url(version: &str) -> String {
    format!(
        "https://github.com/microsoft/PowerToys/releases/download/v{}/{}",
        version,
        setup_filename(version)
    )
}

fn setup_filename(version: &str) -> String {
    format!("PowerToysUserSetup-{}-x64.exe", version)
}

fn find_powertoys_exe() -> Option<PathBuf> {
    find_powertoys_dir().map(|d| d.join("PowerToys.exe"))
}

fn find_powertoys_dir() -> Option<PathBuf> {
    // 用户级（UserSetup，hudo 使用的方式）
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let path = PathBuf::from(local).join("PowerToys");
        if path.join("PowerToys.exe").exists() {
            return Some(path);
        }
    }
    // 机器级（PowerToysSetup 或 winget 默认）
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let path = PathBuf::from(pf).join("PowerToys");
        if path.join("PowerToys.exe").exists() {
            return Some(path);
        }
    }
    None
}

fn file_version(exe: &std::path::Path) -> Option<String> {
    let ps_cmd = format!(
        "(Get-Item '{}').VersionInfo.ProductVersion",
        exe.to_string_lossy().replace('\'', "''")
    );
    crate::installer::winps_command()
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        })
}

/// 拆分注册表命令串为 (exe, 参数)：`"C:\path with space\x.exe" /a /b` 或 `MsiExec.exe /X{...}`
fn split_command_line(cmd: &str) -> Option<(String, Vec<String>)> {
    let cmd = cmd.trim();
    if let Some(rest) = cmd.strip_prefix('"') {
        let end = rest.find('"')?;
        let exe = rest[..end].to_string();
        let args = rest[end + 1..]
            .split_whitespace()
            .map(String::from)
            .collect();
        Some((exe, args))
    } else {
        let mut parts = cmd.split_whitespace();
        let exe = parts.next()?.to_string();
        Some((exe, parts.map(String::from).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_command_line() {
        let (exe, args) = split_command_line(
            r#""C:\Users\a b\Package Cache\{X}\PowerToysUserSetup.exe" /uninstall /quiet"#,
        )
        .unwrap();
        assert_eq!(exe, r"C:\Users\a b\Package Cache\{X}\PowerToysUserSetup.exe");
        assert_eq!(args, vec!["/uninstall", "/quiet"]);

        let (exe, args) = split_command_line("MsiExec.exe /X{GUID}").unwrap();
        assert_eq!(exe, "MsiExec.exe");
        assert_eq!(args, vec!["/X{GUID}"]);
    }
}

/// 从 HKCU/HKLM 的 Uninstall 表找 PowerToys 的静默卸载命令
fn quiet_uninstall_string() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let hive = RegKey::predef(root);
        let Ok(uninstall) = hive.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") else {
            continue;
        };
        for name in uninstall.enum_keys().flatten() {
            let Ok(key) = uninstall.open_subkey(&name) else { continue };
            let display: String = key.get_value("DisplayName").unwrap_or_default();
            if display.starts_with("PowerToys") {
                if let Ok(quiet) = key.get_value::<String, _>("QuietUninstallString") {
                    return Some(quiet);
                }
                if let Ok(cmd) = key.get_value::<String, _>("UninstallString") {
                    return Some(format!("{} /quiet", cmd));
                }
            }
        }
    }
    None
}
