---
name: bque
description: >-

  通用跨工作树全局编译排队、后台 CPU 降级防卡顿与纯净基准测试保护技能。规范 Agent 何时必须走排队调度、何时直跑，提供纯净跑分独占执行（秒级挂起后台构建）、构建管道安全铁律（防 Windows 4KB 管道永久死锁）、全局状态自省与死锁自愈。当需要执行重型构建、编译、检查、单测、性能跑分（bench）、性能画像分析，或遇到 CPU 占满卡顿、命令挂死、编译锁争用时使用。
  Triggers: bque, build-queue, 排队构建, 全局排队, 编译排队, 跑分保护, 纯净跑分, 冻结后台构建, 挂起构建, cpu降级, idle优先级, queue-status, queue-clean, 编译锁, 管道死锁, 管道卫生.

---

<!-- PENGJ_TEMPLATE_START -->

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

## 🚀 2. 三阶梯执行规范（按优先级选用）

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

### 阶梯 3：原生 CLI 直调（独立运行或系统运维）
在没有 justfile 或独立运维排查时直接调用 `bque`：
```powershell
bque run -d "Heavy Task" -p idle -- cargo build --workspace
bque bench -d "Performance Probe" -- cargo bench
bque status
bque clean
```

---

## 🛡️ 3. 构建安全两铁律（硬性红线）

1. **管道卫生安全（严禁截断原生管道）**：
   - **反例（严禁）**：`cargo test 2>&1 | Select-Object -First 100` 或 `cmake --build ... | Select-Object -Last 50`。
   - **原理解析**：原生 Windows 进程不响应 PowerShell 的协同终止信号，4KB 匿名管道缓冲区写满后触发内核级系统调用阻塞，整条调用链永久死锁（卡死 120 秒超时）。
   - **正确姿势**：必须先重定向落盘，再通过 PowerShell 读取：`cargo test *> out.log; Get-Content out.log -TotalCount 100`。
2. **多工作树进程精准终止**：
   - 在清理构建残留或重启服务时，**严禁使用进程名无差别盲杀**（如 `Stop-Process -Name Dunting`）；
   - 必须通过命令行路径精确匹配**当前工作树路径**后再终止，严禁误杀其他 worktree 的前台调试或测试实例。

---

## 🔧 4. 环境检测与安装指引

若系统提示找不到 `bque` 命令，请引导或查阅安装指引：
👉 详见 [references/install.md](references/install.md)。


<!-- PENGJ_TEMPLATE_END -->

<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属排队策略与定制配置

> 可在此登记本项目特殊的耗时任务标签（如 Heavy 测试、特殊离线渲染等）、专属超时阈值或前置环境准备脚本。