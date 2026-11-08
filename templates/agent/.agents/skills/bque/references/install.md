{% if options["skill_lang"] == "en" %}
# bque Installation & Environment Setup Guide

## One-Click Fast Install & Update (Recommended)

Run the following command directly in PowerShell. **No administrator privileges required**; it automatically detects existing `bque` instances in `PATH` and performs hot replacement in-place, or installs to user tools directory and updates `PATH`. Takes effect immediately in the current session:

```powershell
# Standard one-click install/update (PowerShell 7+ / Default)
irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex

# Fallback compatible command (Windows PowerShell 5.1 / Proxy environments)
iex (curl.exe -sL https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | Out-String)
```

- **Default install path**: `$env:LOCALAPPDATA\bque\bin\bque.exe`
- **Key benefits**:
  - **Install equals update**: If an older `bque` exists (including older instances installed via `cargo install` in `.cargo/bin`), the script automatically detects and hot-updates them, eliminating PATH precedence shadowing;
  - Automatically writes to current user persistent `PATH` environment variable (no manual GUI editing, no UAC elevation);
  - Automatically injects into current PowerShell session without restarting the terminal;
  - Out of the box for frontend, C++, or environments without Rust/Cargo installed.

## CLI Self-Update (When bque is already installed)

If `bque` is already installed, update directly via the built-in self-update command:

```powershell
# Check for new version
bque update --check

# One-click upgrade to latest release
bque update
```

The background runner silently and asynchronously checks for updates every 24 hours (cached in `$env:LOCALAPPDATA\bque\update_cache.json`). When a newer version is discovered, a friendly notification will be printed upon command completion (without interrupting any build).

## Manual Binary Installation (Offline / Fallback)

1. Navigate to GitHub Releases:
   👉 **https://github.com/pengjiaxusz/bque/releases/latest**
2. Download the latest `bque-windows-x86_64.zip` (or `bque.exe`).
3. Extract `bque.exe` into a user directory (recommended `$env:LOCALAPPDATA\bque\bin`):
   ```powershell
   $targetDir = "$env:LOCALAPPDATA\bque\bin"
   New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
   # Place extracted bque.exe into $targetDir
   ```
4. Append directory to user `PATH`:
   ```powershell
   [Environment]::SetEnvironmentVariable(
       "Path",
       [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:LOCALAPPDATA\bque\bin",
       "User"
   )
   ```
5. Verify installation:
   ```powershell
   bque --version
   ```

## Build & Install from Source

If a standard Rust toolchain is configured on the machine:
```powershell
git clone https://github.com/pengjiaxusz/bque.git
cd bque
cargo install --path .
```

## Health & Connectivity Verification

Execute introspection command to verify bque kernel objects and lock status:
```powershell
bque status
```
Expected normal output:
```
============================================================
              bque Global Build Queue & Governor            
============================================================

[1] Active Lock Holder:
  (No active lock holder. Build queue is idle)

[2] Queued Waiters (0):
  (Queue is empty)
============================================================
```
{% else %}
# bque 安装与环境配置指南 / Installation & Setup

## 一键快速安装与更新（推荐）

在 PowerShell 中直接运行以下命令。**无需管理员权限**，自动检测 PATH 中已有的 `bque` 实例并执行热替换原地更新，或安装至用户工具目录并配置 PATH，当前终端即刻生效：

```powershell
# 标准一键安装/更新 (PowerShell 7+ / 默认)
irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex

# 兼容备用命令 (Windows PowerShell 5.1 / 代理环境)
iex (curl.exe -sL https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | Out-String)
```

- **默认安装路径**：`$env:LOCALAPPDATA\bque\bin\bque.exe`
- **特性优势**：
  - **安装即更新**：若系统已安装旧版 `bque`（包含通过 `cargo install` 安装在 `.cargo/bin` 的旧实例），脚本会自动探测并一并热更新，杜绝 PATH 优先级遮蔽问题；
  - 自动写入当前用户持久化 `PATH` 环境变量（免手动点系统属性、无需 UAC 提权）；
  - 自动注入当前 PowerShell 会话，无需重启终端；
  - 纯前端、C++ 或无 Rust/Cargo 环境的用户均可开箱即用。

## CLI 自我更新（已安装 bque）

已安装 `bque` 的环境下，可直接通过内置自更新命令一键检测并更新：

```powershell
# 检测是否有新版本
bque update --check

# 一键升级到最新发布版本
bque update
```

后台在每次运行命令时会静默异步检查最新版本（每 24 小时检查一次，保存在 `$env:LOCALAPPDATA\bque\update_cache.json`），若发现新版本将在命令完成后输出温馨更新提示（不中断任何构建）。

## 手动二进制安装（离线 / 备用）

1. 前往 GitHub Releases 页面：
   👉 **https://github.com/pengjiaxusz/bque/releases/latest**
2. 下载最新的 `bque-windows-x86_64.zip`（或 `bque.exe`）。
3. 将解压出的 `bque.exe` 放置于用户目录（推荐 `$env:LOCALAPPDATA\bque\bin`）：
   ```powershell
   $targetDir = "$env:LOCALAPPDATA\bque\bin"
   New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
   # 将解压得到的 bque.exe 放入 $targetDir
   ```
4. 将该目录追加到用户环境变量 `PATH` 中：
   ```powershell
   [Environment]::SetEnvironmentVariable(
       "Path",
       [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:LOCALAPPDATA\bque\bin",
       "User"
   )
   ```
5. 验证安装：
   ```powershell
   bque --version
   ```

## 源码编译安装

若本机已配置标准 Rust 编译环境：
```powershell
git clone https://github.com/pengjiaxusz/bque.git
cd bque
cargo install --path .
```

## 健康度与连通性验证

执行自省命令验证 bque 内核对象与锁状态是否正常：
```powershell
bque status
```
正常输出示例：
```
============================================================
              bque Global Build Queue & Governor            
============================================================

[1] Active Lock Holder:
  (No active lock holder. Build queue is idle)

[2] Queued Waiters (0):
  (Queue is empty)
============================================================
```
{% endif %}
