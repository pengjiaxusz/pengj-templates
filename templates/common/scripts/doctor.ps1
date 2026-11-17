# scripts/doctor.ps1 - {% if options["skill_lang"] | default("zh") == "en" %}project environment check{% else %}项目环境自检{% endif %}.
#
{% if options["skill_lang"] | default("zh") == "en" -%}
# Checks only the tools the selected template layers actually use, then runs
# every custom check in scripts/doctor.d/*.ps1 in sorted order.
#
# Usage:
#   pwsh -NoProfile -File scripts/doctor.ps1          report only
#   pwsh -NoProfile -File scripts/doctor.ps1 --fix    also activate the commit hooks
#   (the just recipe forwards `just doctor --fix` unchanged on both platforms)
#
# Custom checks: drop *.ps1 files into scripts/doctor.d/. Each script runs from
# the project root with DOCTOR_ROOT set to the project root and DOCTOR_FIX=1
# when --fix is given. Exit 0 = pass, non-zero = failure; output is shown as-is.
{%- else -%}
# 只检查所选模板层实际用到的工具，再按文件名顺序运行 scripts/doctor.d/*.ps1
# 下的所有自定义检查。
#
# 用法：
#   pwsh -NoProfile -File scripts/doctor.ps1          仅输出报告
#   pwsh -NoProfile -File scripts/doctor.ps1 --fix    同时激活提交钩子
#   （just 配方在两个平台上都原样转发 `just doctor --fix`）
#
# 自定义检查：把 *.ps1 放进 scripts/doctor.d/。每个脚本从项目根目录运行，
# DOCTOR_ROOT 指向项目根目录，带 --fix 时设置 DOCTOR_FIX=1。
# 退出码 0 为通过，非 0 为失败；输出按原样显示。
{%- endif %}

$Fix = $false
foreach ($arg in $args) {
    switch ($arg.ToLowerInvariant()) {
        "--fix" { $Fix = $true }
        "-fix" { $Fix = $true }
        "-h" {
            Write-Host "{% if options["skill_lang"] | default("zh") == "en" %}usage: pwsh -NoProfile -File scripts/doctor.ps1 [--fix]{% else %}用法：pwsh -NoProfile -File scripts/doctor.ps1 [--fix]{% endif %}"
            exit 0
        }
        "--help" {
            Write-Host "{% if options["skill_lang"] | default("zh") == "en" %}usage: pwsh -NoProfile -File scripts/doctor.ps1 [--fix]{% else %}用法：pwsh -NoProfile -File scripts/doctor.ps1 [--fix]{% endif %}"
            exit 0
        }
        default {
            Write-Host "[doctor] {% if options["skill_lang"] | default("zh") == "en" %}unknown argument:{% else %}未知参数：{% endif %} $arg"
            exit 2
        }
    }
}

$ErrorActionPreference = "Continue"

$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    $root = (Get-Location).Path
}
Set-Location $root
$env:DOCTOR_ROOT = $root
if ($Fix) {
    $env:DOCTOR_FIX = "1"
} else {
    Remove-Item Env:DOCTOR_FIX -ErrorAction SilentlyContinue
}

$script:checks = 0
$script:errors = 0
$script:warnings = 0

function Report-Ok($message) {
    $script:checks++
    Write-Host "[doctor] ok    $message" -ForegroundColor Green
}

function Report-Warn($message) {
    $script:checks++
    $script:warnings++
    Write-Host "[doctor] warn  $message" -ForegroundColor Yellow
}

function Report-Fail($message) {
    $script:checks++
    $script:errors++
    Write-Host "[doctor] error $message" -ForegroundColor Red
}

