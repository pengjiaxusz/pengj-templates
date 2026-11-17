---
name: diagnosing-bugs
description: >-
{%- if options["skill_lang"] == "en" %}
  Diagnosis loop for hard bugs and performance regressions. Use when the user says "diagnose"/"debug this", or reports something broken/throwing/failing/slow.
{%- else %}
  针对疑难 bug 与性能回归的诊断闭环流程。当用户说“诊断/调试这个”，或报告某处损坏、抛错、失败或变慢时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
# Diagnosing Bugs

A discipline for hard bugs. Skip phases only when explicitly justified.

When exploring the codebase, read `GLOSSARY.md` (if it exists) to get a clear mental model of the relevant modules, and check ADRs in the area you're touching.

## Redact

This skill has you show commands, outputs and captured artifacts. **Redact every secret first**: write `<REDACTED>` in its place. Build loops against env vars, so the credential stays in the environment rather than in what you show. Captured artifacts carry auth headers: quote only the lines that carry the signal.

If the redacted output is not enough to diagnose the bug, say so and ask the user.

## Phase 1: Build a feedback loop

**This is the skill.** Everything else is mechanical. If you have a **tight** pass/fail signal for the bug (one that goes red on _this_ bug), you will find the cause; bisection, hypothesis-testing, and instrumentation all just consume it. If you don't have one, no amount of staring at code will save you.

Spend disproportionate effort here. **Be aggressive. Be creative. Refuse to give up.**

### Ways to construct one, in roughly this order

1. **Failing test** at whatever seam reaches the bug: unit, integration, e2e.
2. **Curl / HTTP script** against a running dev server.
3. **CLI invocation** with a fixture input, diffing stdout against a known-good snapshot.
4. **Headless browser script** (Playwright / Puppeteer) that drives the UI and asserts on DOM/console/network.
5. **Replay a captured trace.** Save a real network request / payload / event log to disk; replay it through the code path in isolation.
6. **Throwaway harness.** Spin up a minimal subset of the system (one service, mocked deps) that exercises the bug code path with a single function call.
7. **Property / fuzz loop.** If the bug is "sometimes wrong output", run 1000 random inputs and look for the failure mode.
8. **Bisection harness.** If the bug appeared between two known states (commit, dataset, version), automate "boot at state X, check, repeat" so you can `git bisect run` it.
9. **Differential loop.** Run the same input through old-version vs new-version (or two configs) and diff outputs.
10. **HITL bash script.** Last resort. If a human must click, drive _them_ with `scripts/hitl-loop.template.sh` so the loop is still structured. Captured output feeds back to you.

Build the right feedback loop, and the bug is 90% fixed.

### Tighten the loop

Treat the loop as a product. Once you have _a_ loop, **tighten** it:

- Can I make it faster? (Cache setup, skip unrelated init, narrow the test scope.)
- Can I make the signal sharper? (Assert on the specific symptom, not "didn't crash".)
- Can I make it more deterministic? (Pin time, seed RNG, isolate filesystem, freeze network.)

A 30-second flaky loop is barely better than no loop; a 2-second deterministic one is tight, a debugging superpower.

### Non-deterministic bugs

The goal is not a clean repro but a **higher reproduction rate**. Loop the trigger 100×, parallelise, add stress, narrow timing windows, inject sleeps. A 50%-flake bug is debuggable; 1% is not, so keep raising the rate until it's debuggable.

### When you genuinely cannot build a loop

Stop and say so explicitly. List what you tried. Ask the user for: (a) access to whatever environment reproduces it, (b) a redacted captured artifact (HAR file, log dump, core dump, screen recording with timestamps), or (c) permission to add temporary production instrumentation. Do **not** proceed to hypothesise without a loop.

### Completion criterion: a tight loop that goes red

Phase 1 is done when the loop is **tight** and **red-capable**: you can name **one command** (a script path, a test invocation, a curl) that you have **already run at least once** (show the invocation and its output, redacted), and that is:

- [ ] **Red-capable**: it drives the actual bug code path and asserts the **user's exact symptom**, so it can go red on this bug and green once fixed. Not "runs without erroring"; it must be able to _catch this specific bug_.
- [ ] **Deterministic**: same verdict every run (flaky bugs: a pinned, high reproduction rate, per above).
- [ ] **Fast**: seconds, not minutes.
- [ ] **Agent-runnable**: you can run it unattended; a human in the loop only via `scripts/hitl-loop.template.sh`.

