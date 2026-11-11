{% if options["skill_lang"] == "en" %}
# Domain Architecture Doc Generation Guide (`docs/architecture/<module>.md`)

> **Purpose**: When an authoritative architecture document is missing for a core domain or subsystem, generate a standard, production-grade architecture document following this guide and engineering best practices after user consent.

---

## 1. Universal Architecture Principles & Requirements

1. **Grounded In Reality (Anchor with Code & Tests)**:
   - Architecture documents must not be detached blueprints. Every document must explicitly state its implementation status, point to source files, key types, and the **guardrail tests** protecting it.
2. **Explicit Trade-offs & Costs**:
   - Software architecture is the art of trade-offs. Explicitly document what was sacrificed and why, preventing developers or AI agents from performing destructive "refactoring" in the name of purity.
3. **Physical Guardrails Over Verbal Promises**:
   - Architectural invariants should be enforced by automated tests or compiler barriers whenever possible. The document must link to the test guarding the boundary.
4. **Clear Scope & Agent-Native Ergonomics**:
   - Unambiguous in-scope/out-of-scope boundaries.
   - Prefer single orthogonal APIs and minimal surface area to eliminate LLM hallucinations.
5. **Appropriate Length & Progressive Disclosure**:
   - Keep documents between 200-400 lines, focused on architectural contracts rather than dumping transient code snippets.

---

## 2. Standard Domain Architecture Doc Template

Generated architecture documents must follow this structure:

```markdown
# <Module / Subsystem Name> Architecture Specification

> **Implementation Status**: `Implemented` | `Partially Implemented` | `Blueprint`
>
> | | |
> | :--- | :--- |
> | **Source Paths** | `src/xxx/core.rs`, `src/xxx/mod.rs` |
> | **Key Types & Interfaces** | `EngineCore`, `DataStream`, `SessionState` |
> | **Guardrail Tests** | `tests/layering.rs`, `tests/invariants.rs` (automated CI test guard) |
> | **KPI Budget / Actual** | Processing latency $\le 1.0\text{ms}$ / Actual $0.62\text{ms}$ (if applicable) |

---

## 1. Role & System Boundaries
- **Core Role**: One-sentence summary of the core responsibility and primary problem solved.
- **Scope Delineation**:
  - **In-Scope (Handled by this module)**: ...
  - **Out-of-Scope (Delegated externally)**: ...

## 2. Architectural Trade-offs & Cost Matrix
| Cost Incurred (What was sacrificed / constrained) | Core Value Gained (Why we made this choice) | Grounding Module / Source |
| :--- | :--- | :--- |
| e.g. Forbid deep generic typestates | Drastically reduce compile time & avoid code bloat | `src/xxx/desc.rs` |
| e.g. Callback actions must be serializable enums | Enable state migration, replay & time-travel debugging | `src/xxx/action.rs` |

## 3. Core Data Flow & Lifecycle
- **Core Data Flow / Call Sequence**: Complete data flow from input trigger to final output/presentation.
- **Concurrency & Context Model**: Execution context (main thread, worker pool, async runtime), synchronization and shared state primitives.
- **Lifecycle**: Initialization, runtime update, dynamic reload/reconnection, and clean shutdown sequence.

## 4. Code Module Mapping Table
| Logical Role / Layer | Corresponding File / Directory Path | Responsibilities & Key Interfaces / Types |
| :--- | :--- | :--- |
| Entry & Facade | `src/xxx/entry.*` | Public API surface & parameter validation |
| Core Logic & State | `src/xxx/core.*` | Domain state management & core transformations |
| External Adapter / Driver | `src/xxx/adapter.*` | Low-level calls & external system interaction |

## 5. Architectural Invariants & Physical Guardrails
- **Invariant 1 (Boundary)**: Strict boundary rule (e.g. no direct cross-layer dependency, unidirectional data flow).
  - *Guardrail Test*: Enforced by `tests/layering.rs::test_dependency_direction` (fails CI if broken).
- **Invariant 2 (Performance / Concurrency)**: Memory/thread constraints (e.g. zero frame allocation in steady state, lock-free hot path).
  - *Guardrail Test*: Enforced by benchmark or leak detection test `tests/alloc_guard.rs`.
- **Common Anti-Patterns**: Pitfalls and forbidden practices that agents and developers must strictly avoid.

## 6. (Optional) KPI Budget & Measurement Rigor
| Metric / Dimension | Test Scenario | Budget Limit | Actual Measured | Measurement Command & Notes |
| :--- | :--- | :--- | :--- | :--- |
| Hot path latency | 100k items processing | $\le 1.5\text{ms}$ | **0.82 ms** | `just bench` (steady state, exclude 1st frame) |
```

---

## 3. Self-Registration Feedback Loop

After generating the new domain architecture doc, **always register the new doc entry into the `docs/architecture/README.md` index table** to ensure subsequent agents and developers can discover it immediately via the main index.
{% else %}
# 领域架构文档生成指南 (`docs/architecture/<模块>.md`)

> **用途**：当某核心领域/子系统缺失权威架构文档时，在用户同意生成后，按照本指南的标准架构模型和软件工程最佳实践生成通用的架构文档。

