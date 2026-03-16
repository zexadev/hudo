# hudo installer
# Usage: irm hudo.zexa.cc/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo      = "zexadev/hudo"
$installDir = "$env:USERPROFILE\.hudo\bin"

Write-Host ""
Write-Host "  ==========================================" -ForegroundColor DarkGray
Write-Host "    hudo - Dev Environment Bootstrap Tool" -ForegroundColor Cyan
Write-Host "  ==========================================" -ForegroundColor DarkGray
Write-Host ""

# ── 1. Fetch latest release ──────────────────────────────────────────────────
Write-Host "  > Fetching latest version..." -ForegroundColor Cyan
try {
    $headers = @{ "User-Agent" = "hudo-installer" }
    $release = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/$repo/releases/latest" `
        -Headers $headers `
        -ErrorAction Stop
} catch {
    Write-Host "  x Failed to reach GitHub API, check your network" -ForegroundColor Red
    Write-Host "    $_" -ForegroundColor DarkGray
    return
}

$version = $release.tag_name.TrimStart('v')
$asset   = $release.assets | Where-Object { $_.name -eq "hudo-x86_64-pc-windows-msvc.exe" } | Select-Object -First 1

if (-not $asset) {
    Write-Host "  x hudo-x86_64-pc-windows-msvc.exe not found in release v$version" -ForegroundColor Red
    return
}

$downloadUrl = $asset.browser_download_url
Write-Host "  + Latest version: v$version" -ForegroundColor Green

# ── 2. Check existing installation ───────────────────────────────────────────
$exePath = "$installDir\hudo.exe"
if (Test-Path $exePath) {
    try {
        $currentVer = (& $exePath --version 2>$null) -replace '^hudo\s+', ''
        if ($currentVer -eq $version) {
            Write-Host "  + Already up to date (v$version)" -ForegroundColor Green
            Write-Host ""
            return
        }
        Write-Host "  > Upgrading: v$currentVer -> v$version" -ForegroundColor Cyan
    } catch {
        Write-Host "  > Reinstalling v$version" -ForegroundColor Cyan
    }
} else {
    Write-Host "  > Installing v$version to $installDir" -ForegroundColor Cyan
}

# ── 3. Download hudo.exe ─────────────────────────────────────────────────────
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
$tmpPath = "$env:TEMP\hudo-install.exe"

Write-Host "  > Downloading..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tmpPath -UseBasicParsing -ErrorAction Stop
} catch {
    Write-Host "  x Download failed: $_" -ForegroundColor Red
    return
}

# ── 4. Install (atomic replace) ──────────────────────────────────────────────
Unblock-File -Path $tmpPath
Move-Item -Force $tmpPath $exePath

# ── 5. Add to user PATH ─────────────────────────────────────────────────────
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    Write-Host "  > Added $installDir to user PATH" -ForegroundColor Cyan
}

# ── 6. Done ──────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  + hudo v$version installed successfully!" -ForegroundColor Green
Write-Host "  Restart your terminal and run 'hudo' to get started" -ForegroundColor DarkGray
Write-Host ""
