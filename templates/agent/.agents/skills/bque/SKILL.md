---
name: bque
description: >-
{% if options["skill_lang"] == "en" %}
  Universal cross-worktree build queue serialization, CPU throttling, and clean-room benchmark protection skill. Enforces Idle priority, clean-room benchmark execution (auto-freezing background builds), queue introspection, pipeline hygiene (preventing 4KB anonymous pipe deadlocks on Windows), and deadlock recovery. Use when running builds, cargo, cmake, ninja, pnpm build, tests, benchmarks, profiling, or when encountering CPU contention, hanging commands, or build lock errors.
  Triggers: bque, build-queue, queue, build queue, run build, cargo build, pnpm build, cmake build, ninja, ninja -C, cmake, ctest, cargo test, cargo clippy, bench, benchmark, profiling, queue-status, queue-clean, compile freeze, cpu throttle, pipeline deadlock.
{% else %}
  通用跨工作树全局编译排队、后台 CPU 降级防卡顿与纯净基准测试保护技能。规范 Agent 何时必须走排队调度、何时直跑，提供纯净跑分独占执行（秒级挂起后台构建）、构建管道安全铁律（防 Windows 4KB 管道永久死锁）、全局状态自省与死锁自愈。当需要执行重型构建、编译、检查、单测、性能跑分（bench）、性能画像分析，或遇到 CPU 占满卡顿、命令挂死、编译锁争用时使用。
  Triggers: bque, build-queue, 排队构建, 全局排队, 编译排队, 跑分保护, 纯净跑分, 冻结后台构建, 挂起构建, cpu降级, idle优先级, queue-status, queue-clean, 编译锁, 管道死锁, 管道卫生, ninja, ctest, cmake, cargo, 编译, 构建, 跑测试, 运行测试, 单测, 性能跑分, bench, ninja -C, cmake --build, cargo build, cargo test.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# bque — Universal Build Queue, CPU Governor & Clean-Room Benchmark Protector

A cross-worktree build queue and governor skill. Prevents multi-repository build collisions, keeps foreground interactivity silky-smooth via `Idle` priority lowering, guarantees 0% CPU background disturbance for benchmarks via process freezing, and enforces pipeline hygiene.

---

## ⚡ 1. When to Use vs When NOT to Use bque

| Scenario | Use bque? | Mode / Command | Reason |
| :--- | :---: | :--- | :--- |
| **Heavy Compilation** (`cargo build`, `cmake`, `ninja`, `pnpm build`, `tsc`, `xtask pack`) | **YES** | `just build` or `tools/bque-runner.ps1 -Cmd "<cmd>"` | Serializes builds, lowers process priority to `Idle` to avoid freezing the OS |
| **Workspace Lint & Test** (`cargo check`, `cargo clippy`, `cargo test`, `vitest run`) | **YES** | `just check` / `just test` or `bque-runner.ps1 -Cmd "<cmd>"` | High CPU utilization tasks require queue mutual exclusion and background throttling |
| **Benchmarks & Profiling** (`bench`, `perf`, timing probes) | **YES** | `just bench` or `tools/bque-runner.ps1 -Bench -Cmd "<cmd>"` | **Clean-room mode**: automatically freezes background builds to ensure 0% CPU disturbance |
| **Nested / Recursive invocation** (`BQUE_ACTIVE_PID` set) | **NO** | Direct command (`& <cmd>`) | Already under bque governor supervision; avoids recursive queue deadlock |
| **Fast Formatting & Lint** (`cargo fmt`, single-file `prettier`/`eslint`) | **NO** | Direct command | Fast completion (<1s), negligible CPU usage, no need to queue |
| **Git Operations** (`git status`, `commit`, `push`, `rebase`) | **NO** | Direct command | I/O-bound operational commands, must never queue behind builds |
| **State Inspection** (`queue-status`, `queue-clean`) | **NO** | `just queue-status` / `just queue-clean` | Control and diagnostic tools must run immediately |

See [references/usage-matrix.md](references/usage-matrix.md) for full scenario matrix and command mappings.

---

## 🛑 2. Strictly Prohibited Raw Commands (Mandatory Wrappers)

AI assistants are **STRICTLY PROHIBITED** from executing raw build or compiler commands directly in the terminal! Bypassing `bque` / `just` starves all CPU cores, freezes desktop interactivity, contaminates benchmark accuracy, and triggers Windows 4KB pipe deadlocks. Always convert raw commands into their wrapped forms:

| Prohibited Raw Command (FORBIDDEN) | Hazards & Failure Modes | Mandatory Wrapped Alternative |
| :--- | :--- | :--- |
| ❌ `ninja -C <dir> <target>` / `ninja ...` | Consumes 100% CPU cores, freezes the system, causes worktree conflicts | `just test-build <target>` or `pwsh tools/bque-runner.ps1 -Cmd "ninja -C <dir> <target>"` |
| ❌ `cmake --build <dir> ...` | Causes target lock collisions, starves foreground responsiveness | `pwsh tools/locked-cmake-build.ps1 ...` or `just build` or `pwsh tools/bque-runner.ps1 -Cmd "cmake --build ..."` |
| ❌ `cargo build` / `cargo test` / `cargo clippy` | Cargo target directory lock collisions, heavy CPU contention | `just build` / `just test` / `just check` or `pwsh tools/bque-runner.ps1 -Cmd "cargo ..."` |
| ❌ Raw Benchmark invocation (e.g. `ninja -C target/debug DuntingThumbnailPipelineBenchmark` or `./bench.exe`) | **Fatal Benchmark Corruption**: background builds pollute CPU cache, invalidating benchmark data | `just test-bench` or `pwsh tools/bque-runner.ps1 -Bench -Cmd "..."` (auto-freezes background builds for 0% CPU disturbance) |
| ❌ `ctest ...` / `<TestBinary>.exe` | Unthrottled multi-threaded test runner saturates CPU | `just test` or `pwsh tools/run-tests.ps1` or `pwsh tools/bque-runner.ps1 -Cmd "ctest ..."` |
| ❌ `pnpm build` / `tsc` (in large frontend workspaces) | V8 engine multi-thread memory and CPU spikes | `just build` or `pwsh tools/bque-runner.ps1 -Cmd "pnpm build"` |

---

## 🚀 3. Three Execution Tiers (Priority Order)

### Tier 1: Unified `just` Entry (Recommended)
Always check the repository's `justfile` first. Standard targets automatically invoke the bque runner:
```powershell
just build           # Compile under bque queue & Idle priority
just check           # Check code under bque supervision
just test            # Run test suite safely
just bench           # Clean-room benchmark (freezes background builds)
just queue-status    # Inspect current lock holder and waiters
just queue-clean     # Clean abnormal or abandoned locks
```

### Tier 2: Universal Runner Delegation (When no matching just recipe exists)
If the project has `tools/bque-runner.ps1` and you need a custom build command:
```powershell
# Standard queue execution (-Cmd preserves complex flags and switches)
pwsh tools/bque-runner.ps1 -Desc "Custom Build" -Cmd "cargo build --release"

# Clean-room benchmark execution
pwsh tools/bque-runner.ps1 -Bench -Desc "Custom Benchmark" -Cmd "pnpm bench"
```

### Tier 3: Direct Native CLI (When outside repository)
When executing ad-hoc commands outside any repository that contains `tools/bque-runner.ps1`:
```powershell
bque run -d "Heavy Task" -p idle -- cargo build --workspace
bque bench -d "Performance Probe" -- cargo bench
bque status
bque clean
```
Inside projects, ALWAYS invoke through `just` or `tools/bque-runner.ps1` for automatic PATH/binary detection, recursion guard, and priority management.

---

## 🛡️ 4. Two Hard Ironclad Rules

