use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HudoConfig {
    /// 安装根目录（如 D:\hudo）
    pub root_dir: String,

    #[serde(default)]
    pub java: JavaConfig,

    #[serde(default)]
    pub go: GoConfig,

    #[serde(default)]
    pub versions: VersionConfig,

    #[serde(default)]
    pub mirrors: MirrorConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaConfig {
    #[serde(default = "default_java_version")]
    pub version: String,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            version: default_java_version(),
        }
    }
}

fn default_java_version() -> String {
    "21".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoConfig {
    #[serde(default = "default_go_version")]
    pub version: String,
}

impl Default for GoConfig {
    fn default() -> Self {
        Self {
            version: default_go_version(),
        }
    }
}

fn default_go_version() -> String {
    "latest".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MirrorConfig {
    pub uv: Option<String>,
    pub fnm: Option<String>,
    pub go: Option<String>,
    pub java: Option<String>,
    pub vscode: Option<String>,
    pub pycharm: Option<String>,
    pub maven: Option<String>,
    pub gradle: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VersionConfig {
    pub git: Option<String>,
    pub fnm: Option<String>,
    pub mysql: Option<String>,
    pub pgsql: Option<String>,
    pub pycharm: Option<String>,
    pub maven: Option<String>,
    pub gradle: Option<String>,
}

impl HudoConfig {
    /// 配置文件路径: %USERPROFILE%\.hudo\config.toml
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;
        Ok(home.join(".hudo").join("config.toml"))
    }

    /// 加载配置文件，不存在则返回 None
    pub fn load() -> Result<Option<Self>> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path.display()))?;
        let config: HudoConfig = toml::from_str(&content)
            .with_context(|| format!("配置文件格式错误: {}", path.display()))?;
        Ok(Some(config))
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("序列化配置失败")?;
        std::fs::write(&path, content)
            .with_context(|| format!("无法写入配置文件: {}", path.display()))?;
        Ok(())
    }

    /// 从 root_dir 派生各子目录
    pub fn root_path(&self) -> PathBuf {
        PathBuf::from(&self.root_dir)
    }

    pub fn tools_dir(&self) -> PathBuf {
        self.root_path().join("tools")
    }

    pub fn lang_dir(&self) -> PathBuf {
        self.root_path().join("lang")
    }

    pub fn ide_dir(&self) -> PathBuf {
        self.root_path().join("ide")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root_path().join("cache")
    }

    pub fn state_path(&self) -> PathBuf {
        self.root_path().join("state.json")
    }

    /// 创建安装根目录下的标准子目录
    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [self.tools_dir(), self.lang_dir(), self.ide_dir(), self.cache_dir()] {
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("无法创建目录: {}", dir.display()))?;
        }
        Ok(())
    }

    /// 扫描可用磁盘（Windows 盘符）
    pub fn scan_drives() -> Vec<DriveInfo> {
        let mut drives = Vec::new();
        for letter in b'C'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let path = Path::new(&drive);
            if path.exists() {
                let is_system = letter == b'C';
                // 获取剩余空间（简单实现）
                let free_gb = get_free_space_gb(&drive);
                drives.push(DriveInfo {
                    letter: letter as char,
                    is_system,
                    free_gb,
                });
            }
        }
        drives
    }
}

#[derive(Debug)]
pub struct DriveInfo {
    pub letter: char,
    pub is_system: bool,
    pub free_gb: u64,
}

fn get_free_space_gb(path: &str) -> u64 {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;

    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    let mut free_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;

    unsafe {
        windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes as *mut u64,
            &mut total_bytes as *mut u64,
            &mut total_free as *mut u64,
        );
    }
    free_bytes / (1024 * 1024 * 1024)
}
