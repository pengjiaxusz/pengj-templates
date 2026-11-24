{% if options["skill_lang"] == "en" %}
# bque Command Decision Matrix & Safety Guide

## 1. Command Mapping Matrix

| Original Native Command | Recommended Execution (with bque) | Weight / Mode | Priority Class |
| :--- | :--- | :--- | :--- |
| `cargo build` | `just build` | `heavy` (`100`, auto-yielding) | `Idle` (inherited by tree) |
| `cargo check --workspace` | `just check` | `heavy` (`100`) or `--weight medium` (`50`) | `Idle` |
| `cargo clippy -p <crate>` | `bque run --weight light -- cargo clippy -p <crate>` | `light` (`25`, concurrent) | `Idle` |
| `cargo test --workspace` | `just test` | `heavy` (`100`, auto-yielding) | `Idle` |
| `cargo test -p <crate> --lib` | `bque run --weight light -- cargo test -p <crate> --lib` | `light` (`25`, concurrent) | `Idle` |
| `pnpm build` / `tsc` | `just build` or `bque run -- pnpm build` | `heavy` (`100`) | `Idle` |
| `cmake --build target/debug` | `just build` or `bque run --scope cmake-debug -- cmake --build target/debug` | `heavy` + `--scope` | `Idle` |
| `ninja -C target/debug <target>` | `just test-build <target>` or `bque run --scope cmake-debug -- ninja ...` | `heavy` + `--scope` | `Idle` |
| `ctest --output-on-failure` | `just test` or `bque run -- ctest ...` | `heavy` (`100`) | `Idle` |
| `cargo bench` / `pnpm bench` | `just bench` or `bque bench -- <cmd>` | `bque bench` | `High` (freezes all active tasks & observers) |
| `cargo fmt` / `git ...` | Direct command | **Bypass** | Foreground |
| Automated / Agent build or test | `bque run --stall-timeout 300 --max-duration 1800 -- <cmd>` | Watchdog (`exit 124`) | `Idle` |
| Long-running service (`xtask dev`, `pnpm dev`, `cargo watch`) | `bque run --observer -- <cmd>` (**never** plain `bque run`) | `--observer` (`0` Budget) | `Idle` |

### Key Scheduling Mechanics (v0.3.0+)

- **100-unit Budget & Backfill**: Total host budget is `100`. Multiple `light` (`25`) or `medium` (`50`) tasks run concurrently. Smaller waiters may backfill spare capacity ahead of a blocked heavy waiter until the head waiter has waited `60s` (starvation cap).
- **Automatic Yielding & Soft Oversubscription**: Any running task staying below `5%` host CPU for `--yield-after` seconds (default `10s`) enters `Yielding` and temporarily lends its budget to waiters without being killed (disable with `--no-yield`). If a yielding task later resumes heavy CPU work while a borrower is still running, both stay `Active` under soft oversubscription (`OVERSUBSCRIBED`), splitting the CPU soft-cap proportionally while blocking further admissions.
- **Scope Exclusion (`--scope <key>`)**: Tasks sharing a `--scope` key never overlap — even if the holder enters `Yielding` — protecting shared build directories from corruption.
- **Opt-in Watchdogs (Exit `124`)**: `--stall-timeout <secs>` terminates the child tree if process-tree CPU stays `<0.5%` continuously; `--max-duration <secs>` enforces a hard wall-clock ceiling. Both preserve `Stdio::inherit()` native TTY output and return exit code `124`.
- **Cross-Platform Boundary**: `bque` runs natively on both Windows and Linux/WSL2. Generated `justfile` `[unix]` recipes pass through without `bque` so CI runners stay zero-dependency; invoke `bque` directly on Linux/WSL2 dev hosts when cross-repo queuing is desired.

---

## 2. Windows 4KB Anonymous Pipe Deadlock & Safe Handling

### The Problem
```powershell
# DANGEROUS ANTI-PATTERN: DO NOT RUN THIS!
cargo test 2>&1 | Select-Object -First 100  # pipeline-hygiene: ok anti-pattern doc
```
PowerShell creates a 4KB anonymous pipe. Once `Select-Object -First 100` closes its read end, the upstream native compiler blocks indefinitely on Win32 `WriteFile`.

### The Safe Pattern (Redirect to disk first)
```powershell
cargo test *> test_out.log
Get-Content test_out.log -TotalCount 100
Remove-Item test_out.log -ErrorAction SilentlyContinue
```

---

## 3. Abnormal State Recovery & Orphan Freeze Healing

Every queue record (`tasks/task_<pid>.json`, `waiters/`, `observers/`, `bench_state.json`, `pause_state.json`) is pinned to `(pid, created_at_ms)` and refreshed by a heartbeat. Coordination uses a short-lived `Bque_Coord_Lock` rather than a long-held global mutex:

- Dead or recycled-PID records are reaped automatically on any `status`, `run`, or `bench` call.
- If a `bque bench` process crashes while background builds were suspended, the next `reap()` / `bque status` / `bque clean` verifies recorded process identities and **automatically unfreezes** the orphaned build trees.

```powershell
# 0. Verify installation and host tier (`bque probe --json`)
just doctor

# 1. Inspect active tasks, Budget (used/100), live CPU%, Yielding state, and waiters
bque status

# 2. Clean dead records and auto-unfreeze any orphaned frozen processes
bque clean

# 3. Emergency unblock: drop all state records and resume any frozen processes
bque clean --force

# 4. Manually freeze/resume active builds (optionally keep dev-server observers alive)
bque pause --keep-observers --reason "Gaming or profiling"
bque resume
```

{% else %}
# bque 命令判定矩阵与安全指南

