use anyhow::Result;
use async_trait::async_trait;
use dialoguer::Confirm;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct JdkInstaller;

const JDK_MAJOR_DEFAULT: &str = "21";

#[async_trait]
impl Installer for JdkInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "jdk",
            name: "Java JDK",
            description: "Adoptium Temurin JDK",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 检查 hudo 安装目录
        let java_exe = ctx.config.lang_dir().join("java").join("bin").join("java.exe");
        if java_exe.exists() {
            if let Ok(out) = std::process::Command::new(&java_exe).arg("-version").output() {
                // java -version 输出到 stderr
                let version = String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                return Ok(DetectResult::InstalledByHudo(version));
            }
        }

        // 检查系统 PATH
        if let Ok(out) = std::process::Command::new("java").arg("-version").output() {
            if out.status.success() || !out.stderr.is_empty() {
                let version = String::from_utf8_lossy(&out.stderr)
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
        let major = match config.java.version.as_str() {
            "" => JDK_MAJOR_DEFAULT,
            v => v,
        };
        let base = config.mirrors.java.as_deref()
            .unwrap_or("https://api.adoptium.net/v3/binary/latest");
        let url = format!(
            "{}/{}/ga/windows/x64/jdk/hotspot/normal/eclipse",
            base.trim_end_matches('/'),
            major
        );
        let filename = format!("adoptium-jdk{}-latest.zip", major);
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.lang_dir().join("java");
        let (url, filename) = self.resolve_download(config);

        // 总是下载最新版（API 返回的是 latest）
        let cached = config.cache_dir().join(&filename);
        if cached.exists() {
            std::fs::remove_file(&cached).ok();
        }

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 解压到临时目录
        crate::ui::print_action("解压 JDK...");
        let tmp_dir = config.cache_dir().join("jdk-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有 jdk-21.0.6+7/ 子目录，移到 lang/java/
        let inner = download::find_single_subdir(&tmp_dir).unwrap_or(tmp_dir.clone());
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        std::fs::rename(&inner, &install_dir).ok();
        std::fs::remove_dir_all(&tmp_dir).ok();

        let major = match config.java.version.as_str() {
            "" => JDK_MAJOR_DEFAULT,
            v => v,
        };
        let version = get_java_version(&install_dir).unwrap_or_else(|| format!("JDK {}", major));

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![
            EnvAction::Set {
                name: "JAVA_HOME".to_string(),
                value: install_path.to_string_lossy().to_string(),
            },
            EnvAction::AppendPath {
                path: install_path.join("bin").to_string_lossy().to_string(),
            },
        ]
    }
}


fn get_java_version(install_dir: &PathBuf) -> Option<String> {
    let java_exe = install_dir.join("bin").join("java.exe");
    std::process::Command::new(java_exe)
        .arg("-version")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stderr)
                .lines()
                .next()
                .map(|s| s.to_string())
        })
}

/// 检测 Java 是否可用（hudo 路径优先，然后系统 PATH）
pub fn detect_java(config: &HudoConfig) -> bool {
    let java_hudo = config.lang_dir().join("java").join("bin").join("java.exe");
    if java_hudo.exists() {
        return true;
    }
    std::process::Command::new("java")
        .arg("-version")
        .output()
        .map(|o| o.status.success() || !o.stderr.is_empty())
        .unwrap_or(false)
}

/// 确保 JDK 可用；若不可用则提示用户选择安装或取消
/// `tool_name` 用于提示信息，如 "Maven"、"Gradle"
pub async fn ensure_jdk(ctx: &InstallContext<'_>, tool_name: &str) -> Result<()> {
    if detect_java(ctx.config) {
        return Ok(());
    }

    crate::ui::print_warning(&format!(
        "未检测到 Java，{} 需要 JDK 才能运行",
        tool_name
    ));

    let install_now = Confirm::new()
        .with_prompt("  是否现在安装 Java JDK？")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !install_now {
        anyhow::bail!("请先安装 JDK：hudo install jdk");
    }

    crate::ui::print_title("安装 Java JDK");
    let result = JdkInstaller.install(ctx).await?;
    crate::ui::print_success(&format!(
        "Java {} 安装完成",
        console::style(&result.version).green()
    ));

    // 持久化环境变量
    let install_path = &result.install_path;
    let actions = JdkInstaller.env_actions(install_path, ctx.config);
    for action in &actions {
        match action {
            super::EnvAction::AppendPath { path } => {
                crate::ui::print_info(&format!("PATH += {}", path));
                crate::env::EnvManager::append_to_path(path)?;
            }
            super::EnvAction::Set { name, value } => {
                crate::ui::print_info(&format!("{} = {}", name, value));
                crate::env::EnvManager::set_var(name, value)?;
            }
        }
    }
    if !actions.is_empty() {
        crate::env::EnvManager::broadcast_change();
    }

    // 将 java/bin 和 JAVA_HOME 注入当前进程，让后续工具能立即找到 java
    let java_bin = install_path.join("bin");
    if let Ok(old_path) = std::env::var("PATH") {
        std::env::set_var("PATH", format!("{};{}", java_bin.display(), old_path));
    }
    std::env::set_var("JAVA_HOME", install_path.to_string_lossy().as_ref());

    // 恢复原工具安装标题
    crate::ui::print_title(&format!("安装 {}", tool_name));

    Ok(())
}
