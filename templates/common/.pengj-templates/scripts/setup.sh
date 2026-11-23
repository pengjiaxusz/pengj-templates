#!/bin/sh
{% if options["skill_lang"] | default("zh") == "en" -%}
# Managed by pengj-templates (common layer).
# Zero-prerequisite bootstrap wrapper for project setup: installs or upgrades pengj-templates-cli if missing/outdated, then delegates to `pengj-templates-cli setup --dir .`.
{% else -%}
# 由 pengj-templates（common 层）托管。
# 仓库一键装机的零前置薄引导脚本：若缺少 pengj-templates-cli 或版本过旧不支持 setup 则自动安装/升级，随后转交 `pengj-templates-cli setup --dir .`。
{% endif %}
set -eu

PENGJ_BIN="${XDG_DATA_HOME:-$HOME/.local/share}/pengj-templates/bin"
case ":$PATH:" in
  *":$PENGJ_BIN:"*) ;;
  *) PATH="$PATH:$PENGJ_BIN"; export PATH ;;
esac

NEED_INSTALL=0
if ! command -v pengj-templates-cli >/dev/null 2>&1; then
  NEED_INSTALL=1
elif ! pengj-templates-cli setup --help >/dev/null 2>&1; then
  NEED_INSTALL=2
fi

if [ "$NEED_INSTALL" -ne 0 ]; then
  DRY_RUN=0
  for arg in "$@"; do
    if [ "$arg" = "--dry-run" ]; then
      DRY_RUN=1
      break
    fi
  done

  if [ "$DRY_RUN" -eq 1 ]; then
{% if options["skill_lang"] | default("zh") == "en" -%}
    if [ "$NEED_INSTALL" -eq 2 ]; then
      echo "[setup] [dry-run] Existing pengj-templates-cli does not support the setup subcommand."
      echo "[setup] [dry-run] Would upgrade pengj-templates-cli (target: $PENGJ_BIN) and run: pengj-templates-cli setup --dir ."
    else
      echo "[setup] [dry-run] pengj-templates-cli not found."
      echo "[setup] [dry-run] Would download and install pengj-templates-cli (target: $PENGJ_BIN) and run: pengj-templates-cli setup --dir ."
    fi
{% else -%}
    if [ "$NEED_INSTALL" -eq 2 ]; then
      echo "[setup] [dry-run] 当前已安装的 pengj-templates-cli 版本过旧，不支持 setup 子命令。"
      echo "[setup] [dry-run] 将升级 pengj-templates-cli（目标目录：$PENGJ_BIN）并执行 pengj-templates-cli setup --dir ."
    else
      echo "[setup] [dry-run] 未检测到 pengj-templates-cli。"
      echo "[setup] [dry-run] 将下载安装 pengj-templates-cli（目标目录：$PENGJ_BIN）并执行 pengj-templates-cli setup --dir ."
    fi
{% endif -%}
    exit 0
  fi

{% if options["skill_lang"] | default("zh") == "en" -%}
  if [ "$NEED_INSTALL" -eq 2 ]; then
    echo "[setup] Upgrading pengj-templates-cli in $PENGJ_BIN..." >&2
  else
    echo "[setup] Installing pengj-templates-cli to $PENGJ_BIN..." >&2
  fi
{% else -%}
  if [ "$NEED_INSTALL" -eq 2 ]; then
    echo "[setup] 当前 pengj-templates-cli 版本过旧，正在升级到 $PENGJ_BIN..." >&2
  else
    echo "[setup] 正在安装 pengj-templates-cli 到 $PENGJ_BIN..." >&2
  fi
{% endif -%}
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh
  else
{% if options["skill_lang"] | default("zh") == "en" -%}
    echo "[setup] Error: neither curl nor wget is available to download pengj-templates-cli." >&2
{% else -%}
    echo "[setup] 错误：未找到 curl 或 wget，无法自动下载安装 pengj-templates-cli。" >&2
{% endif -%}
    exit 1
  fi

  if ! pengj-templates-cli setup --help >/dev/null 2>&1; then
{% if options["skill_lang"] | default("zh") == "en" -%}
    echo "[setup] Error: pengj-templates-cli (with setup support) still unavailable in PATH after install (expected: $PENGJ_BIN)." >&2
{% else -%}
    echo "[setup] 错误：安装完成后仍未在 PATH 中找到支持 setup 的 pengj-templates-cli（预期目录：$PENGJ_BIN）。" >&2
{% endif -%}
    exit 1
  fi
fi

if [ -n "${GITHUB_PATH:-}" ] && [ -d "$PENGJ_BIN" ]; then
  if [ ! -f "$GITHUB_PATH" ] || ! grep -Fxq "$PENGJ_BIN" "$GITHUB_PATH" 2>/dev/null; then
    printf '%s\n' "$PENGJ_BIN" >> "$GITHUB_PATH"
  fi
fi

exec pengj-templates-cli setup --dir . "$@"
