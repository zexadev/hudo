mod cc;
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
use dialoguer::{MultiSelect, Select, theme::ColorfulTheme};
use installer::{DetectResult, InstallContext, EnvAction, all_installers};

/// 确保配置已初始化（首次运行引导用户选择安装盘）
fn ensure_config() -> Result<HudoConfig> {
    if let Some(config) = HudoConfig::load()? {
        return Ok(config);
    }

    // 首次运行，引导用户选择安装目录
    ui::print_banner();
    ui::print_title("首次运行 — 选择安装目录");

    let root_dir = {
        #[cfg(windows)]
        { ensure_config_windows()? }
        #[cfg(not(windows))]
        { ensure_config_unix()? }
    };

    let config = HudoConfig {
        root_dir: root_dir.clone(),
        proxy: None,
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

/// 只读命令使用：配置不存在时给提示，不触发首次运行的选盘向导（不在磁盘上留痕）
fn load_config_readonly() -> Result<Option<HudoConfig>> {
    let config = HudoConfig::load()?;
    if config.is_none() {
        ui::print_info("hudo 尚未初始化，运行 hudo setup 开始安装");
    }
    Ok(config)
}

/// Windows：扫描盘符让用户选择
#[cfg(windows)]
fn ensure_config_windows() -> Result<String> {
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
    let selection = if ui::assume_yes() {
        // 非交互：自动选默认盘（首个非系统盘，无则 C），支撑 install.ps1 + 档案的无人值守整机还原
        ui::print_info(&format!("非交互模式，自动选择安装盘: {}:", drives[default].letter));
        default
    } else {
        Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(default)
            .interact()
            .context("磁盘选择被取消")?
    };

    let chosen = &drives[selection];
    let mut root_dir = format!("{}:\\hudo", chosen.letter);

    // C 盘根目录普通用户无写权限，自动回退到用户目录
    if chosen.is_system {
        if let Err(e) = std::fs::create_dir_all(&root_dir) {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                let profile = std::env::var("USERPROFILE")
                    .unwrap_or_else(|_| "C:\\Users\\Default".to_string());
                let fallback = format!("{}\\hudo", profile);
                ui::print_warning(&format!(
                    "C:\\ 根目录需要管理员权限，已自动切换到: {}",
                    fallback
                ));
                root_dir = fallback;
            }
        }
    }

    Ok(root_dir)
}

/// Windows stub（Unix 编译时不需要）
#[cfg(not(windows))]
fn ensure_config_windows() -> Result<String> {
    unreachable!()
}

/// Unix：默认 ~/hudo，允许用户自定义
#[cfg(not(windows))]
fn ensure_config_unix() -> Result<String> {
    use dialoguer::Input;

    let default_dir = HudoConfig::default_root_dir()
        .unwrap_or_else(|_| "/opt/hudo".to_string());
    println!("  {}", console::style(format!("默认安装目录: {}", default_dir)).dim());

    println!();
    let root_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("安装目录")
        .default(default_dir)
        .interact_text()
        .context("目录输入被取消")?;

    Ok(root_dir)
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
            .unwrap_or(None);

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

    // 并行检测该分类下所有工具的安装状态（检测期间显示 spinner，避免界面像卡死）
    let tool_refs: Vec<&dyn installer::Installer> =
        tool_indices.iter().map(|&i| installers[i].as_ref()).collect();
    let sp = ui::spinner("正在检测已安装工具...");
    let tool_data = detect_all_parallel(&tool_refs, config, &reg);
    sp.finish_and_clear();

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
    let mut has_external = false;

    for (info, detect) in &tool_data {
        let status = match detect {
            Ok(DetectResult::InstalledByHudo(ver)) => {
                let short = truncate_version(ver, 16);
                format!("{}", console::style(format!("✓ hudo {}", short)).green())
            }
            Ok(DetectResult::InstalledExternal(ver)) => {
                has_external = true;
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

    println!("  {}", console::style("空格勾选，a 全选/取消全选，回车确认，Esc 返回").dim());
    if has_external {
        println!("  {}", console::style("● 系统 = 已有系统安装，勾选后可选择由 hudo 接管").dim());
    }
    println!();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&labels)
        .defaults(&defaults)
        .interact_opt()
        .unwrap_or(None);

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
    let confirm = ui::confirm_proceed("确认开始？", true)?;

    if !confirm {
        ui::print_info("已取消");
        return Ok(());
    }

    // 逐个安装（中途失败可选择中止，中止走正常汇总而非报错退出）
    let total = selections.len();
    let mut success_count = 0u32;
    let mut fail_names = Vec::new();
    let mut aborted = false;

    for (idx, &sel) in selections.iter().enumerate() {
        let info = installers[tool_indices[sel]].info();
        println!();
        ui::print_step(
            (idx + 1) as u32,
            total as u32,
            &format!("安装 {}", info.name),
        );
        if let Err(e) = cmd_install(config, info.id).await {
            ui::print_error(&format!("{} 安装失败: {:#}", info.name, e));
            fail_names.push(info.name);
            if !ui::confirm_proceed("是否继续安装其余工具？", true).unwrap_or(false) {
                aborted = true;
                break;
            }
        } else {
            success_count += 1;
        }
    }

    // 汇总（boxed 面板）
    let mut summary_lines = Vec::new();
    if fail_names.is_empty() && !aborted {
        summary_lines.push(format!(" {} {} 个工具安装完成", console::style("✓").green().bold(), success_count));
    } else {
        summary_lines.push(format!(" {} {} 个工具安装成功", console::style("✓").green().bold(), success_count));
        if !fail_names.is_empty() {
            summary_lines.push(format!(" {} {} 个失败: {}", console::style("✗").red().bold(), fail_names.len(), fail_names.join(", ")));
        }
        let remaining = total.saturating_sub(success_count as usize + fail_names.len());
        if aborted && remaining > 0 {
            summary_lines.push(format!(" 已中止，{} 个工具未安装", remaining));
        }
    }
    summary_lines.push(String::new());
    summary_lines.push(format!(" {}", console::style("请打开新终端以使环境变量生效").dim()));
    ui::print_box(&summary_lines);
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
            if !skip_configure
                && ui::confirm(&format!("是否重新运行 {} 的配置？", info.name), false)?
            {
                run_configure(inst.as_ref(), &ctx, &info).await;
            }
            return Ok(());
        }
        DetectResult::InstalledExternal(version) => {
            ui::print_warning(&format!("{} 已安装在系统其他位置: {}", info.name, version));
            let reinstall = ui::confirm("是否由 hudo 接管？（将清理旧版并重新安装到 hudo 目录）", false)?;
            if !reinstall {
                // 外部安装不做 hudo 配置：configure 按 hudo 目录写路径，对外部安装是错的
                ui::print_info("跳过安装，使用现有版本");
                return Ok(());
            }
            ui::print_step(1, 2, "卸载旧版...");
            #[cfg(windows)]
            uninstall_from_system(info.id)?;
            #[cfg(not(windows))]
            anyhow::bail!("该工具已安装在系统其他位置，请手动卸载后重试");
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

    // 交互式配置（失败降级为警告：此时安装与记录均已完成，不算安装失败）
    if !skip_configure {
        run_configure(inst.as_ref(), &ctx, &info).await;
    }

    if !actions.is_empty() {
        ui::print_next_step("请打开新终端以使环境变量生效");
    }

    Ok(())
}

/// 不支持自动升级的工具及替代指引
fn upgrade_unsupported_reason(id: &str) -> Option<&'static str> {
    match id {
        // 数据目录/环境在安装目录内，重装即清空，绝不能自动覆盖
        "mysql" | "pgsql" | "redis" => {
            Some("数据目录在安装目录内，自动重装会丢数据；请备份数据后手动 uninstall + install")
        }
        "miniconda" => Some("conda 环境保存在安装目录内；请运行 conda update conda"),
        "rust" => Some("由 rustup 自管理；请运行 rustup update"),
        "chrome" => Some("Chrome 自带自动更新"),
        _ => None,
    }
}

/// 配置锁定优先，未锁定时才发起最新版查询（async fn 未被 await 前不会执行）
async fn lock_or<F>(lock: &Option<String>, latest: F) -> Option<String>
where
    F: std::future::Future<Output = Option<String>>,
{
    match lock {
        Some(v) => Some(v.clone()),
        None => latest.await,
    }
}

/// 解析升级目标版本；None = 查询失败
async fn resolve_upgrade_target(config: &HudoConfig, id: &str) -> Option<String> {
    let v = &config.versions;
    match id {
        "git" => lock_or(&v.git, version::git_latest()).await,
        "gh" => lock_or(&v.gh, version::gh_latest()).await,
        "nodejs" => lock_or(&v.nodejs, version::nodejs_lts_latest()).await,
        "fnm" => lock_or(&v.fnm, version::fnm_latest()).await,
        "bun" => lock_or(&v.bun, version::bun_latest()).await,
        "uv" => lock_or(&v.uv, version::uv_latest()).await,
        "maven" => lock_or(&v.maven, version::maven_latest()).await,
        "gradle" => lock_or(&v.gradle, version::gradle_latest()).await,
        "vscode" => lock_or(&v.vscode, version::vscode_latest()).await,
        "pycharm" => lock_or(&v.pycharm, version::pycharm_latest()).await,
        "idea" => match &v.idea {
            Some(x) => Some(x.clone()),
            None => version::idea_latest().await.map(|(ver, _)| ver),
        },
        "claude-code" => lock_or(&v.claude_code, version::claude_code_latest()).await,
        "go" => match config.go.version.as_str() {
            "latest" | "" => version::go_latest().await,
            s => Some(s.to_string()),
        },
        "jdk" => {
            let major = match config.java.version.as_str() {
                "" => "21",
                m => m,
            };
            version::jdk_latest(major).await
        }
        "c" => version::mingw_latest().await.map(|(_, _, gcc)| gcc),
        "7zip" => lock_or(&v.sevenzip, version::sevenzip_latest()).await,
        "pwsh" => lock_or(&v.pwsh, version::pwsh_latest()).await,
        "dotnet" => lock_or(&v.dotnet, version::dotnet_latest()).await,
        "powertoys" => lock_or(&v.powertoys, version::powertoys_latest()).await,
        "omp" => lock_or(&v.omp, version::omp_latest()).await,
        _ => None,
    }
}

/// 从任意版本输出中提取可比较的版本号：
/// "git version 2.47.1" → "2.47.1"; "v24.14.1" → "24.14.1";
/// "openjdk version \"21.0.11\" 2026-04-15" → "21.0.11"; "go1.24.0" → "1.24.0"（比较用，前缀无关紧要）;
/// "21.0.11+10.0.LTS" → "21.0.11"
fn normalize_version(raw: &str) -> String {
    for token in raw.split_whitespace() {
        let t = token.trim_matches(|c: char| c == '"' || c == '\'' || c == ',');
        let stripped = t.trim_start_matches(|c: char| !c.is_ascii_digit());
        // 要求含 '.'：排除 "2026-04-15" 日期与构建哈希等数字开头的非版本 token
        if !stripped.is_empty() && stripped.contains('.') {
            return stripped.split('+').next().unwrap_or(stripped).to_string();
        }
    }
    raw.trim().to_string()
}

/// 升级 hudo 安装的工具：state.json 记录版本 vs 目标版本（锁定 > API 最新），
/// 不重跑 configure、不改环境变量（安装路径固定不变）
async fn cmd_upgrade(config: &HudoConfig, tool: Option<String>) -> Result<()> {
    ui::print_title("升级工具");

    let installers = all_installers();
    let reg = registry::InstallRegistry::load(&config.state_path())?;

    if let Some(ref t) = tool {
        if installers.iter().all(|i| i.info().id != t.as_str()) {
            anyhow::bail!("未知工具: {}", t);
        }
        if !reg.tools.contains_key(t.as_str()) {
            ui::print_warning(&format!("{} 未由 hudo 安装（无安装记录），如需安装: hudo install {}", t, t));
            return Ok(());
        }
        if let Some(reason) = upgrade_unsupported_reason(t) {
            ui::print_warning(&format!("{} 不支持自动升级: {}", t, reason));
            return Ok(());
        }
    }

    // 候选 = state.json 里记录的 hudo 安装工具（可选按参数过滤），排除不支持项
    let mut skipped: Vec<(&str, &str)> = Vec::new();
    let candidates: Vec<&dyn installer::Installer> = installers
        .iter()
        .map(|b| b.as_ref())
        .filter(|inst| {
            let id = inst.info().id;
            if !reg.tools.contains_key(id) {
                return false;
            }
            if let Some(ref t) = tool {
                if id != t.as_str() {
                    return false;
                }
            }
            if let Some(reason) = upgrade_unsupported_reason(id) {
                skipped.push((id, reason));
                return false;
            }
            true
        })
        .collect();

    if candidates.is_empty() && skipped.is_empty() {
        ui::print_info("尚无 hudo 安装的工具，运行 hudo setup 开始安装");
        return Ok(());
    }

    // 并发解析目标版本（每个查询各自带 5 秒超时）
    let sp = ui::spinner("查询最新版本...");
    let resolved = futures_util::future::join_all(
        candidates
            .iter()
            .map(|inst| resolve_upgrade_target(config, inst.info().id)),
    )
    .await;
    sp.finish_and_clear();

    let mut upgradable: Vec<(&dyn installer::Installer, String, String)> = Vec::new();
    let mut up_to_date = 0u32;
    for (inst, target) in candidates.iter().zip(resolved) {
        let info = inst.info();
        let current = reg
            .tools
            .get(info.id)
            .map(|s| s.version.clone())
            .unwrap_or_default();
        match target {
            None => ui::print_warning(&format!("{} 版本查询失败，跳过", info.name)),
            Some(t) => {
                if normalize_version(&current) == normalize_version(&t) {
                    up_to_date += 1;
                } else {
                    upgradable.push((*inst, current, t));
                }
            }
        }
    }

    for (id, reason) in &skipped {
        ui::print_info(&format!("跳过 {}: {}", id, reason));
    }

    if upgradable.is_empty() {
        ui::print_success(&format!("{} 个工具均已是目标版本", up_to_date));
        return Ok(());
    }

    println!();
    ui::print_info(&format!("{} 个工具可升级:", upgradable.len()));
    for (inst, current, target) in &upgradable {
        println!(
            "    {}  {} {} {}",
            console::style(ui::pad(inst.info().name, 12)).bold(),
            console::style(normalize_version(current)).dim(),
            console::style("→").cyan(),
            console::style(normalize_version(target)).green()
        );
    }
    println!();

    if !ui::confirm_proceed(&format!("升级这 {} 个工具？", upgradable.len()), true)? {
        ui::print_info("已取消");
        return Ok(());
    }

    let ctx = InstallContext { config };
    let total = upgradable.len();
    let mut success = 0u32;
    let mut fail_names: Vec<&str> = Vec::new();
    for (idx, (inst, _, target)) in upgradable.iter().enumerate() {
        let info = inst.info();
        println!();
        ui::print_step((idx + 1) as u32, total as u32, &format!("升级 {}", info.name));
        match inst.install(&ctx).await {
            Ok(result) => {
                let mut reg = registry::InstallRegistry::load(&config.state_path())?;
                reg.mark_installed(info.id, &result.version, &result.install_path.to_string_lossy());
                reg.save(&config.state_path())?;
                ui::print_success(&format!("{} 已升级到 {}", info.name, result.version));
                success += 1;
            }
            Err(e) => {
                ui::print_error(&format!("{} 升级到 {} 失败: {:#}", info.name, target, e));
                fail_names.push(info.name);
                if !ui::confirm_proceed("是否继续升级其余工具？", true).unwrap_or(false) {
                    break;
                }
            }
        }
    }

    println!();
    if fail_names.is_empty() {
        ui::print_success(&format!("{} 个工具升级完成", success));
    } else {
        ui::print_warning(&format!(
            "{} 个升级成功，{} 个失败: {}",
            success,
            fail_names.len(),
            fail_names.join(", ")
        ));
    }
    Ok(())
}

/// 运行工具的交互式配置；失败降级为警告（安装本身已完成并记录到 state.json）
async fn run_configure(
    inst: &dyn installer::Installer,
    ctx: &InstallContext<'_>,
    info: &installer::ToolInfo,
) {
    if let Err(e) = inst.configure(ctx).await {
        ui::print_warning(&format!("{} 已安装，但配置未完成: {:#}", info.name, e));
        ui::print_next_step(&format!("可运行 hudo install {} 重新配置", info.id));
    }
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

    let confirm = ui::confirm_proceed(
        &format!("确认卸载 {}？（将删除安装目录并清理环境变量）", info.name),
        false,
    )?;

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
    if matches!(info.id, "mysql" | "pgsql" | "redis") {
        ui::print_info("即将弹出 UAC 提权窗口以停止并移除服务，请在弹窗中选择\"是\"");
    }
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
            .with_context(|| format!("删除目录失败（若相关程序正在运行，请关闭后重试）: {}", install_path.display()))?;
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
    ui::print_next_step("请打开新终端以使环境变量生效");
    Ok(())
}

/// 卸载系统中已有的工具
#[cfg(windows)]
fn uninstall_from_system(tool_id: &str) -> Result<()> {
    match tool_id {
        "git" => uninstall_via_registry("Git_is1"),
        "uv" => uninstall_uv(),
        "rust" => uninstall_rust(),
        "go" => uninstall_go(),
        "miniconda" => uninstall_miniconda(),
        "vscode" => uninstall_vscode(),
        // 绿色安装的工具：通过 where 找到旧二进制，移除 PATH
        "nodejs" => uninstall_green(&["node"], &[]),
        "bun" => uninstall_green(&["bun"], &[]),
        "jdk" => uninstall_green(&["java"], &["JAVA_HOME"]),
        "c" => uninstall_green(&["gcc"], &[]),
        "mysql" => uninstall_green(&["mysql"], &[]),
        "pgsql" => uninstall_green(&["psql"], &[]),
        "pycharm" => uninstall_green(&["pycharm64"], &[]),
        "idea" => uninstall_green(&["idea64"], &[]),
        "claude-code" => uninstall_claude_code(),
        _ => anyhow::bail!("不支持自动卸载: {}", tool_id),
    }
}

/// 通过注册表查找并运行系统卸载程序（如 Git）
#[cfg(windows)]
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
#[cfg(windows)]
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

/// 卸载系统中已有的 Claude Code（npm 全局安装）
#[cfg(windows)]
fn uninstall_claude_code() -> Result<()> {
    // 尝试 npm uninstall
    let status = std::process::Command::new("cmd")
        .args(["/c", "npm", "uninstall", "-g", "@anthropic-ai/claude-code"])
        .status();

    match status {
        Ok(s) if s.success() => {
            ui::print_success("旧版 Claude Code (npm) 已卸载");
            return Ok(());
        }
        _ => {}
    }

    // npm 不可用或失败，尝试绿色方式清理
    uninstall_green(&["claude"], &[])
}

/// 通用卸载：通过 where 找到旧二进制，从 PATH 移除其所在目录，并清理指定环境变量
#[cfg(windows)]
fn uninstall_green(binaries: &[&str], env_vars: &[&str]) -> Result<()> {
    let mut old_dirs: Vec<String> = Vec::new();
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
                        let dir_str = parent.to_string_lossy().to_string();
                        ui::print_info(&format!("移除用户 PATH: {}", dir_str));
                        env::EnvManager::remove_from_path(&dir_str)?;
                        old_dirs.push(dir_str);
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

    // 用户级清理只覆盖 HKCU：MSI 全局安装的旧版在机器级 PATH 里，仍会优先生效
    if let Some(dir) = machine_path_contains(&old_dirs) {
        ui::print_warning(&format!("旧版位于机器级 PATH，无法以当前权限清理: {}", dir));
        ui::print_next_step("请通过系统\"应用和功能\"卸载旧版，否则新版本可能被旧版遮蔽");
    } else {
        ui::print_success("旧版已清理");
    }
    Ok(())
}

/// 检查机器级 PATH（HKLM）中是否仍包含给定目录，返回第一个命中的目录
#[cfg(windows)]
fn machine_path_contains(dirs: &[String]) -> Option<String> {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let env_key = hklm
        .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
        .ok()?;
    let machine_path: String = env_key.get_value("Path").ok()?;
    let entries: Vec<String> = machine_path
        .split(';')
        .map(|s| s.trim().trim_end_matches('\\').to_lowercase())
        .collect();
    dirs.iter()
        .find(|d| entries.contains(&d.trim_end_matches('\\').to_lowercase()))
        .cloned()
}

/// 卸载系统中的 Rust（通过 rustup self uninstall）
#[cfg(windows)]
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
#[cfg(windows)]
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
#[cfg(windows)]
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
#[cfg(windows)]
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
    // 展示绝对路径，避免用户找不到导出的文件
    let display_path = std::path::absolute(output_path)
        .unwrap_or_else(|_| output_path.to_path_buf());

    ui::print_title("导出环境档案");

    let installers = all_installers();
    let sp = ui::spinner("正在检测已安装工具...");
    let profile_result = profile::HudoProfile::build_from_current(config, &installers).await;
    sp.finish_and_clear();
    let profile = profile_result?;

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
    if !profile.cc_providers.is_empty() {
        println!();
        ui::print_warning("档案将包含 Claude Code API 密钥明文，请勿分享给他人或提交到仓库");
    }

    println!();
    if !ui::confirm_proceed(&format!("导出到 {} ?", display_path.display()), true)? {
        ui::print_info("已取消");
        return Ok(());
    }

    profile.save_to_file(output_path)?;
    ui::print_success(&format!("环境档案已导出到 {}", display_path.display()));

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

    // 收集档案中的配置变更（用户确认后才应用，取消不落盘）
    let mut setting_changes: Vec<(String, String)> = Vec::new();
    if let Some(ref jv) = prof.settings.java_version {
        if config.java.version != *jv {
            setting_changes.push(("java.version".to_string(), jv.clone()));
        }
    }
    if let Some(ref gv) = prof.settings.go_version {
        if config.go.version != *gv {
            setting_changes.push(("go.version".to_string(), gv.clone()));
        }
    }
    for (key, value) in &prof.settings.mirrors {
        if MIRROR_KEYS.contains(&key.as_str()) {
            setting_changes.push((format!("mirrors.{}", key), value.clone()));
        } else {
            ui::print_warning(&format!("跳过未识别配置项: mirrors.{}", key));
        }
    }
    for (key, value) in &prof.settings.versions {
        if VERSION_KEYS.contains(&key.as_str()) {
            setting_changes.push((format!("versions.{}", key), value.clone()));
        } else {
            ui::print_warning(&format!("跳过未识别配置项: versions.{}", key));
        }
    }
    if !setting_changes.is_empty() {
        println!();
        ui::print_info(&format!("档案包含 {} 项配置变更:", setting_changes.len()));
        for (key, value) in &setting_changes {
            println!(
                "    {}  {}",
                console::style(ui::pad(key, 20)).bold(),
                console::style(value).dim()
            );
        }
    }

    if prof.tools.is_empty()
        && setting_changes.is_empty()
        && prof.tool_config.is_empty()
        && prof.cc_providers.is_empty()
    {
        ui::print_info("档案中没有需要应用的内容");
        return Ok(());
    }

    // 检测已安装工具，筛选出需要安装的
    let installers = all_installers();
    let mut to_install = Vec::new();
    {
        let ctx = InstallContext { config: &*config };
        let sp = ui::spinner("正在检测已安装工具...");
        let mut skip_lines: Vec<String> = Vec::new();
        for (tool_id, _ver) in &prof.tools {
            if let Some(inst) = installers.iter().find(|i| i.info().id == tool_id.as_str()) {
                match inst.detect_installed(&ctx).await {
                    Ok(DetectResult::InstalledByHudo(ver)) => {
                        skip_lines.push(format!("{} 已安装 (hudo): {} — 跳过", inst.info().name, ver));
                    }
                    Ok(DetectResult::InstalledExternal(ver)) => {
                        skip_lines.push(format!(
                            "{} 已安装 (系统): {} — 跳过（如需由 hudo 接管，运行 hudo install {}）",
                            inst.info().name, ver, inst.info().id
                        ));
                    }
                    _ => {
                        to_install.push(inst.info());
                    }
                }
            }
        }
        sp.finish_and_clear();
        for line in &skip_lines {
            ui::print_info(line);
        }
    }

    if !to_install.is_empty() {
        println!();
        ui::print_info(&format!("需要安装 {} 个工具:", to_install.len()));
        for info in &to_install {
            println!("    {}  {}", console::style(info.name).bold(), info.description);
        }
    } else if !prof.tools.is_empty() {
        ui::print_success("档案中的工具均已安装");
    }

    // 统一确认：确认前不落盘任何配置，取消即无副作用
    println!();
    let prompt = if to_install.is_empty() {
        "确认应用档案中的配置？"
    } else if setting_changes.is_empty() {
        "确认开始安装？"
    } else {
        "确认应用配置并开始安装？"
    };
    if !ui::confirm_proceed(prompt, true)? {
        ui::print_info("已取消，未做任何修改");
        return Ok(());
    }

    if !setting_changes.is_empty() {
        for (key, value) in &setting_changes {
            apply_config_kv(config, key, value)?;
        }
        config.save()?;
        ui::print_success("配置已更新");
    }

    if !to_install.is_empty() {
        // 批量安装（中止走正常汇总，不以错误退出）
        let total = to_install.len();
        let mut success_count = 0u32;
        let mut fail_names = Vec::new();
        let mut aborted = false;

        for (idx, info) in to_install.iter().enumerate() {
            println!();
            ui::print_step(
                (idx + 1) as u32,
                total as u32,
                &format!("安装 {}", info.name),
            );
            if let Err(e) = cmd_install_inner(config, info.id, false).await {
                ui::print_error(&format!("{} 安装失败: {:#}", info.name, e));
                fail_names.push(info.name);
                if !ui::confirm_proceed("是否继续安装其余工具？", true).unwrap_or(false) {
                    aborted = true;
                    break;
                }
            } else {
                success_count += 1;
            }
        }

        println!();
        println!("{}", console::style("─".repeat(40)).cyan());
        if fail_names.is_empty() && !aborted {
            ui::print_success(&format!("全部 {} 个工具安装完成", success_count));
        } else {
            ui::print_success(&format!("{} 个工具安装成功", success_count));
            if !fail_names.is_empty() {
                ui::print_warning(&format!(
                    "{} 个工具安装失败: {}",
                    fail_names.len(),
                    fail_names.join(", ")
                ));
            }
            if aborted {
                ui::print_info("已按要求中止剩余安装");
            }
        }
    }

    // 应用 tool_config
    if !prof.tool_config.is_empty() {
        println!();
        apply_tool_configs(config, &installers, &prof).await?;
    }

    // 合并 cc_providers（按 name 去重，新的追加）
    if !prof.cc_providers.is_empty() {
        println!();
        let mut store = cc::CcProviders::load()?;
        let mut added = 0u32;
        for p in &prof.cc_providers {
            if !store.providers.iter().any(|e| e.name == p.name) {
                store.providers.push(p.clone());
                added += 1;
            }
        }
        store.save()?;
        ui::print_info(&format!(
            "Claude Code providers: {} 个已存在，新增 {} 个",
            prof.cc_providers.len() as u32 - added,
            added
        ));
    }

    ui::print_next_step("请打开新终端以使环境变量生效");
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

/// 卸载 hudo 自身
#[cfg(windows)]
async fn cmd_self_uninstall() -> Result<()> {
    ui::print_title("卸载 hudo");

    let confirmed = ui::confirm_proceed("确定要卸载 hudo 吗？", false)?;
    if !confirmed {
        println!("  已取消");
        return Ok(());
    }

    let del_config = ui::confirm("同时删除配置文件和缓存？", false).unwrap_or(false);

    let current_exe = std::env::current_exe().context("无法获取当前程序路径")?;
    let bin_dir = current_exe
        .parent()
        .context("无法获取安装目录")?;
    let hudo_home = bin_dir.parent();

    // 从 PATH 中移除 bin 目录
    let bin_str = bin_dir.to_string_lossy().to_string();
    env::EnvManager::remove_from_path(&bin_str).ok();
    env::EnvManager::broadcast_change();
    ui::print_success("已从 PATH 移除");

    // 构建后台清理命令
    let exe_str = current_exe.to_string_lossy().to_string();
    let mut ps_cmd = format!(
        "Start-Sleep -Milliseconds 500; Remove-Item -Force '{}' -ErrorAction SilentlyContinue",
        exe_str
    );
    if del_config {
        if let Some(home) = hudo_home {
            ps_cmd.push_str(&format!(
                "; Remove-Item -Recurse -Force '{}' -ErrorAction SilentlyContinue",
                home.to_string_lossy()
            ));
        }
    }

    // 脱离控制台启动后台清理
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x00000008;
    let _ = crate::installer::winps_command()
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
        .creation_flags(DETACHED_PROCESS)
        .spawn();

    ui::print_success("hudo 已卸载，重启终端后生效");
    Ok(())
}

/// 更新 hudo 到最新版本（自替换）
#[cfg(windows)]
async fn cmd_update() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    ui::print_action("检查最新版本...");
    let latest = match version::hudo_latest().await {
        Some(v) => v,
        None => {
            ui::print_error("无法获取版本信息，请检查网络连接");
            return Ok(());
        }
    };

    if latest == current {
        ui::print_success(&format!("已是最新版本 v{}", current));
        return Ok(());
    }

    println!(
        "  发现新版本: {} → {}",
        console::style(format!("v{}", current)).dim(),
        console::style(format!("v{}", latest)).cyan().bold()
    );

    // 下载新版本
    let url = format!(
        "https://github.com/{}/releases/download/v{}/hudo-x86_64-pc-windows-msvc.exe",
        version::GITHUB_REPO,
        latest
    );
    let tmp = std::env::temp_dir().join("hudo-new.exe");

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("  {spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("下载 hudo v{}...", latest));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let client = download::client_builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let resp = client
        .get(&url)
        .send()
        .await
        .context("下载请求失败")?;

    if !resp.status().is_success() {
        pb.finish_and_clear();
        ui::print_error(&format!("下载失败，HTTP 状态码: {}", resp.status()));
        return Ok(());
    }

    let bytes = resp.bytes().await.context("读取下载内容失败")?;

    pb.finish_and_clear();

    // 验证下载文件完整性：PE 可执行文件必须以 MZ 开头且大小合理
    if bytes.len() < 1_000_000 || bytes.get(0..2) != Some(&[0x4D, 0x5A]) {
        ui::print_error("下载的文件不是有效的可执行程序，可能网络异常，请稍后重试");
        return Ok(());
    }

    std::fs::write(&tmp, &bytes).context("写入临时文件失败")?;

    // 自替换：重命名当前 exe（Windows 允许对运行中的 exe 改名），再移入新文件
    let current_exe = std::env::current_exe().context("无法获取当前程序路径")?;
    let old_exe = current_exe.with_extension("exe.old");

    std::fs::rename(&current_exe, &old_exe)
        .context("重命名当前程序失败（请确认安装目录有写权限）")?;
    if let Err(e) = std::fs::rename(&tmp, &current_exe) {
        // 回滚：恢复原文件，避免留下损坏状态
        let _ = std::fs::rename(&old_exe, &current_exe);
        return Err(e).context("替换程序失败");
    }

    // 后台清理 .old 文件（完全脱离父控制台，避免 hudo 退出时关闭终端窗口）
    let old_str = old_exe.to_string_lossy().to_string();
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x00000008;
    let _ = crate::installer::winps_command()
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &format!(
                "Start-Sleep -Milliseconds 1000; Remove-Item -Force '{}' -ErrorAction SilentlyContinue",
                old_str
            ),
        ])
        .creation_flags(DETACHED_PROCESS)
        .spawn();

    ui::print_success(&format!("hudo 已更新到 v{}，重新打开终端后生效", latest));
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

/// 列出工具状态
/// show_all 显示全部工具（含未安装）；deep 做完整检测（可识别系统安装），否则仅读 state.json
async fn cmd_list(config: &HudoConfig, show_all: bool, deep: bool) -> Result<()> {
    ui::print_title(if show_all {
        "所有可用工具"
    } else if deep {
        "已安装工具"
    } else {
        "hudo 安装的工具"
    });

    let installers = all_installers();
    let reg = registry::InstallRegistry::load(&config.state_path())?;

    // 按分类分组
    let categories = [
        ui::ToolCategory::Tool,
        ui::ToolCategory::Language,
        ui::ToolCategory::Database,
        ui::ToolCategory::Ide,
    ];

    // 收集工具检测结果
    // 快速模式：仅读 state.json，毫秒级（CLI hudo list）
    // 完整模式：并行子进程检测，可识别系统安装（--all / 交互菜单）
    let all_results: Vec<(installer::ToolInfo, Result<DetectResult>)> = if show_all || deep {
        let tool_refs: Vec<&dyn installer::Installer> =
            installers.iter().map(|i| i.as_ref()).collect();
        let sp = ui::spinner("正在检测已安装工具...");
        let results = detect_all_parallel(&tool_refs, config, &reg);
        sp.finish_and_clear();
        results
    } else {
        installers
            .iter()
            .map(|inst| {
                let info = inst.info();
                let detect = fast_detect(info.id, &reg)
                    .map(Ok)
                    .unwrap_or(Ok(DetectResult::NotInstalled));
                (info, detect)
            })
            .collect()
    };

    // 点线连接的总宽度
    let dot_width = 50usize;

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
            let (version_text, extra) = match detect {
                Ok(DetectResult::InstalledByHudo(ver)) => {
                    hudo_count += 1;
                    let date = reg
                        .get(info.id)
                        .map(|s| format!("  {}", console::style(&s.installed_at).dim()))
                        .unwrap_or_default();
                    (console::style(ver).green().to_string(), date)
                }
                Ok(DetectResult::InstalledExternal(ver)) => {
                    external_count += 1;
                    (
                        console::style(ver).green().to_string(),
                        format!("  {}", console::style("(非 hudo)").yellow()),
                    )
                }
                Ok(DetectResult::NotInstalled) => {
                    (console::style("·").dim().to_string(), String::new())
                }
                Err(_) => (console::style("检测失败").red().to_string(), String::new()),
            };
            println!(
                "    {}{}",
                ui::dotfill(info.name, &version_text, dot_width),
                extra,
            );
        }
    }

    if !any_displayed {
        ui::print_info(if show_all || deep {
            "尚未安装任何工具，运行 hudo setup 开始安装"
        } else {
            "hudo 尚未安装任何工具，运行 hudo setup 开始安装"
        });
    }

    println!();
    let total = hudo_count + external_count;
    if total > 0 {
        if show_all || deep {
            ui::print_info(&format!(
                "共 {} 个工具已安装 (hudo: {}, 系统: {})",
                total, hudo_count, external_count
            ));
        } else {
            ui::print_info(&format!("共 {} 个工具由 hudo 安装", total));
        }
        if external_count > 0 {
            ui::print_info("(非 hudo) 为系统已有安装，运行 hudo install <工具> 可选择由 hudo 接管");
        }
    }
    if !show_all && !deep && total > 0 {
        ui::print_info("使用 hudo list --all 查看所有可用工具（含系统安装）");
    }
    ui::print_info(&format!("安装根目录: {}", config.root_dir));
    Ok(())
}

