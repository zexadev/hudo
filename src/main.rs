mod cli;
mod config;
mod download;
mod env;
mod installer;
mod profile;
mod registry;
mod ui;
mod version;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands, ConfigAction};
use config::HudoConfig;
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use installer::{DetectResult, InstallContext, EnvAction, all_installers};

/// 确保配置已初始化（首次运行引导用户选择安装盘）
fn ensure_config() -> Result<HudoConfig> {
    if let Some(config) = HudoConfig::load()? {
        return Ok(config);
    }

    // 首次运行，引导用户选择安装目录
    ui::print_banner();
    ui::print_title("首次运行 — 选择安装目录");
    println!("  {}", console::style("所有开发工具将安装到所选磁盘的 hudo 目录下").dim());

    let drives = HudoConfig::scan_drives();
    if drives.is_empty() {
        anyhow::bail!("未检测到可用磁盘");
    }

    let items: Vec<String> = drives
        .iter()
        .map(|d| {
            if d.is_system {
                format!(
                    "{}:  {}  {}",
                    d.letter,
                    ui::pad(&format!("{}GB 可用", d.free_gb), 12),
                    console::style("(系统盘)").dim()
                )
            } else {
                format!("{}:  {}GB 可用", d.letter, d.free_gb)
            }
        })
        .collect();

    let default = drives
        .iter()
        .position(|d| !d.is_system)
        .unwrap_or(0);

    println!();
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
        versions: Default::default(),
        mirrors: Default::default(),
    };

    config.save()?;
    config.ensure_dirs()?;
    ui::print_success(&format!("已创建 {}", root_dir));

    Ok(config)
}

/// 交互式多选安装（两级：先选分类，再选工具）
async fn cmd_setup(config: &HudoConfig) -> Result<()> {
    let installers = all_installers();
    let categories = [
        ui::ToolCategory::Tool,
        ui::ToolCategory::Language,
        ui::ToolCategory::Database,
        ui::ToolCategory::Ide,
    ];

    loop {
        ui::page_header("选择工具分类");

        // 构建分类菜单项，显示每个分类的工具数量
        let cat_labels: Vec<String> = categories
            .iter()
            .map(|cat| {
                let count = installers
                    .iter()
                    .filter(|i| {
                        std::mem::discriminant(&ui::ToolCategory::from_id(i.info().id))
                            == std::mem::discriminant(cat)
                    })
                    .count();
                let icon = cat.icon();
                format!("{}  {}  {}", icon, ui::pad(cat.label(), 14), console::style(format!("{} 个工具", count)).dim())
            })
            .collect();

        let cat_sel = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择分类 (Esc 返回)")
            .items(&cat_labels)
            .default(0)
            .interact_opt()
            .context("选择被取消")?;

        let cat_idx = match cat_sel {
            Some(i) => i,
            None => break,
        };

        // 筛选该分类下的工具
        let cat = &categories[cat_idx];
        let cat_tools: Vec<usize> = installers
            .iter()
            .enumerate()
            .filter(|(_, i)| {
                std::mem::discriminant(&ui::ToolCategory::from_id(i.info().id))
                    == std::mem::discriminant(cat)
            })
            .map(|(idx, _)| idx)
            .collect();

        // 进入分类内的工具多选
        setup_category(config, &installers, &cat_tools, cat.label()).await?;
    }

    Ok(())
}

