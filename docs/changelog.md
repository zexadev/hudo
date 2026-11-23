---
title: "更新日志"
description: "hudo 版本更新历史记录，包含每个版本的新功能、优化改进和问题修复详情。"
head:
  - - meta
    - name: keywords
      content: "hudo 更新日志, hudo changelog, hudo 版本历史, hudo 新功能"
---

# 更新日志

## v0.4.1 <Badge type="tip" text="最新" />

**修复**
- 在 PowerShell 7 中运行 hudo 时，uv / .NET SDK 等依赖 Windows PowerShell 脚本的安装失败（报 `Get-ExecutionPolicy` 无法加载模块 Microsoft.PowerShell.Security）。原因是子进程继承了 pwsh 7 的 `PSModulePath`，现启动前清除该变量（#1）

## v0.4.0

**新功能**
- 新增 6 款工具：IntelliJ IDEA、7-Zip、PowerShell 7、.NET SDK、PowerToys、Oh My Posh（捆绑 Nerd Font 与 profile 注入），支持工具总数达到 26 款
- `hudo upgrade`：基于精确安装记录批量升级已装工具；MySQL/PostgreSQL/Redis/Miniconda 等数据在安装目录内的工具排除在外，避免误清空
- 全局 `-y`/`--yes` 非交互模式：推进类确认自动确认，可选分支走安全默认，适合脚本化批量部署
- 全局代理配置：作用于版本查询、下载、自更新全部网络出口
- `install.ps1` 支持档案参数：新机器一条命令装 hudo 并自动导入档案、批量安装工具，无人值守还原整套开发环境
- bun/uv/vscode/miniconda 支持版本锁定

**优化**
- MySQL 补动态版本查询，与 PostgreSQL/Redis 的版本解析链对齐
- 接管入口提示更清晰：setup 图例、import 跳过行提示、list 汇总补充操作指引

**修复**
- 档案导出遗漏 7 个配置键，键表统一收敛为 config 单一来源
- config.toml 键名按 serde rename 对齐文档，避免键名与文档不一致

## v0.3.0

**新功能**
- Claude Code API 来源管理全面升级：Provider 支持查看详情与逐字段编辑（回车保留原值），API Key 改用不回显输入、展示时脱敏；添加后可一键切换；删除激活中的 Provider 时提示恢复默认
- 配置菜单新增「设置固定版本」入口；`config set/show` 支持全部 mirrors/versions 键（含 maven/gradle/mysql/pgsql/gh 等），未知键报错列出全部可用键

**优化**
- 交互全面翻新：确认/输入框统一彩色主题；工具检测、档案导出、解压、静默安装等长操作显示 spinner 或进度条；下载进度条增加百分比与速率
- 清屏不再清空终端滚动缓冲：页面切换不闪屏，安装日志可随时回看
- 版本查询失败时明确提示「使用内置默认版本」，不再静默回退
- MySQL/PostgreSQL/Redis 安装完成后以面板汇总连接地址、端口、账号与密码状态；子进程输出全部捕获，不再与 spinner 花屏
- 关键行动提示（如「请打开新终端」）改为醒目样式，单工具安装也会提示

**修复**
- 菜单内操作出错不再导致整个程序退出：报错后回到菜单；批量安装中止走正常汇总而非报错退出
- 菜单里设置镜像/导入档案后本次会话立即生效（原需重启 hudo）
- 菜单导入档案改为询问文件路径，文件不存在时不再崩溃退出；确认后才写入配置，取消无副作用
- 拒绝由 hudo 接管系统已装工具后，不再错误执行 hudo 路径配置
- configure 阶段失败（如 UAC 被拒）不再误报「安装失败」，提示可重新配置
- claude-code 通过 npm 回退安装后可被正确检测与卸载（兼容 claude.cmd）；登录指引修正为 `/login`
- 下载增加 60 秒读超时，断流不再永久冻结，超时可触发镜像回退
- 菜单「查看已安装」可显示系统安装的工具，不再恒显「系统: 0」
- state.json 损坏时先备份为 .bak 再重置；接管清理时检测机器级 PATH 残留并明确告知