fn cmd_config_show(config: &HudoConfig) -> Result<()> {
    ui::print_title("当前配置");

    println!("  {}  {}", ui::pad("root_dir", 20), config.root_dir);
    if let Some(ref p) = config.proxy {
        println!("  {}  {}", ui::pad("proxy", 20), p);
    }
    println!("  {}  {}", ui::pad("java.version", 20), config.java.version);
    println!("  {}  {}", ui::pad("go.version", 20), config.go.version);

    // 遍历 KEYS 展示已设置项：键表来自 config 单一来源，加新键无需改这里
    let versions: Vec<(String, &String)> = config::VersionConfig::KEYS
        .iter()
        .filter_map(|&k| config.versions.get(k).map(|v| (format!("versions.{}", k), v)))
        .collect();
    if !versions.is_empty() {
        println!();
        for (key, val) in &versions {
            println!("  {}  {}", ui::pad(key, 20), val);
        }
    }

    let mirrors: Vec<(String, &String)> = config::MirrorConfig::KEYS
        .iter()
        .filter_map(|&k| config.mirrors.get(k).map(|v| (format!("mirrors.{}", k), v)))
        .collect();
    if !mirrors.is_empty() {
        println!();
        for (key, val) in &mirrors {
            println!("  {}  {}", ui::pad(key, 20), val);
        }
    }
    Ok(())
}

