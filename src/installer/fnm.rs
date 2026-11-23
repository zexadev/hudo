use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{
    clean_fnm_profile_lines, force_remove_dir_all, is_legacy_fnm_dir, DetectResult, EnvAction,
    InstallContext, InstallResult, Installer, ToolInfo,
};
use crate::config::HudoConfig;
use crate::download;

pub struct FnmInstaller;

const FNM_VERSION_DEFAULT: &str = "1.38.1";

#[async_trait]
impl Installer for FnmInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "fnm",
            name: "fnm",
            description: "Node.js 版本管理器（多版本切换）",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let fnm_exe = ctx.config.tools_dir().join("fnm").join("fnm.exe");
        if fnm_exe.exists() {
            if let Ok(out) = std::process::Command::new(&fnm_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        if let Ok(out) = std::process::Command::new("fnm").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let filename = "fnm-windows.zip".to_string();
        match &config.versions.fnm {
            Some(fnm_version) => {
                let default_base = format!(
                    "https://github.com/Schniz/fnm/releases/download/v{}",
                    fnm_version
                );
                let base = config.mirrors.fnm.as_deref().unwrap_or(&default_base);
                let url = format!("{}/{}", base.trim_end_matches('/'), filename);
                (url, filename)
            }
            None => {
                let base = config
                    .mirrors
                    .fnm
                    .as_deref()
                    .unwrap_or("https://github.com/Schniz/fnm/releases/latest/download");
                let url = format!("{}/{}", base.trim_end_matches('/'), filename);
                (url, filename)
            }
        }
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let legacy_node_dir = config.lang_dir().join("node");
        // 旧版 fnm 残留迁移：征得同意后清 lang/node/、PowerShell profile，并移除 state 里的 nodejs 条目
        if is_legacy_fnm_dir(&legacy_node_dir) || legacy_fnm_state_exists(config) {
            crate::ui::print_warning("检测到旧版 fnm 残留（由 hudo 0.2.12 及更早版本安装）");
            let confirm = crate::ui::confirm_proceed("是否清理旧版残留后再安装新版 fnm？", true)?;
            if !confirm {
                anyhow::bail!("已取消，请先运行 `hudo uninstall nodejs` 清理旧版后再试");
            }
            cleanup_legacy_fnm(config)?;
        }

        let fnm_dir = config.tools_dir().join("fnm");
        let node_dir = fnm_node_dir(config);
        let (url, filename) = self.resolve_download(config);

        if config.versions.fnm.is_none() {
            let cached = config.cache_dir().join(&filename);
            if cached.exists() {
                std::fs::remove_file(&cached).ok();
            }
        }

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 fnm...");
        std::fs::create_dir_all(&fnm_dir).ok();
        download::extract_zip(&zip_path, &fnm_dir)?;

        std::fs::create_dir_all(&node_dir).ok();

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

        std::process::Command::new(&fnm_exe)
            .args(["default", "lts-latest"])
            .env("FNM_DIR", &node_dir)
            .status()
            .ok();

        let version = get_fnm_version(&fnm_dir).unwrap_or_else(|| {
            config
                .versions
                .fnm
                .as_deref()
                .unwrap_or(FNM_VERSION_DEFAULT)
                .to_string()
        });

        Ok(InstallResult {
            install_path: fnm_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, config: &HudoConfig) -> Vec<EnvAction> {
        let node_dir = fnm_node_dir(config);
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

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let fnm_dir = ctx.config.tools_dir().join("fnm");
        let fnm_exe = fnm_dir.join("fnm.exe");

        if let Err(e) = write_powershell_profile(&fnm_exe) {
            crate::ui::print_warning(&format!("写入 PowerShell profile 失败: {}", e));
            crate::ui::print_info("请手动在 $PROFILE 中添加：");
            crate::ui::print_info("  fnm env --use-on-cd --shell power-shell | Out-String | Invoke-Expression");
        }

        Ok(())
    }

    async fn pre_uninstall(&self, ctx: &InstallContext<'_>) -> Result<()> {
        // 清 PowerShell profile 里的 fnm 初始化（新旧版写入格式一致）
        match clean_fnm_profile_lines() {
            Ok(true) => crate::ui::print_info("已清理 PowerShell profile 中的 fnm 初始化行"),
            Ok(false) => {}
            Err(e) => crate::ui::print_warning(&format!("清理 PowerShell profile 失败: {}", e)),
        }
        // 同时清掉 fnm 管理的 Node 版本目录
        let node_dir = fnm_node_dir(ctx.config);
        if node_dir.exists() {
            if let Err(e) = force_remove_dir_all(&node_dir) {
                crate::ui::print_warning(&format!("删除 {} 失败: {}", node_dir.display(), e));
            } else {
                crate::ui::print_info(&format!("已删除 {}", node_dir.display()));
            }
        }
        Ok(())
    }
}

/// state.json 里 `nodejs` 条目指向 fnm 时，认定为旧版 fnm 残留
fn legacy_fnm_state_exists(config: &HudoConfig) -> bool {
    let Ok(reg) = crate::registry::InstallRegistry::load(&config.state_path()) else {
        return false;
    };
    let Some(entry) = reg.get("nodejs") else { return false };
    entry.install_path.to_lowercase().contains("fnm")
        || entry.version.to_lowercase().starts_with("fnm ")
}

fn cleanup_legacy_fnm(config: &HudoConfig) -> Result<()> {
    crate::ui::print_action("清理旧版 fnm 残留...");
    let node_dir = config.lang_dir().join("node");
    if node_dir.exists() {
        force_remove_dir_all(&node_dir)
            .with_context(|| format!("删除 {} 失败（可能有进程占用）", node_dir.display()))?;
        crate::ui::print_info(&format!("已删除 {}", node_dir.display()));
    }
    let old_fnm_dir = config.tools_dir().join("fnm");
    if old_fnm_dir.exists() {
        // 只删 fnm.exe 等旧内容；如果目录存在但准备重装新版，重装逻辑会重新解压
        force_remove_dir_all(&old_fnm_dir).ok();
    }
    if legacy_fnm_state_exists(config) {
        let mut reg = crate::registry::InstallRegistry::load(&config.state_path())?;
        reg.remove("nodejs");
        reg.save(&config.state_path()).ok();
    }
    if let Ok(Some(val)) = crate::env::EnvManager::get_var("FNM_DIR") {
        let target = config.lang_dir().join("node").to_string_lossy().to_lowercase();
        if val.to_lowercase() == target {
            crate::env::EnvManager::delete_var("FNM_DIR").ok();
            crate::ui::print_info("已移除旧 FNM_DIR 环境变量");
        }
    }
    match clean_fnm_profile_lines() {
        Ok(true) => crate::ui::print_info("已清理 PowerShell profile 中的 fnm 初始化行"),
        Ok(false) => {}
        Err(e) => crate::ui::print_warning(&format!("清理 PowerShell profile 失败: {}", e)),
    }
    Ok(())
}

/// fnm 管理的 Node.js 版本目录，独立于纯 nodejs 安装器的 lang/node/
fn fnm_node_dir(config: &HudoConfig) -> PathBuf {
    config.lang_dir().join("node-fnm")
}

fn write_powershell_profile(fnm_exe: &std::path::Path) -> Result<()> {
    let output = crate::installer::winps_command()
        .args(["-NoProfile", "-Command", "$PROFILE"])
        .output()
        .context("无法获取 PowerShell profile 路径")?;

    let profile_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if profile_path.is_empty() {
        anyhow::bail!("PowerShell $PROFILE 路径为空");
    }
    let profile_path = std::path::Path::new(&profile_path);

    if let Some(parent) = profile_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let init_line = format!(
        "# fnm (Node.js version manager)\r\n& '{}' env --use-on-cd --shell power-shell | Out-String | Invoke-Expression",
        fnm_exe.display()
    );

    let existing = std::fs::read_to_string(profile_path).unwrap_or_default();
    if existing.contains("fnm env") {
        crate::ui::print_info("PowerShell profile 已包含 fnm 初始化，跳过");
        return Ok(());
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(profile_path)
        .context("打开 PowerShell profile 失败")?;

    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(file)?;
    }
    writeln!(file, "\r\n{}", init_line)?;

    crate::ui::print_success("已写入 PowerShell profile，重开终端后 node 命令即可使用");
    Ok(())
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
