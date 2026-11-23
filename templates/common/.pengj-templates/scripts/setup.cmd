@echo off
setlocal EnableExtensions
chcp 65001 >nul
{% if options["skill_lang"] | default("zh") == "en" -%}
rem Managed by pengj-templates (common layer).
rem Zero-prerequisite bootstrap wrapper for project setup: installs or upgrades pengj-templates-cli if missing/outdated, then delegates to `pengj-templates-cli setup --dir .`.
{% else -%}
rem 由 pengj-templates（common 层）托管。
rem 仓库一键装机的零前置薄引导脚本：若缺少 pengj-templates-cli 或版本过旧不支持 setup 则自动安装/升级，随后转交 `pengj-templates-cli setup --dir .`。
{% endif %}

set "PENGJ_BIN="
if defined LOCALAPPDATA set "PENGJ_BIN=%LOCALAPPDATA%\pengj-templates\bin"
if not defined PENGJ_BIN goto :path_done
echo ";%PATH%;" | findstr /I /C:";%PENGJ_BIN%;" >nul 2>&1
if errorlevel 1 set "PATH=%PATH%;%PENGJ_BIN%"
:path_done

set "NEED_INSTALL=0"
where pengj-templates-cli >nul 2>&1
if errorlevel 1 (
  set "NEED_INSTALL=1"
) else (
  pengj-templates-cli setup --help >nul 2>&1
  if errorlevel 1 set "NEED_INSTALL=2"
)
if "%NEED_INSTALL%"=="0" goto :ensure_github_path

set "DRY_RUN=0"
for %%A in (%*) do (
  if /I "%%~A"=="--dry-run" set "DRY_RUN=1"
)
if "%DRY_RUN%"=="1" goto :dry_run

set "DISPLAY_BIN=%%LOCALAPPDATA%%\pengj-templates\bin"
if defined PENGJ_BIN set "DISPLAY_BIN=%PENGJ_BIN%"
{% if options["skill_lang"] | default("zh") == "en" -%}
if "%NEED_INSTALL%"=="2" (
  >&2 echo [setup] Existing pengj-templates-cli is outdated; upgrading in %DISPLAY_BIN%...
) else (
  >&2 echo [setup] Installing pengj-templates-cli to %DISPLAY_BIN%...
)
{% else -%}
if "%NEED_INSTALL%"=="2" (
  >&2 echo [setup] 当前 pengj-templates-cli 版本过旧，正在升级到 %DISPLAY_BIN%...
) else (
  >&2 echo [setup] 正在安装 pengj-templates-cli 到 %DISPLAY_BIN%...
)
{% endif -%}
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex"
if errorlevel 1 exit /b %ERRORLEVEL%

pengj-templates-cli setup --help >nul 2>&1
if errorlevel 1 (
{% if options["skill_lang"] | default("zh") == "en" -%}
  >&2 echo [setup] Error: pengj-templates-cli (with setup support) still unavailable in PATH after install (expected: %DISPLAY_BIN%).
{% else -%}
  >&2 echo [setup] 错误：安装完成后仍未在 PATH 中找到支持 setup 的 pengj-templates-cli（预期目录：%DISPLAY_BIN%）。
{% endif -%}
  exit /b 1
)
goto :ensure_github_path

:dry_run
set "DISPLAY_BIN=%%LOCALAPPDATA%%\pengj-templates\bin"
if defined PENGJ_BIN set "DISPLAY_BIN=%PENGJ_BIN%"
{% if options["skill_lang"] | default("zh") == "en" -%}
if "%NEED_INSTALL%"=="2" (
  echo [setup] [dry-run] Existing pengj-templates-cli does not support the setup subcommand.
  echo [setup] [dry-run] Would upgrade pengj-templates-cli (target: %DISPLAY_BIN%) and run: pengj-templates-cli setup --dir .
) else (
  echo [setup] [dry-run] pengj-templates-cli not found.
  echo [setup] [dry-run] Would download and install pengj-templates-cli (target: %DISPLAY_BIN%) and run: pengj-templates-cli setup --dir .
)
{% else -%}
if "%NEED_INSTALL%"=="2" (
  echo [setup] [dry-run] 当前已安装的 pengj-templates-cli 版本过旧，不支持 setup 子命令。
  echo [setup] [dry-run] 将升级 pengj-templates-cli（目标目录：%DISPLAY_BIN%）并执行 pengj-templates-cli setup --dir .
) else (
  echo [setup] [dry-run] 未检测到 pengj-templates-cli。
  echo [setup] [dry-run] 将下载安装 pengj-templates-cli（目标目录：%DISPLAY_BIN%）并执行 pengj-templates-cli setup --dir .
)
{% endif -%}
exit /b 0

:ensure_github_path
if defined GITHUB_PATH if defined PENGJ_BIN if exist "%PENGJ_BIN%" (
  findstr /I /X /C:"%PENGJ_BIN%" "%GITHUB_PATH%" >nul 2>&1
  if errorlevel 1 >>"%GITHUB_PATH%" echo %PENGJ_BIN%
)

:run_cli
pengj-templates-cli setup --dir . %*
exit /b %ERRORLEVEL%
