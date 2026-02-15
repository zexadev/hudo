use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct MysqlInstaller;

const MYSQL_VERSION: &str = "8.4.4";

#[async_trait]
impl Installer for MysqlInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "mysql",
            name: "MySQL",
            description: "MySQL Community Server",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let mysql_exe = ctx.config.tools_dir().join("mysql").join("bin").join("mysql.exe");
        if mysql_exe.exists() {
            if let Ok(out) = std::process::Command::new(&mysql_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        if let Ok(out) = std::process::Command::new("mysql").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, _config: &HudoConfig) -> (String, String) {
        // MySQL Community Server zip archive (no installer)
        let filename = format!("mysql-{}-winx64.zip", MYSQL_VERSION);
        let url = format!(
            "https://dev.mysql.com/get/Downloads/MySQL-8.4/{}",
            filename
        );
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("mysql");
        let (url, filename) = self.resolve_download(config);

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 MySQL...");
        let tmp_dir = config.cache_dir().join("mysql-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有 mysql-8.4.4-winx64/ 子目录
        let inner = download::find_single_subdir(&tmp_dir).unwrap_or(tmp_dir.clone());
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        std::fs::rename(&inner, &install_dir).ok();
        std::fs::remove_dir_all(&tmp_dir).ok();

        // 创建 data 目录
        let data_dir = install_dir.join("data");
        std::fs::create_dir_all(&data_dir).ok();

        Ok(InstallResult {
            install_path: install_dir,
            version: MYSQL_VERSION.to_string(),
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![EnvAction::AppendPath {
            path: install_path.join("bin").to_string_lossy().to_string(),
        }]
    }

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let install_dir = ctx.config.tools_dir().join("mysql");
        let mysqld = install_dir.join("bin").join("mysqld.exe");
        let data_dir = install_dir.join("data");

        // 如果 data 目录为空，初始化数据库
        if data_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
            crate::ui::print_action("初始化 MySQL 数据目录...");
            let status = std::process::Command::new(&mysqld)
                .args(["--initialize-insecure", &format!("--datadir={}", data_dir.display())])
                .status();

            match status {
                Ok(s) if s.success() => {
                    crate::ui::print_success("MySQL 数据目录初始化完成（root 用户无密码）");
                    crate::ui::print_info("启动: mysqld --console");
                    crate::ui::print_info("连接: mysql -u root");
                }
                _ => {
                    crate::ui::print_warning("MySQL 数据目录初始化失败，请手动执行: mysqld --initialize-insecure");
                }
            }
        }

        Ok(())
    }
}