/// 分类内的工具多选安装
async fn setup_category(
    config: &HudoConfig,
    installers: &[Box<dyn installer::Installer>],
    tool_indices: &[usize],
    cat_name: &str,
) -> Result<()> {
    ui::page_header(&format!("{} — 选择要安装的工具", cat_name));

    let reg = registry::InstallRegistry::load(&config.state_path())?;

    // 并行检测该分类下所有工具的安装状态
    let tool_refs: Vec<&dyn installer::Installer> =
        tool_indices.iter().map(|&i| installers[i].as_ref()).collect();
    let tool_data = detect_all_parallel(&tool_refs, config, &reg);

    // 计算动态列宽
    let mut name_width = 0usize;
    let mut desc_width = 0usize;
    for (info, _) in &tool_data {
        name_width = name_width.max(console::measure_text_width(info.name));
        desc_width = desc_width.max(console::measure_text_width(info.description));
    }

    // 加 2 列间距
    name_width += 2;
    desc_width += 2;

    // 第二轮：构建标签
    let mut labels = Vec::new();
    let mut defaults = Vec::new();

    for (info, detect) in &tool_data {
        let status = match detect {
            Ok(DetectResult::InstalledByHudo(ver)) => {
                let short = truncate_version(ver, 16);
                format!("{}", console::style(format!("✓ hudo {}", short)).green())
            }
            Ok(DetectResult::InstalledExternal(ver)) => {
                let short = truncate_version(ver, 16);
                format!("{}", console::style(format!("● 系统 {}", short)).yellow())
            }
            Ok(DetectResult::NotInstalled) => String::new(),
            Err(_) => format!("{}", console::style("✗ 检测失败").red()),
        };

        labels.push(format!(
            "{}  {}  {}",
            console::style(ui::pad(info.name, name_width)).bold(),
            ui::pad(info.description, desc_width),
            status
        ));
        defaults.push(false);
    }

    println!("  {}", console::style("空格勾选/取消，回车确认，Esc 返回").dim());
    println!();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&labels)
        .defaults(&defaults)
        .interact_opt()
        .context("选择被取消")?;

    let selections = match selections {
        Some(s) => s,
        None => {
            ui::print_info("已取消");
            return Ok(());
        }
    };

    if selections.is_empty() {
        ui::print_info("未选择任何工具");
        return Ok(());
    }

    // 确认
    let selected_names: Vec<_> = selections
        .iter()
        .map(|&i| installers[tool_indices[i]].info().name)
        .collect();
    println!();
    println!(
        "  即将安装 {} 个工具: {}",
        console::style(selected_names.len()).cyan().bold(),
        selected_names.join(", ")
    );
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
    let mut success_count = 0u32;
    let mut fail_names = Vec::new();

    for (idx, &sel) in selections.iter().enumerate() {
        let info = installers[tool_indices[sel]].info();
        println!();
        ui::print_step(
            (idx + 1) as u32,
            total as u32,
            &format!("安装 {}", info.name),
        );
        if let Err(e) = cmd_install(config, info.id).await {
            ui::print_error(&format!("{} 安装失败: {}", info.name, e));
            fail_names.push(info.name);
            let cont = Confirm::new()
                .with_prompt("  是否继续安装其余工具？")
                .default(true)
                .interact()
                .unwrap_or(false);
            if !cont {
                anyhow::bail!("用户中止安装");
            }
        } else {
            success_count += 1;
        }
    }

    // 汇总
    println!();
    println!("{}", console::style("─".repeat(40)).cyan());
    if fail_names.is_empty() {
        ui::print_success(&format!("全部 {} 个工具安装完成", success_count));
    } else {
        ui::print_success(&format!("{} 个工具安装成功", success_count));
        ui::print_warning(&format!(
            "{} 个工具安装失败: {}",
            fail_names.len(),
            fail_names.join(", ")
        ));
    }
    ui::print_info("请打开新终端以使环境变量生效");
    ui::wait_for_key();
    Ok(())
}

/// 安装单个工具
async fn cmd_install(config: &HudoConfig, tool_id: &str) -> Result<()> {
    cmd_install_inner(config, tool_id, false).await
}

/// 安装单个工具（内部实现，skip_configure 控制是否跳过交互式配置）
async fn cmd_install_inner(config: &HudoConfig, tool_id: &str, skip_configure: bool) -> Result<()> {
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
            if !skip_configure {
                inst.configure(&ctx).await?;
            }
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
                if !skip_configure {
                    inst.configure(&ctx).await?;
                }
                return Ok(());
            }
            ui::print_step(1, 2, "卸载旧版...");
            uninstall_from_system(info.id)?;
        }
        DetectResult::NotInstalled => {}
    }

    // 执行安装
    let result = inst.install(&ctx).await?;
    ui::print_success(&format!(
        "{} {} 安装完成",
        info.name,
        console::style(&result.version).green()
    ));

    // 配置环境变量
    let actions = inst.env_actions(&result.install_path, config);
    if !actions.is_empty() {
        for action in &actions {
            match action {
                EnvAction::Set { name, value } => {
                    env::EnvManager::set_var(name, value)?;
                    ui::print_info(&format!("{} = {}", name, value));
                }
                EnvAction::AppendPath { path } => {
                    env::EnvManager::append_to_path(path)?;
                    ui::print_info(&format!("PATH += {}", path));
                }
            }
        }
        env::EnvManager::broadcast_change();
    }

    // 保存安装状态（在 configure 之前，确保安装失败不影响已安装记录）
    let mut reg = registry::InstallRegistry::load(&config.state_path())?;
    reg.mark_installed(
        info.id,
        &result.version,
        &result.install_path.to_string_lossy(),
    );
    reg.save(&config.state_path())?;

    // 交互式配置
    if !skip_configure {
        inst.configure(&ctx).await?;
    }

    Ok(())
}

