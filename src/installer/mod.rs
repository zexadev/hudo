pub mod git;

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
}

/// 返回所有可用的安装器
pub fn all_installers() -> Vec<Box<dyn Installer>> {
    vec![
        Box::new(git::GitInstaller),
    ]
}
