# JDK

Java 开发工具包，使用 [Eclipse Temurin](https://adoptium.net) 发行版（原 AdoptOpenJDK）。

## 安装

```powershell
hudo install jdk
```

安装到 `{install_root}\lang\java\`，默认安装 JDK 21（LTS），可通过配置文件指定主版本号。

## 安装后

```powershell
java -version
javac -version
```

## 卸载

```powershell
hudo uninstall jdk
```

## 配置文件版本

```toml
[versions]
jdk = "21"   # 主版本号，不填则使用 LTS 默认版本
```
