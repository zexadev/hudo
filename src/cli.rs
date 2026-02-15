use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hudo", version, about = "混沌 - 开发环境一键引导工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 交互式多选安装开发工具
    Setup,
    /// 安装单个工具
    Install {
        /// 工具名称（git, uv, nodejs, bun, rust, go, jdk, c, miniconda, mysql, pgsql, vscode, pycharm）
        tool: String,
    },
    /// 卸载由 hudo 安装的工具
    Uninstall {
        /// 工具名称
        tool: String,
    },
    /// 列出所有工具及安装状态
    List,
    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// 显示当前配置
    Show,
    /// 设置配置项（key=value）
    Set {
        /// 配置键
        key: String,
        /// 配置值
        value: String,
    },
    /// 用编辑器打开配置文件
    Edit,
    /// 重置配置为默认值
    Reset,
}
