# hudo 安装脚本
# 用法: irm https://raw.githubusercontent.com/YOUR_GITHUB_USERNAME/hudo/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo      = "huancheng01/hudo"
$installDir = "$env:USERPROFILE\.hudo\bin"

Write-Host ""
Write-Host "  ███████████████████████████████████████████" -ForegroundColor DarkGray
Write-Host "    hudo  混沌 —— 开发环境一键引导工具" -ForegroundColor Cyan
Write-Host "  ███████████████████████████████████████████" -ForegroundColor DarkGray
Write-Host ""

# ── 1. 获取最新 Release ──────────────────────────────────────────────────────
Write-Host "  ► 获取最新版本..." -ForegroundColor Cyan
try {
    $headers = @{ "User-Agent" = "hudo-installer" }
    $release = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/$repo/releases/latest" `
        -Headers $headers `
        -ErrorAction Stop
} catch {
    Write-Host "  ✗ 无法连接 GitHub API，请检查网络连接" -ForegroundColor Red
    Write-Host "    $_" -ForegroundColor DarkGray
    exit 1
}

$version = $release.tag_name.TrimStart('v')
$asset   = $release.assets | Where-Object { $_.name -eq "hudo.exe" } | Select-Object -First 1

if (-not $asset) {
    Write-Host "  ✗ Release v$version 中未找到 hudo.exe，请检查发布资产" -ForegroundColor Red
    exit 1
}

$downloadUrl = $asset.browser_download_url
Write-Host "  ✓ 最新版本: v$version" -ForegroundColor Green

# ── 2. 检测是否已安装 ────────────────────────────────────────────────────────
$exePath = "$installDir\hudo.exe"
if (Test-Path $exePath) {
    try {
        $currentVer = (& $exePath --version 2>$null) -replace '^hudo\s+', ''
        if ($currentVer -eq $version) {
            Write-Host "  ✓ 已是最新版本 v$version，无需更新" -ForegroundColor Green
            Write-Host ""
            exit 0
        }
        Write-Host "  ► 升级: v$currentVer → v$version" -ForegroundColor Cyan
    } catch {
        Write-Host "  ► 重新安装 v$version" -ForegroundColor Cyan
    }
} else {
    Write-Host "  ► 安装 v$version 到 $installDir" -ForegroundColor Cyan
}

# ── 3. 下载 hudo.exe ─────────────────────────────────────────────────────────
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
$tmpPath = "$env:TEMP\hudo-install.exe"

Write-Host "  ► 下载中..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tmpPath -UseBasicParsing -ErrorAction Stop
} catch {
    Write-Host "  ✗ 下载失败: $_" -ForegroundColor Red
    exit 1
}

# ── 4. 安装（原子替换）──────────────────────────────────────────────────────
Move-Item -Force $tmpPath $exePath

# ── 5. 添加到用户 PATH ───────────────────────────────────────────────────────
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    Write-Host "  ► 已添加 $installDir 到用户 PATH" -ForegroundColor Cyan
}

# ── 6. 完成 ──────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ✓ hudo v$version 安装完成！" -ForegroundColor Green
Write-Host "  重启终端后运行 hudo 开始使用" -ForegroundColor DarkGray
Write-Host ""
