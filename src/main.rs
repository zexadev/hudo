mod cli;
mod config;
mod download;
mod env;
mod installer;
mod registry;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands, ConfigAction};
use config::HudoConfig;
use dialoguer::{Confirm, MultiSelect, Select, theme::ColorfulTheme};
use installer::{DetectResult, InstallContext, EnvAction, all_installers};

/// 确保配置已初始化（首次运行引导用户选择安装盘）
fn ensure_config() -> Result<HudoConfig> {
    if let Some(config) = HudoConfig::load()? {
        return Ok(config);
    }

    // 首次运行，引导用户选择安装目录
    println!();
    ui::print_title("欢迎使用 hudo！首次运行，请选择安装目录。");

    let drives = HudoConfig::scan_drives();
    if drives.is_empty() {
        anyhow::bail!("未检测到可用磁盘");
    }

    let items: Vec<String> = drives
        .iter()
        .map(|d| {
            if d.is_system {
                format!("  {}: (系统盘, 剩余 {}GB)", d.letter, d.free_gb)
            } else {
                format!("  {}: (剩余 {}GB)", d.letter, d.free_gb)
            }
        })
        .collect();

    let default = drives
        .iter()
        .position(|d| !d.is_system)
        .unwrap_or(0);

    println!("检测到以下磁盘：");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(default)
        .interact()
        .context("磁盘选择被取消")?;

    let chosen = &drives[selection];
    let root_dir = format!("{}:\\hudo", chosen.letter);

    let config = HudoConfig {
        root_dir: root_dir.clone(),
        java: Default::default(),
        go: Default::default(),
        mirrors: Default::default(),
    };

    config.save()?;
    config.ensure_dirs()?;
    ui::print_success(&format!("已创建 {}", root_dir));

    Ok(config)
}

/// 交互式多选安装
async fn cmd_setup(config: &HudoConfig) -> Result<()> {
    ui::print_title("hudo setup - 选择要安装的开发工具");

    let installers = all_installers();
    let ctx = InstallContext { config };

    // 检测各工具当前状态，构建选项列表
    let mut labels = Vec::new();
    let mut defaults = Vec::new();

    for inst in &installers {
        let info = inst.info();
        let status = match inst.detect_installed(&ctx).await {
            Ok(DetectResult::InstalledByHudo(ver)) => {
                format!(" {}", console::style(format!("[已安装 {}]", ver)).green())
            }
            Ok(DetectResult::InstalledExternal(ver)) => {
                format!(" {}", console::style(format!("[系统 {}]", ver)).yellow())
            }
            Ok(DetectResult::NotInstalled) => String::new(),
            Err(_) => format!(" {}", console::style("[检测失败]").red()),
        };

        labels.push(format!("{:<8} — {}{}", info.name, info.description, status));

        // 默认勾选未安装的工具
        defaults.push(matches!(
            inst.detect_installed(&ctx).await,
            Ok(DetectResult::NotInstalled)
        ));
    }

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("  用空格勾选，回车确认")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .context("选择被取消")?;

    if selections.is_empty() {
        ui::print_info("未选择任何工具，退出");
        return Ok(());
    }

    // 确认
    let selected_names: Vec<_> = selections.iter().map(|&i| installers[i].info().name).collect();
    println!();
    ui::print_info(&format!("即将安装: {}", selected_names.join(", ")));
    let confirm = Confirm::new()
        .with_prompt("  确认开始？")
        .default(true)
        .interact()
        .context("确认被取消")?;

    if !confirm {
        ui::print_info("已取消");
        return Ok(());
    }

    // 逐个安装
    let total = selections.len();
    for (idx, &sel) in selections.iter().enumerate() {
        let info = installers[sel].info();
        println!();
        ui::print_step((idx + 1) as u32, total as u32, &format!("安装 {}", info.name));
        if let Err(e) = cmd_install(config, info.id).await {
            ui::print_error(&format!("{} 安装失败: {}", info.name, e));
            let cont = Confirm::new()
                .with_prompt("  是否继续安装其余工具？")
                .default(true)
                .interact()
                .unwrap_or(false);
            if !cont {
                anyhow::bail!("用户中止安装");
            }
        }
    }

    println!();
    ui::print_title("安装完成！");
    ui::print_info("请打开新终端以使环境变量生效");
    Ok(())
}

