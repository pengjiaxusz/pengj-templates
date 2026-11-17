#!/bin/sh
# scripts/doctor.sh - {% if options["skill_lang"] | default("zh") == "en" %}project environment check{% else %}项目环境自检{% endif %}.
#
{% if options["skill_lang"] | default("zh") == "en" -%}
# Checks only the tools the selected template layers actually use, then runs
# every custom check in scripts/doctor.d/*.sh in sorted order.
#
# Usage:
#   sh scripts/doctor.sh          report only
#   sh scripts/doctor.sh --fix    also activate the commit hooks and fix modes
#
# Custom checks: drop *.sh files into scripts/doctor.d/. Each script runs from
# the project root with DOCTOR_ROOT set to the project root and DOCTOR_FIX=1
# when --fix is given. Exit 0 = pass, non-zero = failure; output is shown as-is.
{%- else -%}
# 只检查所选模板层实际用到的工具，再按文件名顺序运行 scripts/doctor.d/*.sh
# 下的所有自定义检查。
#
# 用法：
#   sh scripts/doctor.sh          仅输出报告
#   sh scripts/doctor.sh --fix    同时激活提交钩子并修正相关模式
#
# 自定义检查：把 *.sh 放进 scripts/doctor.d/。每个脚本从项目根目录运行，
# DOCTOR_ROOT 指向项目根目录，带 --fix 时设置 DOCTOR_FIX=1。
# 退出码 0 为通过，非 0 为失败；输出按原样显示。
{%- endif %}

set -u

fix=0
for arg in "$@"; do
    case "$arg" in
        --fix) fix=1 ;;
        -h|--help)
            echo "{% if options["skill_lang"] | default("zh") == "en" %}usage: sh scripts/doctor.sh [--fix]{% else %}用法：sh scripts/doctor.sh [--fix]{% endif %}"
            exit 0
            ;;
        *)
            echo "[doctor] {% if options["skill_lang"] | default("zh") == "en" %}unknown argument:{% else %}未知参数：{% endif %} $arg" >&2
            exit 2
            ;;
    esac
done

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$root" || exit 1
export DOCTOR_ROOT="$root"
if [ "$fix" = "1" ]; then
    export DOCTOR_FIX=1
fi

checks=0
errors=0
warnings=0

ok()   { checks=$((checks + 1)); printf '[doctor] ok    %s\n' "$1"; }
warn() { checks=$((checks + 1)); warnings=$((warnings + 1)); printf '[doctor] warn  %s\n' "$1"; }
fail() { checks=$((checks + 1)); errors=$((errors + 1)); printf '[doctor] error %s\n' "$1"; }
have() { command -v "$1" >/dev/null 2>&1; }
is_windows() {
    case "$(uname -s 2>/dev/null || echo unknown)" in
        MINGW*|MSYS*|CYGWIN*) return 0 ;;
        *) return 1 ;;
    esac
}

# --- built-in checks: only what the selected layers use ---------------------

if have git; then
    ok "git $(git --version | sed 's/^git version //')"
else
    fail "{% if options["skill_lang"] | default("zh") == "en" %}git not found{% else %}未找到 git{% endif %}"
fi

{% if 'just' in layers %}
if have just; then
    ok "just $(just --version | sed 's/^just //')"
else
    fail "{% if options["skill_lang"] | default("zh") == "en" %}just not found; install: https://github.com/casey/just#installation{% else %}未找到 just；安装地址：https://github.com/casey/just#installation{% endif %}"
fi
if is_windows; then
    if have pwsh; then
        ok "pwsh $(pwsh --version)"
    else
        fail "{% if options["skill_lang"] | default("zh") == "en" %}pwsh not found; the just recipes use PowerShell 7 on Windows{% else %}未找到 pwsh；Windows 上的 just 配方需要 PowerShell 7{% endif %}"
    fi
fi
{% endif %}

{% if 'rust' in layers or 'rust-workspace' in layers %}
if have cargo; then
    ok "cargo $(cargo --version | sed 's/^cargo //')"
