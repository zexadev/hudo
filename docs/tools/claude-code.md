# Claude Code

Anthropic Claude Code CLI，AI 驱动的命令行编程助手。

## 安装

```powershell
hudo install claude-code
```

从 Google Cloud Storage 下载官方二进制，安装到 `{install_root}\tools\claude-code\claude.exe`，并进行 SHA256 完整性校验。

## 安装后

登录账号：

```powershell
claude login
```

或设置 API Key：

```powershell
$env:ANTHROPIC_API_KEY = "sk-ant-..."
```

## 使用

```powershell
# 在项目目录启动
cd my-project
claude
```

## 卸载

```powershell
hudo uninstall claude-code
```

## 配置文件版本

```toml
[versions]
# 不填则自动获取最新版
claude_code = "1.0.0"
```
