{% if options["skill_lang"] == "en" %}
# Architecture Index Generation Guide (`docs/architecture/README.md`)

> **Purpose**: When `docs/architecture/README.md` does not exist in the project, follow this guide with user approval to scan existing architecture documents and module structures, generating a standard, production-grade architecture index file.

---

## 1. Generation Principles

1. **Sole Authoritative Entry Point**: All architecture discovery routes through `docs/architecture/README.md` without introducing fragmented or competing indexes.
2. **Task-Driven Reading Paths**: Group documents by developer/agent intent (e.g. overview, domain logic, performance tuning, agent integration) so agents can reach the right 1-2 docs in 1 jump.
3. **Progressive Disclosure & Grounding**:
   - Keep the index file itself concise (100-200 lines).
   - Display implementation status (`Implemented` / `Partially Implemented` / `Blueprint`) to show system maturity at a glance.
4. **Authority & Rigor**:
   - Explicitly differentiate between [Authoritative Specification] (mandatory contracts and boundaries) and [Design & Implementation] (concrete subsystem design).

---

## 2. Scanning & Extraction Steps

1. **Scan Architecture Directory**:
   - Check all `.md` files and subdirectories under `docs/architecture/` (or `docs/`).
   - Check auxiliary design notes in `docs/design/` or similar folders.
2. **Extract Metadata**:
   For each architecture document, extract:
   - **Document Path**: Relative markdown link (e.g. `[`System Architecture`](./system-architecture.md)`).
   - **Core Scope / Responsibility**: Concise 1-2 sentence summary of what problem this doc addresses.
   - **Involved Modules / Code Paths**: Code directories or crates (e.g. `src/core/`, `crates/unbound-gpu/`).
   - **Implementation Status**: `Implemented` | `Partially Implemented` | `Blueprint`.
   - **Authority Level**: [Authoritative Specification] or [Design & Implementation].
3. **Output Standard Template**:
   Generate `docs/architecture/README.md` following this structure:

```markdown
# Project Architecture & Core Specification Index

> **AI / Developer Guide**: This file is the **sole authoritative architecture index** for this project.
> Before starting new features, cross-module changes, or performance tuning, consult the relevant documents below.
> Items marked [Authoritative Specification] must be strictly obeyed; [Design & Implementation] guides concrete code.

---

## Reading Paths by Task Intent

- **First time overview** ➔ [`System Principles`](./00-overview/principles.md) ➔ [`Data Flow`](./00-overview/dataflow.md) ➔ [`Milestones & KPIs`](./07-roadmap/milestones.md)
- **Feature / Domain Logic** ➔ [`Domain Model`](./02-ui/dod-model.md) ➔ [`State Tree`](./02-ui/state.md)
- **Infrastructure / Storage / Graphics** ➔ [`Core Engine`](./01-graphics/api.md) ➔ [`Memory Hierarchy`](./01-graphics/memory.md)
- **Performance & Benchmark** ➔ [`Roadmap & Acceptance KPIs`](./07-roadmap/acceptance-kpis.md)
- **AI Agent Integration & Guardrails** ➔ [`Headless Verification`](./05-agent/headless.md) ➔ [`Dependency Layering`](./05-agent/layering.md)

---

## 1. Global Architecture & Core Invariants

| Authoritative Architecture Doc | Status | Core Content / Scope | Related Code Paths | Authority Level |
| :--- | :--- | :--- | :--- | :--- |
| [`System Architecture`](./system-architecture.md) | `Implemented` | Overall layered architecture, core process topology & lifecycle | `src/core/`, `src/bootstrap/` | [Authoritative Specification] |
| [`End-to-End Data Flow`](./dataflow.md) | `Implemented` | Complete lifecycle data flow from external input to processing | Global closed loop | [Authoritative Specification] |
| [`Acceptance KPIs & Benchmarks`](./acceptance-kpis.md) | `Implemented` | Quantified performance budget, measurement commands & criteria | `tests/bench/` | [Authoritative Specification] |

---

## 2. Core Subsystems & Domain Modules

| Domain Architecture Doc | Status | Core Content / Scope | Related Code Paths | Authority Level |
| :--- | :--- | :--- | :--- | :--- |
| [`Communication Protocol`](./communication.md) | `Implemented` | RPC protocols, serialization contracts & sync primitives | `src/ipc/`, `src/transport/` | [Design & Implementation] |
| [`State & Storage Engine`](./storage.md) | `Partially Implemented` | In-memory indexing, transaction isolation & disk persistence | `src/storage/` | [Design & Implementation] |

---

## 3. Associated Design Drafts & Explorations (Non-authoritative)

- `docs/design/xxx/` - Experimental prototype notes and exploratory drafts...
```

