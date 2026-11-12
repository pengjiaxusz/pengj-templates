{% if options["skill_lang"] == "en" %}
# Master Architecture Index Guide (`docs/architecture/README.md`)

Use this guide to generate or update `docs/architecture/README.md` as an ultra-compact routing table for AI Agents.

## 1. Index Design Rules

- **Single Entry Point**: `docs/architecture/README.md` is the only index file.
- **Hard Length Cap**: Maximum **60 lines**.
- **1-Hop Routing**: Map task intents and source directories directly to target `.md` files so the Agent never reads more than 1-2 domain docs per task.
- **Status Visibility**: Tag every doc with `[LANDED]`, `[PARTIAL]`, or `[BLUEPRINT]`, plus `[RULE]` (mandatory global invariant) or `[MOD]` (module spec).

## 2. Standard Master Index Template

Generate `docs/architecture/README.md` using this exact skeleton:

```markdown
# ARCH INDEX (Agent Routing Map)

> Read only the 1-2 specs matching your target paths or task intent.
> Tags: `[RULE]` = mandatory global contract | `[MOD]` = subsystem spec | Status: `LANDED` | `PARTIAL` | `BLUEPRINT`

## TASK ROUTING (1-Hop Lookup)
- Global boundaries & dependency rules -> `./principles.md`
- End-to-end request/data pipeline -> `./dataflow.md`
- Performance budgets & benchmark gates -> `./kpi-budgets.md`
- Adding/modifying domain business logic -> `./domain-core.md`
- Storage, persistence, or caching changes -> `./storage.md`
- IPC, RPC, or network transport changes -> `./transport.md`

## MODULE MAP
- `./principles.md` [RULE][LANDED] | Paths: `src/*` | Global layering, forbidden cross-imports, error contract
- `./dataflow.md` [RULE][LANDED] | Paths: `src/pipeline/` | Ingress -> Core -> Storage -> Egress state transitions
- `./kpi-budgets.md` [RULE][LANDED] | Paths: `benches/`, `tests/` | Latency/memory redlines and CI guard commands
- `./domain-core.md` [MOD][LANDED] | Paths: `src/core/` | State machine, command dispatch, pure business rules
- `./storage.md` [MOD][PARTIAL] | Paths: `src/storage/` | WAL, memory index, snapshot isolation
- `./transport.md` [MOD][LANDED] | Paths: `src/ipc/`, `src/net/` | Framing protocol, backpressure, retry semantics

## DRAFTS (NON-AUTHORITATIVE - DO NOT TREAT AS INVARIANTS)
- `docs/design/` -> Experimental RFCs and historical notes; verify against code before relying on them.
```

## 3. Generation Steps

1. Scan `docs/architecture/` and top-level source directories (`src/`, `crates/`, `pkg/`, `internal/`, `tests/`).
2. Map each existing architecture doc to its source paths, status, and a <= 10-word telegraphic summary.
3. Write `docs/architecture/README.md` keeping total length <= 60 lines.
{% else %}
# 架构主索引指南（`docs/architecture/README.md`）

用本指南生成或更新 `docs/architecture/README.md`，作为面向 AI Agent 的超紧凑路由表。

## 一、索引设计规则

- **单一入口点**：`docs/architecture/README.md` 是唯一的索引文件。
- **硬性长度上限**：最多 **60 行**。
- **1 跳路由**：把任务意图与源码目录直接映射到目标 `.md` 文件，使 Agent 每个任务最多只读 1~2 篇领域文档。
- **状态可见**：每篇文档打上 `[LANDED]`、`[PARTIAL]` 或 `[BLUEPRINT]`，并标注 `[RULE]`（强制全局不变量）或 `[MOD]`（模块规范）。

## 二、标准主索引模板

生成 `docs/architecture/README.md` 时使用以下精确骨架（骨架中的英文标签是给 Agent 的稳定契约，保持英文，不要翻译）：

```markdown
# ARCH INDEX (Agent Routing Map)

> Read only the 1-2 specs matching your target paths or task intent.
> Tags: `[RULE]` = mandatory global contract | `[MOD]` = subsystem spec | Status: `LANDED` | `PARTIAL` | `BLUEPRINT`

## TASK ROUTING (1-Hop Lookup)
- Global boundaries & dependency rules -> `./principles.md`
- End-to-end request/data pipeline -> `./dataflow.md`
- Performance budgets & benchmark gates -> `./kpi-budgets.md`
- Adding/modifying domain business logic -> `./domain-core.md`
- Storage, persistence, or caching changes -> `./storage.md`
- IPC, RPC, or network transport changes -> `./transport.md`

## MODULE MAP
- `./principles.md` [RULE][LANDED] | Paths: `src/*` | Global layering, forbidden cross-imports, error contract
- `./dataflow.md` [RULE][LANDED] | Paths: `src/pipeline/` | Ingress -> Core -> Storage -> Egress state transitions
- `./kpi-budgets.md` [RULE][LANDED] | Paths: `benches/`, `tests/` | Latency/memory redlines and CI guard commands
- `./domain-core.md` [MOD][LANDED] | Paths: `src/core/` | State machine, command dispatch, pure business rules
- `./storage.md` [MOD][PARTIAL] | Paths: `src/storage/` | WAL, memory index, snapshot isolation
- `./transport.md` [MOD][LANDED] | Paths: `src/ipc/`, `src/net/` | Framing protocol, backpressure, retry semantics

## DRAFTS (NON-AUTHORITATIVE - DO NOT TREAT AS INVARIANTS)
- `docs/design/` -> Experimental RFCs and historical notes; verify against code before relying on them.
```

## 三、生成步骤

1. 扫描 `docs/architecture/` 与顶层源码目录（`src/`、`crates/`、`pkg/`、`internal/`、`tests/`）。
2. 把每篇已有架构文档映射到源码路径、状态，以及不超过 10 个词的电报式摘要。
3. 写入 `docs/architecture/README.md`，总量控制在 60 行以内。
{% endif %}
