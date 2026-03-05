use anyhow::{Context, Result};

/// 跨平台环境变量管理器
pub struct EnvManager;

// ── Windows 实现：注册表 ────────────────────────────────────────────────────

#[cfg(windows)]
mod platform {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winreg::enums::*;
    use winreg::RegKey;

    const ENV_KEY: &str = "Environment";

    pub fn get_var(name: &str) -> Result<Option<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey(ENV_KEY).context("无法打开注册表 HKCU\\Environment")?;
        match env.get_value::<String, _>(name) {
            Ok(val) => Ok(Some(val)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("读取环境变量 {} 失败", name)),
        }
    }

    pub fn set_var(name: &str, value: &str) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags(ENV_KEY, KEY_SET_VALUE)
            .context("无法打开注册表 HKCU\\Environment（写入）")?;
        env.set_raw_value(name, &winreg::RegValue {
            vtype: REG_EXPAND_SZ,
            bytes: to_reg_sz(value),
        })
        .with_context(|| format!("设置环境变量 {} 失败", name))?;
        Ok(())
    }

    pub fn delete_var(name: &str) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags(ENV_KEY, KEY_SET_VALUE)
            .context("无法打开注册表 HKCU\\Environment（写入）")?;
        match env.delete_value(name) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e).with_context(|| format!("删除环境变量 {} 失败", name)),
        }
    }

    pub fn append_to_path(new_path: &str) -> Result<()> {
        let current = get_var("Path")?.unwrap_or_default();
        let parts: Vec<&str> = current.split(';').filter(|s| !s.is_empty()).collect();
        let already_exists = parts.iter().any(|p| p.eq_ignore_ascii_case(new_path));
        if already_exists {
            return Ok(());
        }
        let new_value = if current.is_empty() {
            new_path.to_string()
        } else if current.ends_with(';') {
            format!("{}{}", current, new_path)
        } else {
            format!("{};{}", current, new_path)
        };
        set_var("Path", &new_value)
    }

    pub fn remove_from_path(target: &str) -> Result<()> {
        let current = get_var("Path")?.unwrap_or_default();
        let new_parts: Vec<&str> = current
            .split(';')
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case(target))
            .collect();
        let new_value = new_parts.join(";");
        if new_value != current {
            set_var("Path", &new_value)?;
        }
        Ok(())
    }

    pub fn broadcast_change() {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        let env_wide: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mut _result: usize = 0;
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env_wide.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                5000,
                &mut _result,
            );
        }
    }

    fn to_reg_sz(s: &str) -> Vec<u8> {
        let wide: Vec<u16> = OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        wide.iter()
            .flat_map(|&w| w.to_le_bytes())
            .collect()
    }
}

// ── Unix 实现：shell profile ────────────────────────────────────────────────

#[cfg(unix)]
mod platform {
    use super::*;
    use std::path::PathBuf;

    /// 获取用户 shell profile 路径
    fn shell_profile() -> Result<PathBuf> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;

        // 优先检查当前 shell
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return Ok(home.join(".zshrc"));
            }
            if shell.contains("fish") {
                return Ok(home.join(".config").join("fish").join("config.fish"));
            }
        }
        // 默认 bash
        let bashrc = home.join(".bashrc");
        if bashrc.exists() {
            return Ok(bashrc);
        }
        Ok(home.join(".profile"))
    }

    /// hudo 专用的环境配置文件
    fn hudo_env_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;
        Ok(home.join(".hudo").join("env.sh"))
    }

    /// 确保 shell profile source 了 hudo env 文件
    fn ensure_sourced() -> Result<()> {
        let env_path = hudo_env_path()?;
        let profile = shell_profile()?;

        let source_line = format!(". \"{}\"", env_path.display());

        if profile.exists() {
            let content = std::fs::read_to_string(&profile)
                .with_context(|| format!("读取 {} 失败", profile.display()))?;
            if content.contains(&source_line) {
                return Ok(());
            }
        }

        // 追加 source 行
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&profile)
            .with_context(|| format!("写入 {} 失败", profile.display()))?;

        use std::io::Write;
        writeln!(file, "\n# hudo 环境变量")?;
        writeln!(file, "{}", source_line)?;

        Ok(())
    }

    /// 读取 hudo env.sh 的所有行
    fn read_env_lines() -> Result<Vec<String>> {
        let path = hudo_env_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("读取 {} 失败", path.display()))?;
        Ok(content.lines().map(|l| l.to_string()).collect())
    }

    /// 写入所有行到 hudo env.sh
    fn write_env_lines(lines: &[String]) -> Result<()> {
        let path = hudo_env_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = lines.join("\n") + "\n";
        std::fs::write(&path, content)
            .with_context(|| format!("写入 {} 失败", path.display()))
    }

    pub fn get_var(name: &str) -> Result<Option<String>> {
        let prefix = format!("export {}=", name);
        for line in read_env_lines()? {
            if let Some(rest) = line.strip_prefix(&prefix) {
                let val = rest.trim_matches('"').to_string();
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    pub fn set_var(name: &str, value: &str) -> Result<()> {
        ensure_sourced()?;
        let mut lines = read_env_lines()?;
        let prefix = format!("export {}=", name);
        let new_line = format!("export {}=\"{}\"", name, value);

        let mut found = false;
        for line in &mut lines {
            if line.starts_with(&prefix) {
                *line = new_line.clone();
                found = true;
                break;
            }
        }
        if !found {
            lines.push(new_line);
        }
        write_env_lines(&lines)
    }

    pub fn delete_var(name: &str) -> Result<()> {
        let mut lines = read_env_lines()?;
        let prefix = format!("export {}=", name);
        let before = lines.len();
        lines.retain(|l| !l.starts_with(&prefix));
        if lines.len() != before {
            write_env_lines(&lines)?;
        }
        Ok(())
    }

    pub fn append_to_path(new_path: &str) -> Result<()> {
        ensure_sourced()?;
        let mut lines = read_env_lines()?;
        let export_line = format!("export PATH=\"{}:$PATH\"", new_path);

        // 检查是否已存在
        for line in &lines {
            if line.contains(new_path) && line.starts_with("export PATH=") {
                return Ok(());
            }
        }

        lines.push(export_line);
        write_env_lines(&lines)
    }

    pub fn remove_from_path(target: &str) -> Result<()> {
        let mut lines = read_env_lines()?;
        let before = lines.len();
        lines.retain(|l| !(l.starts_with("export PATH=") && l.contains(target)));
        if lines.len() != before {
            write_env_lines(&lines)?;
        }
        Ok(())
    }

    pub fn broadcast_change() {
        // Unix 下无需广播，环境变量在新 shell 中自动生效
    }
}

// ── 统一公共接口 ────────────────────────────────────────────────────────────

impl EnvManager {
    pub fn get_var(name: &str) -> Result<Option<String>> {
        platform::get_var(name)
    }

    pub fn set_var(name: &str, value: &str) -> Result<()> {
        platform::set_var(name, value)
    }

    pub fn append_to_path(new_path: &str) -> Result<()> {
        platform::append_to_path(new_path)
    }

    pub fn delete_var(name: &str) -> Result<()> {
        platform::delete_var(name)
    }

    pub fn remove_from_path(target: &str) -> Result<()> {
        platform::remove_from_path(target)
    }

    pub fn broadcast_change() {
        platform::broadcast_change()
    }
}
