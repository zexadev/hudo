# 配置文件

hudo 的配置文件位于 `%USERPROFILE%\.hudo\config.toml`，首次运行时自动创建。

## 配置项说明

```toml
# 工具安装根目录
install_root = "D:\\hudo"

[versions]
# 固定各工具版本，不填则自动获取最新版
# git = "2.47.0"
# nodejs = "22.0.0"
# go = "1.23.0"

[mirror]
# 自定义下载镜像（可选）
# nodejs = "https://npmmirror.com/mirrors/node"
```

## 修改配置

直接用文本编辑器打开修改：

```powershell
notepad $env:USERPROFILE\.hudo\config.toml
```

## 固定工具版本

如果需要安装指定版本，在 `[versions]` 下添加：

```toml
[versions]
nodejs = "20.11.0"
go = "1.21.0"
```

再次运行 `hudo install` 时会使用指定版本。
