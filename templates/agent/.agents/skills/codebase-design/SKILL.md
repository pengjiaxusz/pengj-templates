---
name: codebase-design
description: >-
{%- if options["skill_lang"] == "en" %}
  Shared vocabulary for designing deep modules. Use when the user wants to design or improve a module's interface, find deepening opportunities, decide where a seam goes, make code more testable or AI-navigable, or when another skill needs the deep-module vocabulary.
{%- else %}
  用于设计深度模块的共享词汇表。当用户想要设计或改进模块接口、寻找深化机会、决定接缝位置、提升代码可测试性或 AI 可导航性，或当其他技能需要深度模块词汇时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
# Codebase Design

Design **deep modules**: a lot of behaviour behind a small interface, placed at a clean seam, testable through that interface. Use this language and these principles wherever code is being designed or restructured. The aim is leverage for callers, locality for maintainers, and testability for everyone.

## Glossary

Use these terms exactly: don't substitute "component," "service," "API," or "boundary." Consistent language is the whole point.

**Module**: anything with an interface and an implementation. Deliberately scale-agnostic: a function, class, package, or tier-spanning slice. _Avoid_: unit, component, service.

**Interface**: everything a caller must know to use the module correctly: the type signature, but also invariants, ordering constraints, error modes, required configuration, and performance characteristics. _Avoid_: API, signature (too narrow, they refer only to the type-level surface).

**Implementation**: what's inside a module, its body of code. Distinct from **Adapter**: a thing can be a small adapter with a large implementation (a Postgres repo) or a large adapter with a small implementation (an in-memory fake). Reach for "adapter" when the seam is the topic; "implementation" otherwise.

**Depth**: leverage at the interface. The amount of behaviour a caller (or test) can exercise per unit of interface they have to learn. A module is **deep** when a large amount of behaviour sits behind a small interface, **shallow** when the interface is nearly as complex as the implementation.

**Seam** _(Michael Feathers)_: a place where you can alter behaviour without editing in that place; the *location* at which a module's interface lives. Where to put the seam is its own design decision, distinct from what goes behind it. _Avoid_: boundary (overloaded with DDD's bounded context).

