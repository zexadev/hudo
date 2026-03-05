#!/bin/sh
# hudo 安装脚本 (macOS / Linux)
# 用法: curl -fsSL https://hudo.zexa.cc/install.sh | bash
set -e

REPO="zexadev/hudo"
INSTALL_DIR="$HOME/.hudo/bin"

# ── 颜色 ──────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    CYAN='\033[0;36m'
    DIM='\033[2m'
    RESET='\033[0m'
else
    RED='' GREEN='' CYAN='' DIM='' RESET=''
fi

info()    { printf "  ${CYAN}► %s${RESET}\n" "$1"; }
success() { printf "  ${GREEN}✓ %s${RESET}\n" "$1"; }
error()   { printf "  ${RED}✗ %s${RESET}\n" "$1"; exit 1; }

# ── Banner ────────────────────────────────────────────────────────────────────
echo ""
printf "  ${DIM}███████████████████████████████████████████${RESET}\n"
printf "    ${CYAN}hudo  混沌 —— 开发环境一键引导工具${RESET}\n"
printf "  ${DIM}███████████████████████████████████████████${RESET}\n"
echo ""

# ── 1. 检测平台和架构 ─────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) PLATFORM="apple-darwin" ;;
    Linux)
        # 检测 musl vs glibc
        if ldd --version 2>&1 | grep -qi musl; then
            PLATFORM="unknown-linux-musl"
        elif [ -f /lib/libc.musl-*.so.1 ] 2>/dev/null; then
            PLATFORM="unknown-linux-musl"
        else
            PLATFORM="unknown-linux-gnu"
        fi
        ;;
    *)      error "不支持的操作系统: $OS" ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH="x86_64" ;;
    arm64|aarch64)   ARCH="aarch64" ;;
    *)               error "不支持的架构: $ARCH" ;;
esac

TARGET="${ARCH}-${PLATFORM}"
ASSET_NAME="hudo-${TARGET}"

info "平台: ${OS} ${ARCH} (${TARGET})"

# ── 2. 获取最新版本 ───────────────────────────────────────────────────────────
info "获取最新版本..."

if command -v curl >/dev/null 2>&1; then
    RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        -H "User-Agent: hudo-installer")" || error "无法连接 GitHub API，请检查网络连接"
elif command -v wget >/dev/null 2>&1; then
    RELEASE_JSON="$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
        --header="User-Agent: hudo-installer")" || error "无法连接 GitHub API，请检查网络连接"
else
    error "需要 curl 或 wget"
fi

# 提取版本号（兼容无 jq 环境）
if command -v jq >/dev/null 2>&1; then
    VERSION="$(echo "$RELEASE_JSON" | jq -r '.tag_name' | sed 's/^v//')"
    DOWNLOAD_URL="$(echo "$RELEASE_JSON" | jq -r ".assets[] | select(.name == \"${ASSET_NAME}\") | .browser_download_url")"
else
    VERSION="$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"v\([^"]*\)".*/\1/')"
    DOWNLOAD_URL="$(echo "$RELEASE_JSON" | grep "browser_download_url" | grep "${ASSET_NAME}\"" | head -1 | sed 's/.*"\(https[^"]*\)".*/\1/')"
fi

if [ -z "$VERSION" ]; then
    error "无法获取版本号"
fi

if [ -z "$DOWNLOAD_URL" ]; then
    error "Release v${VERSION} 中未找到 ${ASSET_NAME}，请检查发布资产"
fi

success "最新版本: v${VERSION}"

# ── 3. 检测是否已安装 ─────────────────────────────────────────────────────────
EXE_PATH="${INSTALL_DIR}/hudo"

if [ -f "$EXE_PATH" ]; then
    CURRENT_VER="$("$EXE_PATH" --version 2>/dev/null | sed 's/^hudo //' || echo "")"
    if [ "$CURRENT_VER" = "$VERSION" ]; then
        success "已是最新版本 v${VERSION}，无需更新"
        echo ""
        exit 0
    fi
    if [ -n "$CURRENT_VER" ]; then
        info "升级: v${CURRENT_VER} → v${VERSION}"
    else
        info "重新安装 v${VERSION}"
    fi
else
    info "安装 v${VERSION} 到 ${INSTALL_DIR}"
fi

# ── 4. 下载 ───────────────────────────────────────────────────────────────────
mkdir -p "$INSTALL_DIR"
TMP_PATH="$(mktemp)"

info "下载中..."
if command -v curl >/dev/null 2>&1; then
    curl -fSL "$DOWNLOAD_URL" -o "$TMP_PATH" || error "下载失败"
else
    wget -q "$DOWNLOAD_URL" -O "$TMP_PATH" || error "下载失败"
fi

# ── 5. 安装 ───────────────────────────────────────────────────────────────────
chmod +x "$TMP_PATH"
mv -f "$TMP_PATH" "$EXE_PATH"

# ── 6. 添加到 PATH ───────────────────────────────────────────────────────────
add_to_path() {
    local profile="$1"
    local line="export PATH=\"${INSTALL_DIR}:\$PATH\""

    if [ -f "$profile" ] && grep -qF "$INSTALL_DIR" "$profile" 2>/dev/null; then
        return 0
    fi

    echo "" >> "$profile"
    echo "# hudo" >> "$profile"
    echo "$line" >> "$profile"
    info "已添加到 $profile"
}

SHELL_NAME="$(basename "${SHELL:-/bin/sh}")"
case "$SHELL_NAME" in
    zsh)  add_to_path "$HOME/.zshrc" ;;
    bash)
        if [ -f "$HOME/.bashrc" ]; then
            add_to_path "$HOME/.bashrc"
        else
            add_to_path "$HOME/.profile"
        fi
        ;;
    fish)
        FISH_CONF="$HOME/.config/fish/config.fish"
        mkdir -p "$(dirname "$FISH_CONF")"
        if ! grep -qF "$INSTALL_DIR" "$FISH_CONF" 2>/dev/null; then
            echo "" >> "$FISH_CONF"
            echo "# hudo" >> "$FISH_CONF"
            echo "set -gx PATH ${INSTALL_DIR} \$PATH" >> "$FISH_CONF"
            info "已添加到 $FISH_CONF"
        fi
        ;;
    *)    add_to_path "$HOME/.profile" ;;
esac

# ── 7. 完成 ───────────────────────────────────────────────────────────────────
echo ""
success "hudo v${VERSION} 安装完成！"
printf "  ${DIM}重启终端后运行 hudo 开始使用${RESET}\n"
echo ""
