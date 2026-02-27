# 安装 hudo

## 系统要求

- Windows 10 / 11
- PowerShell 5.1 或更高版本

## 一键安装

打开 PowerShell，运行：

```powershell
irm https://raw.githubusercontent.com/zexadev/hudo/master/install.ps1 | iex
```

安装完成后，`hudo` 命令即可在新终端中使用。

## 安装位置

| 文件 | 路径 |
|------|------|
| hudo 可执行文件 | `%USERPROFILE%\.hudo\bin\hudo.exe` |
| 配置文件 | `%USERPROFILE%\.hudo\config.toml` |
| 安装记录 | `%USERPROFILE%\.hudo\state.json` |

工具安装到你选择的盘（如 `D:\hudo\tools\`），不占用 C 盘。

## 验证安装

```powershell
hudo -v
```

## 卸载

```powershell
hudo uninstall --self
```
