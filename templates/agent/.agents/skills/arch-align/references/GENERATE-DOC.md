{% if options["skill_lang"] == "en" %}
# Domain Architecture Doc Generation Guide (`docs/architecture/<module>.md`)

> **Purpose**: When an authoritative architecture document is missing for a core domain or subsystem, generate a standard, universal architecture document following this guide and software engineering best practices after user consent.

---

## 1. Universal Architecture Principles & Requirements

1. **Clear Responsibilities & Boundaries**:
   - Explicitly define the role and position of this module within the overall system.
   - Clearly delineate input and output contracts; avoid handling responsibilities outside this module's scope.
2. **Standardization & Generality**:
   - Follow a standardized architecture layout without binding to ad-hoc project conventions.
   - Use precise technical terminology and domain concepts.
3. **Appropriate Length & Progressive Structure**:
   - Keep individual documents within 200-400 lines.
   - Must contain the 4 core sections: System Boundaries, Core Data Flow, Source Mapping, and Invariants & Anti-Patterns.

---

## 2. Standard Domain Architecture Doc Template

The generated architecture document should contain the following standard sections:

```markdown
# <Module / Subsystem Name> Architecture Specification

## 1. Role & System Boundaries
- **Core Role**: One-sentence summary of the core responsibility and primary problem solved.
- **Scope Delineation**:
  - **In-Scope (Handled by this module)**: ...
  - **Out-of-Scope (Delegated externally)**: ...

## 2. Core Data Flow & Lifecycle
- **Core Data Flow / Call Sequence**: Complete data flow from request trigger to final processing.
- **Concurrency & Context Model**: Thread/coroutine/runtime execution context, state sharing, and synchronization mechanisms.
- **Lifecycle**: Initialization, hot-reload / runtime maintenance, and shutdown/cleanup sequence.

## 3. Code Module Mapping Table
| Logical Role / Layer | Corresponding File / Directory Path | Responsibilities & Key Interfaces / Types |
| :--- | :--- | :--- |
| Entry & Facade | `src/xxx/entry.*` | Public API surface & parameter validation |
| Core Logic & State | `src/xxx/core.*` | Domain state management & core transformations |
| External Adapter / Driver | `src/xxx/adapter.*` | Low-level calls & external system interaction |

## 4. Invariants & Forbidden Anti-Patterns
- **Invariant 1**: Non-negotiable constraint (e.g. no cross-layer direct access, unidirectional read-only state).
- **Invariant 2**: Non-functional constraint (memory limits, latency budgets, performance requirements).
- **Common Anti-Patterns**: Pitfalls and forbidden practices to strictly avoid during development.
```

---

## 3. Self-Registration Feedback Loop

After generating the new domain architecture doc, **always register the new doc entry into the `docs/architecture/README.md` index table** to ensure subsequent agents and developers can discover it immediately via the main index.
{% else %}
# 领域架构文档生成指南 (`docs/architecture/<模块>.md`)

> **用途**：当某核心领域/子系统缺失权威架构文档时，在用户同意生成后，按照本指南的标准架构模型和软件工程最佳实践生成通用的架构文档。

---

## 一、 通用架构原则与要求

1. **职责与边界清晰**：
   - 明确说明本模块在整个系统中的角色定位。
   - 清晰划分系统的输入与输出契约，不越界处理非本模块职责。
2. **规范性与通用性**：
   - 遵循统一的软件架构结构，不绑定特定项目的个性化约定。
   - 使用规范的技术专有名词与领域概念，表述精准。
3. **篇幅适中与渐进式表达**：
   - 单篇文档建议控制在 200~400 行以内。
   - 必须包含系统边界、核心数据流、源码映射与架构红线 4 大核心板块。

---

## 二、 领域架构文档标准模板

生成的架构文档应包含以下通用章节：

```markdown
# <模块/子系统名称> 架构规范

## 1. 定位与系统边界
- **核心定位**：一句话阐明本模块的核心职责与解决的核心问题。
- **边界划分**：
  - **本模块负责（In-Scope）**：...
  - **本模块不负责（Out-of-Scope / 委托外部）**：...

## 2. 核心数据流与生命周期
- **核心数据流 / 调用时序**：从请求触发到最终处理的完整数据流向。
- **并发与上下文模型**：运行在何种线程/协程/运行时上下文，状态共享与同步机制。
- **生命周期**：初始化、热更新/运行时维护、销毁释放时序。

## 3. 代码模块映射表
| 逻辑角色 / 层次 | 对应文件 / 目录路径 | 职责与关键接口/类型 |
| :--- | :--- | :--- |
| 入口与门面 | `src/xxx/entry.*` | 对外暴露接口与参数校验 |
| 核心逻辑与状态 | `src/xxx/core.*` | 业务状态管理与核心变换 |
| 外部适配 / 驱动 | `src/xxx/adapter.*` | 底层调用与外部系统交互 |

## 4. 架构红线与禁止事项（Invariants & Anti-Patterns）
- **架构红线 1**：必须遵守的不可变约束（如：禁止跨层直接调用、状态只读单向流等）。
- **架构红线 2**：内存/性能/低延迟等非功能性硬约束。
- **常见反模式（Anti-Patterns）**：开发中绝对要规避的做法。
```

---

## 三、 自举回写闭环

生成完新的领域架构文档后，**必须顺手将新文档条目更新登记到 `docs/architecture/README.md` 的索引表格中**，确保后续其他 Agent 和开发者能立即通过主索引发现该文档。
{% endif %}
