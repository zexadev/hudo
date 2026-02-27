# GitHub CLI

GitHub 官方命令行工具，用于管理 PR、Issue、Release 等。

## 安装

```powershell
hudo install gh
```

安装到 `{install_root}\tools\gh\`，自动获取最新版本。

## 安装后

首次使用需要登录：

```powershell
gh auth login
```

## 常用命令

```powershell
gh repo clone owner/repo
gh pr list
gh issue create
gh release create v1.0.0
```

## 注意

GitHub CLI 的登录状态不会导出到 profile 档案，换电脑后需重新运行 `gh auth login`。

## 卸载

```powershell
hudo uninstall gh
```
