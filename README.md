# hudo 混沌

**Windows 开发环境一键引导工具** · [官网与文档 hudo.zexa.cc](https://hudo.zexa.cc)

一条命令装好 26 款开发工具（Git、Node.js、Rust、Go、JDK、Python、MySQL、VS Code 等），自动配置环境变量。免管理员权限、国内镜像自动回退、环境档案换机一键还原。

> **hudo** is a one-command Windows dev environment bootstrapper: installs 26 developer tools (Git, Node.js, Rust, Go, JDK, Python, MySQL, VS Code, ...) with automatic PATH/env configuration, no admin rights required, China-mirror fallback, and profile export/import for one-command machine migration. Docs: [hudo.zexa.cc](https://hudo.zexa.cc)

> 官方仓库 / Official repository: [github.com/zexadev/hudo](https://github.com/zexadev/hudo) —— 发版与 issue 都在这里，各 fork 不由本仓维护。

![hudo 交互式主菜单](docs/public/img/blog/hudo-menu.png)

---

## 安装

```powershell
irm hudo.zexa.cc/install.ps1 | iex
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
| `hudo upgrade [工具]` | 升级 hudo 安装的工具（省略工具名检查全部） |
| `hudo update` | 更新 hudo 到最新版本 |
| `-y` / `--yes` | 非交互模式（任意命令后附加），跳过确认提示 |

---

## 支持的工具（26 个）

### 工具
| ID | 工具 | 说明 |
|----|------|------|
| `git` | Git | 分布式版本控制系统 |
| `gh` | GitHub CLI | GitHub 命令行工具 |
| `claude-code` | Claude Code | Anthropic AI 编程助手 |
| `7zip` | 7-Zip | 压缩/解压工具（便携免管理员） |
| `pwsh` | PowerShell 7 | 现代跨平台 PowerShell（便携版） |
| `powertoys` | PowerToys | 微软官方效率工具集（用户级静默安装） |
| `omp` | Oh My Posh | 终端主题引擎 + Nerd Font 字体 |

### 语言 & 运行时
| ID | 工具 | 说明 |
|----|------|------|
| `nodejs` | Node.js | Node.js 运行时 |
| `fnm` | fnm | Node.js 多版本管理器 |
| `bun` | Bun | JavaScript/TypeScript 运行时 |
| `uv` | uv | Python 包管理器 |
| `miniconda` | Miniconda | Conda 包管理器（最小安装） |
| `rust` | Rust | Rust 编程语言（via rustup） |
| `go` | Go | Go 编程语言 |
| `jdk` | Java JDK | Adoptium Temurin JDK |
| `c` | C/C++ | GCC 编译器（MinGW-w64） |
| `dotnet` | .NET SDK | C#/F# 开发工具链（默认 LTS） |
| `maven` | Maven | Apache Maven 构建工具 |
| `gradle` | Gradle | Gradle 构建工具 |

### 数据库
| ID | 工具 | 说明 |
|----|------|------|
| `mysql` | MySQL | MySQL Community Server |
| `pgsql` | PostgreSQL | PostgreSQL 数据库 |
| `redis` | Redis | Redis 内存数据库 |

### IDE & 浏览器
| ID | 工具 | 说明 |
|----|------|------|
| `vscode` | VS Code | Visual Studio Code 编辑器 |
| `pycharm` | PyCharm | PyCharm Community IDE |
| `idea` | IntelliJ IDEA | IntelliJ IDEA Community IDE |
| `chrome` | Chrome | Google Chrome 浏览器 |

---

## 特色功能

### 国内镜像自动回退

原地址连接失败时，自动切换到国内镜像下载，无需手动配置：

- Git / Node.js → npmmirror
- Go → golang.google.cn
- JDK / Maven / Gradle → 华为云
- Rust → USTC
- Miniconda → TUNA
- VS Code → Azure 中国 CDN
- Claude Code → npm + npmmirror

### 安装后自动配置

- Maven → 自动生成 `~/.m2/settings.xml`（阿里云镜像）
- Gradle → 自动生成 `~/.gradle/init.gradle`（阿里云仓库）
- Miniconda → 自动 `conda init`（cmd + PowerShell）
- Git → 自动设置 `core.autocrlf=true`
- PostgreSQL → 自动设置 `PGDATA` 环境变量
- Rust → 自动配置 USTC 工具链镜像

### 环境档案迁移

```powershell
# 旧电脑导出
hudo export mysetup.toml

# 新电脑一键恢复
hudo import mysetup.toml
```

---

## 配置文件

配置文件路径：`%USERPROFILE%\.hudo\config.toml`

```toml
root_dir = "D:\\hudo"

[java]
version = "21"

[go]
version = "latest"

[versions]
git     = "2.53.0"
nodejs  = "24.14.1"
mysql   = "8.4.8"
pgsql   = "17.8"

[mirrors]
# 自定义下载镜像（不填使用官方源 + 自动回退国内镜像）
go   = "https://golang.google.cn/dl"
java = "https://mirrors.huaweicloud.com/openjdk"
```

---

## 安装路径

```
D:\hudo\
├── tools\        # git, gh, mysql, pgsql, redis, maven, gradle, miniconda, claude-code
├── lang\         # go, java, node, cargo, gopath
├── ide\          # vscode, pycharm
└── cache\        # 下载缓存
```

---

## 文档 & 博客

- 📖 文档站：[hudo.zexa.cc](https://hudo.zexa.cc)
- 📝 博客教程：[hudo.zexa.cc/blog](https://hudo.zexa.cc/blog/)

---

## 系统要求

- Windows 10 / 11（x64）
- PowerShell 5.1+（系统自带）
- 网络连接

---

## License

MIT
