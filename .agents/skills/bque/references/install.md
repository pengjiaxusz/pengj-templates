# bque 安装与环境配置指南 / Installation & Setup

## 一键快速安装（推荐）

在 PowerShell 中直接运行以下命令。**无需管理员权限**，自动拉取最新 Release 二进制、安装至用户工具目录并配置 PATH，当前终端即刻生效：

```powershell
irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex
```

- **默认安装路径**：`$env:LOCALAPPDATA\bque\bin\bque.exe`
- **特性优势**：
  - 自动写入当前用户持久化 `PATH` 环境变量（免手动点系统属性、无需 UAC 提权）；
  - 自动注入当前 PowerShell 会话，无需重启终端；
  - 纯前端、C++ 或无 Rust/Cargo 环境的用户均可开箱即用。

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