function Test-Tool($name) {
    return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

# --- built-in checks: only what the selected layers use ---------------------

if (Test-Tool git) {
    Report-Ok "git $((& git --version) -replace '^git version ', '')"
} else {
    Report-Fail "{% if options["skill_lang"] | default("zh") == "en" %}git not found{% else %}未找到 git{% endif %}"
}

{% if 'just' in layers %}
if (Test-Tool just) {
    Report-Ok "just $((& just --version) -replace '^just ', '')"
} else {
    Report-Fail "{% if options["skill_lang"] | default("zh") == "en" %}just not found; install: https://github.com/casey/just#installation{% else %}未找到 just；安装地址：https://github.com/casey/just#installation{% endif %}"
}
{% endif %}

{% if 'rust' in layers or 'rust-workspace' in layers %}
if (Test-Tool cargo) {
    Report-Ok "cargo $((& cargo --version) -replace '^cargo ', '')"
} else {
    Report-Fail "{% if options["skill_lang"] | default("zh") == "en" %}cargo not found; install a Rust toolchain{% else %}未找到 cargo；请安装 Rust 工具链{% endif %}"
}
if (Test-Tool rustc) {
    Report-Ok "rustc $((& rustc --version) -replace '^rustc ', '')"
} else {
    Report-Fail "{% if options["skill_lang"] | default("zh") == "en" %}rustc not found; install a Rust toolchain{% else %}未找到 rustc；请安装 Rust 工具链{% endif %}"
}
{% endif %}

{% if 'bque' in layers %}
if (Test-Tool bque) {
    Report-Ok "bque $((& bque --version 2>$null | Select-Object -First 1))"
} else {
    Report-Warn "{% if options["skill_lang"] | default("zh") == "en" %}bque not found; install: irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex{% else %}未找到 bque；安装命令：irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex{% endif %}"
}
{% endif %}

{% if 'gate' in layers or 'agent' in layers %}
if (Test-Tool pengj-templates-cli) {
    Report-Ok "pengj-templates-cli $((& pengj-templates-cli --version 2>$null | Select-Object -First 1))"
} else {
    Report-Warn "{% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli not found; install: irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex{% else %}未找到 pengj-templates-cli；安装命令：irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex{% endif %}"
}
{% endif %}

{% if options["skills"] is defined and 'mcp-bootstrap' in options["skills"] %}
if (Test-Tool node) {
    Report-Ok "node $((& node --version 2>$null))"
} else {
    Report-Warn "{% if options["skill_lang"] | default("zh") == "en" %}node not found; it is required by the mcp-bootstrap skill (install Node.js or drop that skill from options.skills){% else %}未找到 node；mcp-bootstrap 技能需要它（请安装 Node.js，或从 options.skills 中移除该技能）{% endif %}"
}
{% endif %}

{% if 'gate' in layers %}
# --- commit gate activation ------------------------------------------------

$inside = $false
if (Test-Tool git) {
    $inside = [bool](& git rev-parse --show-toplevel 2>$null)
}
if ($inside) {
    $local = (& git config --local --get core.hooksPath 2>$null)

    if (-not (Test-Path ".githooks/pre-commit") -or -not (Test-Path ".githooks/commit-msg")) {
        Report-Fail "{% if options["skill_lang"] | default("zh") == "en" %}.githooks is incomplete; expected pre-commit and commit-msg{% else %}.githooks 不完整；应包含 pre-commit 与 commit-msg{% endif %}"
    } elseif ($local -eq ".githooks") {
        Report-Ok "{% if options["skill_lang"] | default("zh") == "en" %}commit hooks active (core.hooksPath=.githooks){% else %}提交钩子已激活（core.hooksPath=.githooks）{% endif %}"
    } elseif ($local) {
        Report-Warn "{% if options["skill_lang"] | default("zh") == "en" %}core.hooksPath is '$local', not .githooks; the commit gate is not active here{% else %}core.hooksPath 为 '$local'，并非 .githooks；提交门禁未在此生效{% endif %}"
    } elseif ($Fix) {
        # Activate through the first available path, but judge the outcome by
        # the resulting git state: an informational line failing to print must
        # not turn a successful activation into an error.
        if ((Test-Path ".githooks/install.sh") -and (Test-Tool sh)) {
            & sh .githooks/install.sh | Out-Null
        } elseif (Test-Tool pengj-templates-cli) {
            & pengj-templates-cli gate hooks install | Out-Null
        } else {
            & git config core.hooksPath .githooks | Out-Null
        }
        & git update-index --chmod=+x .githooks/pre-commit .githooks/commit-msg 2>$null | Out-Null
        if ((& git config --local --get core.hooksPath 2>$null) -eq ".githooks") {
            Report-Ok "{% if options["skill_lang"] | default("zh") == "en" %}commit hooks activated (core.hooksPath=.githooks){% else %}提交钩子已激活（core.hooksPath=.githooks）{% endif %}"
        } else {
            Report-Fail "{% if options["skill_lang"] | default("zh") == "en" %}could not activate the commit hooks; run: git config core.hooksPath .githooks{% else %}无法激活提交钩子；请运行：git config core.hooksPath .githooks{% endif %}"
        }
    } else {
        Report-Warn "{% if options["skill_lang"] | default("zh") == "en" %}commit hooks are not active; run 'pwsh -NoProfile -File scripts/doctor.ps1 --fix' or 'git config core.hooksPath .githooks'{% else %}提交钩子未激活；请运行 'pwsh -NoProfile -File scripts/doctor.ps1 --fix' 或 'git config core.hooksPath .githooks'{% endif %}"
    }
} else {
    Report-Warn "{% if options["skill_lang"] | default("zh") == "en" %}not a git work tree; skipping the commit hook check{% else %}当前不是 git 工作树；跳过提交钩子检查{% endif %}"
}
{% endif %}

# --- custom checks ---------------------------------------------------------

$custom = 0
$customDir = Join-Path $root "scripts/doctor.d"
if (Test-Path $customDir) {
    Get-ChildItem -Path $customDir -Filter "*.ps1" | Sort-Object Name | ForEach-Object {
        $custom++
        $name = $_.Name
        & pwsh -NoProfile -File $_.FullName
        if ($LASTEXITCODE -eq 0) {
            $script:checks++
            Write-Host "[doctor] custom ok    $name" -ForegroundColor Green
        } else {
            $script:checks++
            $script:errors++
            Write-Host "[doctor] custom error $name" -ForegroundColor Red
        }
    }
}
if ($custom -eq 0) {
    Write-Host "{% if options["skill_lang"] | default("zh") == "en" %}[doctor] custom: none (add *.ps1 scripts to scripts/doctor.d/ to extend this check){% else %}[doctor] 自定义检查：无（在 scripts/doctor.d/ 下添加 *.ps1 脚本即可扩展本检查）{% endif %}"
}

Write-Host "{% if options["skill_lang"] | default("zh") == "en" %}[doctor] $script:checks checks: $script:errors errors, $script:warnings warnings{% else %}[doctor] 共 $script:checks 项检查：$script:errors 个错误，$script:warnings 个警告{% endif %}"
if ($script:errors -gt 0) {
    exit 1
}
exit 0
