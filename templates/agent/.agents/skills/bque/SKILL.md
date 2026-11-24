---
name: bque
description: >-
{% if options["skill_lang"] == "en" %}
  Cross-platform (Windows & Linux/WSL2) build queue with 100-unit budget scheduling, automatic low-CPU yielding, mutual-exclusion scopes, stall/duration watchdogs (exit 124), CPU throttling, clean-room benchmark protection, and Windows 4KB pipe hygiene. Use when running builds, cargo, cmake, ninja, pnpm build, tests, benchmarks, profiling, or when encountering CPU contention, hanging commands, or build lock collisions.
  Triggers: bque, build-queue, queue, build queue, run build, cargo build, pnpm build, cmake build, ninja, ninja -C, cmake, ctest, cargo test, cargo clippy, bench, benchmark, profiling, queue-status, queue-clean, compile freeze, cpu throttle, pipeline deadlock, stall-timeout, yielding, budget.
{% else %}
  跨平台（Windows 与 Linux/WSL2）全局编译预算调度、低占用自动让槽（Yielding）、局部互斥作用域（Scope）、停滞/超时看门狗（退出码 124）、后台 CPU 降级与纯净基准测试保护技能。规范 Agent 如何按任务权重分级接入 `bque`、防范 Windows 4KB 管道死锁、以及排查队列异常与孤儿冻结进程。当需要执行构建、编译、检查、单测、性能跑分（bench）、性能画像，或遇到任务卡死堵队、CPU 争抢、编译锁冲突时使用。
  Triggers: bque, build-queue, 排队构建, 全局排队, 预算调度, 自动让槽, 编译排队, 跑分保护, 纯净跑分, 冻结后台构建, 挂起构建, cpu降级, idle优先级, queue-status, queue-clean, 编译锁, 管道死锁, 管道卫生, 看门狗, stall-timeout, ninja, ctest, cmake, cargo, 编译, 构建, 跑测试, 运行测试, 单测, 性能跑分, bench.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# bque — Cross-Platform Budget Scheduler, Activity Watchdog & Benchmark Protector

A daemonless build queue and resource governor for Windows and Linux/WSL2. Instead of a blind single-slot lock, `bque` coordinates tasks through a **100-unit shared Budget**, automatically lends capacity (**Yielding**) when a running task stays at low CPU, protects shared build directories with optional **Scope** locks, catches hung tasks with opt-in **Watchdogs**, and freezes background builds for 0% CPU interference during benchmarks.

---

## 1. Decision Table: Weight Tiers, Modes & Bypass

| Scenario | Use bque? | Recommended Invocation | Reason |
| :--- | :---: | :--- | :--- |
| **Heavy compilation** (`cargo build`, `cmake --build`, `ninja`, `pnpm build`, `tsc`) | **YES (`heavy`)** | `just build` or `bque run -- <cmd>` | Claims default `weight=100` (serializes active heavy builds); yields automatically if CPU stays `<5%` for 10s |
| **Workspace check / full test** (`cargo clippy --workspace`, `cargo test --workspace`) | **YES (`heavy` or `medium`)** | `just check` / `just test` or `bque run --weight medium -- <cmd>` | Throttled to `Idle`; `medium` (`50`) allows two moderate tasks to overlap when safe |
| **Targeted single-crate check / unit test / linter** | **YES (`light`)** | `bque run --weight light -- cargo check -p <crate>` | Claims `25/100` budget; runs concurrently with other light tasks and backfills idle capacity |
| **Shared build directory collision risk** (e.g. same CMake/Ninja build dir) | **YES (`--scope`)** | `bque run --scope <dir-key> -- <cmd>` | Strict mutual exclusion per scope — never overlaps even if the holder enters `Yielding` |
| **Automated / Agent long commands** (risk of hangs or infinite loops) | **YES (Watchdog)** | `bque run --stall-timeout 300 --max-duration 1800 -- <cmd>` | Kills child tree and exits **`124`** on `<0.5%` CPU stall or hard duration limit while keeping native TTY |
| **Benchmarks & Profiling** (`cargo bench`, `perf`, timing probes) | **YES (`bench`)** | `just bench` or `bque bench -- <cmd>` | **Clean-room mode**: freezes all active tasks & observers with RAII auto-resume; elevates to `High` |
| **Long-running service** (dev server, watch loop, `xtask dev`, `pnpm dev`) | **YES (`--observer`)** | `bque run --observer -- <cmd>` | Registered in status and throttled, but consumes `0` Budget and never blocks waiters |
| **Nested / recursive call** (`BQUE_ACTIVE_PID` set) | **NO** | Direct command or `bque run` (native auto-bypass) | Already supervised by outer `bque`; avoids nested queuing |
| **Fast formatting & Git** (`cargo fmt`, `git status`, `commit`, `push`) | **NO** | Direct command | Sub-second or I/O/network ops must never queue behind builds |
| **Queue control & inspection** (`status`, `clean`, `pause`, `resume`, `probe`) | **NO** | `bque status` / `bque clean` / `bque pause` | Control-plane commands execute immediately |

