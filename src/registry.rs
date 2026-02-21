use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 单个工具的安装状态
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolState {
    pub version: String,
    pub install_path: String,
    pub installed_at: String,
}

/// 所有工具的安装状态（保存在 state.json）
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstallRegistry {
    pub tools: HashMap<String, ToolState>,
}

impl InstallRegistry {
    /// 从 state.json 加载
    pub fn load(state_path: &Path) -> Result<Self> {
        if !state_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(state_path)
            .with_context(|| format!("无法读取状态文件: {}", state_path.display()))?;
        match serde_json::from_str::<InstallRegistry>(&content) {
            Ok(registry) => Ok(registry),
            Err(_) => {
                eprintln!(
                    "  {} 状态文件损坏，已重置: {}",
                    console::style("⚠").yellow(),
                    state_path.display()
                );
                Ok(Self::default())
            }
        }
    }

    /// 保存到 state.json
    pub fn save(&self, state_path: &Path) -> Result<()> {
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let content = serde_json::to_string_pretty(self).context("序列化状态失败")?;
        std::fs::write(state_path, content)
            .with_context(|| format!("无法写入状态文件: {}", state_path.display()))?;
        Ok(())
    }

    /// 记录工具安装状态
    pub fn mark_installed(&mut self, tool_id: &str, version: &str, install_path: &str) {
        let now = current_timestamp();
        self.tools.insert(
            tool_id.to_string(),
            ToolState {
                version: version.to_string(),
                install_path: install_path.to_string(),
                installed_at: now,
            },
        );
    }

    /// 查询工具是否已安装
    #[allow(dead_code)]
    pub fn get(&self, tool_id: &str) -> Option<&ToolState> {
        self.tools.get(tool_id)
    }

    /// 移除工具安装记录
    pub fn remove(&mut self, tool_id: &str) {
        self.tools.remove(tool_id);
    }
}

/// 可读的本地时间戳（通过 Windows API，不引入 chrono 依赖）
pub fn current_timestamp() -> String {
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;

    let mut st = windows_sys::Win32::Foundation::SYSTEMTIME {
        wYear: 0, wMonth: 0, wDayOfWeek: 0, wDay: 0,
        wHour: 0, wMinute: 0, wSecond: 0, wMilliseconds: 0,
    };
    unsafe { GetLocalTime(&mut st) };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
    )
}