/// mirrors.* 支持的键（与 config::MirrorConfig 字段一一对应）
const MIRROR_KEYS: &[&str] = config::MirrorConfig::KEYS;

/// versions.* 支持的键（与 config::VersionConfig 字段一一对应）
const VERSION_KEYS: &[&str] = config::VersionConfig::KEYS;

/// 将 key=value 写入 config（不落盘、不打印，供 config set 与档案导入共用）
fn apply_config_kv(config: &mut HudoConfig, key: &str, value: &str) -> Result<()> {
    let val = Some(value.to_string());
    let handled = match key {
        "root_dir" => {
            config.root_dir = value.to_string();
            true
        }
        "java.version" => {
            config.java.version = value.to_string();
            true
        }
        "go.version" => {
            config.go.version = value.to_string();
            true
        }
        // 空值/off 视为清除代理（Option 键没有别的清除入口）
        "proxy" => {
            config.proxy = match value {
                "" | "off" | "none" => None,
                v => Some(v.to_string()),
            };
            true
        }
        k => {
            if let Some(m) = k.strip_prefix("mirrors.") {
                config.mirrors.set(m, val)
            } else if let Some(v) = k.strip_prefix("versions.") {
                config.versions.set(v, val)
            } else {
                false
            }
        }
    };
    if !handled {
        anyhow::bail!(
            "未知配置项: {}。可用: root_dir, proxy, java.version, go.version, mirrors.{{{}}}, versions.{{{}}}",
            key,
            MIRROR_KEYS.join("|"),
            VERSION_KEYS.join("|")
        );
    }
    Ok(())
}

