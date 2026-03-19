# VS Code

Microsoft Visual Studio Code，轻量级代码编辑器。

## 安装

```powershell
hudo install vscode
```

安装到 `{install_root}\ide\vscode\`，免安装版（zip）。

安装时自动注册 Windows 右键菜单「通过 Code 打开」，支持：

- 右键文件 → 通过 Code 打开
- 右键文件夹 → 通过 Code 打开
- 右键文件夹空白处 → 通过 Code 打开

## 安装后

```powershell
code --version
code .
```

## 卸载

```powershell
hudo uninstall vscode
```

卸载时自动清理右键菜单注册表项。
