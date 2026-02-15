use anyhow::{Context, Result};
use async_trait::async_trait;
use dialoguer::Input;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::{download, ui};

pub struct GitInstaller;

const GIT_VERSION: &str = "2.47.1.2";
const GIT_TAG: &str = "v2.47.1.windows.2";

#[async_trait]
impl Installer for GitInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "git",
            name: "Git",
            description: "分布式版本控制系统",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        // 先检查 hudo 安装目录
        let git_exe = ctx.config.tools_dir().join("git").join("cmd").join("git.exe");
        if git_exe.exists() {
            if let Ok(out) = std::process::Command::new(&git_exe).arg("--version").output() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledByHudo(version));
            }
        }

        // 再检查系统 PATH
        if let Ok(out) = std::process::Command::new("git").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        let filename = format!("Git-{}-64-bit.exe", GIT_VERSION);
        let url = format!(
            "https://github.com/git-for-windows/git/releases/download/{}/{}",
            GIT_TAG, filename
        );
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("git");
        let (url, filename) = self.resolve_download(config);

        // 下载安装包
        let exe_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 静默安装到指定目录
        crate::ui::print_action("安装 Git（静默模式）...");
        download::run_installer(
            &exe_path,
            &[
                "/VERYSILENT",
                "/NORESTART",
                &format!("/DIR={}", install_dir.display()),
                "/NOICONS",
                "/COMPONENTS=ext,ext\\shellhere,ext\\guihere,gitlfs,assoc,assoc_sh,scalar",
            ],
        )
        .context("Git 安装失败")?;

        Ok(InstallResult {
            install_path: install_dir,
            version: GIT_VERSION.to_string(),
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![EnvAction::AppendPath {
            path: install_path.join("cmd").to_string_lossy().to_string(),
        }]
    }

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        ui::print_title("配置 Git");

        let git = find_git(ctx.config);

        // 读取当前配置
        let current_name = git_config_get(&git, "user.name");
        let current_email = git_config_get(&git, "user.email");

        ui::print_info("Git 需要你的身份信息，用于标记 commit 的作者");
        ui::print_info("这不是登录账号，只是显示在代码历史中的名字和邮箱");
        println!();

        // user.name
        let name: String = match current_name {
            Some(ref d) => Input::new()
                .with_prompt("  user.name")
                .default(d.clone())
                .interact_text()
                .context("输入 user.name 失败")?,
            None => Input::new()
                .with_prompt("  user.name")
                .interact_text()
                .context("输入 user.name 失败")?,
        };

        // user.email
        let email: String = match current_email {
            Some(ref d) => Input::new()
                .with_prompt("  user.email")
                .default(d.clone())
                .interact_text()
                .context("输入 user.email 失败")?,
            None => Input::new()
                .with_prompt("  user.email")
                .interact_text()
                .context("输入 user.email 失败")?,
        };

        // 写入 git global config
        git_config_set(&git, "user.name", &name)?;
        git_config_set(&git, "user.email", &email)?;

        ui::print_success("Git 配置成功");

        Ok(())
    }

    fn export_config(&self, ctx: &InstallContext<'_>) -> Vec<(String, String)> {
        let git = find_git(ctx.config);
        let mut entries = Vec::new();
        if let Some(name) = git_config_get(&git, "user.name") {
            entries.push(("user_name".to_string(), name));
        }
        if let Some(email) = git_config_get(&git, "user.email") {
            entries.push(("user_email".to_string(), email));
        }
        entries
    }

    async fn import_config(&self, ctx: &InstallContext<'_>, entries: &[(String, String)]) -> Result<()> {
        let git = find_git(ctx.config);
        for (key, value) in entries {
            let git_key = match key.as_str() {
                "user_name" => "user.name",
                "user_email" => "user.email",
                _ => continue,
            };
            git_config_set(&git, git_key, value)?;
        }
        Ok(())
    }
}

/// 找到可用的 git 可执行文件路径（优先 hudo 目录）
fn find_git(config: &HudoConfig) -> String {
    let hudo_git = config.tools_dir().join("git").join("cmd").join("git.exe");
    if hudo_git.exists() {
        return hudo_git.to_string_lossy().to_string();
    }
    "git".to_string()
}

/// 读取 git global 配置项
fn git_config_get(git: &str, key: &str) -> Option<String> {
    std::process::Command::new(git)
        .args(["config", "--global", key])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 设置 git global 配置项
fn git_config_set(git: &str, key: &str, value: &str) -> Result<()> {
    let status = std::process::Command::new(git)
        .args(["config", "--global", key, value])
        .status()
        .with_context(|| format!("执行 git config --global {} 失败", key))?;
    if !status.success() {
        anyhow::bail!("git config --global {} 设置失败", key);
    }
    Ok(())
}