fn cmd_config_set(config: &mut HudoConfig, key: &str, value: &str) -> Result<()> {
    // root_dir 特殊：不迁移已装工具，改后旧目录中的安装记录不再显示
    if key == "root_dir" && config.root_dir != value {
        let has_records = registry::InstallRegistry::load(&config.state_path())
            .map(|r| !r.tools.is_empty())
            .unwrap_or(false);
        if has_records {
            ui::print_warning("修改 root_dir 不会迁移已安装的工具，旧目录中的安装记录将不再显示");
            if !ui::confirm_proceed("确认修改？", false)? {
                ui::print_info("已取消");
                return Ok(());
            }
        }
    }
    apply_config_kv(config, key, value)?;
    config.save()?;
    ui::print_success(&format!("已设置 {} = {}", key, value));
    Ok(())
}

fn cmd_config_edit() -> Result<()> {
    let path = HudoConfig::config_path()?;
    if !path.exists() {
        ui::print_info("配置文件尚不存在，请先运行 hudo setup 完成初始化");
        return Ok(());
    }
    let default_editor = if cfg!(windows) { "notepad" } else { "vi" };
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| default_editor.to_string());
    ui::print_info(&format!("已用 {} 打开 {}，保存并关闭编辑器后继续", editor, path.display()));
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
/// 按字符边界截断，多字节字符不会 panic
fn truncate_version(ver: &str, max_len: usize) -> String {
    // 尝试提取纯版本号（数字.数字 开头的部分）
    let trimmed = ver.trim();
    let version_part = trimmed
        .split_whitespace()
        .find(|s| s.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap_or(trimmed);
    if version_part.chars().count() <= max_len {
        return version_part.to_string();
    }
    let cut: String = version_part.chars().take(max_len.saturating_sub(1)).collect();
    format!("{}…", cut)
}

/// 交互式主菜单（子流程出错只展示错误并回到菜单，不退出程序）
async fn interactive_menu(config: &mut HudoConfig) -> Result<()> {
    loop {
        ui::page_header("主菜单");

        let menu_items = &[
            "[+] 安装工具",
            "[^] 升级工具",
            "[=] 查看已安装",
            "[-] 卸载工具",
            "[>] 环境档案",
            "[*] 配置",
            "[K] Claude Code API 来源",
            "[Q] 退出",
        ];

        let selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作 (Esc 退出)")
            .items(menu_items)
            .default(0)
            .interact_opt()
        {
            Ok(sel) => sel,
            Err(_) => break, // 终端 IO 异常按退出处理
        };

        let result = match selection {
            Some(0) => cmd_setup(config).await,
            Some(1) => {
                ui::page_header("升级工具");
                let r = cmd_upgrade(config, None).await;
                ui::wait_for_key();
                r
            }
            Some(2) => {
                let r = cmd_list(config, false, true).await;
                ui::wait_for_key();
                r
            }
            Some(3) => interactive_uninstall(config).await,
            Some(4) => interactive_profile(config).await,
            Some(5) => interactive_config(config).await,
            Some(6) => cc::cmd_cc(),
            Some(7) | None => break,
            _ => unreachable!(),
        };

        // 错误边界：子流程失败不退出程序，展示后回到菜单
        if let Err(e) = result {
            ui::print_error(&format!("{:#}", e));
            ui::wait_for_key();
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
    let sp = ui::spinner("正在检测已安装工具...");
    let results = detect_all_parallel(&refs, config, &reg);
    sp.finish_and_clear();

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
        .unwrap_or(None);

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
async fn interactive_profile(config: &mut HudoConfig) -> Result<()> {
    loop {
        ui::page_header("环境档案");

        let menu_items = &[
            "[>] 导出环境档案",
            "[<] 导入环境档案",
            "[B] 返回",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("环境档案 (Esc 返回)")
            .items(menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => {
                cmd_export(config, None).await?;
                ui::wait_for_key();
            }
            Some(1) => {
                let path = ui::input_text("档案文件路径", Some("hudo-profile.toml"), false)?;
                if !std::path::Path::new(&path).exists() {
                    ui::print_error(&format!("文件不存在: {}", path));
                    ui::wait_for_key();
                    continue;
                }
                cmd_import(config, &path).await?;
                ui::wait_for_key();
            }
            Some(2) | None => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// 交互式配置子菜单
async fn interactive_config(config: &mut HudoConfig) -> Result<()> {
    loop {
        ui::page_header("配置管理");

        let menu_items = &[
            "[=] 查看配置",
            "[M] 设置镜像",
            "[V] 设置固定版本",
            "[E] 编辑配置文件",
            "[R] 重置配置",
            "[B] 返回",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("配置管理 (Esc 返回)")
            .items(menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => {
                cmd_config_show(config)?;
                ui::wait_for_key();
            }
            Some(1) => {
                interactive_config_set(config, "mirrors", MIRROR_KEYS)?;
            }
            Some(2) => {
                interactive_config_set(config, "versions", VERSION_KEYS)?;
            }
            Some(3) => {
                cmd_config_edit()?;
                // 用户可能改了文件，重新加载保证本次会话生效
                if let Some(new_config) = HudoConfig::load()? {
                    *config = new_config;
                    ui::print_success("配置已重新加载");
                }
                ui::wait_for_key();
            }
            Some(4) => {
                cmd_config_reset()?;
                ui::wait_for_key();
            }
            Some(5) | None => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// 交互式设置 mirrors.* / versions.* 配置项（写入的是真实 config，本次会话立即生效）
fn interactive_config_set(config: &mut HudoConfig, prefix: &str, keys: &[&str]) -> Result<()> {
    let labels: Vec<String> = keys.iter().map(|k| format!("{}.{}", prefix, k)).collect();
    let key_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要设置的配置项 (Esc 返回)")
        .items(&labels)
        .interact_opt()
        .unwrap_or(None);

    if let Some(idx) = key_sel {
        let value = ui::input_text(&format!("输入 {} 的值（留空取消）", labels[idx]), None, true)?;
        if value.is_empty() {
            ui::print_info("已取消");
        } else {
            cmd_config_set(config, &labels[idx], &value)?;
        }
        ui::wait_for_key();
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.yes {
        ui::set_assume_yes(true);
    }

    // 进程级代理注入：静默读取配置（不触发首次初始化），对版本查询/下载/自更新统一生效
    if let Ok(Some(cfg)) = HudoConfig::load() {
        if let Some(ref p) = cfg.proxy {
            download::set_proxy(p);
        }
    }

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
            Commands::Uninstall { tool, uninstall_self } => {
                if uninstall_self {
                    #[cfg(windows)]
                    cmd_self_uninstall().await?;
                    #[cfg(not(windows))]
                    {
                        ui::print_error("Linux/macOS 暂不支持自卸载，请手动删除 hudo 目录");
                    }
                } else if let Some(t) = tool {
                    if let Some(config) = load_config_readonly()? {
                        cmd_uninstall(&config, &t.to_lowercase()).await?;
                    }
                } else {
                    eprintln!("请指定工具名称，或使用 --self 卸载 hudo 自身");
                    eprintln!("示例: hudo uninstall git");
                    eprintln!("      hudo uninstall --self");
                    std::process::exit(1);
                }
            }
            Commands::Export { file } => {
                if let Some(config) = load_config_readonly()? {
                    cmd_export(&config, file).await?;
                }
            }
            Commands::Import { file } => {
                let mut config = ensure_config()?;
                cmd_import(&mut config, &file).await?;
            }
            Commands::Upgrade { tool } => {
                if let Some(config) = load_config_readonly()? {
                    cmd_upgrade(&config, tool.map(|t| t.to_lowercase())).await?;
                }
            }
            Commands::List { all } => {
                if let Some(config) = load_config_readonly()? {
                    cmd_list(&config, all, all).await?;
                }
            }
            Commands::Config { action } => match action {
                ConfigAction::Show => {
                    if let Some(config) = load_config_readonly()? {
                        cmd_config_show(&config)?;
                    }
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
            Commands::Update => {
                #[cfg(windows)]
                cmd_update().await?;
                #[cfg(not(windows))]
                {
                    ui::print_error("Linux/macOS 暂不支持自更新，请重新下载安装");
                }
            }
            Commands::Cc => {
                cc::cmd_cc()?;
            }
        },
        None => {
            let mut config = ensure_config()?;
            interactive_menu(&mut config).await?;
        }
    }

    Ok(())
}
