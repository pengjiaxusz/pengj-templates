---
name: tdd
description: >-
{%- if options["skill_lang"] == "en" %}
  Test-driven development. Use when the user wants to build features or fix bugs test-first, mentions "red-green-refactor", or wants integration tests.
{%- else %}
  测试驱动开发（TDD）。当用户希望测试先行地开发特性或修复 bug、提到 "red-green-refactor" 或需要集成测试时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
# Test-Driven Development

TDD is the red → green loop. This skill is the reference that makes that loop produce tests worth keeping: what a good test is, where tests go, the anti-patterns, and the rules of the loop. Every section applies on every cycle: consult them before and during the loop, not after.

When exploring the codebase, read `GLOSSARY.md` (if it exists) so test names and interface vocabulary match the project's domain language, and respect ADRs in the area you're touching.

## What a good test is

Tests verify behavior through public interfaces, not implementation details. Code can change entirely; tests shouldn't. A good test reads like a specification: "user can checkout with valid cart" tells you exactly what capability exists, and it survives refactors because it doesn't care about internal structure.

See [tests.md](tests.md) for examples and [mocking.md](mocking.md) for mocking guidelines.

## Seams: where tests go

A **seam** is the public boundary you test at: the interface where you observe behavior without reaching inside. Tests live at seams, never against internals.

**Test only at pre-agreed seams.** Before writing any test, write down the seams under test and confirm them with the user. No test is written at an unconfirmed seam. You can't test everything, so agreeing the seams up front is how testing effort lands on the critical paths and complex logic instead of every edge case.

Ask: "What's the public interface, and which seams should we test?" Give each proposed seam a one-line note on what it catches and what it misses.

When the shape of that interface is itself in question (how deep the module is, where the seam belongs, what the interface should expose), call the Skill tool with "codebase-design" for the vocabulary. It is the shared source of the module, interface, depth, seam, adapter, leverage and locality terms, and it is a reference to consult, not a session to run.

## Anti-patterns

- **Implementation-coupled**: mocks internal collaborators, tests private methods, or verifies through a side channel (querying the database instead of using the interface). The tell: the test breaks when you refactor but behavior hasn't changed.
- **Tautological**: the assertion recomputes the expected value the way the code does (`expect(add(a, b)).toBe(a + b)`, a snapshot derived by hand the same way, a constant asserted equal to itself), so it passes by construction and can never disagree with the code. Expected values must come from an independent source of truth: a known-good literal, a worked example, the spec.
- **Horizontal slicing**: writing all tests first, then all implementation. Bulk tests verify _imagined_ behavior: you test the _shape_ of things rather than user-facing behavior, the tests go insensitive to real changes, and you commit to test structure before understanding the implementation. Work in **vertical slices** instead: one test → one implementation → repeat, each test a **tracer bullet** that responds to what the last cycle taught you.

## Rules of the loop

- **Red before green.** Write the failing test first, then only enough code to pass it. Don't anticipate future tests or add speculative features.
- **One slice at a time.** One seam, one test, one minimal implementation per cycle.
- **Refactoring is not part of the loop.** It belongs to the review stage (see the `code-review` skill), not the red → green implementation cycle.
{%- else -%}
# 测试驱动开发（Test-Driven Development）

TDD 是红 → 绿的循环。本技能是让这套循环产出值得保留的测试的权威参考：什么是好测试、测试写在哪里、反模式以及循环纪律。每个循环都适用全部章节：在循环开始前与循环过程中查阅，不要事后才看。

探索代码库时，若存在 `GLOSSARY.md` 请先阅读，使测试命名和接口词汇与项目领域语言一致，并尊重所触及区域的 ADR。

## 什么是好测试

测试通过公开接口验证行为，而不是验证实现细节。代码可以彻底重写，测试不应该跟着碎掉。好测试读起来像一份规格说明：“持有效购物车的用户可以结账”清楚说明了系统具备什么能力，并且因为不关心内部结构而在重构后依然稳健。

示例见 [tests.md](tests.md)，mock 指南见 [mocking.md](mocking.md)。

## 接缝（Seams）：测试写在哪里

**接缝**是你开展测试的公开边界——在那个接口处观察行为，而不伸手进内部。测试写在接缝处，绝不针对内部实现。

**只在预先对齐好的接缝处写测试。** 动手写任何测试之前，先列出待测接缝并与用户确认。不在未经确认的接缝处写测试。你不可能测遍一切，事先对齐接缝才能把测试精力集中在核心路径与复杂逻辑上，而不是穷举细枝末节。

提问：“公开接口是什么？我们应该在哪些接缝处写测试？”为每个提议的接缝附上一行说明：它能兜住什么、会漏掉什么。

当接口本身的形状尚存疑问时（模块该多深、接缝放哪里、接口暴露什么），调用 Skill 工具执行 "codebase-design" 获取词汇表。它是 `module`、`interface`、`depth`、`seam`、`adapter`、`leverage` 与 `locality` 术语的共享来源，作为参考查阅即可，无需另开一场设计会话。

## 反模式

- **与实现耦合**：mock 内部协作者、测试私有方法，或通过旁路渠道验证（直接查数据库而不是走接口）。典型征兆：行为完全没变，一重构测试就挂。
- **同义反复（Tautological）**：断言用和业务代码一样的算法重算期望值（`expect(add(a, b)).toBe(a + b)`、用同套逻辑手推快照、拿常量跟自己比），导致测试在构造上天然通过、永远无法与实现唱反调。期望值必须来自独立事实来源：已知正确的字面量、手算样例或 spec。
- **水平切片（Horizontal slicing）**：先把所有测试全写完，再写所有实现。批量先写的测试验证的是*想象中*的行为：你测的是事物的*形状*而不是面向用户的行为，测试对真实变更变得麻木，而且你在真正理解实现之前就把测试结构锁死了。改用**垂直切片（vertical slices）**：一个测试 → 一份实现 → 重复，每个测试都是一颗**曳光弹（tracer bullet）**，根据上一轮学到的反馈调整方向。

## 循环规则

- **先红后绿。** 先写出失败的测试，再只写刚好让它通过的代码。不要提前预判未来测试，也不要加 spec 没要求的功能。
- **一次一个切片。** 每个循环只针对一个接缝、写一个测试、做一份最小实现。
- **重构不属于本循环。** 重构属于评审阶段（见 `code-review` 技能），不要混在红 → 绿实现循环里。
{%- endif %}