Full command mapping and recovery details: [references/usage-matrix.md](references/usage-matrix.md).

---

## 2. Prohibited Raw Commands (Mandatory Wrappers)

AI assistants must **never** run raw heavy build, test, or benchmark commands directly in the terminal on managed workstations. Always convert them to wrapped forms:

| Prohibited Raw Command | Hazards | Mandatory Wrapped Alternative |
| :--- | :--- | :--- |
| `ninja -C <dir> <target>` | Saturates all cores, freezes desktop, collides across worktrees | `just test-build <target>` or `bque run --scope <dir> -- ninja -C <dir> <target>` |
| `cmake --build <dir> ...` | Build directory lock contention, starves foreground UI | `just build` or `bque run --scope <dir> -- cmake --build <dir> ...` |
| `cargo build` / `cargo test` / `cargo clippy` | CPU saturation and cross-worktree contention | `just build` / `just test` / `just check`, or `bque run [--weight light] -- cargo ...` |
| Raw benchmark binary / `cargo bench` | Background builds pollute L3 cache and skew timings | `just bench` or `bque bench -- <cmd>` |
| `ctest ...` / `<TestBinary>.exe` | Unthrottled multi-threaded test bursts | `just test` or `bque run -- ctest ...` |
| `pnpm build` / `tsc` (large workspaces) | V8 CPU and memory spikes | `just build` or `bque run -- pnpm build` |

---

## 3. Two Execution Tiers & Platform Boundary

### Tier 1: Unified `just` Entry (Preferred)
Check the repository's `justfile` first; standard targets already wire `bque`:
```powershell
just build           # Queued build at Idle priority (default weight=100, auto-yielding)
just check           # Queued clippy/check
just test            # Queued test suite
just bench           # Clean-room benchmark (freezes active tasks & observers)
bque status          # Inspect active tasks, Budget (used/100), live CPU%, waiters
bque clean           # Reap dead records and auto-unfreeze orphaned frozen processes
just doctor          # Verify toolchain, bque version, and host resource tier (`bque probe`)
```

**Platform note**: `bque` works natively on **both Windows and Linux/WSL2** (cross-platform budget scheduling, yielding, watchdogs, and freezing). In generated `justfile` templates, `[unix]` recipes intentionally pass through without `bque` so CI runners need zero extra setup; on Linux/WSL2 workstations with `bque` installed, invoke `bque run` / `bque bench` directly whenever you want cross-repo coordination.

### Tier 2: Direct Native `bque` CLI (Ad-hoc & Tuned Calls)
When you need custom flags (`--weight`, `--scope`, `--stall-timeout`, `--max-duration`, `--observer`, `--cwd`), call `bque` directly:
```powershell
# Default heavy build (weight=100, auto-yields after 10s at <5% CPU)
bque run -- cargo build --release

# Lightweight concurrent check (weight=25)
bque run --weight light -- cargo clippy -p my-crate

# Strict directory isolation + watchdog for automated runs (exit 124 on stall/timeout)
bque run --scope target-debug --stall-timeout 300 --max-duration 1800 -- cargo test

# Pause active builds manually (optionally keep dev-server observers running)
bque pause --keep-observers --reason "Interactive profiling"
bque resume
```

---

## 4. Two Hard Hygiene Rules