/// 安装单个工具
async fn cmd_install(config: &HudoConfig, tool_id: &str) -> Result<()> {
    let installers = all_installers();

    let available: Vec<_> = installers.iter().map(|i| i.info().id).collect();
    let inst = installers
        .iter()
        .find(|i| i.info().id == tool_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "未知工具 '{}'，可用: {}",
                tool_id,
                available.join(", ")
            )
        })?;

    let info = inst.info();
    ui::print_title(&format!("安装 {}", info.name));

    let ctx = InstallContext { config };

    // 检测是否已安装
    let detect = inst.detect_installed(&ctx).await?;
    match &detect {
        DetectResult::InstalledByHudo(version) => {
            ui::print_success(&format!("{} 已安装 (hudo): {}", info.name, version));
            inst.configure(&ctx).await?;
            return Ok(());
        }
        DetectResult::InstalledExternal(version) => {
            ui::print_warning(&format!("{} 已安装在系统其他位置: {}", info.name, version));
            let reinstall = Confirm::new()
                .with_prompt("  是否由 hudo 接管？（将清理旧版并重新安装到 hudo 目录）")
                .default(false)
                .interact()
                .context("选择被取消")?;
            if !reinstall {
                ui::print_info("跳过安装，使用现有版本");
                inst.configure(&ctx).await?;
                return Ok(());
            }
            ui::print_step(1, 2, "卸载旧版...");
            uninstall_from_system(info.id)?;
        }
        DetectResult::NotInstalled => {}
    }

    // 执行安装
    let result = inst.install(&ctx).await?;
    ui::print_success(&format!("{} 安装完成 ({})", info.name, result.version));

    // 配置环境变量
    let actions = inst.env_actions(&result.install_path, config);
    for action in &actions {
        match action {
            EnvAction::Set { name, value } => {
                env::EnvManager::set_var(name, value)?;
                ui::print_success(&format!("设置 {} = {}", name, value));
            }
            EnvAction::AppendPath { path } => {
                env::EnvManager::append_to_path(path)?;
                ui::print_success(&format!("PATH += {}", path));
            }
        }
    }

    if !actions.is_empty() {
        env::EnvManager::broadcast_change();
        ui::print_info("环境变量已更新，新终端生效");
    }

    // 交互式配置
    inst.configure(&ctx).await?;

    // 保存安装状态
    let mut reg = registry::InstallRegistry::load(&config.state_path())?;
    reg.mark_installed(
        info.id,
        &result.version,
        &result.install_path.to_string_lossy(),
    );
    reg.save(&config.state_path())?;

    Ok(())
}

/// 卸载系统中已有的工具
fn uninstall_from_system(tool_id: &str) -> Result<()> {
    match tool_id {
        "git" => uninstall_via_registry("Git_is1"),
        "uv" => uninstall_uv(),
        _ => anyhow::bail!("不支持自动卸载: {}", tool_id),
    }
}

/// 通过注册表查找并运行系统卸载程序（如 Git）
fn uninstall_via_registry(uninstall_key: &str) -> Result<()> {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let path = format!(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
        uninstall_key
    );

    let uninstall_string: String = hklm
        .open_subkey(&path)
        .and_then(|key| key.get_value("UninstallString"))
        .or_else(|_| {
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            hkcu.open_subkey(&path)
                .and_then(|key| key.get_value("UninstallString"))
        })
        .context("未找到卸载程序，请手动卸载后重试")?;

    let uninstall_string = uninstall_string.trim_matches('"').to_string();

    let status = std::process::Command::new(&uninstall_string)
        .args(["/VERYSILENT", "/NORESTART"])
        .status()
        .with_context(|| format!("运行卸载程序失败: {}", uninstall_string))?;

    if !status.success() {
        anyhow::bail!("卸载程序退出码: {}", status.code().unwrap_or(-1));
    }

    ui::print_success("旧版已卸载");
    Ok(())
}