## v0.2.14

**修复**
- 升级用户卡点：旧版 hudo (≤0.2.12) 用 `nodejs` id 记录 fnm，`hudo uninstall fnm` 找不到、`hudo uninstall nodejs` 不清 `lang/node/` 和 PowerShell profile
- `hudo uninstall nodejs` 现在识别旧版 fnm 条目，一并清理 `lang/node/`、`tools/fnm/`、`FNM_DIR` 环境变量和 profile 中的 fnm 初始化行
- `hudo uninstall fnm` 卸载时同步清理 PowerShell profile 和 `lang/node-fnm/`
- `hudo install nodejs` / `hudo install fnm` 检测到旧版 fnm 残留时，交互确认后自动迁移
- 新增 `force_remove_dir_all` 工具：递归清除只读属性、junction 回退 `cmd rd /s /q`

## v0.2.13

**新功能**
- 新增 fnm 安装器，作为 Node.js 多版本管理选项与纯 `nodejs` 安装器并存
- fnm 管理目录独立为 `lang/node-fnm/`，与 `lang/node/` 解耦，可同时安装互不干扰

**优化**
- Node.js 安装在目录被 fnm 占用或文件锁定时给出明确错误提示，并指引清理步骤

## v0.2.12

**修复**
- `hudo update` 自更新增加下载完整性校验：检查 HTTP 状态码和 PE 文件头，防止网络异常导致损坏的二进制替换正常程序

## v0.2.11

**新功能**
- Claude Code API 来源管理新增「恢复默认」选项，一键清除自定义配置回到官方默认

## v0.2.10

**修复**
- Redis 安装 404 错误：redis-windows 新版 release tag 不再带 `.1` 后缀，URL 拼接已适配

## v0.2.9

**优化**
- `hudo list` 默认只读 state.json，毫秒级响应，不再启动子进程检测
- `hudo list --all` 保持完整并行检测
- 菜单 emoji 全部替换为 ASCII 图标，兼容 Windows 10 旧控制台
- 恢复大 Logo ASCII art，硬编码 + 蓝紫逐行渐变色

**修复**
- CI release notes 提取失败（CRLF 换行符 + awk 范围匹配问题）

## v0.2.8

**改进**
- 恢复大 Logo ASCII art，硬编码 + 蓝紫逐行渐变色，移除 figlet-rs 运行时依赖
- README 全面更新，补全所有工具和特色功能介绍
- 文档站 SEO 优化：完善 meta/OG 标签、Schema.org 结构化数据、favicon
- 新增博客栏目，21 篇教程文章覆盖全部工具

## v0.2.7

**改进**
- 下载自动回退国内镜像：Git/Node.js/Go/JDK/Maven/Gradle/Rust/Miniconda/VS Code，原地址连接失败时自动切换
- Claude Code 安装支持 npm 镜像回退，GCS 不可达时通过 npmmirror 安装
- Miniconda 安装后自动执行 `conda init`，`conda activate` 开箱即用
- Maven 安装后自动生成 `settings.xml`，配置阿里云中央仓库镜像
- Gradle 安装后自动生成 `init.gradle`，配置阿里云仓库镜像
- PostgreSQL 安装后自动设置 `PGDATA` 环境变量
- Git 安装后自动设置 `core.autocrlf=true`
- Rust 安装时自动使用 USTC 镜像下载工具链
- CLI 输出美化：精简 Banner、边框标题、点线列表、boxed 安装摘要
- 移除 figlet-rs 依赖，减少二进制体积

## v0.2.6

**改进**
- Node.js 安装改为直接下载官方预编译包，移除 fnm 依赖，CMD/PowerShell/Git Bash 均可直接使用

## v0.2.5

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