1. **Windows 4KB Anonymous Pipe Hygiene (Strictly Forbidden)**:
   - **FORBIDDEN**: Never pipe native console build commands into PowerShell truncating filters (e.g., `cargo test 2>&1 | Select-Object -First 100` # pipeline-hygiene: ok anti-pattern doc).
   - **Why**: Native Windows processes ignore PowerShell cooperative breaks; once the 4KB pipe buffer fills, `WriteFile` blocks forever.
   - **Fix**: Redirect to a file first, then inspect: `cargo test *> out.log; Get-Content out.log -TotalCount 100`.
2. **Scoped Process Termination**:
   - Never kill processes blindly by image name (`Stop-Process -Name ...`). Match strictly by worktree path or use `bque`'s own `--stall-timeout` / `--max-duration`.

---

## 5. Environment & Installation

Run `just doctor` to check `bque` availability and host tier (`bque probe --json`). Full setup guide: [references/install.md](references/install.md).

{% else %}
# bque — 跨平台预算调度、活跃度看门狗与纯净跑分保护技能

面向 Windows 与 Linux/WSL2 的无守护进程构建调度与资源管控工具。`bque` 采用 **100 单位共享预算（Budget）+ 低占用自动让槽（Yielding）**模型，既保证重型编译互不踩踏、前台丝滑，又彻底消除“前序任务卡住或低占用导致后面全傻等”的队头阻塞；同时支持 `--scope` 局部互斥、`--stall-timeout` / `--max-duration` 看门狗以及 0% 干扰的纯净跑分冻结。

---

## 1. 选型决策表：权重分级、模式与旁路

| 场景分类 | 是否使用 bque？ | 推荐调用方式 | 选型理由 |
| :--- | :---: | :--- | :--- |
| **重型编译构建**（`cargo build`、`cmake --build`、`ninja`、`pnpm build`、`tsc`） | **必须（`heavy`）** | `just build` 或 `bque run -- <命令>` | 默认 `weight=100` 保持重编译串行；若持续 10s CPU `<5%` 会自动进入 `Yielding` 让出配额 |
| **工作区全量检查与测试**（`cargo clippy --workspace`、`cargo test --workspace`） | **必须（`heavy`/`medium`）** | `just check` / `just test` 或 `bque run --weight medium -- <命令>` | 降为 `Idle` 优先级防卡前台；中等负载可用 `medium`（`50`）允许双任务并发 |
| **单包轻量检查 / 单测 / Linter** | **推荐（`light`）** | `bque run --weight light -- cargo check -p <crate>` | 仅占 `25/100` 预算，可与其他轻任务并发或通过 Backfill 利用空闲配额 |
| **共享构建目录防踩踏**（如同一 CMake/Ninja 构建目录） | **必须（`--scope`）** | `bque run --scope <目录键> -- <命令>` | 同一 Scope 严格互斥——**即使持有者进入 `Yielding` 也不会并发** |
| **Agent / 自动化长任务**（防 0% CPU 挂死或死循环） | **推荐（看门狗）** | `bque run --stall-timeout 300 --max-duration 1800 -- <命令>` | 保留原生 TTY，连续 `<0.5%` CPU 或超总时长时强杀进程树并返回退出码 **`124`** |
| **性能跑分与基准测试**（`cargo bench`、`perf`、耗时画像） | **必须（`bench`）** | `just bench` 或 `bque bench -- <命令>` | **纯净跑分模式**：带 RAII 守卫冻结所有活跃任务与观察者，提权至 `High`，退出自动恢复 |
| **常驻服务**（dev server、watch 循环、`xtask dev`、`pnpm dev`） | **必须（`--observer`）** | `bque run --observer -- <命令>` | 登记状态并限流，但消耗 `0` 预算、绝不阻塞队列 |
| **递归/嵌套调用**（已设 `BQUE_ACTIVE_PID`） | **无需** | 原生命令直跑或 `bque run`（自动穿透） | 外层已受 `bque` 纳管，避免重复排队 |
| **秒级格式化与 Git 操作**（`cargo fmt`、`git status`/`commit`/`push`） | **无需** | 原生命令直跑 | 轻量或 I/O/网络操作不应堵在编译队列后 |
| **控制面与自省**（`status`、`clean`、`pause`、`resume`、`probe`） | **无需** | `bque status` / `bque clean` / `bque pause` | 诊断与控制命令瞬时直达 |

完整命令对照与异常自愈见 [references/usage-matrix.md](references/usage-matrix.md)。

---

## 2. 严禁裸跑原生重型命令（强制包装表）

AI 助手在开发机终端中**严禁**直接裸跑重型编译、测试或基准测试命令，必须按以下方式接入 `bque`：

| 严禁裸跑的原生命令 | 危害与问题 | 强制替换调用方式 |
| :--- | :--- | :--- |
| `ninja -C <dir> <target>` | 打满全部核心、卡死桌面、多工作树冲突 | `just test-build <target>` 或 `bque run --scope <dir> -- ninja -C <dir> <target>` |
| `cmake --build <dir> ...` | 构建目录锁争用、前台交互卡顿 | `just build` 或 `bque run --scope <dir> -- cmake --build <dir> ...` |
| `cargo build` / `cargo test` / `cargo clippy` | CPU 抢占、多工作树互踩 | `just build` / `just test` / `just check`，或 `bque run [--weight light] -- cargo ...` |
| 原生基准测试 / `cargo bench` | 后台构建污染 L3 缓存导致跑分失真 | `just bench` 或 `bque bench -- <命令>` |
| `ctest ...` / `<TestBinary>.exe` | 未降级多线程测试瞬间挤爆 CPU | `just test` 或 `bque run -- ctest ...` |
| `pnpm build` / `tsc`（大型前端） | V8 内存与 CPU 尖峰卡顿 | `just build` 或 `bque run -- pnpm build` |

---

## 3. 双阶梯执行规范与平台边界

### 阶梯 1：优先使用 `just` 统一入口
优先查阅仓库 `justfile`，标准配方已默认接入 `bque`：
```powershell
just build           # 预算排队 + Idle 优先级构建（默认 weight=100，支持自动让槽）
just check           # 排队代码检查
just test            # 排队运行测试
just bench           # 纯净基准测试（安全冻结全部活跃任务与观察者）
bque status          # 查看活跃任务、预算占用（used/100）、实时 CPU%、Yielding 与等待队列
bque clean           # 清理僵尸记录并自动解冻异常崩溃遗留的冻结进程
just doctor          # 自检工具链、bque 版本与机器资源档位（bque probe）
```

**平台说明**：`bque` 原生支持 **Windows 与 Linux/WSL2 双平台**（预算准入、进程树 CPU 活跃度采样、自动让槽、看门狗与冻结均跨平台可用）。模板生成的 `justfile` 中 `[unix]` 分支默认直跑透传是为了让 CI Runner 零额外依赖；在已装 `bque` 的 Linux/WSL2 开发机上需要跨仓库协调时，直接调用 `bque run` / `bque bench` 即可。

### 阶梯 2：原生 `bque` CLI 直调（定制权重、作用域与看门狗）
需要精细控制 `--weight`、`--scope`、`--stall-timeout`、`--max-duration`、`--observer` 或 `--cwd` 时，直接调用原生 CLI：
```powershell
# 默认重型构建（weight=100，持续 10s CPU <5% 自动让槽）
bque run -- cargo build --release

# 单包轻量并发检查（weight=25）
bque run --weight light -- cargo clippy -p my-crate

# 强目录隔离 + 自动化看门狗（停滞或超时强杀进程树并返回退出码 124）
bque run --scope target-debug --stall-timeout 300 --max-duration 1800 -- cargo test

# 手动冻结后台构建（可用 --keep-observers 保留 dev server 不被冻住）
bque pause --keep-observers --reason "交互式性能分析"
bque resume
```

---

## 4. 构建安全两铁律

1. **Windows 4KB 匿名管道卫生（严禁截断原生管道）**：
   - **反例（严禁）**：`cargo test 2>&1 | Select-Object -First 100`（# pipeline-hygiene: ok 反例演示）。
   - **原理**：Windows 原生进程不响应 PowerShell 协同断开，4KB 管道写满后 `WriteFile` 永久阻塞。
   - **正确姿势**：先重定向落盘再读取：`cargo test *> out.log; Get-Content out.log -TotalCount 100`。
2. **精准进程终止**：
   - 严禁按进程名无差别盲杀（`Stop-Process -Name ...`）；必须按工作树路径精确匹配，或直接交给 `bque` 的 `--stall-timeout` / `--max-duration` 清理整棵进程树。

---

## 5. 环境检测与安装指引

运行 `just doctor` 可一次检测 `bque` 安装状态与主机资源档位（`bque probe --json`）。完整安装指南见 [references/install.md](references/install.md)。

{% endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- The following is the project-specific area: template updates only replace the managed block above; this area is owned by the project and fully preserved. -->
## Project-Specific Queue Policies & Custom Configuration

> Register project-specific weight tiers, `--scope` keys, watchdog timeouts (`--stall-timeout` / `--max-duration`), or setup scripts here.
{% else %}
<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属排队策略与定制配置

> 可在此登记本项目的任务权重约定、`--scope` 互斥键、看门狗超时阈值（`--stall-timeout` / `--max-duration`）或前置准备脚本。
{% endif %}
