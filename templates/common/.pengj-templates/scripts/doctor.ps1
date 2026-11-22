{%- if options["skill_lang"] | default("zh") == "en" -%}
# Managed by pengj-templates (common layer).
# Non-interactive bootstrap wrapper for repository environment checks.
# Never downloads/installs tools automatically and never prompts for y/n input.

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$DoctorArgs
)

$ErrorActionPreference = 'Stop'

$pengjBin = if ($env:LOCALAPPDATA) {
    Join-Path $env:LOCALAPPDATA 'pengj-templates\bin'
} else {
    ''
}
if ($pengjBin) {
    $pathParts = ($env:PATH -split ';') | Where-Object { $_ }
    if ($pathParts -notcontains $pengjBin) {
        $env:PATH = "$env:PATH;$pengjBin"
    }
}

$hasJust = [bool](Get-Command just -ErrorAction SilentlyContinue)
$hasCli = [bool](Get-Command pengj-templates-cli -ErrorAction SilentlyContinue)
$hasJustfile = (Test-Path -LiteralPath 'justfile') -or (Test-Path -LiteralPath 'Justfile')

if ($hasJust -and $hasCli -and $hasJustfile) {
    & just doctor @DoctorArgs
    exit $LASTEXITCODE
}

if ($hasCli) {
    & pengj-templates-cli doctor --dir . @DoctorArgs
    exit $LASTEXITCODE
}

$displayBin = if ($pengjBin) { $pengjBin } else { '%LOCALAPPDATA%\pengj-templates\bin' }
[Console]::Error.WriteLine('[doctor] Missing required local tool(s). Nothing was downloaded or modified.')
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('  - pengj-templates-cli (~5-10 MB)')
[Console]::Error.WriteLine("    Default install path: $displayBin")
[Console]::Error.WriteLine('    Install (PowerShell):')
[Console]::Error.WriteLine('      irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex')

if (-not $hasJust) {
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('  - just (~3-5 MB, optional if calling the CLI directly)')
    [Console]::Error.WriteLine('    Typical install path: WinGet / Scoop shims or %USERPROFILE%\.cargo\bin')
    [Console]::Error.WriteLine('    Install: winget install Casey.Just / scoop install just / https://github.com/casey/just#installation')
}

[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('After installing, re-run:')
[Console]::Error.WriteLine('  pwsh -File .pengj-templates/scripts/doctor.ps1')
[Console]::Error.WriteLine('  # or: just doctor (pass --fix explicitly to fix local hooks/links)')
exit 1
{%- else -%}
# 由 pengj-templates（common 层）托管。
# 仓库环境体检的无交互引导脚本：永不自动下载/安装，永不弹 y/n 阻塞输入。

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$DoctorArgs
)

$ErrorActionPreference = 'Stop'

$pengjBin = if ($env:LOCALAPPDATA) {
    Join-Path $env:LOCALAPPDATA 'pengj-templates\bin'
} else {
    ''
}
if ($pengjBin) {
    $pathParts = ($env:PATH -split ';') | Where-Object { $_ }
    if ($pathParts -notcontains $pengjBin) {
        $env:PATH = "$env:PATH;$pengjBin"
    }
}

$hasJust = [bool](Get-Command just -ErrorAction SilentlyContinue)
$hasCli = [bool](Get-Command pengj-templates-cli -ErrorAction SilentlyContinue)
$hasJustfile = (Test-Path -LiteralPath 'justfile') -or (Test-Path -LiteralPath 'Justfile')

if ($hasJust -and $hasCli -and $hasJustfile) {
    & just doctor @DoctorArgs
    exit $LASTEXITCODE
}

if ($hasCli) {
    & pengj-templates-cli doctor --dir . @DoctorArgs
    exit $LASTEXITCODE
}

$displayBin = if ($pengjBin) { $pengjBin } else { '%LOCALAPPDATA%\pengj-templates\bin' }
[Console]::Error.WriteLine('[doctor] 检测到缺失必要工具（本脚本不会自动下载或修改本机环境）：')
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('  - pengj-templates-cli（约 5–10 MB）')
[Console]::Error.WriteLine("    默认安装路径：$displayBin")
[Console]::Error.WriteLine('    安装命令（PowerShell）：')
[Console]::Error.WriteLine('      irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex')

if (-not $hasJust) {
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('  - just（约 3–5 MB；若直接调用 CLI 可选装）')
    [Console]::Error.WriteLine('    常见安装位置：WinGet / Scoop shims 或 %USERPROFILE%\.cargo\bin')
    [Console]::Error.WriteLine('    安装方式：winget install Casey.Just / scoop install just / https://github.com/casey/just#installation')
}

[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('安装完成后请重新运行：')
[Console]::Error.WriteLine('  pwsh -File .pengj-templates/scripts/doctor.ps1')
[Console]::Error.WriteLine('  # 或：just doctor（如需修复本地钩子/链接，再显式追加 --fix）')
exit 1
{%- endif %}
