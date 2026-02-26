pub mod bun;
pub mod chrome;
pub mod claude_code;
pub mod gh;
pub mod git;
pub mod go;
pub mod gradle;
pub mod jdk;
pub mod maven;
pub mod miniconda;
pub mod mingw;
pub mod mysql;
pub mod nodejs;
pub mod pgsql;
pub mod pycharm;
pub mod rustup;
pub mod uv;
pub mod vscode;

use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::config::HudoConfig;

/// 工具基本信息
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// 工具标识符（如 "git"）
    pub id: &'static str,
    /// 显示名称（如 "Git"）
    pub name: &'static str,
    /// 简短描述
    pub description: &'static str,
}

/// 环境变量操作
#[derive(Debug, Clone)]
pub enum EnvAction {
    /// 设置环境变量
    Set { name: String, value: String },
    /// 追加到 PATH
    AppendPath { path: String },
}

/// 检测结果
#[derive(Debug)]
pub enum DetectResult {
    /// 未安装
    NotInstalled,
    /// 已由 hudo 安装在 hudo 目录
    InstalledByHudo(String),
    /// 已安装在系统其他位置（非 hudo 管理）
    InstalledExternal(String),
}

/// 安装结果
#[derive(Debug)]
pub struct InstallResult {
    /// 安装路径
    pub install_path: PathBuf,
    /// 安装的版本
    pub version: String,
}

/// 安装上下文，传递给安装器
pub struct InstallContext<'a> {
    pub config: &'a HudoConfig,
}

/// 安装器 trait
#[async_trait]
pub trait Installer: Send + Sync {
    /// 工具基本信息
    fn info(&self) -> ToolInfo;

    /// 检测是否已安装
    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult>;

    /// 返回 (下载 URL, 缓存文件名)
    fn resolve_download(&self, config: &HudoConfig) -> (String, String);

    /// 执行安装
    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult>;

    /// 安装后需要执行的环境变量操作
    fn env_actions(&self, install_path: &PathBuf, config: &HudoConfig) -> Vec<EnvAction>;

    /// 安装后的交互式配置（默认无操作）
    async fn configure(&self, _ctx: &InstallContext<'_>) -> Result<()> {
        Ok(())
    }

    /// 卸载前的清理操作（默认无操作）
    /// 在删除安装目录之前调用，用于停止服务等操作
    async fn pre_uninstall(&self, _ctx: &InstallContext<'_>) -> Result<()> {
        Ok(())
    }

    /// 导出工具配置（如 Git 的 user.name/user.email），默认返回空
    fn export_config(&self, _ctx: &InstallContext<'_>) -> Vec<(String, String)> {
        vec![]
    }

    /// 导入工具配置，默认无操作
    async fn import_config(&self, _ctx: &InstallContext<'_>, _entries: &[(String, String)]) -> Result<()> {
        Ok(())
    }
}

// ── Windows 服务管理工具（mysql、pgsql 共用） ───────────────────────────────

pub enum ServiceState {
    Running,
    Stopped,
    NotFound,
}

pub fn query_service_exists(name: &str) -> bool {
    std::process::Command::new("sc")
        .args(["query", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn query_service_state(name: &str) -> ServiceState {
    match std::process::Command::new("sc").args(["query", name]).output() {
        Ok(out) if out.status.success() => {
            if String::from_utf8_lossy(&out.stdout).contains("RUNNING") {
                ServiceState::Running
            } else {
                ServiceState::Stopped
            }
        }
        _ => ServiceState::NotFound,
    }
}

/// 通过 PowerShell Start-Process -Verb RunAs 以管理员身份运行命令
pub fn run_as_admin(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let prog_escaped = program.replace('\'', "''");
    let arg_list: Vec<String> = args
        .iter()
        .map(|a| format!("'{}'", a.replace('\'', "''")))
        .collect();

    let ps_cmd = format!(
        "try {{ \
           $p = Start-Process -FilePath '{}' -ArgumentList @({}) \
                -Verb RunAs -Wait -PassThru -WindowStyle Hidden; \
           if ($p) {{ exit $p.ExitCode }} else {{ exit 1 }} \
         }} catch {{ exit 1 }}",
        prog_escaped,
        arg_list.join(", ")
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.trim().is_empty() {
            anyhow::bail!("管理员权限操作失败（用户可能拒绝了 UAC 提示）: {}", program)
        } else {
            anyhow::bail!("管理员权限操作失败: {}\n{}", program, stderr.trim())
        }
    }
}

/// 返回所有可用的安装器
pub fn all_installers() -> Vec<Box<dyn Installer>> {
    vec![
        // 工具
        Box::new(git::GitInstaller),
        Box::new(gh::GhInstaller),
        Box::new(claude_code::ClaudeCodeInstaller),
        // 语言环境 — 按语言分组
        Box::new(uv::UvInstaller),           // Python
        Box::new(miniconda::MinicondaInstaller), // Python
        Box::new(nodejs::NodejsInstaller),   // JavaScript
        Box::new(bun::BunInstaller),         // JavaScript
        Box::new(rustup::RustupInstaller),   // Rust
        Box::new(go::GoInstaller),           // Go
        Box::new(jdk::JdkInstaller),         // Java
        Box::new(maven::MavenInstaller),     // Java 构建
        Box::new(gradle::GradleInstaller),   // Java/Android 构建
        Box::new(mingw::MingwInstaller),     // C/C++
        // 数据库
        Box::new(mysql::MysqlInstaller),
        Box::new(pgsql::PgsqlInstaller),
        // 编辑器 / IDE
        Box::new(vscode::VscodeInstaller),
        Box::new(pycharm::PycharmInstaller),
        Box::new(chrome::ChromeInstaller),
    ]
}