/// 卸载 hudo 管理的工具
async fn cmd_uninstall(config: &HudoConfig, tool_id: &str) -> Result<()> {
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
    let ctx = InstallContext { config };

    // 检测是否由 hudo 安装
    let detect = inst.detect_installed(&ctx).await?;
    match &detect {
        DetectResult::InstalledByHudo(ver) => {
            ui::print_title(&format!("卸载 {} ({})", info.name, ver));
        }
        _ => {
            ui::print_warning(&format!("{} 未由 hudo 安装，无需卸载", info.name));
            return Ok(());
        }
    }

    let confirm = Confirm::new()
        .with_prompt(format!("  确认卸载 {}？（将删除安装目录并清理环境变量）", info.name))
        .default(false)
        .interact()
        .context("选择被取消")?;

    if !confirm {
        ui::print_info("已取消");
        return Ok(());
    }

    // 获取安装路径（从 env_actions 推断或从 registry 读取）
    let reg = registry::InstallRegistry::load(&config.state_path())?;
    let install_path = reg
        .get(info.id)
        .map(|s| std::path::PathBuf::from(&s.install_path))
        .unwrap_or_else(|| {
            // 回退：根据工具类型推断默认路径
            match info.id {
                "vscode" | "pycharm" => config.ide_dir().join(info.id),
                "go" | "jdk" => config.lang_dir().join(match info.id {
                    "jdk" => "java",
                    other => other,
                }),
                "rust" => config.lang_dir().join("cargo"),
                _ => config.tools_dir().join(info.id),
            }
        });

    // 1. 卸载前清理（停止服务等）
    inst.pre_uninstall(&ctx).await?;

    // 2. 清理环境变量
    let actions = inst.env_actions(&install_path, config);
    for action in &actions {
        match action {
            EnvAction::Set { name, .. } => {
                if env::EnvManager::get_var(name)?.is_some() {
                    env::EnvManager::delete_var(name)?;
                    ui::print_info(&format!("移除环境变量: {}", name));
                }
            }
            EnvAction::AppendPath { path } => {
                env::EnvManager::remove_from_path(path)?;
                ui::print_info(&format!("PATH -= {}", path));
            }
        }
    }

    // 3. Rust 特殊处理：同时删除 rustup 目录
    if info.id == "rust" {
        let rustup_home = config.tools_dir().join("rustup");
        if rustup_home.exists() {
            std::fs::remove_dir_all(&rustup_home).ok();
            ui::print_info(&format!("已删除 {}", rustup_home.display()));
        }
    }

    // 3. 删除安装目录
    if install_path.exists() {
        std::fs::remove_dir_all(&install_path)
            .with_context(|| format!("删除目录失败: {}", install_path.display()))?;
        ui::print_info(&format!("已删除 {}", install_path.display()));
    }

    // 4. 更新 state.json
    let mut reg = registry::InstallRegistry::load(&config.state_path())?;
    reg.remove(info.id);
    reg.save(&config.state_path())?;

    if !actions.is_empty() {
        env::EnvManager::broadcast_change();
    }

    ui::print_success(&format!("{} 已卸载", info.name));
    ui::print_info("请打开新终端以使环境变量生效");
    Ok(())
}

