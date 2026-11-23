pub mod claude_code;

// Windows 专属安装器
#[cfg(windows)]
pub mod bun;
#[cfg(windows)]
pub mod chrome;
#[cfg(windows)]
pub mod dotnet;
#[cfg(windows)]
pub mod gh;
#[cfg(windows)]
pub mod git;
#[cfg(windows)]
pub mod go;
#[cfg(windows)]
pub mod gradle;
#[cfg(windows)]
pub mod jdk;
#[cfg(windows)]
pub mod maven;
#[cfg(windows)]
pub mod miniconda;
#[cfg(windows)]
pub mod mingw;
#[cfg(windows)]
pub mod mysql;
#[cfg(windows)]
pub mod fnm;
#[cfg(windows)]
pub mod nodejs;
#[cfg(windows)]
pub mod pgsql;
#[cfg(windows)]
pub mod redis;
#[cfg(windows)]
pub mod idea;
#[cfg(windows)]
pub mod pycharm;
#[cfg(windows)]
pub mod omp;
#[cfg(windows)]
pub mod powertoys;
#[cfg(windows)]
pub mod pwsh;
#[cfg(windows)]
pub mod rustup;
#[cfg(windows)]
pub mod sevenzip;
#[cfg(windows)]
pub mod uv;
#[cfg(windows)]
pub mod vscode;

