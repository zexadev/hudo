use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

/// 右键菜单注册表路径
const CONTEXT_MENU_KEYS: &[&str] = &[
    r"Software\Classes\*\shell\VSCode",                    // 右键文件
    r"Software\Classes\Directory\shell\VSCode",            // 右键文件夹
    r"Software\Classes\Directory\Background\shell\VSCode", // 右键文件夹空白处
];

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
        // 1. hudo 安装目录
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

        // 2. 系统常见安装路径（官方安装程序）
        let candidate_paths = {
            let mut paths = vec![];
            // 用户级安装: %LOCALAPPDATA%\Programs\Microsoft VS Code\Code.exe
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                paths.push(
                    std::path::PathBuf::from(&local)
                        .join("Programs")
                        .join("Microsoft VS Code")
                        .join("Code.exe"),
                );
            }
            // 系统级安装: %ProgramFiles%\Microsoft VS Code\Code.exe
            if let Ok(pf) = std::env::var("ProgramFiles") {
                paths.push(
                    std::path::PathBuf::from(&pf)
                        .join("Microsoft VS Code")
                        .join("Code.exe"),
                );
            }
            paths
        };

        for path in &candidate_paths {
            if path.exists() {
                if let Ok(out) = std::process::Command::new(path).arg("--version").output() {
                    if out.status.success() {
                        let version = String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .next()
                            .unwrap_or("unknown")
                            .to_string();
                        return Ok(DetectResult::InstalledExternal(version));
                    }
                }
            }
        }

        // 3. PATH 上的 code 命令（通过 cmd /c 处理 .cmd 扩展名）
        if let Ok(out) = std::process::Command::new("cmd")
            .args(["/c", "code", "--version"])
            .output()
        {
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

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        register_context_menu(ctx.config)
    }

    async fn pre_uninstall(&self, _ctx: &InstallContext<'_>) -> Result<()> {
        unregister_context_menu();
        Ok(())
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

/// 注册 Windows 右键菜单「通过 Code 打开」
fn register_context_menu(config: &HudoConfig) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let code_exe = config.ide_dir().join("vscode").join("Code.exe");
    let code_path = code_exe.to_string_lossy();
    let icon_value = format!("{},0", code_path);

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for key_path in CONTEXT_MENU_KEYS {
        let (shell_key, _) = hkcu
            .create_subkey(key_path)
            .with_context(|| format!("创建注册表项 {} 失败", key_path))?;
        shell_key.set_value("", &"通过 Code 打开")?;
        shell_key.set_value("Icon", &icon_value)?;

        let (cmd_key, _) = shell_key.create_subkey("command")?;
        // 文件夹空白处使用 %V 表示当前目录，文件/文件夹使用 %1
        let cmd_value = if key_path.contains("Background") {
            format!("\"{}\" \"%V\"", code_path)
        } else {
            format!("\"{}\" \"%1\"", code_path)
        };
        cmd_key.set_value("", &cmd_value)?;
    }

    crate::ui::print_action("已注册右键菜单「通过 Code 打开」");
    Ok(())
}

/// 卸载时清理右键菜单注册表项
fn unregister_context_menu() {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for key_path in CONTEXT_MENU_KEYS {
        let _ = hkcu.delete_subkey_all(key_path);
    }
}
