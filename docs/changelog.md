# 更新日志

## v0.2.5 <Badge type="tip" text="最新" />

**新增**
- 新增 Redis 安装器：使用 redis-windows 预编译包，自动注册 Windows 服务

## v0.2.4

**新增**
- VS Code 安装后自动注册右键菜单「通过 Code 打开」

## v0.2.3

**修复**
- Node.js 安装后 `node` 命令不可用：安装时自动设置 PowerShell 执行策略（`RemoteSigned`）
- MinGW-w64 下载失败：改为从 GitHub API 动态获取最新版本，不再依赖硬编码 URL

## v0.2.2

**修复**
- 适配 Claude Code 新版 manifest 结构（platforms.checksum），修复安装时找不到执行文件

**优化**
- 补全文档站 SEO 配置（Open Graph、sitemap、robots.txt）

## v0.2.1

**修复**
- 安装脚本 (install.ps1) 改用纯 ASCII 英文，解决中文 Windows 控制台乱码
- Claude Code 安装 SHA256 校验失败时自动清除缓存重试
- 支持自动卸载系统已有的 Claude Code (npm)，解决 hudo 接管报错

## v0.2.0

**新增**
- 跨平台支持 (Linux/macOS)
- Claude Code 模型配置
- CI 构建流程

## v0.1.5

**新增**
- 新增 Google Chrome 安装器（企业版 MSI，静默安装，自动 UAC 提权）

**修复**
- 修复首次运行选择 C 盘时因权限不足无法创建安装目录的问题，自动回退到 `%USERPROFILE%\hudo`
- 修复 Profile 导出遗漏 `mysql`/`pgsql`/`maven`/`gradle` 镜像源配置
- 修复 Profile 导出未包含 `versions.*` 版本锁定字段（git/gh/fnm/mysql/pgsql/pycharm）

## v0.1.4

- 新增 Claude Code 安装器（GCS 二进制分发，含 SHA256 校验）
- 新增 `hudo cc` 命令：管理 Claude Code API 来源（Provider 增删切换）
- 主菜单新增「Claude Code API 来源」入口
- 导出/导入 profile 时自动包含 cc_providers

## v0.1.3

**新增**
- `hudo uninstall --self`：卸载 hudo 自身，可选同时删除配置和缓存

**修复**
- 修复 `hudo update` 后终端窗口自动关闭
- 修复 `hudo` 无参数运行时报 `version` 参数缺失
- 版本标志由 `-V` 改为 `-v`

## v0.1.2

- 版本标志由 `-V` 改为 `-v`
- 修复 `hudo update` 后终端窗口自动关闭
- 修复 `hudo` 无参数运行时报 `version` 参数缺失

## v0.1.1

**新增**
- GitHub CLI：`hudo install gh`，安装后自动引导登录

**修复**
- 修复 Gradle / Maven 检测失败（`.bat`/`.cmd` 需通过 `cmd /c` 执行）
- 修复 VS Code 检测：补充 `%LOCALAPPDATA%` 和 `%ProgramFiles%` 路径
- 修复分类图标在 Windows 10 控制台显示为问号（emoji → ASCII `[T][L][D][E]`）
- 修复 GitHub CLI 安装后路径检测不一致

## v0.1.0

首次发布。

**支持安装的工具（15 个）**

- 版本控制：Git
- 语言 & 运行时：Python（uv）、Node.js（fnm）、Bun、Rust（rustup）、Go、JDK（Temurin）、MinGW-w64、Miniconda
- 构建工具：Maven、Gradle
- 数据库：MySQL、PostgreSQL
- IDE：VS Code、PyCharm Community

**主要特性**

- 交互式菜单，按分类勾选后一键安装
- 自动配置环境变量，装完即用
- 版本自动获取（Git、Go、PostgreSQL、PyCharm 等）
- 环境档案导出/导入（`hudo profile export/import`）
- 数据库自动初始化并注册 Windows 服务
- `hudo update` 自更新
