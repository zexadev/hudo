use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winreg::enums::*;
use winreg::RegKey;

const ENV_KEY: &str = "Environment";

/// Windows 用户级环境变量管理器
pub struct EnvManager;

impl EnvManager {
    /// 读取用户环境变量
    pub fn get_var(name: &str) -> Result<Option<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey(ENV_KEY).context("无法打开注册表 HKCU\\Environment")?;
        match env.get_value::<String, _>(name) {
            Ok(val) => Ok(Some(val)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("读取环境变量 {} 失败", name)),
        }
    }

    /// 设置用户环境变量（REG_EXPAND_SZ 类型，支持 %VAR% 展开）
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

    /// 往 PATH 追加路径（大小写不敏感去重）
    pub fn append_to_path(new_path: &str) -> Result<()> {
        let current = Self::get_var("Path")?.unwrap_or_default();

        // 分割现有 PATH，检查是否已存在
        let parts: Vec<&str> = current.split(';').filter(|s| !s.is_empty()).collect();
        let already_exists = parts
            .iter()
            .any(|p| p.eq_ignore_ascii_case(new_path));

        if already_exists {
            return Ok(());
        }

        // 追加新路径
        let new_value = if current.is_empty() {
            new_path.to_string()
        } else if current.ends_with(';') {
            format!("{}{}", current, new_path)
        } else {
            format!("{};{}", current, new_path)
        };

        Self::set_var("Path", &new_value)?;
        Ok(())
    }

    /// 广播 WM_SETTINGCHANGE，通知系统环境变量已更新
    pub fn broadcast_change() {
        use windows_sys::Win32::Foundation::*;
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
}

/// 将字符串转为 REG_EXPAND_SZ 所需的字节格式（UTF-16LE + null terminator）
fn to_reg_sz(s: &str) -> Vec<u8> {
    let wide: Vec<u16> = OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    wide.iter()
        .flat_map(|&w| w.to_le_bytes())
        .collect()
}
