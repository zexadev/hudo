use anyhow::{Context, Result};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ui;

// ── Provider 配置 ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CcProviders {
    #[serde(default)]
    pub providers: Vec<CcProvider>,
}

impl CcProviders {
    fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;
        Ok(home.join(".hudo").join("cc-providers.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(&path)
            .with_context(|| format!("读取 {} 失败", path.display()))?;
        toml::from_str(&s).with_context(|| format!("解析 {} 失败", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self).context("序列化 providers 失败")?;
        std::fs::write(&path, s)
            .with_context(|| format!("写入 {} 失败", path.display()))
    }
}

// ── Claude settings.json ──────────────────────────────────────────────────────

fn claude_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    // claude 使用 ~/.claude/settings.json
    Ok(home.join(".claude").join("settings.json"))
}

/// 读取 ~/.claude/settings.json，不存在则返回空对象
fn read_settings() -> Result<serde_json::Value> {
    let path = claude_settings_path()?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let s = std::fs::read_to_string(&path)
        .with_context(|| format!("读取 {} 失败", path.display()))?;
    serde_json::from_str(&s).with_context(|| format!("解析 {} 失败", path.display()))
}

/// 将修改后的 settings 写回
fn write_settings(val: &serde_json::Value) -> Result<()> {
    let path = claude_settings_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(val).context("序列化 settings.json 失败")?;
    std::fs::write(&path, s)
        .with_context(|| format!("写入 {} 失败", path.display()))
}

/// 将 provider 写入 claude settings.json
fn apply_provider(p: &CcProvider) -> Result<()> {
    let mut settings = read_settings()?;

    // 确保 env 对象存在
    if settings.get("env").is_none() {
        settings["env"] = serde_json::json!({});
    }

    settings["env"]["ANTHROPIC_AUTH_TOKEN"] = serde_json::Value::String(p.api_key.clone());
    settings["env"]["ANTHROPIC_BASE_URL"] = serde_json::Value::String(p.base_url.clone());

    write_settings(&settings)
}

/// 从 claude settings.json 读取当前激活的 base_url
fn current_base_url() -> Option<String> {
    read_settings().ok().and_then(|s| {
        s["env"]["ANTHROPIC_BASE_URL"]
            .as_str()
            .map(|v| v.to_string())
    })
}

// ── 交互菜单 ──────────────────────────────────────────────────────────────────

pub fn cmd_cc() -> Result<()> {
    ui::print_title("Claude Code API 来源管理");

    loop {
        let mut store = CcProviders::load()?;
        let active_url = current_base_url();

        if store.providers.is_empty() {
            println!("  {}", console::style("暂无 Provider，请先添加").dim());
            println!();
            let items = ["添加 Provider", "退出"];
            let sel = Select::with_theme(&ColorfulTheme::default())
                .items(&items)
                .default(0)
                .interact_opt()?;
            match sel {
                Some(0) => {
                    add_provider(&mut store)?;
                    store.save()?;
                }
                _ => break,
            }
            continue;
        }

        // 构建列表项：当前激活的前面显示 *
        let items: Vec<String> = store
            .providers
            .iter()
            .map(|p| {
                let active = active_url.as_deref() == Some(&p.base_url);
                let mark = if active {
                    console::style("* ").green().to_string()
                } else {
                    "  ".to_string()
                };
                format!("{}{:<20}  {}", mark, p.name, console::style(&p.base_url).dim())
            })
            .chain(std::iter::once("  [+] 添加 Provider".to_string()))
            .chain(std::iter::once("  [x] 删除 Provider".to_string()))
            .chain(std::iter::once("  退出".to_string()))
            .collect();

        let n = store.providers.len();
        let sel = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择 Provider（* = 当前激活）")
            .items(&items)
            .default(0)
            .interact_opt()?;

        match sel {
            None => break,
            Some(i) if i < n => {
                // 切换到选中的 provider
                let p = &store.providers[i];
                apply_provider(p)?;
                ui::print_success(&format!("已切换到 [{}]  {}", p.name, p.base_url));
                ui::print_info("重启终端或 Claude Code 后生效");
                break;
            }
            Some(i) if i == n => {
                // 添加
                add_provider(&mut store)?;
                store.save()?;
            }
            Some(i) if i == n + 1 => {
                // 删除
                if delete_provider(&mut store)? {
                    store.save()?;
                }
            }
            _ => break,
        }
    }

    Ok(())
}

/// 交互式添加 Provider
fn add_provider(store: &mut CcProviders) -> Result<()> {
    println!();
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("名称（如: 官方 / 中转）")
        .interact_text()?;

    let base_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Base URL（如: https://api.anthropic.com）")
        .interact_text()?;

    let api_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("API Key（sk-ant-...）")
        .interact_text()?;

    store.providers.push(CcProvider {
        name,
        base_url,
        api_key,
    });

    ui::print_success("Provider 已添加");
    Ok(())
}

/// 交互式删除 Provider，返回是否删除了
fn delete_provider(store: &mut CcProviders) -> Result<bool> {
    if store.providers.is_empty() {
        return Ok(false);
    }
    let items: Vec<String> = store
        .providers
        .iter()
        .map(|p| format!("{} — {}", p.name, p.base_url))
        .chain(std::iter::once("取消".to_string()))
        .collect();

    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要删除的 Provider")
        .items(&items)
        .default(0)
        .interact_opt()?;

    match sel {
        Some(i) if i < store.providers.len() => {
            let removed = store.providers.remove(i);
            ui::print_success(&format!("已删除 [{}]", removed.name));
            Ok(true)
        }
        _ => Ok(false),
    }
}
