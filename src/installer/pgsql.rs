use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{
    query_service_exists, query_service_state, run_as_admin, DetectResult, EnvAction,
    InstallContext, InstallResult, Installer, ServiceState, ToolInfo,
};
use crate::config::HudoConfig;
use crate::download;

pub struct PgsqlInstaller;

const PG_VERSION_DEFAULT: &str = "17.8";
const PG_SERVICE_NAME: &str = "PostgreSQL";

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
                    let version = parse_pgsql_version(&String::from_utf8_lossy(&out.stdout));
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        if let Ok(out) = std::process::Command::new("psql").arg("--version").output() {
            if out.status.success() {
                let version = parse_pgsql_version(&String::from_utf8_lossy(&out.stdout));
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.pgsql.as_deref().unwrap_or(PG_VERSION_DEFAULT);
        let filename = format!("postgresql-{}-1-windows-x64-binaries.zip", version);
        let base = config
            .mirrors
            .pgsql
            .as_deref()
            .unwrap_or("https://get.enterprisedb.com/postgresql");
        let url = format!("{}/{}", base.trim_end_matches('/'), filename);
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("pgsql");

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
        let base = config
            .mirrors
            .pgsql
            .as_deref()
            .unwrap_or("https://get.enterprisedb.com/postgresql");
        let url = format!("{}/{}", base.trim_end_matches('/'), filename);

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
            let sub = download::find_single_subdir(&tmp_dir).unwrap_or(tmp_dir.clone());
            std::fs::rename(&sub, &install_dir).ok();
        }
        std::fs::remove_dir_all(&tmp_dir).ok();

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
        let pg_ctl = install_dir.join("bin").join("pg_ctl.exe");
        let data_dir = install_dir.join("data");

        // 1. 初始化数据目录（无需管理员权限）
        let is_data_empty = data_dir
            .read_dir()
            .map(|mut d| d.next().is_none())
            .unwrap_or(true);

        if is_data_empty {
            crate::ui::print_action("初始化 PostgreSQL 数据目录...");
            let status = std::process::Command::new(&initdb)
                .args([
                    "-D",
                    &data_dir.to_string_lossy(),
                    "-U",
                    "postgres",
                    "-E",
                    "UTF8",
                    "--no-locale",
                ])
                .status();

            match status {
                Ok(s) if s.success() => {
                    crate::ui::print_success("数据目录初始化完成");
                }
                _ => {
                    crate::ui::print_warning("PostgreSQL 初始化失败，请手动执行: initdb -D <data_dir>");
                    return Ok(());
                }
            }
        }

        // 2. 注册 Windows 服务（需要管理员权限）
        if !query_service_exists(PG_SERVICE_NAME) {
            crate::ui::print_action("注册 PostgreSQL Windows 服务...");
            let pg_ctl_str = pg_ctl.to_string_lossy().to_string();
            let data_str = data_dir.to_string_lossy().to_string();

            // 先直接尝试（hudo 以管理员运行时无需 UAC）
            let _ = std::process::Command::new(&pg_ctl_str)
                .args(["register", "-N", PG_SERVICE_NAME, "-D", &data_str])
                .status();

            // pg_ctl register 权限不足时可能返回 0，用 sc query 验证
            if !query_service_exists(PG_SERVICE_NAME) {
                crate::ui::print_info("需要管理员权限，请在弹出的 UAC 窗口中点击\"是\"...");
                run_as_admin(&pg_ctl_str, &["register", "-N", PG_SERVICE_NAME, "-D", &data_str])?;

                if !query_service_exists(PG_SERVICE_NAME) {
                    anyhow::bail!("PostgreSQL 服务注册失败，请以管理员身份运行 hudo 后重试");
                }
            }
            crate::ui::print_success("PostgreSQL 服务注册成功");
        } else {
            crate::ui::print_info("PostgreSQL 服务已存在，跳过注册");
        }

        // 3. 启动服务
        match query_service_state(PG_SERVICE_NAME) {
            ServiceState::Running => {
                crate::ui::print_success("PostgreSQL 服务已在运行");
            }
            ServiceState::Stopped => {
                let pb = indicatif::ProgressBar::new_spinner();
                pb.set_style(
                    indicatif::ProgressStyle::default_spinner()
                        .template("  {spinner:.cyan} {msg}")
                        .unwrap(),
                );
                pb.set_message("PostgreSQL 服务启动中...");
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                let direct_ok = tokio::task::spawn_blocking(|| {
                    std::process::Command::new("net")
                        .args(["start", PG_SERVICE_NAME])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                })
                .await
                .unwrap_or(false);

                pb.finish_and_clear();

                if direct_ok {
                    crate::ui::print_success("PostgreSQL 服务已启动");
                } else {
                    crate::ui::print_info("需要管理员权限，请在弹出的 UAC 窗口中点击\"是\"...");
                    match run_as_admin("net", &["start", PG_SERVICE_NAME]) {
                        Ok(_) => crate::ui::print_success("PostgreSQL 服务已启动"),
                        Err(_) => {
                            crate::ui::print_warning("PostgreSQL 服务未能自动启动");
                            crate::ui::print_info("请以管理员身份手动运行: net start PostgreSQL");
                        }
                    }
                }
            }
            ServiceState::NotFound => {
                crate::ui::print_warning("PostgreSQL 服务未找到，请重新安装");
                return Ok(());
            }
        }

        crate::ui::print_info("连接: psql -U postgres");
        crate::ui::print_info("停止: net stop PostgreSQL");
        crate::ui::print_info("卸载服务: pg_ctl unregister -N PostgreSQL（需管理员）");

        Ok(())
    }

    async fn pre_uninstall(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let pg_ctl = ctx
            .config
            .tools_dir()
            .join("pgsql")
            .join("bin")
            .join("pg_ctl.exe");
        let pg_ctl_str = pg_ctl.to_string_lossy().to_string();

        crate::ui::print_action("停止 PostgreSQL 服务...");
        let _ = run_as_admin("net", &["stop", PG_SERVICE_NAME]);

        crate::ui::print_action("移除 PostgreSQL 服务注册...");
        let _ = run_as_admin(&pg_ctl_str, &["unregister", "-N", PG_SERVICE_NAME]);

        Ok(())
    }
}

/// 从 `psql --version` 输出中提取版本号
/// "psql (PostgreSQL) 17.8" → "17.8"
fn parse_pgsql_version(output: &str) -> String {
    output
        .split(')')
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("已安装")
        .to_string()
}