4. **Complete Generation**:
   Write to `docs/architecture/README.md` and summarize the generated structure to the user.
{% else %}
# 架构主索引生成指南 (`docs/architecture/README.md`)

> **用途**：当项目中不存在 `docs/architecture/README.md` 时，通过本指南在用户授权后扫描现有架构文档与模块结构，生成标准、现代化的架构主索引文件。

---

## 一、 生成原则

1. **唯一权威入口**：所有架构发现统一路由至 `docs/architecture/README.md`，不引入分散或多头入口。
2. **按任务动机导引（Task-Driven Routing Paths）**：在索引顶部根据开发意图（如总览、业务开发、底层重构、性能压测、Agent 接入）组织阅读路径，让 Agent 1 跳命中核心文档。
3. **渐进式披露与成熟度透视**：
   - 保持索引文件精炼（100~200 行以内）。
   - 在表格中直接透出【实现状态】（`已落地` / `部分落地` / `蓝图`），让系统成熟度一目了然。
4. **分级严谨**：
   - 明确区分【权威规范】（不可动摇的契约与红线）与【设计实现】（具体子系统的方案指南）。

---

## 二、 扫描与提炼步骤

1. **扫描架构目录**：
   - 检查 `docs/architecture/` 目录下的所有 `.md` 文件与子目录（若项目已有 `docs/` 分卷则一并扫描）。
   - 检查是否存在 `docs/design/`、`docs/设计/` 等辅助设计目录。
2. **提取元数据**：
   针对每个架构文档，提取以下信息：
   - **文档路径**：相对 Markdown 链接（如 `[`系统架构`](./system-architecture.md)`）。
   - **核心职责/内容**：用 1~2 句话精炼概括该架构解决的问题。
   - **涉及模块 / 代码目录**：代码库中的实际对应目录或包（如 `src/core/`, `crates/unbound-gpu/`）。
   - **实现状态**：`已落地` | `部分落地` | `蓝图`。
   - **规范级别**：【权威规范】或【设计实现】。
3. **输出标准模板**：
   按以下通用格式生成 `docs/architecture/README.md`：

```markdown
# 项目架构与核心规范索引

> **AI / 开发者查阅指南**：本文件是项目的**唯一权威架构主索引**。
> 处理新需求、跨模块改动或定位复杂问题前，请根据当前任务意图查阅对应文档。
> 标注【权威规范】的必须严格遵守，【设计实现】用于指导具体编码。

---

## 按任务动机查阅导引（Reading Paths）

- **首次了解项目 / 全局速览** ➔ [`设计准则`](./principles.md) ➔ [`端到端数据流`](./dataflow.md) ➔ [`量化验收红线`](./acceptance-kpis.md)
- **开发业务模块 / 领域逻辑** ➔ [`领域数据模型`](./domain-model.md) ➔ [`状态流转`](./state.md)
- **底层基础设施 / 通信 / 存储** ➔ [`底层引擎契约`](./engine.md) ➔ [`显存/内存分层`](./memory.md)
- **性能优化 / 预算压测** ➔ [`量化验收红线与测量口径`](./acceptance-kpis.md)
- **AI Agent 友好度与护栏接入** ➔ [`无头闭环验证`](./headless.md) ➔ [`依赖分层护栏`](./layering.md)

---

## 1. 全局系统架构与核心骨架

| 权威架构文档 | 状态 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- | :--- |
| [`系统架构定位`](./principles.md) | `已落地` | 系统整体分层架构、核心进程拓扑与全局第一性原理 | `src/core/`, `src/bootstrap/` | 【权威规范】 |
| [`端到端数据流`](./dataflow.md) | `已落地` | 从外部输入到核心处理、持久化与响应呈现的全链路闭环 | 全局闭环数据流 | 【权威规范】 |
| [`量化验收红线`](./acceptance-kpis.md) | `已落地` | 吞吐量、延迟预算、物理测试命令与严密测量口径 | `tests/bench/` | 【权威规范】 |

---

## 2. 核心子系统与领域模块

| 领域架构文档 | 状态 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- | :--- |
| [`通信协议与 RPC`](./communication.md) | `已落地` | 跨服务通信协议、序列化契约与并发同步原语 | `src/ipc/`, `src/transport/` | 【设计实现】 |
| [`状态与存储引擎`](./storage.md) | `部分落地` | 内存索引、事务隔离与持久化存储 | `src/storage/` | 【设计实现】 |

---

## 3. 关联设计草稿与探索（非权威架构，仅作参考）

- `docs/design/xxx/` - 某实验性功能设计草稿...
```

4. **完成生成**：
   写入 `docs/architecture/README.md`，并在会话中向用户汇总已收录的架构章节与阅读路径。
{% endif %}