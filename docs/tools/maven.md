# Maven

Apache Maven，Java 项目构建与依赖管理工具。

## 安装

```powershell
hudo install maven
```

安装到 `{install_root}\tools\maven\`，自动获取最新版本。需要先安装 JDK。

## 安装后

```powershell
mvn --version
mvn clean install
```

## 卸载

```powershell
hudo uninstall maven
```

## 配置文件版本

```toml
[versions]
# 不填则自动获取最新版
maven = "3.9.9"
```
