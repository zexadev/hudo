# Node.js

JavaScript 运行时，通过 [fnm](https://github.com/Schniz/fnm) 管理版本。

## 安装

```powershell
hudo install nodejs
```

安装 fnm 到 `{install_root}\tools\fnm\`，并通过 fnm 安装最新 LTS 版本的 Node.js。

## 安装后

安装完成后重新打开终端即可使用 `node`、`npm`、`fnm` 命令。

> hudo 安装时会自动设置 PowerShell 执行策略（`RemoteSigned`）并写入 `$PROFILE`，确保 fnm 初始化脚本可以运行。
> 如果重开终端后命令仍不可用，手动执行：
> ```powershell
> Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
> ```

```powershell
node --version
npm --version
fnm list
```

切换版本：

```powershell
fnm install 20
fnm use 20
```

## 卸载

```powershell
hudo uninstall nodejs
```

## 配置文件版本

```toml
[versions]
nodejs = "22.0.0"
```
