# hudo 混沌

**Windows 开发环境一键引导工具**

用一条命令装好开发所需的全部工具，并自动配置好环境变量。

---

## 安装

```powershell
irm https://raw.githubusercontent.com/huancheng01/hudo/master/install.ps1 | iex
```

安装到 `%USERPROFILE%\.hudo\bin\`，自动写入用户 PATH，无需管理员权限。

---

## 快速开始

```
hudo
```

无参数运行进入交互式菜单，按分类勾选工具后一键安装。

---

## 命令

| 命令 | 说明 |
|------|------|
| `hudo` | 交互式菜单（推荐） |
| `hudo setup` | 交互式多选安装 |
| `hudo install <工具>` | 安装单个工具 |
| `hudo uninstall <工具>` | 卸载工具 |
| `hudo list` | 查看已安装工具 |
| `hudo list --all` | 查看所有可用工具 |
| `hudo export [文件]` | 导出环境档案 |
| `hudo import <文件>` | 从档案恢复环境 |
| `hudo config show` | 显示当前配置 |
| `hudo config set <key> <value>` | 修改配置项 |
| `hudo config edit` | 用编辑器打开配置文件 |
| `hudo update` | 更新 hudo 到最新版本 |

---

## 支持的工具

### 工具
| ID | 工具 | 说明 |
|----|------|------|
| `git` | Git | 分布式版本控制系统 |

### 语言 & 运行时
| ID | 工具 | 说明 |
|----|------|------|
| `uv` | uv | Python 包管理器与项目管理工具 |
| `nodejs` | Node.js | Node.js 运行时（via fnm） |
| `bun` | Bun | JavaScript/TypeScript 运行时与包管理器 |
| `rust` | Rust | Rust 编程语言（via rustup） |
| `go` | Go | Go 编程语言 |
| `jdk` | Java JDK | Adoptium Temurin JDK |
| `c` | C/C++ | GCC 编译器（MinGW-w64） |
| `miniconda` | Miniconda | Conda 包管理器（最小安装） |
| `maven` | Maven | Apache Maven 构建工具 |
| `gradle` | Gradle | Gradle 构建工具 |

### 数据库
| ID | 工具 | 说明 |
|----|------|------|
| `mysql` | MySQL | MySQL Community Server（zip 解压 + 服务注册） |
| `pgsql` | PostgreSQL | PostgreSQL 数据库（zip 解压 + 服务注册） |

### IDE
| ID | 工具 | 说明 |
|----|------|------|
| `vscode` | VS Code | Visual Studio Code 编辑器 |
| `pycharm` | PyCharm | PyCharm Community IDE |

---

## 配置文件

配置文件路径：`%USERPROFILE%\.hudo\config.toml`

```toml
# 所有工具的安装根目录
root_dir = "D:\\hudo"

[java]
version = "21"          # JDK 版本

[go]
version = "latest"      # Go 版本，"latest" 自动获取最新

[versions]
# 锁定特定工具版本，不填则自动获取最新
git     = "2.47.1.2"
fnm     = "1.38.1"
mysql   = "8.4.8"
pgsql   = "17.8"
pycharm = "2024.3.5"
maven   = "3.9.9"
gradle  = "8.12.1"

[mirrors]
# 自定义下载镜像（不填使用官方源）
uv      = "https://mirror.example.com/uv"
go      = "https://mirrors.aliyun.com/golang"
java    = "https://mirrors.tuna.tsinghua.edu.cn/Adoptium"
mysql   = "https://mirrors.aliyun.com/mysql/Downloads"
maven   = "https://mirrors.aliyun.com/apache/maven"
```

### 常用配置命令

```powershell
# 锁定 JDK 版本为 17
hudo config set java.version 17

# 使用阿里云 Go 镜像
hudo config set mirrors.go https://mirrors.aliyun.com/golang

# 锁定 MySQL 版本
hudo config set versions.mysql 8.0.40
```

---

## 安装路径

所有工具安装在 `root_dir` 下的固定目录：

```
D:\hudo\
├── tools\        # git, mysql, pgsql
├── lang\         # go, jdk, mingw, miniconda, maven, gradle
├── ide\          # vscode, pycharm
└── cache\        # 下载缓存
```

Node.js（fnm）、Rust（rustup）、uv、Bun 使用其官方安装路径。

---

## 环境档案

将当前环境导出为 TOML 文件，可在新机器上一键还原：

```powershell
# 导出
hudo export hudo-profile.toml

# 在新机器上还原
hudo import hudo-profile.toml
```

档案示例：

```toml
[hudo]
version = "0.1.0"
exported_at = "2026-02-24 10:00:00"

[tools]
git     = "2.47.1.2"
rust    = "1.85.0"
nodejs  = "v22.14.0"
uv      = "0.6.0"

[tool_config.git]
user_name  = "yourname"
user_email = "you@example.com"
```

---

## 数据库说明

### MySQL
- 安装方式：zip 解压 + 自动初始化（`--initialize-insecure`，root 无密码）
- 服务名：`MySQL`，端口 `3306`
- 连接：`mysql -u root`
- 停止服务：`net stop MySQL`

### PostgreSQL
- 安装方式：zip 解压 + `initdb`（用户 `postgres`，UTF-8，无密码）
- 服务名：`PostgreSQL`，端口 `5432`
- 连接：`psql -U postgres`
- 停止服务：`net stop PostgreSQL`

服务注册需要管理员权限，hudo 会自动弹出 UAC 提示。

---

## 自更新

```powershell
hudo update
```

检查 GitHub Releases 最新版本，有更新时自动下载并替换当前程序。

---

## 系统要求

- Windows 10 / 11（x64）
- PowerShell 5.1+（系统自带）
- 网络连接

---

## License

MIT
