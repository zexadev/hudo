pub mod bun;
pub mod git;
pub mod go;
pub mod jdk;
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

    /// 导出工具配置（如 Git 的 user.name/user.email），默认返回空
    fn export_config(&self, _ctx: &InstallContext<'_>) -> Vec<(String, String)> {
        vec![]
    }

    /// 导入工具配置，默认无操作
    async fn import_config(&self, _ctx: &InstallContext<'_>, _entries: &[(String, String)]) -> Result<()> {
        Ok(())
    }
}

/// 返回所有可用的安装器
pub fn all_installers() -> Vec<Box<dyn Installer>> {
    vec![
        // 工具
        Box::new(git::GitInstaller),
        // 语言环境 — 按语言分组
        Box::new(uv::UvInstaller),           // Python
        Box::new(miniconda::MinicondaInstaller), // Python
        Box::new(nodejs::NodejsInstaller),   // JavaScript
        Box::new(bun::BunInstaller),         // JavaScript
        Box::new(rustup::RustupInstaller),   // Rust
        Box::new(go::GoInstaller),           // Go
        Box::new(jdk::JdkInstaller),         // Java
        Box::new(mingw::MingwInstaller),     // C/C++
        // 数据库
        Box::new(mysql::MysqlInstaller),
        Box::new(pgsql::PgsqlInstaller),
        // 编辑器 / IDE
        Box::new(vscode::VscodeInstaller),
        Box::new(pycharm::PycharmInstaller),
    ]
}
