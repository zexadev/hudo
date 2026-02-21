use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct NodejsInstaller;

const FNM_VERSION_DEFAULT: &str = "1.38.1";

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
        let fnm_dir = config.tools_dir().join("fnm");
        let node_dir = config.lang_dir().join("node");
        let (url, filename) = self.resolve_download(config);

        // 使用 latest redirect 时删除缓存（版本未知，文件名相同但内容可能变化）
        if config.versions.fnm.is_none() {
            let cached = config.cache_dir().join(&filename);
            if cached.exists() {
                std::fs::remove_file(&cached).ok();
            }
        }

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

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let fnm_dir = ctx.config.tools_dir().join("fnm");
        let fnm_exe = fnm_dir.join("fnm.exe");

        // 写入 PowerShell profile
        if let Err(e) = write_powershell_profile(&fnm_exe) {
            crate::ui::print_warning(&format!("写入 PowerShell profile 失败: {}", e));
            crate::ui::print_info("请手动在 $PROFILE 中添加：");
            crate::ui::print_info("  fnm env --use-on-cd --shell power-shell | Out-String | Invoke-Expression");
        }

        Ok(())
    }
}

/// 将 fnm 初始化行写入 PowerShell profile（幂等，已存在则跳过）
fn write_powershell_profile(fnm_exe: &std::path::Path) -> Result<()> {
    // 获取 PowerShell profile 路径
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "$PROFILE"])
        .output()
        .context("无法获取 PowerShell profile 路径")?;

    let profile_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if profile_path.is_empty() {
        anyhow::bail!("PowerShell $PROFILE 路径为空");
    }
    let profile_path = std::path::Path::new(&profile_path);

    // 确保 profile 目录存在
    if let Some(parent) = profile_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // fnm 初始化行，使用 fnm.exe 的绝对路径确保可用
    let init_line = format!(
        "# fnm (Node.js version manager)\r\n& '{}' env --use-on-cd --shell power-shell | Out-String | Invoke-Expression",
        fnm_exe.display()
    );

    // 读取现有 profile 内容，已存在则跳过
    let existing = std::fs::read_to_string(profile_path).unwrap_or_default();
    if existing.contains("fnm env") {
        crate::ui::print_info("PowerShell profile 已包含 fnm 初始化，跳过");
        return Ok(());
    }

    // 追加写入
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