## 1. 原生命令到 bque 调度的映射对照表

| 原始原生命令 | 推荐执行方式（接入 bque） | 权重 / 模式 | 进程优先级类别 |
| :--- | :--- | :--- | :--- |
| `cargo build` | `just build` | `heavy`（`100`，支持自动让槽） | `Idle`（子进程树全量继承） |
| `cargo check --workspace` | `just check` | `heavy`（`100`）或 `--weight medium`（`50`） | `Idle` |
| `cargo clippy -p <crate>` | `bque run --weight light -- cargo clippy -p <crate>` | `light`（`25`，可并发） | `Idle` |
| `cargo test --workspace` | `just test` | `heavy`（`100`，支持自动让槽） | `Idle` |
| `cargo test -p <crate> --lib` | `bque run --weight light -- cargo test -p <crate> --lib` | `light`（`25`，可并发） | `Idle` |
| `pnpm build` / `tsc` | `just build` 或 `bque run -- pnpm build` | `heavy`（`100`） | `Idle` |
| `cmake --build target/debug` | `just build` 或 `bque run --scope cmake-debug -- cmake --build target/debug` | `heavy` + `--scope` | `Idle` |
| `ninja -C target/debug <目标>` | `just test-build <目标>` 或 `bque run --scope cmake-debug -- ninja ...` | `heavy` + `--scope` | `Idle` |
| `ctest --output-on-failure` | `just test` 或 `bque run -- ctest ...` | `heavy`（`100`） | `Idle` |
| `cargo bench` / `pnpm bench` | `just bench` 或 `bque bench -- <命令>` | `bque bench` | `High`（冻结所有活跃任务与观察者） |
| `cargo fmt` / `git ...` | 直接运行 | **旁路直跑** | 前台默认优先级 |
| Agent / 自动化构建或测试 | `bque run --stall-timeout 300 --max-duration 1800 -- <命令>` | 看门狗（退出码 `124`） | `Idle` |
| 常驻服务（`xtask dev`、`pnpm dev`、`cargo watch`） | `bque run --observer -- <命令>`（**绝不**裸 `bque run`） | `--observer`（`0` 预算） | `Idle` |

### 核心调度机制（v0.3.0+）

- **100 单位预算与有限 Backfill**：整机共享 `100` 预算。多个 `light`（`25`）或 `medium`（`50`）任务可直接并发；当队头重任务暂时放不下时，更小的等待者可在剩余预算内插队（Backfill），但队头等待满 `60s` 后锁定插队防止饥饿。
- **自动让槽（Yielding）与软超发（Oversubscription）**：任意运行中任务若整棵进程树连续 `--yield-after` 秒（默认 `10s`）整机 CPU `<5%`，自动进入 `Yielding` 并把预算临时借给等待者，不杀进程（可用 `--no-yield` 关闭）。若原任务之后恢复高 CPU，双方以软超发（`OVERSUBSCRIBED`）共生，由 Governor 按权重平分 CPU 软上限并暂停放行新任务。
- **局部互斥作用域（`--scope <key>`）**：相同 Scope 键的任务**即使持有者进入 `Yielding` 也绝不并发**，用于保护共享构建目录。
- **可选看门狗（退出码 `124`）**：`--stall-timeout <秒>` 在进程树持续 `<0.5%` CPU 时强杀；`--max-duration <秒>` 限制硬总时长。两者均保留 `Stdio::inherit()` 原生 TTY 输出并返回退出码 `124`。
- **跨平台边界**：`bque` 原生支持 Windows 与 Linux/WSL2。模板 `justfile` 的 `[unix]` 分支保持直跑透传是为了让 CI 零依赖；Linux/WSL2 开发机按需直接调原生 `bque` CLI 即可。

---

## 2. Windows 4KB 匿名管道死锁原理解析与防范姿势

### 致命陷阱（反例）
```powershell
# 严禁写法（反例）：千万不要这样写！
cargo test 2>&1 | Select-Object -First 100  # pipeline-hygiene: ok 反例演示
```
PowerShell 匿名管道缓冲区仅 4KB，`Select-Object -First 100` 提前关闭读端后，上游原生编译器会在 Win32 `WriteFile` 上无限阻塞。

### 正确姿势（先重定向落盘，再读取截断）
```powershell
cargo test *> test_out.log
Get-Content test_out.log -TotalCount 100
Remove-Item test_out.log -ErrorAction SilentlyContinue
```

---

## 3. 异常状态诊断与孤儿冻结自愈

每条队列记录（`tasks/task_<pid>.json`、`waiters/`、`observers/`、`bench_state.json`、`pause_state.json`）均绑定 `(pid, created_at_ms)` 进程实例身份并由心跳刷新，底层协调改为短时 `Bque_Coord_Lock`（不再长期独占全局编译锁）：

- 进程退出或 PID 复用时，任意 `status` / `run` / `bench` 都会自动清理僵尸记录；
- 若 `bque bench` 在冻结后台构建后异常崩溃，下一次 `reap()` / `bque status` / `bque clean` 会核对进程身份并**自动解冻**残留的孤儿冻结进程树。

```powershell
# 0. 自检安装状态与主机资源档位（bque probe --json）
just doctor

# 1. 查看活跃任务、预算占用（used/100）、实时 CPU%、Yielding 状态与等待队列
bque status

# 2. 清理死记录并自动解冻孤儿冻结进程
bque clean

# 3. 应急解锁：强制清空所有状态记录并恢复冻结中的进程
bque clean --force

# 4. 手动冻结/恢复活跃构建（可用 --keep-observers 保留 dev server 运行）
bque pause --keep-observers --reason "打游戏或性能分析"
bque resume
```

{% endif %}