**Adapter**: a concrete thing that satisfies an interface at a seam. Describes *role* (what slot it fills), not substance (what's inside).

**Leverage**: what callers get from depth. More capability per unit of interface they learn. One implementation pays back across N call sites and M tests.

**Locality**: what maintainers get from depth. Change, bugs, knowledge, and verification concentrate in one place rather than spreading across callers. Fix once, fixed everywhere.

## Deep vs shallow

**Deep module** = small interface + lots of implementation:

```
┌─────────────────────┐
│   Small Interface   │  ← Few methods, simple params
├─────────────────────┤
│                     │
│  Deep Implementation│  ← Complex logic hidden
│                     │
└─────────────────────┘
```

**Shallow module** = large interface + little implementation (avoid):

```
┌─────────────────────────────────┐
│       Large Interface           │  ← Many methods, complex params
├─────────────────────────────────┤
│  Thin Implementation            │  ← Just passes through
└─────────────────────────────────┘
```

When designing an interface, ask:

- Can I reduce the number of methods?
- Can I simplify the parameters?
- Can I hide more complexity inside?

## Principles

- **Depth is a property of the interface, not the implementation.** A deep module can be internally composed of small, mockable, swappable parts; they just aren't part of the interface. A module can have **internal seams** (private to its implementation, used by its own tests) as well as the **external seam** at its interface.
- **The deletion test.** Imagine deleting the module. If complexity vanishes, it was a pass-through. If complexity reappears across N callers, it was earning its keep.
- **The interface is the test surface.** Callers and tests cross the same seam. If you want to test *past* the interface, the module is probably the wrong shape.
- **One adapter means a hypothetical seam. Two adapters means a real one.** Don't introduce a seam unless something actually varies across it.

## Designing for testability

Good interfaces make testing natural:

1. **Accept dependencies, don't create them.**

   ```typescript
   // Testable
   function processOrder(order, paymentGateway) {}

   // Hard to test
   function processOrder(order) {
     const gateway = new StripeGateway();
   }
   ```

2. **Return results, don't produce side effects.**

   ```typescript
   // Testable
   function calculateDiscount(cart): Discount {}

   // Hard to test
   function applyDiscount(cart): void {
     cart.total -= discount;
   }
   ```

3. **Small surface area.** Fewer methods = fewer tests needed. Fewer params = simpler test setup.

## Relationships

- A **Module** has exactly one **Interface** (the surface it presents to callers and tests).
- **Depth** is a property of a **Module**, measured against its **Interface**.
- A **Seam** is where a **Module**'s **Interface** lives.
- An **Adapter** sits at a **Seam** and satisfies the **Interface**.
- **Depth** produces **Leverage** for callers and **Locality** for maintainers.

## Rejected framings

- **Depth as ratio of implementation-lines to interface-lines** (Ousterhout): rewards padding the implementation. We use depth-as-leverage instead.
- **"Interface" as the TypeScript `interface` keyword or a class's public methods**: too narrow: interface here includes every fact a caller must know.
- **"Boundary"**: overloaded with DDD's bounded context. Say **seam** or **interface**.

## Going deeper

- **Deepening a cluster given its dependencies**, see [DEEPENING.md](DEEPENING.md): dependency categories, seam discipline, and replace-don't-layer testing.
- **Exploring alternative interfaces**, see [DESIGN-IT-TWICE.md](DESIGN-IT-TWICE.md): spin up parallel sub-agents to design the interface several radically different ways, then compare on depth, locality, and seam placement.
{%- else -%}
# 代码库设计（Codebase Design）

设计**深度模块（deep modules）**：在干净的接缝（seam）处，用一个精简的接口（interface）封装大量行为，并通过该接口直接测试。在任何设计或重构代码的地方，统一使用这套语言与原则。目标是给调用方带来杠杆，给维护者带来局部性，给所有人带来可测试性。

## 词汇表

精确使用以下术语：不要随意替换成 "component"、"service"、"API" 或 "boundary"。语言一致正是这套体系的意义所在。

**Module（模块）**：任何同时拥有接口和实现的东西。刻意保持尺度无关：它可以是一个函数、类、包，或跨层切片。_避免使用_：unit、component、service。

**Interface（接口）**：调用方为了正确使用该模块所必须知道的一切——不仅包括类型签名，还包括不变量、顺序约束、错误模式、必需配置以及性能特征。_避免使用_：API、signature（太窄，只指代类型层面的表面）。

**Implementation（实现）**：模块内部的代码主体。注意区分于 **Adapter（适配器）**：一个东西可以是小适配器配大实现（如 Postgres 仓储），也可以是大适配器配小实现（如内存假实现）。讨论接缝时用 "adapter"，其余情况用 "implementation"。

**Depth（深度）**：接口处的杠杆率。调用方（或测试）每学习一单位接口，能驱动多少行为。当精简接口背后藏着大量行为时，模块是**深的（deep）**；当接口几乎和实现一样复杂时，模块是**浅的（shallow）**。

**Seam（接缝）** _（Michael Feathers）_：无需修改原地代码即可改变行为的位置；即模块接口所在的*物理/逻辑位置*。把接缝放在哪里本身是一个独立的设计决策，区别于接缝背后放什么。_避免使用_：boundary（容易与 DDD 的限界上下文混淆）。

**Adapter（适配器）**：在接缝处满足接口契约的具体实现体。它描述的是*角色*（填补哪个槽位），而不是材质（里面写了什么）。

**Leverage（杠杆）**：调用方从深度中获得的收益——每学一单位接口换取更多能力。一份实现在 N 个调用点与 M 个测试中反复回本。

**Locality（局部性）**：维护者从深度中获得的收益——变更、bug、知识与验证都集中在同一处，而不是散落到各个调用方。修一次，处处修好。

## 深模块 vs 浅模块

**深度模块** = 小接口 + 厚实现：

```
┌─────────────────────┐
│   Small Interface   │  ← 方法少、参数简单
├─────────────────────┤
│                     │
│  Deep Implementation│  ← 复杂逻辑隐藏在内部
│                     │
└─────────────────────┘
```

**浅度模块** = 大接口 + 薄实现（应避免）：

```
┌─────────────────────────────────┐
│       Large Interface           │  ← 方法多、参数复杂
├─────────────────────────────────┤
│  Thin Implementation            │  ← 只是透传
└─────────────────────────────────┘
```

设计接口时追问自己：

- 能不能减少方法数量？
- 能不能简化参数？
- 能不能把更多复杂度藏到内部？

## 原则

- **深度是接口的属性，而不是实现的属性。** 深度模块内部完全可以由小巧、可 mock、可替换的零件组成，只是不要把它们暴露到接口上。模块既可以有接口处的**外部接缝**，也可以有**内部接缝**（对实现私有、供自身测试使用）。
- **删除测试（The deletion test）。** 想象把该模块删掉。如果复杂度凭空消失，说明它只是个透传层；如果复杂度在 N 个调用方处重新冒出来，说明它物有所值。
- **接口即测试面。** 调用方与测试跨越同一道接缝。如果你总想穿透接口去测里面的东西，说明模块形状大概画错了。
- **一个适配器是假想接缝，两个适配器才是真接缝。** 除非真的有东西会在接缝两端发生变化，否则不要硬造接缝。

## 面向可测试性设计

好接口让测试水到渠成：

1. **接收依赖，不要在内部创建依赖。**

   ```typescript
   // 可测
   function processOrder(order, paymentGateway) {}

   // 难测
   function processOrder(order) {
     const gateway = new StripeGateway();
   }
   ```

2. **返回结果，不要产生副作用。**

   ```typescript
   // 可测
   function calculateDiscount(cart): Discount {}

   // 难测
   function applyDiscount(cart): void {
     cart.total -= discount;
   }
   ```

3. **缩小表面积。** 方法越少，需要的测试越少；参数越少，测试夹具越简单。

## 概念关系

- 一个 **Module** 恰好拥有一个 **Interface**（面向调用方与测试的表面）。
- **Depth** 是 **Module** 的属性，依其 **Interface** 来衡量。
- **Seam** 是 **Module** 的 **Interface** 所在的位置。
- **Adapter** 位于 **Seam** 处并满足 **Interface**。
- **Depth** 为调用方带来 **Leverage**，为维护者带来 **Locality**。

## 舍弃的表述

- **把深度定义为“实现行数 / 接口行数”之比**（Ousterhout）：这会变相奖励把实现写啰嗦。我们改用“作为杠杆的深度”。
- **把 "Interface" 狭义理解为 TypeScript 的 `interface` 关键字或类的公开方法**：太窄，这里的接口包含调用方必须知道的一切事实。
- **"Boundary"**：在 DDD 中已被限界上下文占满。请说 **seam** 或 **interface**。

## 深入阅读

- **按依赖类型深化一组模块**：见 [DEEPENING.md](DEEPENING.md)（依赖分类、接缝纪律与“替换而非叠床”的测试策略）。
- **探索替代接口方案**：见 [DESIGN-IT-TWICE.md](DESIGN-IT-TWICE.md)（并行拉起多个 sub-agent 设计截然不同的接口，再按深度、局部性与接缝位置对比）。
{%- endif %}
