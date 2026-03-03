# hudo 项目 Claude 工作指南

## 重要原则

**每执行一步前必须先向用户汇报计划，等待确认后再执行。**
不得连续执行多个破坏性或不可逆操作（如 git push、release 发布、tag 删除）而不经用户同意。

---

## 发布流程

发布新版本时，按以下步骤逐一执行，每步执行前告知用户：

1. **确认工作区干净**
   ```bash
   git status
   git log --oneline -3
   ```

2. **更新版本号**（`Cargo.toml` 中的 `version`）
   - patch 修复：0.1.x → 0.1.(x+1)
   - 新功能：0.1.x → 0.2.0

3. **提交版本号变更**（由用户自行 commit，或由 Claude 执行前告知）
   ```bash
   git add Cargo.toml Cargo.lock
   git commit -m "bump version to x.y.z"
   ```

4. **推送代码**
   ```bash
   git push origin master
   ```

5. **打 Tag 并推送**
   ```bash
   git tag vx.y.z
   git push origin vx.y.z
   ```
   若 tag 已存在需要移动（含新提交），先删除再重建：
   ```bash
   git tag -d vx.y.z
   git push origin :refs/tags/vx.y.z
   git tag vx.y.z
   git push origin vx.y.z
   ```

6. **构建 Release 二进制**
   ```bash
   cargo build --release
   # 产物：target/release/hudo.exe
   ```

7. **创建 GitHub Release**（使用 gh CLI）
   ```bash
   "D:/hudo/tools/gh/gh.exe" release create vx.y.z target/release/hudo.exe \
     --title "hudo vx.y.z" \
     --notes "..."
   ```
   注意：gh.exe 在 `D:\hudo\tools\gh\gh.exe`（不在 bin/ 子目录）

---

## 项目结构

```
src/
├── main.rs          # CLI 路由、交互菜单、install/uninstall/list 等命令
├── cli.rs           # clap CLI 定义
├── config.rs        # HudoConfig、VersionConfig、MirrorConfig
├── ui.rs            # 输出样式、ToolCategory
├── version.rs       # 各工具版本查询（GitHub API / 官方 API）
├── download.rs      # 下载、解压工具函数
├── registry.rs      # state.json 安装记录
├── env.rs           # 环境变量写入（User PATH / 系统变量）
├── profile.rs       # export/import 档案
└── installer/
    ├── mod.rs       # Installer trait、共享服务工具（run_as_admin 等）
    ├── git.rs
    ├── gh.rs        # GitHub CLI
    ├── go.rs
    ├── jdk.rs
    ├── maven.rs
    ├── gradle.rs
    ├── nodejs.rs
    ├── bun.rs
    ├── rustup.rs
    ├── uv.rs
    ├── mingw.rs
    ├── miniconda.rs
    ├── mysql.rs
    ├── pgsql.rs
    ├── vscode.rs
    ├── pycharm.rs
    ├── chrome.rs
    └── claude_code.rs
```

---

## 文档同步规则

**每次修改代码后，必须同步更新 `docs/` 文档：**

- 新增安装器 → 在 `docs/tools/` 创建对应工具页，并更新 `docs/tools/index.md` 和 `docs/.vitepress/config.ts` sidebar
- 修改工具行为/路径/版本逻辑 → 更新对应 `docs/tools/<name>.md`
- 新增 CLI 命令 → 更新 `docs/guide/quickstart.md`
- 修改配置字段 → 更新 `docs/guide/config.md`
- 发布新版本 → 更新 `docs/changelog.md` 和 `docs/.vitepress/config.ts` 中的版本号

**每次修改 CLAUDE.md 规则或项目结构时，同步更新本文件，保证 AI 不读代码也能了解项目全貌。**

---

## 项目概览（快速参考）

- **名称**：hudo（混沌）— Windows 开发环境一键引导工具
- **语言**：Rust，edition 2021，Windows 平台
- **当前版本**：0.1.5
- **GitHub**：`zexadev/hudo`
- **品牌**：Zexa（zexa.cc）

### 安装方式
```powershell
irm https://raw.githubusercontent.com/zexadev/hudo/master/install.ps1 | iex
```

### 目录约定
- 配置文件：`%USERPROFILE%\.hudo\config.toml`
- 安装根目录：`{用户选的盘}:\hudo\`
  - `tools/` — CLI 工具（git, gh, fnm, rustup, uv, bun, mingw64, claude-code）
  - `lang/` — 语言环境（go, java, cargo, gopath, pgsql, mysql）
  - `ide/` — IDE（vscode, pycharm）
  - `cache/` — 下载缓存

### 支持的工具（19 个）
| 分类 | 工具 ID |
|------|---------|
| 版本控制 | git, gh |
| 运行时 | nodejs, bun, uv（Python）, miniconda, go, rust |
| JVM | jdk, maven, gradle |
| 数据库 | mysql, pgsql |
| IDE | vscode, pycharm |
| 系统工具 | c（MinGW）, chrome, claude-code |

### 核心架构
- `Installer` trait：每个工具实现 `info / detect_installed / resolve_download / install / env_actions / configure`
- `DetectResult`：`NotInstalled / InstalledByHudo / InstalledExternal`
- 版本查询：运行时动态调用 GitHub API / 官方 API，失败时回退到 `DEFAULT_VERSION` 常量
- 环境变量：写入 `HKCU\Environment`，无需管理员权限

### 文档站
- 框架：VitePress，位于 `docs/`
- 主题：自定义 `HomePage.vue`（`layout: page`），不使用 VitePress 默认 home layout
- 部署：Cloudflare Pages，域名 zexa.cc

---

## 新增安装器清单

添加一个新工具需要：

1. 新建 `src/installer/<name>.rs`，实现 `Installer` trait
2. `src/installer/mod.rs`：`pub mod <name>;` + 加入 `all_installers()`
3. `src/config.rs`：`VersionConfig` 和/或 `MirrorConfig` 添加字段（如需）
4. `src/version.rs`：添加版本查询函数（如需）
5. `src/ui.rs`：`ToolCategory::from_id()` 添加 id 映射

---

## 已知注意事项

- **`.bat`/`.cmd` 文件**不能直接用 `Command::new()` 执行，必须通过 `cmd /c <file>` 调用（影响 gradle、maven 检测）
- **emoji 图标**在 Windows 10 旧控制台不支持，使用 ASCII `[T][L][D][E]` 代替
- **gh.exe 路径**：安装后在 `tools/gh/gh.exe`（不在 `bin/` 子目录），检测和 PATH 已兼容两种结构
- **Windows 服务注册**（MySQL/PostgreSQL）：`mysqld --install` 退出码不可信，必须用 `sc query` 二次验证；服务注册和停止需要 UAC 提权（`run_as_admin`）
- **`reg.save()` 必须在 `configure()` 之前**：否则 configure 失败时工具不会被记录到 state.json
- **`detect_all_parallel`** 用于卸载列表，不能用 `fast_detect`（后者只读 state.json）
- **gh auth token** 不导出到 profile 文件（安全考虑），新设备安装后自动引导 `gh auth login`
