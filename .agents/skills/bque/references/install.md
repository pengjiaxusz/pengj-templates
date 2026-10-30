# bque 安装与环境配置指南 / Installation & Setup


## 预编译二进制下载（推荐）

1. 前往 GitHub Releases 页面：
   👉 **https://github.com/pengjiaxusz/bque/releases/latest**
2. 下载最新的 `bque-windows-x86_64.zip`（或 `bque.exe`）。
3. 将解压出的 `bque.exe` 放置于固定的系统工具目录（如 `C:\Tools`，或通过 Scoop / Scoop shims 管理）。
4. 将该目录添加到系统环境变量 `PATH` 中：
   ```powershell
   # 永久追加到当前用户 PATH 环境变量
   [Environment]::SetEnvironmentVariable(
       "Path",
       [Environment]::GetEnvironmentVariable("Path", "User") + ";C:\Tools",
       "User"
   )
   ```
5. 重启终端或刷新环境变量，验证安装：
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

