# Gradle

Gradle 构建工具，支持 Java、Kotlin、Android 项目。

## 安装

```powershell
hudo install gradle
```

安装到 `{install_root}\tools\gradle\`，自动获取最新版本。需要先安装 JDK。

## 安装后

```powershell
gradle --version
gradle build
```

## 卸载

```powershell
hudo uninstall gradle
```

## 配置文件版本

```toml
[versions]
# 不填则自动获取最新版
gradle = "8.12.1"
```
