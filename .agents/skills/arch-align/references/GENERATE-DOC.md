
# 领域架构规范指南（`docs/architecture/<module>.md`）

用本指南编写或压缩专门供 AI Agent 消费的领域架构文件。

## 一、Agent 原生写作规则

- **硬性长度上限**：每文件最多 **80 行**（目标 40~60 行）。
- **零散文**：不写完整句子、导语摘要或设计历史。用片段与符号（`->`、`!=`、`<=`、`::`）。
- **符号锚定**：每个论断必须指向真实路径或符号（`src/foo/bar.rs::Engine`）。
- **只记增量**：省略标准框架模式与显而易见的目录清单，只记录读单个文件看不出来的东西。
- **禁止 Markdown 表格**：表格在竖线、补齐与冗长表头上浪费 token。改用紧凑的标签列表。

## 二、标准 Agent 原生模板

生成 `docs/architecture/<module>.md` 时使用以下精确骨架（骨架中的英文标签是给 Agent 的稳定契约，保持英文，不要翻译）：

```markdown
# ARCH: <module-name>
STATUS: LANDED | PARTIAL | BLUEPRINT
PATHS: `src/xxx/`, `crates/yyy/`
ENTRY: `src/xxx/mod.rs::PublicFacade`, `src/xxx/core.rs::CoreState`
GUARDS: `cargo test --test layering` | `pytest tests/arch/test_bounds.py`

## SCOPE
- [IN] <核心职责 1 —— 电报式短语>
- [IN] <核心职责 2>
- [OUT] <委托出去的关注点 1> -> delegated to `src/other/mod.rs`
- [OUT] <禁止承担的关注点 2> -> handled by external caller

## TOPOLOGY & FLOW
- Main: `InputReq` -> `Validator::check` [sync] -> `CoreState::apply` [mut, single-thread] -> `WalWriter::append` [async, bounded-chan(1024)]
- Lifecycle: `init(Config)` -> `run_loop()` -> `shutdown(drain_timeout=5s)`
- Concurrency: `CoreState` owned by 1 worker thread; external callers send `CmdEnum` via `mpsc`; zero locks on hot path.

## TRADEOFFS (DO NOT REFACTOR)
- [COST] `StateNode` 禁止嵌套泛型 -> [WHY] 避免单态化膨胀，增量编译 < 300ms -> [AT] `src/xxx/desc.rs`
- [COST] 回调只能使用可序列化的 `ActionId` 枚举 -> [WHY] 确定性回放与快照恢复的前提 -> [AT] `src/xxx/action.rs`

## INVARIANTS & RED LINES
- [NEVER] `src/xxx/core.rs` 引入 `src/xxx/adapter/*` -> [WHY] 保持 core 纯净、无 I/O -> [GUARD] `tests/layering.rs::core_no_io`
- [NEVER] `process_frame()` 内堆分配 -> [WHY] 帧预算 <= 1.0ms（实测 0.62ms） -> [GUARD] `tests/alloc_guard.rs`
- [MUST] 所有状态变更在返回前发出 `ChangeEvent` -> [GUARD] `tests/invariants.rs::audit_trail`

## EXTENSION RECIPE (OPTIONAL - MAX 5 LINES)
- 新增命令：1) 在 `src/xxx/action.rs::Cmd` 加枚举分支 -> 2) 在 `CoreState::dispatch` 中处理 -> 3) 在 `tests/replay.rs` 补回放用例。
```

## 三、保存前压缩检查

- [ ] 文件是否 ≤ 80 行？
- [ ] 所有 Markdown 表格是否已移除，并替换为 `[IN]`、`[COST]`、`[NEVER]` 标签列表？
- [ ] 每条 `[COST]` 是否说明了 `[WHY]`，让 Agent 重构时不会把它“修好”？
- [ ] 新文件是否已登记到 `docs/architecture/README.md`？
