# bque 命令判定矩阵与安全指南 / Usage Matrix & Safety Guide


## 1. 原生命令到 bque 调度的映射对照表

| 原始原生命令 | 推荐执行方式（接入 bque） | 底层 bque 模式 | 进程优先级类别 |
| :--- | :--- | :--- | :--- |
| `cargo build` | `just build` | `bque run` | `Idle`（编译子进程树全量继承） |
| `cargo check --workspace` | `just check` | `bque run` | `Idle` |
| `cargo clippy -- -D warnings` | `just clippy` 或 `bque-runner.ps1 -Cmd "..."` | `bque run` | `Idle` |
| `cargo test --workspace` | `just test` | `bque run` | `Idle` |
| `pnpm build` | `just build` 或 `bque-runner.ps1 -Cmd "pnpm build"` | `bque run` | `Idle` |
| `cmake --build target/debug` | `just build` | `bque run` | `Idle` |
| `cargo bench` | `just bench` | `bque bench` | `High`（自动秒级挂起后台编译） |
| `pnpm bench` | `just bench` | `bque bench` | `High`（自动秒级挂起后台编译） |
| `cargo fmt` | `cargo fmt`（直接运行） | **旁路直跑** | 前台默认优先级 |
| `git status` / `git commit` | 直接运行 | **旁路直跑** | 前台默认优先级 |

---

## 2. Windows 4KB 匿名管道死锁原理解析与防范姿势

### 致命陷阱（反例）
在 PowerShell 中对原生控制台程序使用管道截断：
```powershell
# 严禁写法（反例）：千万不要这样写！
cargo test 2>&1 | Select-Object -First 100
```
- PowerShell 为管道创建了仅 4KB 容量的 Windows 匿名管道缓冲区。
- `Select-Object -First 100` 在读取满足 100 行后协同结束读取，不再消费管道数据。
- 上游的原生编译器进程（`cargo.exe`、`cl.exe` 或 `ninja.exe`）继续向标准输出写日志。
- 一旦 4KB 缓冲区写满，底层内核 `WriteFile` 系统调用无限期阻塞等待读者读取。
- 整个进程树彻底卡死，直到 120 秒工具调用超时被强杀。

### 正确姿势（先重定向落盘，再读取截断）
```powershell
# 正确姿势：使用 *> 将全部输出重定向至临时文件，再通过 Get-Content 读取
cargo test *> test_out.log
Get-Content test_out.log -TotalCount 100
Remove-Item test_out.log -ErrorAction SilentlyContinue
```

---

## 3. 异常状态诊断与死锁自愈

当由于断电、强制杀进程或异常崩溃导致编译锁残留时：
```powershell
# 1. 检查当前持锁者与排队队列
just queue-status   # 或 bque status

# 2. 清理残留锁与孤儿等待记录
just queue-clean    # 或 bque clean

# 3. 手动暂停与恢复后台构建（在进行重度游戏或系统级压测时）
bque pause
bque resume
```

