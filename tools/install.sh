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

REPO="pengjiaxusz/pengj-templates"
BASE_URL="https://github.com/$REPO/releases/latest/download"

# 1. Detect the platform and the architecture.
os=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$os" in
    linux) os="linux" ;;
    darwin) os="macos" ;;
    *)
        echo "[install] unsupported platform: $(uname -s)" >&2
        exit 1
        ;;
esac

case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="arm64" ;;
    *)
        echo "[install] unsupported architecture: $(uname -m)" >&2
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
echo "        pengj-templates-cli Installer / Updater ($os)          "
echo "================================================================"
echo ""
echo "--> Platform: $os-$arch"
echo "--> Install root: $install_root"

mkdir -p "$bin_dir"

# 3. Download the portable bundle (CLI + templates).
tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t pengj-templates)
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

bundle="$tmp_dir/pengj-templates-cli-$os-$arch.tar.gz"
url="$BASE_URL/pengj-templates-cli-$os-$arch.tar.gz"

echo "--> Downloading $url"
if command -v curl >/dev/null 2>&1; then
    curl -fLs --connect-timeout 15 --max-time 180 -o "$bundle" "$url"
elif command -v wget >/dev/null 2>&1; then
    wget -q --timeout=30 -O "$bundle" "$url"
else
    echo "[install] neither curl nor wget is available" >&2
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

echo "--> Installed CLI: $exe_path"
echo "--> Installed templates: $templates_dir"

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
    echo "--> Linked: $link_dir/pengj-templates-cli"
else
    echo "--> No writable PATH directory found; add $bin_dir to your PATH manually." >&2
fi

# 7. Verify.
echo ""
if "$exe_path" --version >/dev/null 2>&1; then
    echo "[OK] pengj-templates-cli installed: $("$exe_path" --version)"
else
    echo "[install] the CLI was installed but could not be executed" >&2
    exit 1
fi

# 8. The commit gate is activated per project.
#
# Git keeps hook activation in .git/config, which does not travel with a
# clone. The project's own environment check (scripts/doctor.sh --fix) is the
# single activation entry point: it works with or without this CLI and only
# touches repositories that ship .githooks/.
echo ""
echo "The commit gate ships inside this binary, activated per project:"
echo "  just doctor --fix                 # inside a generated project"
echo "  sh scripts/doctor.sh --fix        # without just"
echo "  pengj-templates-cli gate check    # run the code language guard directly"
