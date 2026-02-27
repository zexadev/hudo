# Node.js

JavaScript 运行时，通过 [fnm](https://github.com/Schniz/fnm) 管理版本。

## 安装

```powershell
hudo install nodejs
```

安装 fnm 到 `{install_root}\tools\fnm\`，并通过 fnm 安装最新 LTS 版本的 Node.js。

## 安装后

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