If you catch yourself reading code to build a theory before this command exists, **stop: jumping straight to a hypothesis is the exact failure this skill prevents.** No red-capable command, no Phase 2.

## Phase 2: Reproduce + minimise

Run the loop. Watch it go red as the bug appears.

Confirm:

- [ ] The loop produces the failure mode the **user** described, not a different failure that happens to be nearby. Wrong bug = wrong fix.
- [ ] The failure is reproducible across multiple runs (or, for non-deterministic bugs, reproducible at a high enough rate to debug against).
- [ ] You have captured the exact symptom (error message, wrong output, slow timing) so later phases can verify the fix actually addresses it.

### Minimise

Once it's red, shrink the repro to the **smallest scenario that still goes red**. Cut inputs, callers, config, data, and steps **one at a time**, re-running the loop after each cut, and keep only what's load-bearing for the failure.

Why bother: a minimal repro shrinks the hypothesis space in Phase 3 (fewer moving parts left to suspect) and becomes the clean regression test in Phase 5.

Done when **every remaining element is load-bearing**: removing any one of them makes the loop go green.

Do not proceed until you have reproduced **and** minimised.

## Phase 3: Hypothesise

Generate **3–5 ranked hypotheses** before testing any of them. Single-hypothesis generation anchors on the first plausible idea.

Each hypothesis must be **falsifiable**: state the prediction it makes.

> Format: "If <X> is the cause, then <changing Y> will make the bug disappear / <changing Z> will make it worse."

If you cannot state the prediction, the hypothesis is a vibe: discard or sharpen it.

**Show the ranked list to the user before testing.** They often have domain knowledge that re-ranks instantly ("we just deployed a change to #3"), or know hypotheses they've already ruled out. Cheap checkpoint, big time saver. Don't block on it; proceed with your ranking if the user is AFK.

## Phase 4: Instrument

Each probe must map to a specific prediction from Phase 3. **Change one variable at a time.**

Tool preference:

1. **Debugger / REPL inspection** if the env supports it. One breakpoint beats ten logs.
2. **Targeted logs** at the boundaries that distinguish hypotheses.
3. Never "log everything and grep".

**Tag every debug log** with a unique prefix, e.g. `[DEBUG-a4f2]`. Cleanup at the end becomes a single grep. Untagged logs survive; tagged logs die.

**Perf branch.** For performance regressions, logs are usually wrong. Instead: establish a baseline measurement (timing harness, `performance.now()`, profiler, query plan), then bisect. Measure first, fix second.

## Phase 5: Fix + regression test

Write the regression test **before the fix**, but only if there is a **correct seam** for it.

