# Git

分布式版本控制系统。

## 安装

```powershell
hudo install git
```

安装到 `{install_root}\tools\git\`，自动获取最新版本。

## 安装后

```powershell
git --version
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

## 卸载

```powershell
hudo uninstall git
```

## 配置文件版本

```toml
[versions]
# 不填则自动获取最新版
git = "2.47.0"
```
