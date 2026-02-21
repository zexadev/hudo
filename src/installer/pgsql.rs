use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct PgsqlInstaller;

const PG_VERSION_DEFAULT: &str = "17.4";

#[async_trait]
impl Installer for PgsqlInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "pgsql",
            name: "PostgreSQL",
            description: "PostgreSQL 数据库",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let psql_exe = ctx.config.tools_dir().join("pgsql").join("bin").join("psql.exe");
        if psql_exe.exists() {
            if let Ok(out) = std::process::Command::new(&psql_exe).arg("--version").output() {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        if let Ok(out) = std::process::Command::new("psql").arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.pgsql.as_deref().unwrap_or(PG_VERSION_DEFAULT);
        // PostgreSQL binaries zip from EDB (official distribution)
        let filename = format!("postgresql-{}-1-windows-x64-binaries.zip", version);
        let url = format!(
            "https://get.enterprisedb.com/postgresql/{}",
            filename
        );
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("pgsql");

        // 解析版本: config > API > hardcoded
        let version = match &config.versions.pgsql {
            Some(v) => v.clone(),
            None => {
                crate::ui::print_action("查询 PostgreSQL 最新版本...");
                crate::version::pgsql_latest()
                    .await
                    .unwrap_or_else(|| PG_VERSION_DEFAULT.to_string())
            }
        };

        let filename = format!("postgresql-{}-1-windows-x64-binaries.zip", version);
        let url = format!("https://get.enterprisedb.com/postgresql/{}", filename);

        let zip_path = download::download(&url, &config.cache_dir(), &filename).await?;

        crate::ui::print_action("解压 PostgreSQL...");
        let tmp_dir = config.cache_dir().join("pgsql-extract");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }
        download::extract_zip(&zip_path, &tmp_dir)?;

        // zip 内有 pgsql/ 子目录
        let inner = tmp_dir.join("pgsql");
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        if inner.exists() {
            std::fs::rename(&inner, &install_dir).ok();
        } else {
            // 可能直接解压到顶层
            let sub = download::find_single_subdir(&tmp_dir).unwrap_or(tmp_dir.clone());
            std::fs::rename(&sub, &install_dir).ok();
        }
        std::fs::remove_dir_all(&tmp_dir).ok();

        // 创建 data 目录
        let data_dir = install_dir.join("data");
        std::fs::create_dir_all(&data_dir).ok();

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![EnvAction::AppendPath {
            path: install_path.join("bin").to_string_lossy().to_string(),
        }]
    }

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let install_dir = ctx.config.tools_dir().join("pgsql");
        let initdb = install_dir.join("bin").join("initdb.exe");
        let data_dir = install_dir.join("data");

        if data_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
            crate::ui::print_action("初始化 PostgreSQL 数据目录...");
            let status = std::process::Command::new(&initdb)
                .args(["-D", &data_dir.to_string_lossy(), "-U", "postgres", "-E", "UTF8", "--no-locale"])
                .status();

            match status {
                Ok(s) if s.success() => {
                    crate::ui::print_success("PostgreSQL 数据目录初始化完成");
                    crate::ui::print_info("启动: pg_ctl -D <data_dir> start");
                    crate::ui::print_info("连接: psql -U postgres");
                }
                _ => {
                    crate::ui::print_warning("PostgreSQL 初始化失败，请手动执行: initdb -D <data_dir>");
                }
            }
        }

        Ok(())
    }
}

