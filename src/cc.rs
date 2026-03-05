use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ui;

// ── Provider 配置 ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub reasoning_model: Option<String>,
    #[serde(default)]
    pub haiku_model: Option<String>,
    #[serde(default)]
    pub sonnet_model: Option<String>,
    #[serde(default)]
    pub opus_model: Option<String>,
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

/// 将 provider 写入 claude settings.json，并确保 onboarding 已标记完成
fn apply_provider(p: &CcProvider) -> Result<()> {
    let mut settings = read_settings()?;

    // 确保 env 对象存在
    if settings.get("env").is_none() {
        settings["env"] = serde_json::json!({});
    }

    settings["env"]["ANTHROPIC_AUTH_TOKEN"] = serde_json::Value::String(p.api_key.clone());
    settings["env"]["ANTHROPIC_BASE_URL"] = serde_json::Value::String(p.base_url.clone());

    // 写入模型配置（有值则设置，无值则清除）
    let model_fields: &[(&str, &Option<String>)] = &[
        ("ANTHROPIC_MODEL", &p.model),
        ("ANTHROPIC_REASONING_MODEL", &p.reasoning_model),
        ("ANTHROPIC_DEFAULT_HAIKU_MODEL", &p.haiku_model),
        ("ANTHROPIC_DEFAULT_SONNET_MODEL", &p.sonnet_model),
        ("ANTHROPIC_DEFAULT_OPUS_MODEL", &p.opus_model),
    ];
    for (key, val) in model_fields {
        match val {
            Some(v) => {
                settings["env"][key] = serde_json::Value::String(v.clone());
            }
            None => {
                if let Some(env) = settings["env"].as_object_mut() {
                    env.remove(*key);
                }
            }
        }
    }

    write_settings(&settings)?;

    // 使用第三方 API 时，需要在 ~/.claude.json 中标记 onboarding 已完成
    // 否则 Claude Code 会卡在引导流程
    ensure_onboarding_completed()
}

/// 确保 ~/.claude.json 中 hasCompletedOnboarding = true
fn ensure_onboarding_completed() -> Result<()> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    let path = home.join(".claude.json");

    let mut val = if path.exists() {
        let s = std::fs::read_to_string(&path)
            .with_context(|| format!("读取 {} 失败", path.display()))?;
        serde_json::from_str(&s)
            .with_context(|| format!("解析 {} 失败", path.display()))?
    } else {
        serde_json::json!({})
    };

    if val.get("hasCompletedOnboarding") == Some(&serde_json::Value::Bool(true)) {
        return Ok(());
    }

    val["hasCompletedOnboarding"] = serde_json::Value::Bool(true);

    let s = serde_json::to_string_pretty(&val).context("序列化 .claude.json 失败")?;
    std::fs::write(&path, s)
        .with_context(|| format!("写入 {} 失败", path.display()))?;

    Ok(())
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

    // 可选：配置自定义模型
    let (model, reasoning_model, haiku_model, sonnet_model, opus_model) =
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("是否配置自定义模型？（第三方 API 通常需要）")
            .default(false)
            .interact()?
        {
            let ask = |prompt: &str| -> Result<Option<String>> {
                let v: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(prompt)
                    .allow_empty(true)
                    .interact_text()?;
                Ok(if v.is_empty() { None } else { Some(v) })
            };
            (
                ask("默认模型 (ANTHROPIC_MODEL，回车跳过)")?,
                ask("推理模型 (ANTHROPIC_REASONING_MODEL，回车跳过)")?,
                ask("Haiku 模型 (ANTHROPIC_DEFAULT_HAIKU_MODEL，回车跳过)")?,
                ask("Sonnet 模型 (ANTHROPIC_DEFAULT_SONNET_MODEL，回车跳过)")?,
                ask("Opus 模型 (ANTHROPIC_DEFAULT_OPUS_MODEL，回车跳过)")?,
            )
        } else {
            (None, None, None, None, None)
        };

    store.providers.push(CcProvider {
        name,
        base_url,
        api_key,
        model,
        reasoning_model,
        haiku_model,
        sonnet_model,
        opus_model,
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
