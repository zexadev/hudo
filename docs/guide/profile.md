# 配置档案

配置档案功能让你在多台电脑间同步开发环境配置。

## 导出档案

```powershell
hudo profile export
```

会在当前目录生成 `hudo-profile.toml`，记录当前已安装的所有工具及版本。

## 导入档案

在新电脑上安装好 hudo 后：

```powershell
hudo profile import
```

按提示选择档案文件，hudo 会自动安装文件中记录的所有工具。

## 档案文件格式

```toml
[tools]
git = "2.47.0"
nodejs = "22.0.0"
go = "1.23.0"
vscode = "1.95.0"
```

## 注意事项

- GitHub CLI 的登录状态**不会**导出到档案（出于安全考虑）
- 导入后会自动提示运行 `gh auth login` 完成认证
- 档案文件可以提交到团队仓库，统一团队开发环境
