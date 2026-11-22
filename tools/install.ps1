# tools/install.ps1
#
# pengj-templates-cli One-Liner Quick Installer & Updater for Windows
#
# Usage:
#   irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex
#
# The installer ships the CLI and the template tree together: the template
# engine reads `templates/` at runtime, so a binary without its templates is
# useless. The commit gate rides along in the same binary.

$ErrorActionPreference = "Stop"

# 0. Language: Chinese UI -> Chinese messages, everything else -> English.
#    The installer ships standalone (it is not a rendered template), so it
#    detects the OS UI language at runtime instead of reading `skill_lang`.
#    `irm ... | iex` runs this in the caller's scope, so keep helpers flat.
try {
    $uiName = [System.Globalization.CultureInfo]::CurrentUICulture.Name
} catch {
    $uiName = ""
}
if (-not $uiName) { $uiName = "$env:LANG" }
$IsZh = [bool]($uiName -match '^zh(?:[-_]|$)')
function T([string]$zhText, [string]$enText) { if ($IsZh) { $zhText } else { $enText } }

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls11 -bor [Net.SecurityProtocolType]::Tls

$repo = "pengjiaxusz/pengj-templates"
$installRoot = Join-Path $env:LOCALAPPDATA "pengj-templates"
$binDir = Join-Path $installRoot "bin"
$templatesDir = Join-Path $installRoot "templates"
$exePath = Join-Path $binDir "pengj-templates-cli.exe"

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host (T "        pengj-templates-cli 安装 / 更新程序 (Windows)       " "        pengj-templates-cli Installer / Updater (Windows)       ") -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Detect the architecture so the right asset is fetched.
$arch = switch ($env:PROCESSOR_ARCHITEW6432) {
    "ARM64" { "arm64" }
    default {
        if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x86_64" }
    }
}

$existing = @()
$existingCommands = @(Get-Command pengj-templates-cli -All -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
foreach ($cmd in $existingCommands) {
    if (-not $existing.Contains($cmd)) {
        $existing += $cmd
    }
}

$oldVersion = ""
if ($existing.Count -gt 0) {
    try {
        $oldVersion = (& $existing[0] --version 2>$null | Select-Object -First 1)
    } catch {}
}

Write-Host (T "--> 目标架构: $arch" "--> Target architecture: $arch") -ForegroundColor Cyan

if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
}

# 2. Download the portable bundle (CLI + templates) with curl and proxy fallback.
function Download-FileWithFallback {
    param (
        [string]$Url,
        [string]$Destination
    )

    $curlPath = Join-Path $env:SystemRoot "System32\curl.exe"
    if (Test-Path $curlPath) {
        & $curlPath -fLs --connect-timeout 15 --max-time 120 -o $Destination $Url 2>$null
        if ((Test-Path $Destination) -and ((Get-Item $Destination).Length -gt 100000)) {
            return $true
        }
    }

    try {
        $proxy = [System.Net.WebRequest]::GetSystemWebProxy()
        $proxy.Credentials = [System.Net.CredentialCache]::DefaultCredentials

        $req = [System.Net.HttpWebRequest]::Create($Url)
        $req.Proxy = $proxy
        $req.Timeout = 60000
        $req.UserAgent = "pengj-templates-installer"
        $resp = $req.GetResponse()
        $stream = $resp.GetResponseStream()
        $fs = [System.IO.File]::Create($Destination)
        $stream.CopyTo($fs)
        $fs.Close()
        $stream.Close()
        $resp.Close()

        if ((Test-Path $Destination) -and ((Get-Item $Destination).Length -gt 100000)) {
            return $true
        }
    } catch {}

    try {
        Invoke-RestMethod -Uri $Url -OutFile $Destination -TimeoutSec 60
        if ((Test-Path $Destination) -and ((Get-Item $Destination).Length -gt 100000)) {
            return $true
        }
    } catch {}

    return $false
}

$baseUrl = "https://github.com/$repo/releases/latest/download"
$bundleUrl = "$baseUrl/pengj-templates-cli-windows-$arch.zip"
$tempZip = Join-Path $env:TEMP "pengj-templates-$([System.Guid]::NewGuid().ToString('N')).zip"
$tempExtract = Join-Path $env:TEMP "pengj-templates-$([System.Guid]::NewGuid().ToString('N'))"

Write-Host (T "--> 正在下载 $bundleUrl" "--> Downloading $bundleUrl") -ForegroundColor Cyan
if (-not (Download-FileWithFallback -Url $bundleUrl -Destination $tempZip)) {
    Write-Host (T "从 GitHub Releases 下载发布包失败。" "Failed to download the release bundle from GitHub Releases.") -ForegroundColor Red
    Write-Host (T "请检查网络连接，或手动从这里下载:" "Check your network connection or download manually from:") -ForegroundColor Yellow
    Write-Host "https://github.com/$repo/releases/latest" -ForegroundColor Yellow
    exit 1
}

