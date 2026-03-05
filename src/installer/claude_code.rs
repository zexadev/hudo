use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{DetectResult, EnvAction, InstallContext, InstallResult, Installer, ToolInfo};
use crate::config::HudoConfig;
use crate::download;
use crate::ui;

pub struct ClaudeCodeInstaller;

const DEFAULT_VERSION: &str = "1.0.0";
const GCS_BUCKET: &str = "https://storage.googleapis.com/claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819/claude-code-releases";

/// 可执行文件名：Windows 为 claude.exe，其他平台为 claude
fn exe_name() -> &'static str {
    if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    }
}

/// 检测 Linux 是否为 musl libc（而非 glibc）
#[cfg(target_os = "linux")]
fn is_musl() -> bool {
    // 方法 1：检查 musl 动态库文件
    if let Ok(entries) = std::fs::read_dir("/lib") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("libc.musl-") && name.ends_with(".so.1") {
                    return true;
                }
            }
        }
    }
    // 方法 2：通过 ldd 输出判断
    if let Ok(out) = std::process::Command::new("ldd").arg("--version").output() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        if stderr.contains("musl") || stdout.contains("musl") {
            return true;
        }
    }
    false
}

/// 根据操作系统和 CPU 架构返回 GCS 平台标识
fn platform_key() -> String {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    };

    match std::env::consts::OS {
        "windows" => format!("win32-{}", arch),
        "macos" => format!("darwin-{}", arch),
        "linux" => {
            #[cfg(target_os = "linux")]
            if is_musl() {
                return format!("linux-{}-musl", arch);
            }
            format!("linux-{}", arch)
        }
        other => format!("{}-{}", other, arch),
    }
}

/// 运行 claude --version 提取版本号
/// 输出格式：类似 "claude v1.0.0" 或 "1.0.0"
fn parse_claude_version(output: &str) -> String {
    output
        .lines()
        .next()
        .map(|l| l.trim())
        .map(|l| {
            // 去掉可能的 "claude " 前缀
            l.strip_prefix("claude ")
                .or_else(|| l.strip_prefix("Claude Code "))
                .unwrap_or(l)
        })
        .map(|s| s.trim_start_matches('v').to_string())
        .unwrap_or_else(|| "已安装".to_string())
}

/// 获取 manifest.json 中目标平台的 SHA256
async fn fetch_manifest_sha256(version: &str, platform: &str) -> Result<String> {
    let url = format!("{}/{}/manifest.json", GCS_BUCKET, version);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let manifest: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("获取 manifest 失败: {}", url))?
        .error_for_status()
        .with_context(|| format!("manifest HTTP 错误: {}", url))?
        .json()
        .await
        .context("解析 manifest JSON 失败")?;

    // manifest 结构: { "win32-x64": { "sha256": "..." }, ... }
    let sha = manifest[platform]["sha256"]
        .as_str()
        .with_context(|| format!("manifest 中找不到平台 {} 的 SHA256", platform))?;
    Ok(sha.to_string())
}

/// 计算文件 SHA256
fn sha256_file(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("无法打开文件: {}", path.display()))?;
    std::io::copy(&mut file, &mut hasher).context("计算 SHA256 失败")?;
    Ok(format!("{:x}", hasher.finalize()))
}

#[async_trait]
impl Installer for ClaudeCodeInstaller {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            id: "claude-code",
            name: "Claude Code",
            description: "Anthropic Claude AI 命令行工具",
        }
    }

    async fn detect_installed(&self, ctx: &InstallContext<'_>) -> Result<DetectResult> {
        let exe = ctx.config.tools_dir().join("claude-code").join(exe_name());
        if exe.exists() {
            if let Ok(out) = std::process::Command::new(&exe).arg("--version").output() {
                if out.status.success() {
                    let version = parse_claude_version(&String::from_utf8_lossy(&out.stdout));
                    return Ok(DetectResult::InstalledByHudo(version));
                }
            }
            return Ok(DetectResult::InstalledByHudo("已安装".to_string()));
        }

        // 回退检查系统 PATH
        if let Ok(out) = std::process::Command::new("claude").arg("--version").output() {
            if out.status.success() {
                let version = parse_claude_version(&String::from_utf8_lossy(&out.stdout));
                return Ok(DetectResult::InstalledExternal(version));
            }
        }

        Ok(DetectResult::NotInstalled)
    }

    fn resolve_download(&self, config: &HudoConfig) -> (String, String) {
        let version = config
            .versions
            .claude_code
            .as_deref()
            .unwrap_or(DEFAULT_VERSION);
        let platform = platform_key();
        let exe = exe_name();
        let url = format!("{}/{}/{}/{}", GCS_BUCKET, version, platform, exe);
        let filename = format!("claude-{}-{}{}", version, platform,
            if cfg!(windows) { ".exe" } else { "" });
        (url, filename)
    }

    async fn install(&self, ctx: &InstallContext<'_>) -> Result<InstallResult> {
        let config = ctx.config;
        let install_dir = config.tools_dir().join("claude-code");

        // 1. 确定版本
        let version = match &config.versions.claude_code {
            Some(v) => v.clone(),
            None => {
                ui::print_action("查询 Claude Code 最新版本...");
                crate::version::claude_code_latest()
                    .await
                    .unwrap_or_else(|| DEFAULT_VERSION.to_string())
            }
        };

        let platform = platform_key();
        let exe = exe_name();

        // 2. 获取 manifest SHA256
        ui::print_action("获取校验信息...");
        let expected_sha = fetch_manifest_sha256(&version, &platform).await?;

        // 3. 下载可执行文件
        let filename = format!("claude-{}-{}{}", version, platform,
            if cfg!(windows) { ".exe" } else { "" });
        let url = format!("{}/{}/{}/{}", GCS_BUCKET, version, platform, exe);
        let cached_path = download::download(&url, &config.cache_dir(), &filename).await?;

        // 4. SHA256 校验
        ui::print_action("校验文件完整性...");
        let actual_sha = sha256_file(&cached_path)?;
        if actual_sha != expected_sha {
            std::fs::remove_file(&cached_path).ok();
            bail!(
                "SHA256 校验失败！\n  预期: {}\n  实际: {}\n已删除损坏文件，请重试",
                expected_sha,
                actual_sha
            );
        }
        ui::print_success("SHA256 校验通过");

        // 5. 安装到 tools/claude-code/
        std::fs::create_dir_all(&install_dir)
            .with_context(|| format!("无法创建目录: {}", install_dir.display()))?;

        let dest_exe = install_dir.join(exe);
        std::fs::copy(&cached_path, &dest_exe)
            .with_context(|| format!("复制文件失败: {}", dest_exe.display()))?;

        // Linux/macOS 需要设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dest_exe, std::fs::Permissions::from_mode(0o755))
                .with_context(|| format!("设置执行权限失败: {}", dest_exe.display()))?;
        }

        Ok(InstallResult {
            install_path: install_dir,
            version,
        })
    }

    fn env_actions(&self, install_path: &PathBuf, _config: &HudoConfig) -> Vec<EnvAction> {
        vec![EnvAction::AppendPath {
            path: install_path.to_string_lossy().to_string(),
        }]
    }

    async fn configure(&self, _ctx: &InstallContext<'_>) -> Result<()> {
        ui::print_title("配置 Claude Code");
        ui::print_info("运行以下命令登录 Claude:");
        ui::print_info("  claude login");
        ui::print_info("或设置环境变量 ANTHROPIC_API_KEY");
        Ok(())
    }
}
