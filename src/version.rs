use reqwest::Client;

/// 复用同一个 client，带 5 秒超时
fn make_client() -> reqwest::Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
}

/// Git: GitHub API → tag "v2.47.1.windows.2" → "2.47.1.2"
pub async fn git_latest() -> Option<String> {
    let client = make_client().ok()?;
    let resp: serde_json::Value = client
        .get("https://api.github.com/repos/git-for-windows/git/releases/latest")
        .header("User-Agent", "hudo")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let tag = resp["tag_name"].as_str()?;
    parse_git_tag(tag)
}

/// "v2.47.1.windows.2" → "2.47.1.2", "v2.53.0.windows.1" → "2.53.0"
fn parse_git_tag(tag: &str) -> Option<String> {
    let tag = tag.strip_prefix('v')?;
    let parts: Vec<&str> = tag.split('.').collect();
    // ["2","47","1","windows","2"] or ["2","53","0","windows","1"]
    let idx = parts.iter().position(|&p| p == "windows")?;
    let ver_parts = &parts[..idx]; // ["2","47","1"]
    let win_patch = parts.get(idx + 1)?; // "2" or "1"
    if *win_patch == "1" {
        Some(ver_parts.join(".")) // "2.53.0"
    } else {
        Some(format!("{}.{}", ver_parts.join("."), win_patch)) // "2.47.1.2"
    }
}

/// Go: go.dev/dl API → "1.24.0"
pub async fn go_latest() -> Option<String> {
    let client = make_client().ok()?;
    let resp: Vec<serde_json::Value> = client
        .get("https://go.dev/dl/?mode=json")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let ver = resp.first()?["version"].as_str()?; // "go1.24.0"
    Some(ver.strip_prefix("go")?.to_string())
}

/// PostgreSQL: versions.json → 当前大版本最新
pub async fn pgsql_latest() -> Option<String> {
    let client = make_client().ok()?;
    let resp: Vec<serde_json::Value> = client
        .get("https://www.postgresql.org/versions.json")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    resp.iter()
        .find(|v| v["current"].as_bool() == Some(true))
        .and_then(|v| v["latestMinor"].as_str())
        .map(|s| s.to_string())
}

/// PyCharm: JetBrains API → 最新 CE 版本号
pub async fn pycharm_latest() -> Option<String> {
    let client = make_client().ok()?;
    let resp: serde_json::Value = client
        .get("https://data.services.jetbrains.com/products/releases?code=PCC&latest=true&type=release")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    resp["PCC"][0]["version"].as_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_tag_with_patch() {
        assert_eq!(
            parse_git_tag("v2.47.1.windows.2"),
            Some("2.47.1.2".to_string())
        );
    }

    #[test]
    fn test_parse_git_tag_without_patch() {
        assert_eq!(
            parse_git_tag("v2.53.0.windows.1"),
            Some("2.53.0".to_string())
        );
    }

    #[test]
    fn test_parse_git_tag_invalid() {
        assert_eq!(parse_git_tag("invalid"), None);
        assert_eq!(parse_git_tag("2.47.1"), None);
    }
}
