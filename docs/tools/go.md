# Go

Go 编程语言官方工具链。

## 安装

```powershell
hudo install go
```

安装到 `{install_root}\lang\go\`，自动获取最新版本，自动设置 `GOPATH` 到 `{install_root}\lang\gopath\`。

## 安装后

```powershell
go version
go env GOPATH
```

## 卸载

```powershell
hudo uninstall go
```

## 配置文件版本

```toml
[versions]
go = "1.23.0"
```