---

## 一、 通用架构原则与要求

1. **实事求是，代码与测试三元锚定（Grounded In Reality）**：
   - 架构文档绝不是脱离代码的纸上谈兵。每篇文档开头必须明确标注【实现状态】（已落地 / 部分落地 / 蓝图），并给出对应的源码路径、核心类型和**守卫测试用例（Guardrail Tests）**。
2. **显式代价与取舍矩阵（Explicit Trade-offs）**：
   - 软件工程没有银弹，架构即取舍。必须诚实记录为了换取核心指标所放弃的特性与接受的限制，防止开发者或 AI Agent 以“优雅”之名进行反向破坏性重构。
3. **物理测试护栏高于口头约定（Physical Guardrails）**：
   - 能用编译器或自动化测试守住的红线，不要留给文档口头规约。架构文档必须指明是哪个测试在看守该边界。
4. **系统边界清晰与 Agent 友好度（Agent-Native Ergonomics）**：
   - 清晰划分 In-Scope 与 Out-of-Scope。
   - 提倡正交极简的单一路标 API，拒绝花哨冗余的语法糖与多重写法，从根源消除大模型代码幻觉。
5. **篇幅适中与渐进式表达**：
   - 单篇文档建议控制在 200~400 行以内，聚焦系统契约与设计精髓，不堆砌大段易漂移的瞬态业务代码。

---

## 二、 领域架构文档标准模板

生成的架构文档应包含以下通用章节：

```markdown
# <模块/子系统名称> 架构规范

> **实现状态**：`已落地` | `部分落地` | `蓝图`
>
> | | |
> | :--- | :--- |
> | **源码路径** | `src/xxx/core.rs`, `src/xxx/mod.rs` |
> | **关键类型/接口** | `EngineCore`, `DataStream`, `SessionState` |
> | **守卫测试** | `tests/layering.rs`, `tests/invariants.rs`（物理测试用例，CI 失败拦截） |
> | **验收实测** | 处理耗时 $\le 1.0\text{ms}$ / 实测 $0.62\text{ms}$（若适用） |

---

## 1. 定位与系统边界
- **核心定位**：一句话阐明本模块的核心职责与解决的核心问题。
- **边界划分**：
  - **本模块负责（In-Scope）**：...
  - **本模块不负责（Out-of-Scope / 委托外部）**：...

## 2. 核心取舍与显式代价（Trade-off Matrix）
| 承担的代价（放弃了什么 / 接受的约束） | 换取的核心价值（为何这么做） | 对应落地模块 / 源码 |
| :--- | :--- | :--- |
| 例：禁止深层泛型嵌套状态树 | 杜绝单态化代码膨胀，增量编译缩短至 300ms 内 | `src/xxx/desc.rs` |
| 例：组件回调只能是可序列化枚举 | 获得状态可迁移、可回放与时间旅行调试能力 | `src/xxx/action.rs` |

## 3. 核心数据流与生命周期
- **核心数据流 / 调用时序**：从外部输入触发到内部处理、状态流转与最终产出的全链路时序。
- **并发与上下文模型**：运行在何种线程/协程/异步运行时，状态共享与无锁/互斥同步机制。
- **生命周期**：初始化、稳态运行、热重载/断线重连、销毁释放时序。

## 4. 代码模块映射表
| 逻辑角色 / 层次 | 对应文件 / 目录路径 | 职责与关键接口/类型 |
| :--- | :--- | :--- |
| 入口与门面 | `src/xxx/entry.*` | 对外暴露接口与参数校验 |
| 核心逻辑与状态 | `src/xxx/core.*` | 业务状态管理与核心变换 |
| 外部适配 / 驱动 | `src/xxx/adapter.*` | 底层调用与外部系统交互 |

## 5. 架构红线与物理守卫护栏（Invariants & Guardrail Tests）
- **架构红线 1（分层边界）**：必须遵守的不可变约束（如：禁止跨层直接调用、状态单向只读流动）。
  - *物理守卫测试*：由 `tests/layering.rs::test_dependency_direction` 在 CI 中硬性检验，破坏则单测失败。
- **架构红线 2（资源/性能）**：内存、并发或延迟等非功能性硬约束（如：热路径零每帧内存分配）。
  - *物理守卫测试*：由性能基准或分配测试 `tests/alloc_guard.rs` 守卫。
- **常见反模式（Anti-Patterns）**：开发中绝对要规避的做法。

## 6. （可选）量化指标与验收红线（KPI Budget）
| 考核维度 | 核心测试场景 | 量化红线（预算） | 实测值 | 测量命令与口径说明 |
| :--- | :--- | :--- | :--- | :--- |
| 热路径耗时 | 10 万节点并发处理 | $\le 1.5\text{ms}$ | **0.82 ms** | `just bench`（排除第一帧冷启动） |
```

---

## 三、 自举回写闭环

生成完新的领域架构文档后，**必须顺手将新文档条目更新登记到 `docs/architecture/README.md` 的索引表格中**，确保后续其他 Agent 和开发者能立即通过主索引发现该文档。
{% endif %}