A correct seam is one where the test exercises the **real bug pattern** as it occurs at the call site. If the only available seam is too shallow (single-caller test when the bug needs multiple callers, unit test that can't replicate the chain that triggered the bug), a regression test there gives false confidence.

**If no correct seam exists, that itself is the finding.** Note it. The codebase architecture is preventing the bug from being locked down. Flag this for the next phase.

If a correct seam exists:

1. Turn the minimised repro into a failing test at that seam.
2. Watch it fail. If you forced the red by mutating code or a fixture, `diff` against a pristine copy to prove the mutation landed before you trust it.
3. Apply the fix.
4. Watch it pass.
5. Re-run the Phase 1 feedback loop against the original (un-minimised) scenario.

## Phase 6: Cleanup

Required before declaring done:

- [ ] Original repro no longer reproduces (re-run the Phase 1 loop)
- [ ] Regression test passes (or absence of seam is documented)
- [ ] All `[DEBUG-...]` instrumentation removed (`grep` the prefix)
- [ ] Throwaway prototypes deleted (or moved to a clearly-marked debug location)
- [ ] The hypothesis that turned out correct is stated in the commit / PR message, so the next debugger learns
{%- else -%}
# Bug 诊断流程（Diagnosing Bugs）

一套针对疑难 bug 的工程纪律。除非有明确正当理由，否则不要跳过任何阶段。

探索代码库时，若存在 `GLOSSARY.md`，先阅读它以建立相关模块的清晰心智模型，并查阅所触及区域的 ADR。

## 脱敏（Redact）

本技能要求你展示命令、输出和捕获的产物。**务必先对所有敏感信息脱敏**：用 `<REDACTED>` 替换密钥/凭据。围绕环境变量构建验证回路，使凭据留在环境中而非出现在展示输出里。捕获的网络产物会携带认证头：只引用真正包含信号的那几行。

如果脱敏后的输出不足以诊断问题，如实说明并向用户请求补充。

## 阶段 1：构建反馈回路

**这就是本技能的核心。** 其余步骤都是机械动作。只要你手头有一个针对该 bug 的**紧凑（tight）**通过/失败信号（一条稳定能在*这个* bug 上亮红灯的命令），你就一定能找到根因；二分查找、假设验证和插桩都只是它的消费者。如果连它都没有，盯再久代码也救不了你。

在这里投入不成比例的精力。**积极主动，发挥创意，绝不轻易放弃。**

### 构建回路的常见手段（大致按优先级排序）

1. **失败测试**：在触及该 bug 的任意接缝处写失败测试（单元、集成或 e2e）。
2. **Curl / HTTP 脚本**：直接打向运行中的开发服务器。
3. **CLI 调用**：喂入固定测试夹具输入，把 stdout 与已知正常快照做 diff。
4. **无头浏览器脚本**（Playwright / Puppeteer）：驱动 UI 并对 DOM / console / network 做断言。
5. **回放捕获轨迹**：把真实网络请求 / payload / 事件日志保存到磁盘，在隔离路径中回放。
6. **一次性测试夹具（Throwaway harness）**：拉起系统的最小子集（单个服务 + mock 依赖），用一次函数调用触发问题路径。
7. **属性 / 模糊循环**：如果 bug 是“偶尔输出错误”，跑 1000 组随机输入捕捉失败模式。
8. **二分查找夹具**：如果 bug 出现在两个已知状态（commit、数据集、版本）之间，自动化“在状态 X 启动、检查、重复”，从而交给 `git bisect run`。
9. **差分回路**：把同一份输入分别跑在旧版本与新版本（或两套配置）上，对比输出差异。
10. **HITL bash 脚本**：最后手段。如果必须由人工点击触发，用 `scripts/hitl-loop.template.sh` 驱动*人工操作*，让循环依然保持结构化，捕获的输出会回传给你。

把反馈回路搭对了，bug 就已经解决 90% 了。

### 收紧回路（Tighten the loop）

把反馈回路当成产品来打磨。一旦搭出*一个*回路，立刻**收紧**它：

- 能跑得更快吗？（缓存初始化、跳过无关引导、缩小测试范围。）
- 能让信号更锋利吗？（直接断言具体症状，而不是“没有崩溃”。）
- 能让它更确定吗？（固定时间、固定随机种子、隔离文件系统、冻结网络。）

一个跑 30 秒还时灵时不灵的回路几乎等于没有；一个 2 秒出结果的确定性回路，才是调试的超级武器。

### 非确定性 bug

目标不是追求“干净复现”，而是**拉高复现率**。循环触发 100 次、并发压测、人为加压、收窄时间窗口、注入 sleep。50% 复现率的 bug 可以调试；1% 的没法调，所以先把复现率拉高到可调试为止。

### 确实无法构建回路时

停下来并明确说明。列出你尝试过的所有方式。向用户请求：(a) 能复现该问题的环境访问权限；(b) 已脱敏的捕获产物（HAR 文件、日志转储、core dump、带时间戳的录屏）；或 (c) 添加临时生产环境埋点的许可。**没有回路，绝不进入猜想阶段。**

### 完成标准：一条能稳定亮红灯的紧凑回路

满足以下条件才算完成阶段 1：你能说出**一条命令**（脚本路径、测试命令或 curl），你已经**至少亲自运行过一次**（展示调用命令及其脱敏输出），并且它满足：

- [ ] **能亮红灯（Red-capable）**：确实跑过真实 bug 路径，并断言**用户的精确症状**，因此它能在这个 bug 上失败、修复后转绿。不是“能跑完不报错”，而必须能*抓到这个特定 bug*。
- [ ] **确定性**：每次运行结论一致（对于偶发 bug，按上文要求做到固定且高复现率）。
- [ ] **快速**：秒级完成，而不是分钟级。
- [ ] **Agent 可无人值守运行**：你能自己跑；只有实在需要人工介入时才走 `scripts/hitl-loop.template.sh`。

如果你发现自己还没拿到这条能亮红灯的命令，就已经开始读代码脑补理论了——**立刻停手：直接跳去猜想正是本技能要杜绝的失败模式。** 没有红灯命令，就没有阶段 2。

## 阶段 2：复现并最小化

运行回路，亲眼看着它因该 bug 亮红灯。

确认：

- [ ] 回路触发的正是**用户**描述的失败模式，而不是旁边碰巧撞上的另一个错误。找错 bug = 修错地方。
- [ ] 失败可在多次运行中稳定复现（或对非确定性 bug 达到足够高的复现率）。
- [ ] 已捕获精确症状（报错信息、错误输出、慢耗时），以便后续阶段验证修复确实解决了它。

### 最小化

一旦稳定亮红灯，把复现场景缩减到**仍然能亮红灯的最小场景**。**每次只砍掉一项**（输入、调用方、配置、数据或步骤），每次砍完重跑回路，只保留对触发失败真正起承重作用的要素。

为什么费这个劲：最小复现能大幅压缩阶段 3 的假设空间（可疑活动部件更少），而且它能直接变成阶段 5 干净的回归测试。

完成标准：**剩下的每一个元素都是承重件**——拿掉其中任意一个，回路就会变绿。

在完成复现**并且**最小化之前，不要进入下一步。

## 阶段 3：提出假设

在动手验证任何猜想之前，先列出 **3–5 条按优先级排序的假设**。只提一个假设很容易锚定在第一个听起来合理的念头上。

每条假设都必须是**可证伪的**：明确写出它做出的预测。

> 格式：“如果原因是 `<X>`，那么 `<改变 Y>` 会让 bug 消失 / `<改变 Z>` 会让情况恶化。”

如果写不出这条预测，这个假设就只是一种感觉：丢弃它或把它写精确。

**动手测试前先把排序后的假设清单展示给用户。** 用户往往拥有领域背景，能瞬间帮你重排优先级（“我们刚对 #3 发过一版改动”），或者指出哪些假设已经被排除过。这是一个低成本、高回报的检查点。如果用户不在电脑前，不要阻塞，直接按你的排序推进。

## 阶段 4：插桩验证

每一次探测都必须对应阶段 3 的某条具体预测。**每次只改变一个变量。**

工具优先级：

1. **调试器 / REPL 检查**（如果环境支持）。一个断点胜过十行日志。
2. 在能区分不同假设的边界处打**定向日志**。
3. 绝不“全量打日志再 grep”。

**给每条调试日志加上唯一前缀标签**，例如 `[DEBUG-a4f2]`。结束清理时只需一次 grep。不带标签的日志会遗留下来，带标签的日志一网打尽。

**性能分支：** 对于性能回归，打日志通常走错了方向。应先建立基线测量（计时夹具、`performance.now()`、profiler、查询计划），再做二分。先测量，再修复。

## 阶段 5：修复 + 回归测试

**在写修复之前先写回归测试**，但前提是存在**正确的接缝**。

正确的接缝是指测试能够以调用方真实发生的方式演练**真实的 bug 模式**。如果手头唯一的接缝太浅（例如 bug 需要多个调用方联动却只写了单调用方测试，或单元测试无法复现触发链），在那里写的回归测试只会制造虚假的安全感。

**如果不存在正确的接缝，这件事本身就是重要发现。** 记录下来——说明代码库架构正在阻碍该 bug 被测试锁死，并在下一阶段标出。

如果存在正确接缝：

1. 把最小化复现写成该接缝处的失败测试。
2. 亲眼看着它失败。如果你是通过改代码或改夹具强行制造红灯，务必与干净副本做 `diff` 确认变异确实生效，再相信这个红灯。
3. 实施修复。
4. 亲眼看着它通过。
5. 用原始（未最小化的）场景重跑阶段 1 的反馈回路。

## 阶段 6：清理收尾

宣布完成前必须全部勾完：

- [ ] 原始复现不再触发（重跑阶段 1 回路）
- [ ] 回归测试通过（或已记录缺失合适接缝的现状）
- [ ] 所有 `[DEBUG-...]` 插桩日志已清除（用 `grep` 核对前缀）
- [ ] 一次性原型代码已删除（或移至明确标记的调试位置）
- [ ] 在 commit / PR 信息中写明最终被证实的根因假设，让下一位排查者有所积累
{%- endif %}