/// 卸载系统中已有的 uv（绿色安装，无注册表卸载器）
fn uninstall_uv() -> Result<()> {
    // 找到旧 uv 的位置
    let output = std::process::Command::new("where")
        .arg("uv")
        .output()
        .context("查找 uv 位置失败")?;

    if !output.status.success() {
        ui::print_warning("未找到旧版 uv，跳过卸载");
        return Ok(());
    }

    let uv_path = String::from_utf8_lossy(&output.stdout);
    let uv_path = uv_path.lines().next().unwrap_or("").trim();
    let old_dir = std::path::Path::new(uv_path)
        .parent()
        .context("无法确定 uv 所在目录")?;

    // 1. 清理缓存
    ui::print_info("清理 uv 缓存...");
    std::process::Command::new(uv_path)
        .args(["cache", "clean"])
        .status()
        .ok();

    // 2. 删除旧二进制文件
    for bin in &["uv.exe", "uvx.exe", "uvw.exe"] {
        let p = old_dir.join(bin);
        if p.exists() {
            std::fs::remove_file(&p).ok();
        }
    }

    // 3. 从 PATH 移除旧目录
    env::EnvManager::remove_from_path(&old_dir.to_string_lossy())?;

    // 4. 清理 receipt 文件
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let receipt = std::path::Path::new(&local).join("uv").join("uv-receipt.json");
        if receipt.exists() {
            std::fs::remove_file(&receipt).ok();
        }
    }

    env::EnvManager::broadcast_change();
    ui::print_success("旧版 uv 已清理");
    Ok(())
}

/// 列出所有工具状态
async fn cmd_list(config: &HudoConfig) -> Result<()> {
    ui::print_title("开发工具状态");

    let installers = all_installers();
    let ctx = InstallContext { config };
    let reg = registry::InstallRegistry::load(&config.state_path())?;

    for inst in &installers {
        let info = inst.info();
        let status = match inst.detect_installed(&ctx).await {
            Ok(DetectResult::InstalledByHudo(ver)) => {
                let extra = reg.get(info.id)
                    .map(|s| format!("  安装于 {}", console::style(&s.installed_at).dim()))
                    .unwrap_or_default();
                format!("{}{}", console::style(ver).green(), extra)
            }
            Ok(DetectResult::InstalledExternal(ver)) => {
                format!("{} {}", console::style(ver).green(), console::style("(非 hudo 管理)").yellow())
            }
            Ok(DetectResult::NotInstalled) => console::style("未安装").dim().to_string(),
            Err(_) => console::style("检测失败").red().to_string(),
        };
        println!("  {:<8} {:<24} {}", info.name, info.description, status);
    }

    println!();
    ui::print_info(&format!("安装根目录: {}", config.root_dir));
    Ok(())
}

fn cmd_config_show(config: &HudoConfig) -> Result<()> {
    ui::print_title("当前配置");
    let content = toml::to_string_pretty(config).context("序列化配置失败")?;
    println!("{}", content);
    Ok(())
}

fn cmd_config_set(config: &mut HudoConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "root_dir" => config.root_dir = value.to_string(),
        "java.version" => config.java.version = value.to_string(),
        "go.version" => config.go.version = value.to_string(),
        "mirrors.uv" => config.mirrors.uv = Some(value.to_string()),
        "mirrors.fnm" => config.mirrors.fnm = Some(value.to_string()),
        "mirrors.go" => config.mirrors.go = Some(value.to_string()),
        "mirrors.java" => config.mirrors.java = Some(value.to_string()),
        "mirrors.vscode" => config.mirrors.vscode = Some(value.to_string()),
        "mirrors.pycharm" => config.mirrors.pycharm = Some(value.to_string()),
        _ => anyhow::bail!("未知配置项: {}。可用: root_dir, java.version, go.version, mirrors.*", key),
    }
    config.save()?;
    ui::print_success(&format!("已设置 {} = {}", key, value));
    Ok(())
}

fn cmd_config_edit() -> Result<()> {
    let path = HudoConfig::config_path()?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
    std::process::Command::new(&editor)
        .arg(path.to_str().unwrap())
        .status()
        .with_context(|| format!("无法启动编辑器: {}", editor))?;
    Ok(())
}

fn cmd_config_reset() -> Result<()> {
    let path = HudoConfig::config_path()?;
    if path.exists() {
        std::fs::remove_file(&path).context("无法删除配置文件")?;
        ui::print_success("配置已重置，下次运行将重新引导");
    } else {
        ui::print_info("配置文件不存在，无需重置");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            let config = ensure_config()?;
            cmd_setup(&config).await?;
        }
        Commands::Install { tool } => {
            let config = ensure_config()?;
            cmd_install(&config, &tool.to_lowercase()).await?;
        }
        Commands::List => {
            let config = ensure_config()?;
            cmd_list(&config).await?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let config = ensure_config()?;
                cmd_config_show(&config)?;
            }
            ConfigAction::Set { key, value } => {
                let mut config = ensure_config()?;
                cmd_config_set(&mut config, &key, &value)?;
            }
            ConfigAction::Edit => {
                cmd_config_edit()?;
            }
            ConfigAction::Reset => {
                cmd_config_reset()?;
            }
        },
    }

    Ok(())
}
