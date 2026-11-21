#!/bin/sh
# tools/install.sh
#
# pengj-templates-cli One-Liner Quick Installer & Updater for Linux / macOS
#
# Usage:
#   curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh
#
# The installer ships the CLI and the template tree together: the template
# engine reads `templates/` at runtime, so a binary without its templates is
# useless. The commit gate rides along in the same binary.

set -eu

# 0. Language: Chinese locale -> Chinese messages, everything else -> English.
#    The installer ships standalone (it is not a rendered template), so it
#    detects the locale at runtime instead of reading `skill_lang`.
case "${LC_ALL:-${LC_MESSAGES:-${LANG:-}}}" in
    zh*) LANG_IS_ZH=1 ;;
    *)   LANG_IS_ZH=0 ;;
esac
# t <zh> <en>: print the message for the active locale.
t() { if [ "$LANG_IS_ZH" = "1" ]; then printf '%s' "$1"; else printf '%s' "$2"; fi; }

REPO="pengjiaxusz/pengj-templates"
BASE_URL="https://github.com/$REPO/releases/latest/download"

# 1. Detect the platform and the architecture.
os=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$os" in
    linux) os="linux" ;;
    darwin) os="macos" ;;
    *)
        echo "[install] $(t '不支持的平台:' 'unsupported platform:') $(uname -s)" >&2
        exit 1
        ;;
esac

case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="arm64" ;;
    *)
        echo "[install] $(t '不支持的架构:' 'unsupported architecture:') $(uname -m)" >&2
        exit 1
        ;;
esac

# 2. Resolve the install layout: <install root>/bin/pengj-templates-cli plus
#    <install root>/templates. The CLI looks for the templates next to itself.
if [ -n "${PENGJ_TEMPLATES_INSTALL_DIR:-}" ]; then
    install_root="$PENGJ_TEMPLATES_INSTALL_DIR"
else
    install_root="${XDG_DATA_HOME:-${HOME}/.local/share}/pengj-templates"
fi

bin_dir="$install_root/bin"
templates_dir="$install_root/templates"
exe_path="$bin_dir/pengj-templates-cli"

echo ""
echo "================================================================"
echo "        pengj-templates-cli $(t '安装 / 更新程序' 'Installer / Updater') ($os)          "
echo "================================================================"
echo ""
echo "--> $(t '平台' 'Platform'): $os-$arch"
echo "--> $(t '安装根目录' 'Install root'): $install_root"

mkdir -p "$bin_dir"

# 3. Download the portable bundle (CLI + templates).
tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t pengj-templates)
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

bundle="$tmp_dir/pengj-templates-cli-$os-$arch.tar.gz"
url="$BASE_URL/pengj-templates-cli-$os-$arch.tar.gz"

echo "--> $(t '正在下载' 'Downloading') $url"
if command -v curl >/dev/null 2>&1; then
    curl -fLs --connect-timeout 15 --max-time 180 -o "$bundle" "$url"
elif command -v wget >/dev/null 2>&1; then
    wget -q --timeout=30 -O "$bundle" "$url"
else
    echo "[install] $(t 'curl 和 wget 均不可用' 'neither curl nor wget is available')" >&2
    exit 1
fi

# 4. Unpack the bundle.
tar -xzf "$bundle" -C "$tmp_dir"
bundle_root="$tmp_dir/pengj-templates"
[ -d "$bundle_root" ] || bundle_root="$tmp_dir"

# 5. Deploy: CLI into bin/, templates next to it.
cp -f "$bundle_root/pengj-templates-cli" "$exe_path"
chmod +x "$exe_path"
rm -rf "$templates_dir"
cp -R "$bundle_root/templates" "$templates_dir"

echo "--> $(t '已安装 CLI' 'Installed CLI'): $exe_path"
echo "--> $(t '已安装模板' 'Installed templates'): $templates_dir"

# 6. Make the CLI reachable: a symlink in a directory that is already on PATH.
link_dir=""
for candidate in "${HOME}/.local/bin" "${HOME}/bin" "/usr/local/bin"; do
    if [ -d "$candidate" ] && [ -w "$candidate" ]; then
        link_dir="$candidate"
        break
    fi
done

if [ -n "$link_dir" ]; then
    ln -sf "$exe_path" "$link_dir/pengj-templates-cli"
    echo "--> $(t '已链接' 'Linked'): $link_dir/pengj-templates-cli"
else
    echo "--> $(t '未找到可写的 PATH 目录，请手动将' 'No writable PATH directory found; add') $bin_dir $(t '加入 PATH。' 'to your PATH manually.')" >&2
fi

# 7. Verify.
echo ""
if "$exe_path" --version >/dev/null 2>&1; then
    echo "[OK] pengj-templates-cli $(t '已安装' 'installed'): $("$exe_path" --version)"
else
    echo "[install] $(t 'CLI 已安装但无法执行' 'the CLI was installed but could not be executed')" >&2
    exit 1
fi

# 8. The commit gate is activated per project.
#
# Git keeps hook activation in .git/config, which does not travel with a
# clone. The project's own environment check (.pengj-templates/scripts/doctor.sh --fix) is the
# single activation entry point: it works with or without this CLI and only
# touches repositories that ship .githooks/.
echo ""
echo "$(t '提交门禁随此二进制一起发布，按项目分别激活:' 'The commit gate ships inside this binary, activated per project:')"
echo "  just doctor --fix                 # $(t '在已生成的项目内' 'inside a generated project')"
echo "  sh .pengj-templates/scripts/doctor.sh --fix        # $(t '没有 just 时' 'without just')"
echo "  pengj-templates-cli gate check    # $(t '直接运行代码语言守卫' 'run the code language guard directly')"

# 9. Next time, upgrade from the CLI itself.
echo ""
echo "$(t '以后升级不用再跑这个脚本:' 'Next time, upgrade without this script:')"
echo "  pengj-templates-cli self update   # $(t '二进制与 templates 一起换' 'binary and templates replaced together')"