1. **Pipeline Hygiene (Strictly Forbidden)**:
   - **FORBIDDEN**: Never pipe native console build commands into PowerShell truncation filters (e.g., `cargo test 2>&1 | Select-Object -First 100` # pipeline-hygiene: ok anti-pattern doc).
   - **Root Cause**: Windows native processes do not handle PowerShell cooperative breaks; when the 4KB anonymous pipe buffer fills up, the process blocks on write, resulting in a permanent 120s deadlock.
   - **Solution**: Always redirect to disk first, then inspect: `cargo test *> out.log; Get-Content out.log -TotalCount 100`.
2. **Process Termination Hygiene**:
   - When cleaning or rebuilding, **never kill processes by name blindly** (`Stop-Process -Name ...`).
   - Filter strictly by the current worktree's path before terminating to avoid killing processes in other working trees.

---

## 🔧 5. Environment & Installation

If `bque` is missing from system `PATH`, inspect and follow the installation guide:
👉 See [references/install.md](references/install.md).

{% else %}
# bque — 全局编译排队、CPU 调速与纯净跑分保护技能

用于协调跨工作树构建排队、前台防卡顿与纯净基准测试的工程基础设施技能。通过 Windows 内核原生对象将重型编译置于 `Idle` 空闲优先级，让路前台交互；在性能跑分时秒级挂起（Freeze）后台编译，保证 0% 后台 CPU 干扰与干净的 L3 缓存。

---

## ⚡ 1. 何时使用 vs 何时不使用 bque

| 场景分类 | 是否使用 bque？ | 推荐调用方式 | 选型理由 |
| :--- | :---: | :--- | :--- |
| **重型编译构建**（`cargo build`、`cmake`、`ninja`、`pnpm build`、`tsc`、`xtask pack`） | **必须使用** | `just build` 或 `tools/bque-runner.ps1 -Cmd "<命令>"` | 跨工作树串行排队，进程优先级降为 `Idle`，杜绝卡死前台 IDE 与桌面 |
| **工作区检查与测试**（`cargo check`、`cargo clippy`、`cargo test`、`vitest run`） | **必须使用** | `just check` / `just test` 或 `bque-runner.ps1 -Cmd "<命令>"` | 高负载 CPU 任务，接入排队避免多工作树争抢挤爆 CPU 与磁盘 IO |
| **性能跑分与基准测试**（`bench`、`perf`、耗时画像探针） | **必须使用（Bench 模式）** | `just bench` 或 `tools/bque-runner.ps1 -Bench -Cmd "<命令>"` | **纯净跑分保护**：秒级挂起后台正在跑的编译，提供干净 CPU 环境，结束后原位恢复 |
| **递归/嵌套会话**（检测到 `BQUE_ACTIVE_PID` 环境变量） | **禁止使用** | 原生命令直跑（`& <命令>`） | 当前进程树已被外层 bque 纳管，直接执行避免重复排队造成嵌套死锁 |
| **轻量格式化与单文件检查**（`cargo fmt`、单文件 `prettier`/`eslint`） | **无需使用** | 原生命令直跑 | 秒级极速完成，几乎不占 CPU，无需额外排队开销 |
| **Git 工作流操作**（`git status`、`commit`、`push`、`rebase`） | **无需使用** | 原生命令直跑 | I/O 与网络操作，绝对严禁堵在编译队列后面 |
| **队列自省与运维**（`queue-status`、`queue-clean`） | **无需使用** | `just queue-status` / `just queue-clean` | 控制面与排查诊断工具，必须瞬时直达 |

完整场景判定表与命令映射详见 [references/usage-matrix.md](references/usage-matrix.md)。

---

## 🛑 2. 严禁裸跑原生构建/测试命令（硬性禁令与自动转换表）

AI 助手**绝对严禁**在终端中直接裸跑任何底层构建或编译测试命令！绕过 `bque` / `just` 会吃满所有 CPU 逻辑核心导致系统交互冻结、跨工作树产生编译锁争用、破坏性能跑分置信度，并在 PowerShell 管道下引发 4KB 匿名管道死锁。必须严格按下表映射为包装调用：

| 严禁裸跑的原生命令 (FORBIDDEN RAW) | 危害与问题 | 强制替换调用的包装形式 (MANDATORY WRAPPED FORM) |
| :--- | :--- | :--- |
| ❌ `ninja -C <dir> <target>` / `ninja ...` | 抢占全部 CPU 逻辑核心导致系统卡死，多工作树冲突 | `just test-build <target>` 或 `pwsh tools/bque-runner.ps1 -Cmd "ninja -C <dir> <target>"` |
| ❌ `cmake --build <dir> ...` | 争用 CMake/Ninja 编译锁，打死前台交互 | `pwsh tools/locked-cmake-build.ps1 ...` 或 `just build` 或 `pwsh tools/bque-runner.ps1 -Cmd "cmake --build ..."` |
| ❌ `cargo build` / `cargo test` / `cargo clippy` | 产生 cargo target/ 锁争用，打满 CPU | `just build` / `just test` / `just check` 或 `pwsh tools/bque-runner.ps1 -Cmd "cargo ..."` |
| ❌ 原生基准测试（如 `ninja -C target/debug DuntingThumbnailPipelineBenchmark` 或 `./bench.exe`） | **致命跑分污染**：未冻结后台构建，CPU 与 L3 缓存严重受扰，测试数据失真 | `just test-bench` 或 `pwsh tools/bque-runner.ps1 -Bench -Cmd "..."`（自动挂起后台编译以提供 0% 干扰环境） |
| ❌ `ctest ...` / `<TestBinary>.exe` | 未降级的多线程并发测试抢占 CPU 造成前台卡顿 | `just test` 或 `pwsh tools/run-tests.ps1` 或 `pwsh tools/bque-runner.ps1 -Cmd "ctest ..."` |
| ❌ `pnpm build` / `tsc`（大型前端项目） | Node.js/V8 引擎内存与 CPU 突增卡顿 | `just build` 或 `pwsh tools/bque-runner.ps1 -Cmd "pnpm build"` |

---

## 🚀 3. 三阶梯执行规范（按优先级选用）

### 阶梯 1：统一优先使用 `just` 命令（首选）
项目日常命令已由 `justfile` 自动接入 `bque-runner.ps1`，AI 助手应优先运行 `just`：
```powershell
just build           # 全局排队 + Idle 优先级构建
just check           # 全局排队代码检查
just test            # 全局排队单测运行
just bench           # 纯净基准测试（自动冻结后台其他编译）
just queue-status    # 查看当前全局持锁者、排队任务与状态
just queue-clean     # 应急清理异常残留的编译锁状态
```

### 阶梯 2：通用代理脚本委派（当无对应 just 命令或需临时定制时）
若项目根目录存在 `tools/bque-runner.ps1`，需执行非预设的重型命令时使用：
```powershell
# 常规排队任务（-Cmd 支持复杂参数与带引号指令）
pwsh tools/bque-runner.ps1 -Desc "Custom Build" -Cmd "cargo build --release"

# 纯净跑分模式（独占排他锁，秒级挂起后台所有编译任务）
pwsh tools/bque-runner.ps1 -Bench -Desc "Custom Benchmark" -Cmd "pnpm bench"
```

### 阶梯 3：原生 CLI 直调（仅限无 runner 的外部目录或全局运维）
在没有 `tools/bque-runner.ps1` 的独立外部目录执行临时排队时调用：
```powershell
bque run -d "Heavy Task" -p idle -- cargo build --workspace
bque bench -d "Performance Probe" -- cargo bench
bque status
bque clean
```
在任何已接入的项目仓库内，**必须一律通过 `just` 或 `tools/bque-runner.ps1` 调度**，享受二进制自动探测、PATH 自动注入、防重复进队死锁与 CPU 优先级自动降级保护。

---

## 🛡️ 4. 构建安全两铁律（硬性红线）

1. **管道卫生安全（严禁截断原生管道）**：
   - **反例（严禁）**：`cargo test 2>&1 | Select-Object -First 100` 或 `cmake --build ... | Select-Object -Last 50`（# pipeline-hygiene: ok 反例演示）。
   - **原理解析**：原生 Windows 进程不响应 PowerShell 的协同终止信号，4KB 匿名管道缓冲区写满后触发内核级系统调用阻塞，整条调用链永久死锁（卡死 120 秒超时）。
   - **正确姿势**：必须先重定向落盘，再通过 PowerShell 读取：`cargo test *> out.log; Get-Content out.log -TotalCount 100`。
2. **多工作树进程精准终止**：
   - 在清理构建残留或重启服务时，**严禁使用进程名无差别盲杀**（如 `Stop-Process -Name Dunting`）；
   - 必须通过命令行路径精确匹配**当前工作树路径**后再终止，严禁误杀其他 worktree 的前台调试或测试实例。

---

## 🔧 5. 环境检测与安装指引

若系统提示找不到 `bque` 命令，请引导或查阅安装指引：
👉 详见 [references/install.md](references/install.md)。

{% endif %}
<!-- PENGJ_TEMPLATE_END -->

<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属排队策略与定制配置

> 可在此登记本项目特殊的耗时任务标签（如 Heavy 测试、特殊离线渲染等）、专属超时阈值或前置环境准备脚本。