else
    fail "{% if options["skill_lang"] | default("zh") == "en" %}cargo not found; install a Rust toolchain{% else %}未找到 cargo；请安装 Rust 工具链{% endif %}"
fi
if have rustc; then
    ok "rustc $(rustc --version | sed 's/^rustc //')"
else
    fail "{% if options["skill_lang"] | default("zh") == "en" %}rustc not found; install a Rust toolchain{% else %}未找到 rustc；请安装 Rust 工具链{% endif %}"
fi
{% endif %}

{% if 'bque' in layers %}
if is_windows; then
    if have bque; then
        ok "bque $(bque --version 2>/dev/null | head -n 1)"
    else
        warn "{% if options["skill_lang"] | default("zh") == "en" %}bque not found; install: irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex{% else %}未找到 bque；安装命令：irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex{% endif %}"
    fi
else
    ok "{% if options["skill_lang"] | default("zh") == "en" %}bque not required on this platform{% else %}当前平台不需要 bque{% endif %}"
fi
{% endif %}

{% if 'gate' in layers or 'agent' in layers %}
if have pengj-templates-cli; then
    ok "pengj-templates-cli $(pengj-templates-cli --version 2>/dev/null | head -n 1)"
else
    warn "{% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli not found; install: curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh{% else %}未找到 pengj-templates-cli；安装命令：curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh{% endif %}"
fi
{% endif %}

{% if options["skills"] is defined and 'mcp-bootstrap' in options["skills"] %}
if have node; then
    ok "node $(node --version 2>/dev/null)"
else
    warn "{% if options["skill_lang"] | default("zh") == "en" %}node not found; it is required by the mcp-bootstrap skill (install Node.js or drop that skill from options.skills){% else %}未找到 node；mcp-bootstrap 技能需要它（请安装 Node.js，或从 options.skills 中移除该技能）{% endif %}"
fi
{% endif %}

{% if 'gate' in layers %}
# --- commit gate activation ------------------------------------------------

if have git && git rev-parse --show-toplevel >/dev/null 2>&1; then
    local_hooks=$(git config --local --get core.hooksPath 2>/dev/null || true)

    if [ ! -f .githooks/pre-commit ] || [ ! -f .githooks/commit-msg ]; then
        fail "{% if options["skill_lang"] | default("zh") == "en" %}.githooks is incomplete; expected pre-commit and commit-msg{% else %}.githooks 不完整；应包含 pre-commit 与 commit-msg{% endif %}"
    elif [ "$local_hooks" = ".githooks" ]; then
        ok "{% if options["skill_lang"] | default("zh") == "en" %}commit hooks active (core.hooksPath=.githooks){% else %}提交钩子已激活（core.hooksPath=.githooks）{% endif %}"
    elif [ -n "$local_hooks" ]; then
        warn "{% if options["skill_lang"] | default("zh") == "en" %}core.hooksPath is '$local_hooks', not .githooks; the commit gate is not active here{% else %}core.hooksPath 为 '$local_hooks'，并非 .githooks；提交门禁未在此生效{% endif %}"
    elif [ "$fix" = "1" ]; then
        # Activate through the zero-dependency installer shipped by the gate
        # layer, but judge the outcome by the resulting git state: an
        # informational line failing to print must not turn a successful
        # activation into an error.
        if [ -f .githooks/install.sh ]; then
            sh .githooks/install.sh >/dev/null || true
        fi
        # Record the mode bit when the hooks are already tracked, so clones on
        # Unix do not silently skip them.
        git update-index --chmod=+x .githooks/pre-commit .githooks/commit-msg 2>/dev/null || true
        if [ "$(git config --local --get core.hooksPath 2>/dev/null || true)" = ".githooks" ]; then
            ok "{% if options["skill_lang"] | default("zh") == "en" %}commit hooks activated (core.hooksPath=.githooks){% else %}提交钩子已激活（core.hooksPath=.githooks）{% endif %}"
        else
            fail "{% if options["skill_lang"] | default("zh") == "en" %}could not activate the commit hooks; run: git config core.hooksPath .githooks{% else %}无法激活提交钩子；请运行：git config core.hooksPath .githooks{% endif %}"
        fi
    else
        warn "{% if options["skill_lang"] | default("zh") == "en" %}commit hooks are not active; run 'sh scripts/doctor.sh --fix' or 'git config core.hooksPath .githooks'{% else %}提交钩子未激活；请运行 'sh scripts/doctor.sh --fix' 或 'git config core.hooksPath .githooks'{% endif %}"
    fi

    # git silently skips a hook without the executable bit on Unix
    if ! is_windows; then
        for hook in pre-commit commit-msg; do
            if [ -f ".githooks/$hook" ] && [ ! -x ".githooks/$hook" ]; then
                if [ "$fix" = "1" ]; then
                    chmod +x ".githooks/$hook" && ok "{% if options["skill_lang"] | default("zh") == "en" %}.githooks/$hook is executable{% else %}.githooks/$hook 已具备可执行权限{% endif %}"
                else
                    warn "{% if options["skill_lang"] | default("zh") == "en" %}.githooks/$hook is not executable; run with --fix{% else %}.githooks/$hook 不可执行；请带 --fix 运行{% endif %}"
                fi
            fi
        done
    fi