/// 卸载系统中已有的工具
fn uninstall_from_system(tool_id: &str) -> Result<()> {
    match tool_id {
        "git" => uninstall_via_registry("Git_is1"),
        "uv" => uninstall_uv(),
        "rust" => uninstall_rust(),
        "go" => uninstall_go(),
        "miniconda" => uninstall_miniconda(),
        "vscode" => uninstall_vscode(),
        // 绿色安装的工具：通过 where 找到旧二进制，移除 PATH
        "nodejs" => uninstall_green(&["fnm", "node"], &["FNM_DIR"]),
        "bun" => uninstall_green(&["bun"], &[]),
        "jdk" => uninstall_green(&["java"], &["JAVA_HOME"]),
        "c" => uninstall_green(&["gcc"], &[]),
        "mysql" => uninstall_green(&["mysql"], &[]),
        "pgsql" => uninstall_green(&["psql"], &[]),
        "pycharm" => uninstall_green(&["pycharm64"], &[]),
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

/// 通用卸载：通过 where 找到旧二进制，从 PATH 移除其所在目录，并清理指定环境变量
fn uninstall_green(binaries: &[&str], env_vars: &[&str]) -> Result<()> {
    for bin in binaries {
        let bin_name = format!("{}.exe", bin);
        if let Ok(output) = std::process::Command::new("where").arg(&bin_name).output() {
            if output.status.success() {
                let paths = String::from_utf8_lossy(&output.stdout);
                for line in paths.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(parent) = std::path::Path::new(line).parent() {
                        let dir_str = parent.to_string_lossy();
                        ui::print_info(&format!("移除 PATH: {}", dir_str));
                        env::EnvManager::remove_from_path(&dir_str)?;
                    }
                }
            }
        }
    }

    for var in env_vars {
        if env::EnvManager::get_var(var)?.is_some() {
            ui::print_info(&format!("移除环境变量: {}", var));
            env::EnvManager::delete_var(var)?;
        }
    }

    env::EnvManager::broadcast_change();
    ui::print_success("旧版已清理");
    Ok(())
}

/// 卸载系统中的 Rust（通过 rustup self uninstall）
fn uninstall_rust() -> Result<()> {
    // 先尝试 rustup self uninstall
    if let Ok(output) = std::process::Command::new("where").arg("rustup").output() {
        if output.status.success() {
            ui::print_info("执行 rustup self uninstall...");
            let status = std::process::Command::new("rustup")
                .args(["self", "uninstall", "-y"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    // 清理 PATH 和环境变量
                    for var in &["RUSTUP_HOME", "CARGO_HOME"] {
                        if env::EnvManager::get_var(var)?.is_some() {
                            env::EnvManager::delete_var(var)?;
                        }
                    }
                    env::EnvManager::broadcast_change();
                    ui::print_success("旧版 Rust 已卸载");
                    return Ok(());
                }
                _ => {
                    ui::print_warning("rustup self uninstall 失败，尝试手动清理 PATH");
                }
            }
        }
    }

    // 回退：手动清理 PATH
    uninstall_green(&["rustc", "cargo"], &["RUSTUP_HOME", "CARGO_HOME"])
}

/// 卸载系统中的 Go（可能是 MSI 安装或绿色安装）
fn uninstall_go() -> Result<()> {
    // 先尝试注册表卸载器（Go 官方 MSI 的注册表键名可能有变化）
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let uninstall_path = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
    if let Ok(uninstall_key) = hklm.open_subkey(uninstall_path) {
        for name in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(sub) = uninstall_key.open_subkey(&name) {
                let display: std::result::Result<String, _> = sub.get_value("DisplayName");
                if let Ok(display) = display {
                    if display.contains("Go Programming Language") {
                        if let Ok(cmd) = sub.get_value::<String, _>("UninstallString") {
                            ui::print_info(&format!("找到 Go MSI 卸载器: {}", cmd));
                            let cmd = cmd.trim_matches('"').to_string();
                            // MSI 静默卸载
                            let status = std::process::Command::new("msiexec")
                                .args(["/x", &cmd, "/qn", "/norestart"])
                                .status();
                            if let Ok(s) = status {
                                if s.success() {
                                    for var in &["GOROOT", "GOPATH"] {
                                        if env::EnvManager::get_var(var)?.is_some() {
                                            env::EnvManager::delete_var(var)?;
                                        }
                                    }
                                    env::EnvManager::broadcast_change();
                                    ui::print_success("旧版 Go (MSI) 已卸载");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 回退：绿色安装方式清理
    uninstall_green(&["go"], &["GOROOT", "GOPATH"])
}

/// 卸载系统中的 Miniconda
fn uninstall_miniconda() -> Result<()> {
    // 找到 conda 位置
    if let Ok(output) = std::process::Command::new("where").arg("conda").output() {
        if output.status.success() {
            let conda_path = String::from_utf8_lossy(&output.stdout);
            let conda_path = conda_path.lines().next().unwrap_or("").trim();
            if !conda_path.is_empty() {
                // conda 在 Scripts/conda.exe，安装目录是上两级
                let conda_dir = std::path::Path::new(conda_path)
                    .parent()  // Scripts/
                    .and_then(|p| p.parent());  // install root

                if let Some(install_root) = conda_dir {
                    let uninstaller = install_root.join("Uninstall-Miniconda3.exe");
                    if uninstaller.exists() {
                        ui::print_info("执行 Miniconda 卸载程序...");
                        let status = std::process::Command::new(&uninstaller)
                            .args(["/S"])
                            .status();
                        if let Ok(s) = status {
                            if s.success() {
                                env::EnvManager::broadcast_change();
                                ui::print_success("旧版 Miniconda 已卸载");
                                return Ok(());
                            }
                        }
                        ui::print_warning("Miniconda 卸载程序失败，尝试手动清理 PATH");
                    }
                }
            }
        }
    }

    uninstall_green(&["conda"], &[])
}

/// 卸载系统中的 VS Code
fn uninstall_vscode() -> Result<()> {
    // 检查注册表中的 VS Code 卸载器（用户安装或系统安装）
    for (hive, hive_name) in &[
        (winreg::enums::HKEY_CURRENT_USER, "HKCU"),
        (winreg::enums::HKEY_LOCAL_MACHINE, "HKLM"),
    ] {
        let root = winreg::RegKey::predef(*hive);
        let uninstall_path = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
        if let Ok(uninstall_key) = root.open_subkey(uninstall_path) {
            for name in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(sub) = uninstall_key.open_subkey(&name) {
                    let display: std::result::Result<String, _> = sub.get_value("DisplayName");
                    if let Ok(display) = display {
                        if display.contains("Visual Studio Code") || display.contains("VS Code") {
                            if let Ok(cmd) = sub.get_value::<String, _>("UninstallString") {
                                ui::print_info(&format!("找到 VS Code 卸载器 ({}): {}", hive_name, cmd));
                                let cmd = cmd.trim_matches('"').to_string();
                                let status = std::process::Command::new(&cmd)
                                    .args(["/VERYSILENT", "/NORESTART"])
                                    .status();
                                if let Ok(s) = status {
                                    if s.success() {
                                        env::EnvManager::broadcast_change();
                                        ui::print_success("旧版 VS Code 已卸载");
                                        return Ok(());
                                    }
                                }
                                ui::print_warning("VS Code 卸载程序失败，尝试手动清理 PATH");
                            }
                        }
                    }
                }
            }
        }
    }

    // 回退：绿色安装方式清理（portable 模式 code.cmd 在 PATH 里）
    uninstall_green(&["code"], &[])
}

/// 导出 profile
async fn cmd_export(config: &HudoConfig, file: Option<String>) -> Result<()> {
    let output_path = file.unwrap_or_else(|| "hudo-profile.toml".to_string());
    let output_path = std::path::Path::new(&output_path);

    ui::print_title("导出环境档案");

    let installers = all_installers();
    let profile = profile::HudoProfile::build_from_current(config, &installers).await?;

    if profile.tools.is_empty() {
        ui::print_warning("未检测到任何已安装工具，无需导出");
        return Ok(());
    }

    // 展示摘要
    ui::print_info(&format!("检测到 {} 个已安装工具:", profile.tools.len()));
    for (id, ver) in &profile.tools {
        println!(
            "    {}  {}",
            console::style(ui::pad(id, 14)).bold(),
            console::style(ver).dim()
        );
    }
    if !profile.tool_config.is_empty() {
        println!();
        ui::print_info(&format!("包含 {} 个工具的配置", profile.tool_config.len()));
    }

    println!();
    let confirm = Confirm::new()
        .with_prompt(format!("  导出到 {} ?", output_path.display()))
        .default(true)
        .interact_opt()
        .context("确认被取消")?;

    if confirm != Some(true) {
        ui::print_info("已取消");
        return Ok(());
    }

    profile.save_to_file(output_path)?;
    ui::print_success(&format!("环境档案已导出到 {}", output_path.display()));

    Ok(())
}

/// 导入 profile 并安装工具
async fn cmd_import(config: &mut HudoConfig, file: &str) -> Result<()> {
    let file_path = std::path::Path::new(file);
    if !file_path.exists() {
        anyhow::bail!("文件不存在: {}", file);
    }

    ui::print_title("导入环境档案");

    let prof = profile::HudoProfile::load_from_file(file_path)?;
    ui::print_info(&format!(
        "档案版本: {}  导出时间: {}",
        prof.hudo.version, prof.hudo.exported_at
    ));

    // 应用 settings
    let mut settings_changed = false;
    if let Some(ref jv) = prof.settings.java_version {
        if config.java.version != *jv {
            config.java.version = jv.clone();
            ui::print_info(&format!("java.version = {}", jv));
            settings_changed = true;
        }
    }
    if let Some(ref gv) = prof.settings.go_version {
        if config.go.version != *gv {
            config.go.version = gv.clone();
            ui::print_info(&format!("go.version = {}", gv));
            settings_changed = true;
        }
    }
    // 应用 mirrors
    for (key, value) in &prof.settings.mirrors {
        match key.as_str() {
            "uv" => config.mirrors.uv = Some(value.clone()),
            "fnm" => config.mirrors.fnm = Some(value.clone()),
            "go" => config.mirrors.go = Some(value.clone()),
            "java" => config.mirrors.java = Some(value.clone()),
            "vscode" => config.mirrors.vscode = Some(value.clone()),
            "pycharm" => config.mirrors.pycharm = Some(value.clone()),
            _ => {}
        }
        ui::print_info(&format!("mirrors.{} = {}", key, value));
        settings_changed = true;
    }
    if settings_changed {
        config.save()?;
        ui::print_success("配置已更新");
        println!();
    }

    if prof.tools.is_empty() {
        ui::print_info("档案中没有工具需要安装");
        return Ok(());
    }

    // 检测已安装工具，筛选出需要安装的
    let installers = all_installers();
    let ctx = InstallContext { config };
    let mut to_install = Vec::new();

    for (tool_id, _ver) in &prof.tools {
        if let Some(inst) = installers.iter().find(|i| i.info().id == tool_id.as_str()) {
            match inst.detect_installed(&ctx).await {
                Ok(DetectResult::InstalledByHudo(ver)) => {
                    ui::print_info(&format!(
                        "{} 已安装 (hudo): {} — 跳过",
                        inst.info().name,
                        ver
                    ));
                }
                Ok(DetectResult::InstalledExternal(ver)) => {
                    ui::print_info(&format!(
                        "{} 已安装 (系统): {} — 跳过",
                        inst.info().name,
                        ver
                    ));
                }
                _ => {
                    to_install.push(inst.info());
                }
            }
        }
    }

    if to_install.is_empty() {
        ui::print_success("所有工具已安装，无需操作");
    } else {
        println!();
        ui::print_info(&format!("需要安装 {} 个工具:", to_install.len()));
        for info in &to_install {
            println!("    {}  {}", console::style(info.name).bold(), info.description);
        }

        println!();
        let confirm = Confirm::new()
            .with_prompt("  确认开始安装？")
            .default(true)
            .interact_opt()
            .context("确认被取消")?;

        if confirm != Some(true) {
            ui::print_info("已取消");
            return Ok(());
        }

        // 批量安装（skip_configure=true）
        let total = to_install.len();
        let mut success_count = 0u32;
        let mut fail_names = Vec::new();

        for (idx, info) in to_install.iter().enumerate() {
            println!();
            ui::print_step(
                (idx + 1) as u32,
                total as u32,
                &format!("安装 {}", info.name),
            );
            if let Err(e) = cmd_install_inner(config, info.id, false).await {
                ui::print_error(&format!("{} 安装失败: {}", info.name, e));
                fail_names.push(info.name);
                let cont = Confirm::new()
                    .with_prompt("  是否继续安装其余工具？")
                    .default(true)
                    .interact()
                    .unwrap_or(false);
                if !cont {
                    anyhow::bail!("用户中止安装");
                }
            } else {
                success_count += 1;
            }
        }

        println!();
        println!("{}", console::style("─".repeat(40)).cyan());
        if fail_names.is_empty() {
            ui::print_success(&format!("全部 {} 个工具安装完成", success_count));
        } else {
            ui::print_success(&format!("{} 个工具安装成功", success_count));
            ui::print_warning(&format!(
                "{} 个工具安装失败: {}",
                fail_names.len(),
                fail_names.join(", ")
            ));
        }
    }

    // 应用 tool_config
    if !prof.tool_config.is_empty() {
        println!();
        apply_tool_configs(config, &installers, &prof).await?;
    }

    ui::print_info("请打开新终端以使环境变量生效");
    ui::wait_for_key();
    Ok(())
}

/// 遍历 profile 中的 tool_config，调用各安装器的 import_config
async fn apply_tool_configs(
    config: &HudoConfig,
    installers: &[Box<dyn installer::Installer>],
    prof: &profile::HudoProfile,
) -> Result<()> {
    let ctx = InstallContext { config };
    for (tool_id, entries) in &prof.tool_config {
        if let Some(inst) = installers.iter().find(|i| i.info().id == tool_id.as_str()) {
            let pairs: Vec<(String, String)> = entries
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            if !pairs.is_empty() {
                ui::print_info(&format!("应用 {} 配置...", inst.info().name));
                inst.import_config(&ctx, &pairs).await?;
            }
        }
    }
    ui::print_success("工具配置已应用");
    Ok(())
}

/// 快速检测：从 state.json 读取版本，仅做路径存在检查，无需子进程
fn fast_detect(id: &str, reg: &registry::InstallRegistry) -> Option<DetectResult> {
    let state = reg.get(id)?;
    let path = std::path::Path::new(&state.install_path);
    if path.exists() {
        Some(DetectResult::InstalledByHudo(state.version.clone()))
    } else {
        None
    }
}

/// 并行检测工具安装状态：
/// - hudo 工具：读 state.json，无子进程，近乎瞬间
/// - 外部工具：并行在独立线程中运行子进程检测
fn detect_all_parallel(
    tools: &[&dyn installer::Installer],
    config: &HudoConfig,
    reg: &registry::InstallRegistry,
) -> Vec<(installer::ToolInfo, Result<DetectResult>)> {
    // 第一步：state.json 快速检测
    let mut results: Vec<Option<Result<DetectResult>>> = tools
        .iter()
        .map(|inst| fast_detect(inst.info().id, reg).map(Ok))
        .collect();

    // 找出需要子进程检测的工具（不在 state.json 中的）
    let pending: Vec<usize> = results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| if r.is_none() { Some(i) } else { None })
        .collect();

    if !pending.is_empty() {
        // 获取当前 tokio runtime 句柄，供非 tokio 线程使用
        let handle = tokio::runtime::Handle::current();
        std::thread::scope(|s| {
            // 并行启动所有子进程检测
            let handles: Vec<(usize, _)> = pending
                .iter()
                .map(|&i| {
                    let inst = tools[i];
                    let handle = handle.clone();
                    let config = config;
                    (
                        i,
                        s.spawn(move || {
                            let ctx = InstallContext { config };
                            handle.block_on(inst.detect_installed(&ctx))
                        }),
                    )
                })
                .collect();

            // 等待所有线程完成（已并行执行）
            for (i, h) in handles {
                results[i] = Some(
                    h.join()
                        .unwrap_or_else(|_| Err(anyhow::anyhow!("检测线程崩溃"))),
                );
            }
        });
    }

    tools
        .iter()
        .zip(results.into_iter())
        .map(|(inst, r)| (inst.info(), r.unwrap_or(Ok(DetectResult::NotInstalled))))
        .collect()
}

/// 列出所有工具状态
async fn cmd_list(config: &HudoConfig, show_all: bool) -> Result<()> {
    ui::print_title(if show_all { "所有可用工具" } else { "已安装工具" });

    let installers = all_installers();
    let reg = registry::InstallRegistry::load(&config.state_path())?;

    // 按分类分组
    let categories = [
        ui::ToolCategory::Tool,
        ui::ToolCategory::Language,
        ui::ToolCategory::Database,
        ui::ToolCategory::Ide,
    ];

    // 收集所有工具的检测结果（并行）
    let tool_refs: Vec<&dyn installer::Installer> =
        installers.iter().map(|i| i.as_ref()).collect();
    let all_results = detect_all_parallel(&tool_refs, config, &reg);

    // 计算已安装工具的动态列宽（仅基于要显示的工具）
    let mut name_width = 0usize;
    let mut desc_width = 0usize;
    for (info, detect) in &all_results {
        let is_installed = matches!(detect, Ok(DetectResult::InstalledByHudo(_)) | Ok(DetectResult::InstalledExternal(_)));
        if show_all || is_installed {
            name_width = name_width.max(console::measure_text_width(info.name));
            desc_width = desc_width.max(console::measure_text_width(info.description));
        }
    }
    name_width += 2;
    desc_width += 2;

    let mut hudo_count = 0u32;
    let mut external_count = 0u32;
    let mut any_displayed = false;

    for cat in &categories {
        // 筛选该分类下要显示的工具
        let cat_entries: Vec<_> = all_results
            .iter()
            .filter(|(info, detect)| {
                let in_cat = std::mem::discriminant(&ui::ToolCategory::from_id(info.id))
                    == std::mem::discriminant(cat);
                if !in_cat {
                    return false;
                }
                if show_all {
                    return true;
                }
                matches!(detect, Ok(DetectResult::InstalledByHudo(_)) | Ok(DetectResult::InstalledExternal(_)))
            })
            .collect();

        if cat_entries.is_empty() {
            continue;
        }

        ui::print_section(cat.label());
        any_displayed = true;

        for (info, detect) in &cat_entries {
            let status = match detect {
                Ok(DetectResult::InstalledByHudo(ver)) => {
                    hudo_count += 1;
                    let extra = reg
                        .get(info.id)
                        .map(|s| {
                            format!("  {}", console::style(format!("({})", s.installed_at)).dim())
                        })
                        .unwrap_or_default();
                    format!("{}{}", console::style(ver).green(), extra)
                }
                Ok(DetectResult::InstalledExternal(ver)) => {
                    external_count += 1;
                    format!(
                        "{} {}",
                        console::style(ver).green(),
                        console::style("(非 hudo)").yellow()
                    )
                }
                Ok(DetectResult::NotInstalled) => {
                    console::style("·").dim().to_string()
                }
                Err(_) => console::style("检测失败").red().to_string(),
            };
            println!(
                "    {}  {}  {}",
                console::style(ui::pad(info.name, name_width)).bold(),
                ui::pad(info.description, desc_width),
                status,
            );
        }
    }

    if !any_displayed {
        ui::print_info("尚未安装任何工具，运行 hudo setup 开始安装");
    }

    println!();
    let total = hudo_count + external_count;
    if total > 0 {
        ui::print_info(&format!(
            "共 {} 个工具已安装 (hudo: {}, 系统: {})",
            total, hudo_count, external_count
        ));
    }
    if !show_all && total > 0 {
        ui::print_info("使用 hudo list --all 查看所有可用工具");
    }
    ui::print_info(&format!("安装根目录: {}", config.root_dir));
    Ok(())
}

fn cmd_config_show(config: &HudoConfig) -> Result<()> {
    ui::print_title("当前配置");

    println!("  {}  {}", ui::pad("root_dir", 20), config.root_dir);
    println!("  {}  {}", ui::pad("java.version", 20), config.java.version);
    println!("  {}  {}", ui::pad("go.version", 20), config.go.version);

    let versions = [
        ("versions.git", &config.versions.git),
        ("versions.fnm", &config.versions.fnm),
        ("versions.mysql", &config.versions.mysql),
        ("versions.pgsql", &config.versions.pgsql),
        ("versions.pycharm", &config.versions.pycharm),
    ];
    let has_versions = versions.iter().any(|(_, v)| v.is_some());
    if has_versions {
        println!();
        for (key, val) in &versions {
            if let Some(v) = val {
                println!("  {}  {}", ui::pad(key, 20), v);
            }
        }
    }

    let mirrors = [
        ("mirrors.uv", &config.mirrors.uv),
        ("mirrors.fnm", &config.mirrors.fnm),
        ("mirrors.go", &config.mirrors.go),
        ("mirrors.java", &config.mirrors.java),
        ("mirrors.vscode", &config.mirrors.vscode),
        ("mirrors.pycharm", &config.mirrors.pycharm),
    ];
    let has_mirrors = mirrors.iter().any(|(_, v)| v.is_some());
    if has_mirrors {
        println!();
        for (key, val) in &mirrors {
            if let Some(v) = val {
                println!("  {}  {}", ui::pad(key, 20), v);
            }
        }
    }
    Ok(())
}

fn cmd_config_set(config: &mut HudoConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "root_dir" => config.root_dir = value.to_string(),
        "java.version" => config.java.version = value.to_string(),
        "go.version" => config.go.version = value.to_string(),
        "versions.git" => config.versions.git = Some(value.to_string()),
        "versions.fnm" => config.versions.fnm = Some(value.to_string()),
        "versions.mysql" => config.versions.mysql = Some(value.to_string()),
        "versions.pgsql" => config.versions.pgsql = Some(value.to_string()),
        "versions.pycharm" => config.versions.pycharm = Some(value.to_string()),
        "mirrors.uv" => config.mirrors.uv = Some(value.to_string()),
        "mirrors.fnm" => config.mirrors.fnm = Some(value.to_string()),
        "mirrors.go" => config.mirrors.go = Some(value.to_string()),
        "mirrors.java" => config.mirrors.java = Some(value.to_string()),
        "mirrors.vscode" => config.mirrors.vscode = Some(value.to_string()),
        "mirrors.pycharm" => config.mirrors.pycharm = Some(value.to_string()),
        _ => anyhow::bail!("未知配置项: {}。可用: root_dir, java.version, go.version, versions.*, mirrors.*", key),
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

/// 截断版本号字符串，保留关键部分（如 "git version 2.47.1.windows.2" → "2.47.1"）
fn truncate_version(ver: &str, max_len: usize) -> String {
    // 尝试提取纯版本号（数字.数字 开头的部分）
    let trimmed = ver.trim();
    let version_part = trimmed
        .split_whitespace()
        .find(|s| s.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap_or(trimmed);
    if version_part.len() <= max_len {
        version_part.to_string()
    } else {
        format!("{}…", &version_part[..max_len - 1])
    }
}

/// 交互式主菜单
async fn interactive_menu(config: &HudoConfig) -> Result<()> {
    loop {
        ui::page_header("主菜单");

        let menu_items = &[
            "📦  安装工具",
            "📋  查看已安装",
            "🗑   卸载工具",
            "📁  环境档案",
            "⚙   配置",
            "🚪  退出",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作 (Esc 退出)")
            .items(menu_items)
            .default(0)
            .interact_opt()
            .context("选择被取消")?;

        match selection {
            Some(0) => { cmd_setup(config).await?; }
            Some(1) => { cmd_list(config, false).await?; ui::wait_for_key(); }
            Some(2) => { interactive_uninstall(config).await?; }
            Some(3) => { interactive_profile(config).await?; }
            Some(4) => { interactive_config(config).await?; }
            Some(5) | None => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// 交互式卸载：列出已安装工具供用户选择
async fn interactive_uninstall(config: &HudoConfig) -> Result<()> {
    ui::page_header("卸载工具");

    let installers = all_installers();
    let reg = registry::InstallRegistry::load(&config.state_path())?;

    let refs: Vec<&dyn installer::Installer> = installers.iter().map(|b| b.as_ref()).collect();
    let results = detect_all_parallel(&refs, config, &reg);

    let mut installed = Vec::new();
    for (info, result) in &results {
        if let Ok(DetectResult::InstalledByHudo(ver)) = result {
            installed.push((info.id, info.name, ver.clone()));
        }
    }

    if installed.is_empty() {
        ui::print_info("当前没有由 hudo 安装的工具");
        ui::wait_for_key();
        return Ok(());
    }

    let labels: Vec<String> = installed
        .iter()
        .map(|(_, name, ver)| {
            format!(
                "{}  {}",
                ui::pad(name, 14),
                console::style(ver).dim()
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要卸载的工具 (Esc 返回)")
        .items(&labels)
        .interact_opt()
        .context("选择被取消")?;

    match selection {
        Some(idx) => {
            let (tool_id, _, _) = &installed[idx];
            cmd_uninstall(config, tool_id).await?;
            ui::wait_for_key();
        }
        None => {}
    }

    Ok(())
}

/// 交互式环境档案子菜单（导出 / 导入）
async fn interactive_profile(config: &HudoConfig) -> Result<()> {
    loop {
        ui::page_header("环境档案");

        let menu_items = &[
            "📤  导出环境档案",
            "📥  导入环境档案",
            "↩   返回",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("环境档案 (Esc 返回)")
            .items(menu_items)
            .default(0)
            .interact_opt()
            .context("选择被取消")?;

        match selection {
            Some(0) => {
                cmd_export(config, None).await?;
                ui::wait_for_key();
            }
            Some(1) => {
                let mut config = config.clone();
                cmd_import(&mut config, "hudo-profile.toml").await?;
                ui::wait_for_key();
            }
            Some(2) | None => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// 交互式配置子菜单
async fn interactive_config(config: &HudoConfig) -> Result<()> {
    loop {
        ui::page_header("配置管理");

        let menu_items = &[
            "📄  查看配置",
            "🌐  设置镜像",
            "📝  编辑配置文件",
            "🔄  重置配置",
            "↩   返回",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("配置管理 (Esc 返回)")
            .items(menu_items)
            .default(0)
            .interact_opt()
            .context("选择被取消")?;

        match selection {
            Some(0) => {
                cmd_config_show(config)?;
                ui::wait_for_key();
            }
            Some(1) => {
                let mirror_keys = &[
                    "mirrors.uv",
                    "mirrors.fnm",
                    "mirrors.go",
                    "mirrors.java",
                    "mirrors.vscode",
                    "mirrors.pycharm",
                ];

                let key_sel = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("选择要设置的镜像")
                    .items(mirror_keys)
                    .interact_opt()
                    .context("选择被取消")?;

                if let Some(idx) = key_sel {
                    let value: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("输入 {} 的值", mirror_keys[idx]))
                        .interact_text()
                        .context("输入被取消")?;

                    let mut config = config.clone();
                    cmd_config_set(&mut config, mirror_keys[idx], &value)?;
                }
                ui::wait_for_key();
            }
            Some(2) => cmd_config_edit()?,
            Some(3) => { cmd_config_reset()?; ui::wait_for_key(); }
            Some(4) | None => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Setup => {
                let config = ensure_config()?;
                cmd_setup(&config).await?;
            }
            Commands::Install { tool } => {
                let config = ensure_config()?;
                cmd_install(&config, &tool.to_lowercase()).await?;
            }
            Commands::Uninstall { tool } => {
                let config = ensure_config()?;
                cmd_uninstall(&config, &tool.to_lowercase()).await?;
            }
            Commands::Export { file } => {
                let config = ensure_config()?;
                cmd_export(&config, file).await?;
            }
            Commands::Import { file } => {
                let mut config = ensure_config()?;
                cmd_import(&mut config, &file).await?;
            }
            Commands::List { all } => {
                let config = ensure_config()?;
                cmd_list(&config, all).await?;
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
        },
        None => {
            let config = ensure_config()?;
            interactive_menu(&config).await?;
        }
    }

    Ok(())
}
