use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;

pub struct MysqlInstaller;

const MYSQL_VERSION_DEFAULT: &str = "8.4.8";
const MYSQL_SERVICE_NAME: &str = "MySQL";

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
                    let version = parse_mysql_version(&String::from_utf8_lossy(&out.stdout));
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
        }

        if let Ok(out) = std::process::Command::new("mysql").arg("--version").output() {
            if out.status.success() {
                let version = parse_mysql_version(&String::from_utf8_lossy(&out.stdout));
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config.versions.mysql.as_deref().unwrap_or(MYSQL_VERSION_DEFAULT);
        let filename = format!("mysql-{}-winx64.zip", version);
        let major_minor = version.rsplitn(2, '.').last().unwrap_or(version);
        let base = config
            .mirrors
            .mysql
            .as_deref()
            .unwrap_or("https://cdn.mysql.com/Downloads");
        let url = format!(
            "{}/MySQL-{}/{}",
            base.trim_end_matches('/'),
            major_minor,
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

        let inner = download::find_single_subdir(&tmp_dir).unwrap_or(tmp_dir.clone());
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).ok();
        }
        std::fs::rename(&inner, &install_dir).ok();
        std::fs::remove_dir_all(&tmp_dir).ok();

        let version = config
            .versions
            .mysql
            .as_deref()
            .unwrap_or(MYSQL_VERSION_DEFAULT);

        Ok(InstallResult {
            install_path: install_dir,
            version: version.to_string(),
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![EnvAction::AppendPath {
            path: install_path.join("bin").to_string_lossy().to_string(),
        }]
    }

    async fn pre_uninstall(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let mysqld = ctx
            .config
            .tools_dir()
            .join("mysql")
            .join("bin")
            .join("mysqld.exe");

        // 停止服务（忽略失败：可能服务未运行）
        crate::ui::print_action("停止 MySQL 服务...");
        let _ = run_as_admin("net", &["stop", MYSQL_SERVICE_NAME]);

        // 移除服务注册（忽略失败：可能服务未注册）
        crate::ui::print_action("移除 MySQL 服务注册...");
        let mysqld_str = mysqld.to_string_lossy().to_string();
        let _ = run_as_admin(&mysqld_str, &["--remove", MYSQL_SERVICE_NAME]);

        Ok(())
    }

    async fn configure(&self, ctx: &InstallContext<'_>) -> Result<()> {
        let install_dir = ctx.config.tools_dir().join("mysql");
        let mysqld = install_dir.join("bin").join("mysqld.exe");
        let data_dir = install_dir.join("data");

        // 1. 生成 my.ini
        crate::ui::print_action("生成 my.ini...");
        let my_ini = write_my_ini(&install_dir)?;
        crate::ui::print_info(&format!("配置文件: {}", my_ini.display()));

        // 2. 初始化数据目录（若为空，不需要管理员权限）
        let is_data_empty = data_dir
            .read_dir()
            .map(|mut d| d.next().is_none())
            .unwrap_or(true);

        if is_data_empty {
            crate::ui::print_action("初始化 MySQL 数据目录...");
            let basedir_arg = format!("--basedir={}", install_dir.display());
            let datadir_arg = format!("--datadir={}", data_dir.display());
            let status = std::process::Command::new(&mysqld)
                .args(["--initialize-insecure", &basedir_arg, &datadir_arg])
                .status();

            match status {
                Ok(s) if s.success() => {
                    crate::ui::print_success("数据目录初始化完成（root 用户无密码）");
                }
                _ => {
                    crate::ui::print_warning("数据目录初始化失败");
                    crate::ui::print_info(&format!(
                        "  请手动执行: {} --initialize-insecure {} {}",
                        mysqld.display(),
                        basedir_arg,
                        datadir_arg
                    ));
                    return Ok(());
                }
            }
        }

        // 3. 注册 Windows 服务（需要管理员权限）
        if !query_service_exists(MYSQL_SERVICE_NAME) {
            crate::ui::print_action("注册 MySQL Windows 服务...");
            let mysqld_str = mysqld.to_string_lossy().to_string();
            let defaults_arg = format!("--defaults-file={}", my_ini.display());

            // 先直接尝试（hudo 以管理员运行时无需 UAC）
            let _ = std::process::Command::new(&mysqld_str)
                .args(["--install", MYSQL_SERVICE_NAME, &defaults_arg])
                .status();

            // mysqld --install 权限不足时可能返回 0，用 sc query 验证注册是否成功
            if !query_service_exists(MYSQL_SERVICE_NAME) {
                crate::ui::print_info("需要管理员权限，请在弹出的 UAC 窗口中点击\"是\"...");
                run_as_admin(&mysqld_str, &["--install", MYSQL_SERVICE_NAME, &defaults_arg])?;

                if !query_service_exists(MYSQL_SERVICE_NAME) {
                    anyhow::bail!("MySQL 服务注册失败，请以管理员身份运行 hudo 后重试");
                }
            }
            crate::ui::print_success("MySQL 服务注册成功");
        } else {
            crate::ui::print_info("MySQL 服务已存在，跳过注册");
        }

        // 4. 启动服务
        match query_service_state(MYSQL_SERVICE_NAME) {
            ServiceState::Running => {
                crate::ui::print_success("MySQL 服务已在运行");
            }
            ServiceState::Stopped => {
                // net start 是同步阻塞调用，用 spinner 显示等待状态
                let pb = indicatif::ProgressBar::new_spinner();
                pb.set_style(
                    indicatif::ProgressStyle::default_spinner()
                        .template("  {spinner:.cyan} {msg}")
                        .unwrap(),
                );
                pb.set_message("MySQL 服务启动中...");
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                let direct_ok = tokio::task::spawn_blocking(|| {
                    std::process::Command::new("net")
                        .args(["start", MYSQL_SERVICE_NAME])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                })
                .await
                .unwrap_or(false);

                pb.finish_and_clear();

                if direct_ok {
                    crate::ui::print_success("MySQL 服务已启动");
                } else {
                    // 需要提权，触发 UAC
                    crate::ui::print_info("需要管理员权限，请在弹出的 UAC 窗口中点击\"是\"...");
                    match run_as_admin("net", &["start", MYSQL_SERVICE_NAME]) {
                        Ok(_) => crate::ui::print_success("MySQL 服务已启动"),
                        Err(_) => {
                            crate::ui::print_warning("MySQL 服务未能自动启动");
                            crate::ui::print_info("请以管理员身份手动运行: net start MySQL");
                        }
                    }
                }
            }
            ServiceState::NotFound => {
                crate::ui::print_warning("MySQL 服务未找到，请重新安装");
                return Ok(());
            }
        }
        crate::ui::print_info("连接: mysql -u root");
        crate::ui::print_info("停止: net stop MySQL");
        crate::ui::print_info("卸载服务: mysqld --remove MySQL（需管理员）");

        Ok(())
    }
}

/// 生成 my.ini 配置文件
fn write_my_ini(install_dir: &PathBuf) -> Result<PathBuf> {
    let my_ini = install_dir.join("my.ini");
    // MySQL 配置文件中路径使用正斜杠
    let basedir = install_dir.to_string_lossy().replace('\\', "/");
    let datadir = install_dir.join("data").to_string_lossy().replace('\\', "/");

    let content = format!(
        "[mysqld]\n\
        basedir={basedir}\n\
        datadir={datadir}\n\
        port=3306\n\
        character-set-server=utf8mb4\n\
        collation-server=utf8mb4_unicode_ci\n\
        default-storage-engine=INNODB\n\
        max_connections=150\n\
        innodb_buffer_pool_size=128M\n\
        \n\
        [mysql]\n\
        default-character-set=utf8mb4\n\
        \n\
        [client]\n\
        default-character-set=utf8mb4\n\
        port=3306\n",
        basedir = basedir,
        datadir = datadir,
    );

    std::fs::write(&my_ini, content)?;
    Ok(my_ini)
}

use super::{query_service_exists, query_service_state, run_as_admin, ServiceState};

/// 从 `mysql --version` 输出中提取版本号
/// "D:\...\mysql.exe  Ver 8.4.8 for Win64 ..." → "8.4.8"
fn parse_mysql_version(output: &str) -> String {
    output
        .split("Ver ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("已安装")
        .to_string()
}