else
    warn "{% if options["skill_lang"] | default("zh") == "en" %}not a git work tree; skipping the commit hook check{% else %}当前不是 git 工作树；跳过提交钩子检查{% endif %}"
fi
{% endif %}

{% if 'agent' in layers %}
# --- agent client workspace links -----------------------------------------

if [ -d .agents ]; then
    if have pengj-templates-cli; then
        if pengj-templates-cli repo status --dir . >/dev/null 2>&1; then
            ok "{% if options["skill_lang"] | default("zh") == "en" %}workspace links ready (agent clients){% else %}工作区链接已就绪（agent 客户端）{% endif %}"
        elif [ "$fix" = "1" ] && pengj-templates-cli repo init --dir . >/dev/null 2>&1 \
            && pengj-templates-cli repo status --dir . >/dev/null 2>&1; then
            ok "{% if options["skill_lang"] | default("zh") == "en" %}workspace links created (agent clients){% else %}工作区链接已创建（agent 客户端）{% endif %}"
        elif [ "$fix" = "1" ]; then
            fail "{% if options["skill_lang"] | default("zh") == "en" %}could not create the workspace links; run: pengj-templates-cli repo init{% else %}无法创建工作区链接；请运行：pengj-templates-cli repo init{% endif %}"
        else
            warn "{% if options["skill_lang"] | default("zh") == "en" %}workspace links are not ready; run 'just doctor --fix'{% else %}工作区链接未就绪；请运行 'just doctor --fix'{% endif %}"
        fi
    else
        warn "{% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli not found; skipping the workspace link check{% else %}未找到 pengj-templates-cli；跳过工作区链接检查{% endif %}"
    fi
fi
{% endif %}

# --- custom checks ---------------------------------------------------------

custom=0
if [ -d scripts/doctor.d ]; then
    for script in scripts/doctor.d/*.sh; do
        [ -f "$script" ] || continue
        custom=$((custom + 1))
        name=$(basename "$script")
        if sh "$script"; then
            checks=$((checks + 1))
            printf '[doctor] custom ok    %s\n' "$name"
        else
            checks=$((checks + 1))
            errors=$((errors + 1))
            printf '[doctor] custom error %s\n' "$name"
        fi
    done
fi
if [ "$custom" -eq 0 ]; then
    printf '[doctor] {% if options["skill_lang"] | default("zh") == "en" %}custom: none (add *.sh scripts to scripts/doctor.d/ to extend this check){% else %}自定义检查：无（在 scripts/doctor.d/ 下添加 *.sh 脚本即可扩展本检查）{% endif %}\n'
fi

printf '[doctor] {% if options["skill_lang"] | default("zh") == "en" %}%d checks: %d errors, %d warnings{% else %}共 %d 项检查：%d 个错误，%d 个警告{% endif %}\n' "$checks" "$errors" "$warnings"
if [ "$errors" -gt 0 ]; then
    exit 1
fi
exit 0