use anyhow::{Context, Result};
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

    /// 卸载前的清理操作（默认无操作）
    /// 在删除安装目录之前调用，用于停止服务等操作
    async fn pre_uninstall(&self, _ctx: &InstallContext<'_>) -> Result<()> {
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

// ── Windows 服务管理工具（mysql、pgsql 共用） ───────────────────────────────

#[cfg(windows)]
pub enum ServiceState {
    Running,
    Stopped,
    NotFound,
}

#[cfg(windows)]
pub fn query_service_exists(name: &str) -> bool {
    std::process::Command::new("sc")
        .args(["query", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn query_service_state(name: &str) -> ServiceState {
    match std::process::Command::new("sc").args(["query", name]).output() {
        Ok(out) if out.status.success() => {
            if String::from_utf8_lossy(&out.stdout).contains("RUNNING") {
                ServiceState::Running
            } else {
                ServiceState::Stopped
            }
        }
        _ => ServiceState::NotFound,
    }
}

/// 强力删除目录：先递归清除只读属性，再调用 remove_dir_all；
/// 仍失败则回退到 `cmd /c rd /s /q` 处理 junction 等 Rust std 不友好的场景。
#[cfg(windows)]
pub fn force_remove_dir_all(path: &std::path::Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    clear_readonly_recursive(path).ok();
    if let Err(e) = std::fs::remove_dir_all(path) {
        // 回退：cmd 的 rd /s /q 能处理 junction 与一些 std 卡住的边界场景
        let status = std::process::Command::new("cmd")
            .args(["/c", "rd", "/s", "/q", &path.to_string_lossy()])
            .status();
        match status {
            Ok(s) if s.success() && !path.exists() => Ok(()),
            _ => Err(anyhow::anyhow!(
                "删除目录失败: {} ({})",
                path.display(),
                e
            )),
        }
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn clear_readonly_recursive(path: &std::path::Path) -> std::io::Result<()> {
    let md = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    let mut perm = md.permissions();
    if perm.readonly() {
        perm.set_readonly(false);
        std::fs::set_permissions(path, perm).ok();
    }
    if md.is_dir() && !md.file_type().is_symlink() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                clear_readonly_recursive(&entry.path()).ok();
            }
        }
    }
    Ok(())
}

/// 启动 Windows PowerShell 5.1（powershell.exe）的统一入口。
///
/// 必须清掉继承的 `PSModulePath`：pwsh 7 会把自己的模块目录前插进该变量，
/// 经 hudo.exe 透传给 5.1 后，5.1 优先命中 Core 版 `Microsoft.PowerShell.Security`
/// 等模块并加载失败（`Get-ExecutionPolicy` 无法加载模块 → 脚本退出码 1）。
/// 变量缺省时 5.1 会按注册表 + 默认目录自行重建模块路径。
pub fn winps_command() -> std::process::Command {
    let mut cmd = std::process::Command::new("powershell");
    cmd.env_remove("PSModulePath");
    cmd
}

/// 旧版 hudo 的 fnm 安装痕迹检测：`lang/node/` 下存在 `node-versions/` 或 `aliases/`
#[cfg(windows)]
pub fn is_legacy_fnm_dir(node_dir: &std::path::Path) -> bool {
    node_dir.join("node-versions").exists() || node_dir.join("aliases").exists()
}

/// 清理 PowerShell `$PROFILE` 中由 hudo 写入的 fnm 初始化行（旧版或新版均能匹配）。
/// 返回值：是否实际修改了文件。
#[cfg(windows)]
pub fn clean_fnm_profile_lines() -> anyhow::Result<bool> {
    let output = winps_command()
        .args(["-NoProfile", "-Command", "$PROFILE"])
        .output();
    let Ok(out) = output else { return Ok(false) };
    let profile_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if profile_path.is_empty() {
        return Ok(false);
    }
    let profile_path = std::path::PathBuf::from(&profile_path);
    if !profile_path.exists() {
        return Ok(false);
    }

    let content = match std::fs::read_to_string(&profile_path) {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };

    let line_sep = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let marker_cmd = "fnm env --use-on-cd --shell power-shell";
    let marker_comment = "# fnm (Node.js version manager)";

    let mut kept: Vec<&str> = Vec::new();
    for line in content.split(line_sep) {
        let t = line.trim();
        if t.contains(marker_cmd) || t == marker_comment {
            continue;
        }
        kept.push(line);
    }

    let new_content = kept.join(line_sep);
    // 去掉末尾多余空行
    let new_content = new_content.trim_end_matches(&['\r', '\n'][..]).to_string();

    if new_content == content.trim_end_matches(&['\r', '\n'][..]) {
        return Ok(false);
    }

    let final_content = if new_content.is_empty() {
        String::new()
    } else {
        format!("{}{}", new_content, line_sep)
    };
    std::fs::write(&profile_path, final_content)
        .with_context(|| format!("写入 PowerShell profile 失败: {}", profile_path.display()))?;
    Ok(true)
}

/// 通过 PowerShell Start-Process -Verb RunAs 以管理员身份运行命令
#[cfg(windows)]
pub fn run_as_admin(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let prog_escaped = program.replace('\'', "''");
    let arg_list: Vec<String> = args
        .iter()
        .map(|a| format!("'{}'", a.replace('\'', "''")))
        .collect();

    let ps_cmd = format!(
        "try {{ \
           $p = Start-Process -FilePath '{}' -ArgumentList @({}) \
                -Verb RunAs -Wait -PassThru -WindowStyle Hidden; \
           if ($p) {{ exit $p.ExitCode }} else {{ exit 1 }} \
         }} catch {{ exit 1 }}",
        prog_escaped,
        arg_list.join(", ")
    );

    let output = winps_command()
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.trim().is_empty() {
            anyhow::bail!("管理员权限操作失败（用户可能拒绝了 UAC 提示）: {}", program)
        } else {
            anyhow::bail!("管理员权限操作失败: {}\n{}", program, stderr.trim())
        }
    }
}

/// 返回所有可用的安装器
pub fn all_installers() -> Vec<Box<dyn Installer>> {
    let mut list: Vec<Box<dyn Installer>> = vec![
        Box::new(claude_code::ClaudeCodeInstaller),
    ];

    #[cfg(windows)]
    {
        list.insert(0, Box::new(git::GitInstaller));
        list.insert(1, Box::new(gh::GhInstaller));
        // 语言环境 — 按语言分组
        list.push(Box::new(uv::UvInstaller));           // Python
        list.push(Box::new(miniconda::MinicondaInstaller)); // Python
        list.push(Box::new(nodejs::NodejsInstaller));   // JavaScript
        list.push(Box::new(fnm::FnmInstaller));         // JavaScript（Node.js 版本管理）
        list.push(Box::new(bun::BunInstaller));         // JavaScript
        list.push(Box::new(rustup::RustupInstaller));   // Rust
        list.push(Box::new(go::GoInstaller));           // Go
        list.push(Box::new(jdk::JdkInstaller));         // Java
        list.push(Box::new(maven::MavenInstaller));     // Java 构建
        list.push(Box::new(gradle::GradleInstaller));   // Java/Android 构建
        list.push(Box::new(mingw::MingwInstaller));     // C/C++
        list.push(Box::new(dotnet::DotnetInstaller));   // .NET
        // 数据库
        list.push(Box::new(mysql::MysqlInstaller));
        list.push(Box::new(pgsql::PgsqlInstaller));
        list.push(Box::new(redis::RedisInstaller));
        // 编辑器 / IDE
        // 系统工具
        list.push(Box::new(sevenzip::SevenzipInstaller));
        list.push(Box::new(pwsh::PwshInstaller));
        list.push(Box::new(powertoys::PowertoysInstaller));
        list.push(Box::new(omp::OmpInstaller));
        // 编辑器 / IDE
        list.push(Box::new(vscode::VscodeInstaller));
        list.push(Box::new(pycharm::PycharmInstaller));
        list.push(Box::new(idea::IdeaInstaller));
        list.push(Box::new(chrome::ChromeInstaller));
    }

    list
}