# 3. Unpack the bundle.
try {
    Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force
} catch {
    Write-Host (T "解压发布包失败: $_" "Failed to extract the release bundle: $_") -ForegroundColor Red
    Remove-Item $tempZip -Force -ErrorAction SilentlyContinue
    Remove-Item $tempExtract -Recurse -Force -ErrorAction SilentlyContinue
    exit 1
} finally {
    Remove-Item $tempZip -Force -ErrorAction SilentlyContinue
}

$bundleRoot = Join-Path $tempExtract "pengj-templates"
if (-not (Test-Path $bundleRoot)) {
    $bundleRoot = $tempExtract
}

# 4. Deploy: CLI into bin/, templates next to it so the engine finds them at runtime.
Copy-Item -Path (Join-Path $bundleRoot "pengj-templates-cli.exe") -Destination $exePath -Force
if (Test-Path $templatesDir) {
    Remove-Item $templatesDir -Recurse -Force -ErrorAction SilentlyContinue
}
Copy-Item -Path (Join-Path $bundleRoot "templates") -Destination $templatesDir -Recurse -Force

Remove-Item $tempExtract -Recurse -Force -ErrorAction SilentlyContinue

Write-Host (T "--> 已安装 CLI: $exePath" "--> Installed CLI: $exePath") -ForegroundColor Cyan
Write-Host (T "--> 已安装模板: $templatesDir" "--> Installed templates: $templatesDir") -ForegroundColor Cyan

# 5. Also refresh any pengj-templates-cli already on PATH, so an old install
#    never shadows the new one.
foreach ($target in $existing) {
    if ($target -eq $exePath) { continue }
    try {
        $oldBackup = "$target.$([System.Guid]::NewGuid().ToString('N')).old"
        if (Test-Path $target) {
            Move-Item -Path $target -Destination $oldBackup -Force -ErrorAction Stop
        }
        Copy-Item -Path $exePath -Destination $target -Force
        Remove-Item $oldBackup -Force -ErrorAction SilentlyContinue
        Write-Host (T "--> 已刷新已有安装: $target" "--> Refreshed existing install: $target") -ForegroundColor Cyan
    } catch {
        Write-Host (T "警告: 无法替换 $target（文件被占用）: $_" "Warning: could not replace $target (file locked): $_") -ForegroundColor Yellow
    }
}

# 6. Ensure bin/ is in the user PATH.
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$pathList = if ($userPath) { $userPath -split ';' } else { @() }
if ($pathList -notcontains $binDir) {
    Write-Host (T "--> 正在将 $binDir 加入 User PATH..." "--> Adding $binDir to User PATH...") -ForegroundColor Cyan
    $newPath = if ($userPath) { "$userPath;$binDir" } else { $binDir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
}

# 7. Make it usable in the current session too.
if (($env:Path -split ';') -notcontains $binDir) {
    $env:Path = "$binDir;$env:Path"
}

# 8. Verify.
Write-Host ""
$newVersion = (& $exePath --version 2>$null | Select-Object -First 1)
if ($oldVersion) {
    Write-Host (T "[OK] pengj-templates-cli 已更新: $oldVersion -> $newVersion" "[OK] pengj-templates-cli updated: $oldVersion -> $newVersion") -ForegroundColor Green
} else {
    Write-Host (T "[OK] pengj-templates-cli 已安装: $newVersion" "[OK] pengj-templates-cli installed: $newVersion") -ForegroundColor Green
}

# 9. The commit gate is activated per project.
#
# Git keeps hook activation in .git/config, which does not travel with a
# clone. The built-in environment check (`pengj-templates-cli doctor --fix`) is
# the unified activation entry point and only touches repositories that ship .githooks/.
Write-Host ""
Write-Host (T "提交门禁随此二进制一起发布，按项目分别激活:" "The commit gate ships inside this binary, activated per project:") -ForegroundColor Cyan
Write-Host (T "  just doctor --fix                    # 在已生成的项目内" "  just doctor --fix                    # inside a generated project") -ForegroundColor Gray
Write-Host (T "  pengj-templates-cli doctor --fix     # 没有 just 时" "  pengj-templates-cli doctor --fix     # without just") -ForegroundColor Gray
Write-Host (T "  pengj-templates-cli gate check       # 直接运行代码语言守卫" "  pengj-templates-cli gate check       # run the code language guard directly") -ForegroundColor Gray
Write-Host ""
Write-Host (T "当前会话及新开的终端窗口均可直接使用。" "Ready to use in the current session and in new terminal windows.") -ForegroundColor Green

# 10. Next time, upgrade from the CLI itself.
Write-Host ""
Write-Host (T "以后升级不用再跑这个脚本:" "Next time, upgrade without this script:") -ForegroundColor Cyan
Write-Host (T "  pengj-templates-cli self update       # 二进制与 templates 一起换" "  pengj-templates-cli self update       # binary and templates replaced together") -ForegroundColor Gray
