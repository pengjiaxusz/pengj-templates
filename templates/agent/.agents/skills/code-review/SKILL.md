---
name: code-review
description: >-
{%- if options["skill_lang"] == "en" %}
  Review the changes since a fixed point (commit, branch, tag, or merge-base) along two axes: Standards (does the code follow this repo's documented coding standards?) and Spec (does the code match what the originating issue/spec asked for?). Runs both reviews in parallel sub-agents and reports them side by side. Use when the user wants to review a branch, a PR, work-in-progress changes, or asks to "review since X".
{%- else %}
  沿双轴评审自某个固定基准点（commit、分支、tag 或 merge-base）以来的改动：规范轴（是否符合仓库文档化的编码规范）与需求轴（是否匹配源 issue/spec 的要求）。通过并行 sub-agent 分别执行两项评审并列汇报。当用户要求评审分支、PR、未提交/进行中改动或“评审自 X 以来的变更”时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
Two-axis review of the diff between `HEAD` and a fixed point the user supplies:

- **Standards**: does the code conform to this repo's documented coding standards?
- **Spec**: does the code faithfully implement the originating issue / spec?

Both axes run as **parallel sub-agents** so they don't pollute each other's context, then this skill aggregates their findings. Start every sub-agent with a fresh context window.

The issue tracker should have been provided to you. If not, tell the user to run `/setup-matt-pocock-skills`.

## Process

### 1. Pin the fixed point

Whatever the user said is the fixed point (a commit SHA, branch name, tag, `main`, `HEAD~5`, etc.). If they didn't specify one, ask for it.

Capture the diff command once: `git diff <fixed-point>...HEAD` (three-dot, so the comparison is against the merge-base). Also note the list of commits via `git log <fixed-point>..HEAD --oneline`.

Before going further, confirm the fixed point resolves (`git rev-parse <fixed-point>`) and the diff is non-empty. A bad ref or empty diff should fail here, not inside two parallel sub-agents.

### 2. Identify the spec source

Look for the originating spec, in this order:

1. Issue references in the commit messages (`#123`, `Closes #45`, GitLab `!67`, etc.), fetched via the workflow in the tracker doc.
2. A path the user passed as an argument.
3. A spec file under `docs/`, `specs/`, or `.scratch/` matching the branch name or feature.
4. If nothing is found, ask the user where the spec is. If they say there isn't one, the **Spec** sub-agent will skip and report "no spec available".

### 3. Identify the standards sources

Search the repo for every file that documents how code should be written. When `CODING_STANDARDS.md` or `CONTRIBUTING.md` exists, it must be on the list.

On top of whatever the repo documents, the Standards axis always carries the **smell baseline** below: a fixed set of Fowler code smells (_Refactoring_, ch.3) that applies even when a repo documents nothing. Two rules bind it:

- **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
- **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation. Like any standard here, skip anything tooling already enforces.

Each smell reads *what it is* → *how to fix*; match it against the diff:

- **Mysterious Name**: a function, variable, or type whose name doesn't reveal what it does or holds. → rename it; if no honest name comes, the design's murky.
- **Duplicated Code**: the same logic shape appears in more than one hunk or file in the change. → extract the shared shape, call it from both.
- **Feature Envy**: a method that reaches into another object's data more than its own. → move the method onto the data it envies.
- **Data Clumps**: the same few fields or params keep travelling together (a type wanting to be born). → bundle them into one type, pass that.
- **Primitive Obsession**: a primitive or string standing in for a domain concept that deserves its own type. → give the concept its own small type.
- **Repeated Switches**: the same `switch`/`if`-cascade on the same type recurs across the change. → replace with polymorphism, or one map both sites share.
- **Shotgun Surgery**: one logical change forces scattered edits across many files in the diff. → gather what changes together into one module.
- **Divergent Change**: one file or module is edited for several unrelated reasons. → split so each module changes for one reason.
- **Speculative Generality**: abstraction, parameters, or hooks added for needs the spec doesn't have. → delete it; inline back until a real need shows.
- **Message Chains**: long `a.b().c().d()` navigation the caller shouldn't depend on. → hide the walk behind one method on the first object.
- **Middle Man**: a class or function that mostly just delegates onward. → cut it, call the real target direct.
- **Refused Bequest**: a subclass or implementer that ignores or overrides most of what it inherits. → drop the inheritance, use composition.

### 4. Spawn both sub-agents in parallel

Issue both sub-agent calls together, in the foreground, and aggregate the reports they return.

**Standards sub-agent prompt** should include:

- The full diff command and commit list.
- The list of standards-source files you found in step 3, **plus the smell baseline from step 3** pasted in full (the sub-agent has no other access to it).
- The brief: "Report, per file/hunk where relevant, (a) every place the diff violates a documented standard: cite the standard (file + the rule); and (b) any baseline smell you spot: name it and quote the hunk. Distinguish hard violations from judgement calls: documented-standard breaches can be hard, but baseline smells are always judgement calls, and a documented repo standard overrides the baseline. Skip anything tooling enforces. Under 400 words."

**Spec sub-agent prompt** should include:

- The diff command and commit list.
- The path or fetched contents of the spec.
- The brief: "Report: (a) requirements the spec asked for that are missing or partial; (b) behaviour in the diff that wasn't asked for (scope creep); (c) requirements that look implemented but where the implementation looks wrong. Quote the spec line for each finding. Under 400 words."

If the spec is missing, skip the Spec sub-agent and note this in the final report.

### 5. Aggregate

Present the two reports under `## Standards` and `## Spec` headings, verbatim or lightly cleaned. Do **not** merge or rerank findings, because the two axes are deliberately separate (see _Why two axes_).

End with a one-line summary: total findings per axis, and the worst issue _within each axis_ (if any). Don't pick a single winner across axes: that's the reranking the separation exists to prevent.

## Why two axes

A change can pass one axis and fail the other:

- Code that follows every standard but implements the wrong thing → **Standards pass, Spec fail.**
- Code that does exactly what the issue asked but breaks the project's conventions → **Spec pass, Standards fail.**

Reporting them separately stops one axis from masking the other.
{%- else -%}
沿双轴对 `HEAD` 与用户指定的固定基准点之间的 diff 进行评审：

- **规范（Standards）**：代码是否符合本仓库成文的编码规范？
- **需求（Spec）**：代码是否忠实实现了源 issue / spec 的要求？

两条轴各自作为**并行 sub-agent** 运行，避免互相污染上下文，随后由本技能汇总双方发现。每个 sub-agent 都必须从全新的上下文窗口启动。

issue tracker 的配置应当已经提供给你；如果没有，请提示用户先运行 `/setup-matt-pocock-skills`。

## 流程

### 1. 锚定基准点

用户指定的任意位置即为基准点（commit SHA、分支名、tag、`main`、`HEAD~5` 等）。如果用户未指定，向其询问。

确定一次 diff 命令：`git diff <fixed-point>...HEAD`（使用三点语法，从而基于 merge-base 对比）。同时通过 `git log <fixed-point>..HEAD --oneline` 获取提交列表。

继续之前，先确认基准点可解析（`git rev-parse <fixed-point>`）且 diff 非空。无效的引用或空 diff 应当在这一步直接失败，不要拖到两个并行 sub-agent 里才报错。

### 2. 定位 Spec 来源

按以下顺序寻找源 spec：

1. 提交信息中的 issue 引用（`#123`、`Closes #45`、GitLab `!67` 等），按 tracker 文档中的流程拉取。
2. 用户作为参数传入的路径。
3. `docs/`、`specs/` 或 `.scratch/` 下与分支名或特性名匹配的 spec 文件。
4. 如果都找不到，询问用户 spec 在哪里。如果用户表示没有 spec，**Spec** sub-agent 将跳过并汇报“无可用 spec”。

### 3. 定位规范来源

在仓库中搜索所有记录了代码编写规范的文件。只要存在 `CODING_STANDARDS.md` 或 `CONTRIBUTING.md`，它们必须列入清单。

除了仓库成文规范之外，规范轴还始终携带下方的**代码异味基线（smell baseline）**：一组固定的 Fowler 代码异味（《重构》第 3 章），即使仓库没有任何成文规范也同样适用。它受两条规则约束：

- **仓库规范优先。** 仓库成文规范永远胜出；只要仓库规范明确认可某项基线会标记的做法，就压制该异味告警。
- **始终属于判断性提示。** 每条异味都是带标签的启发式提示（如“疑似依恋情结 / Feature Envy”），绝不是硬性违规。与这里的任何规范一样，工具链已经自动强制检查的内容直接跳过。

每条异味按 *它是什么* → *如何修复* 阅读，并与 diff 对照：

- **神秘命名（Mysterious Name）**：函数、变量或类型的名字无法揭示它做什么或装什么。→ 重命名；如果起不出一个诚实的名字，说明设计本身浑浊。
- **重复代码（Duplicated Code）**：同一套逻辑形态在本次改动的多个 hunk 或多个文件中重复出现。→ 提取共享形态，从两处分别调用。
- **依恋情结（Feature Envy）**：一个方法访问另一个对象的数据比访问自己的数据还要频繁。→ 把该方法搬到它所依恋的数据所在之处。
- **数据泥团（Data Clumps）**：相同的几个字段或参数总是结伴出现（一个呼之欲出的类型）。→ 把它们打包成一个类型，传递该类型。
- **基本类型偏执（Primitive Obsession）**：本该拥有自己独立类型的领域概念却一直用原始类型或字符串凑合。→ 为该概念建一个专属的小类型。
- **重复的 switch（Repeated Switches）**：针对同一类型的相同 `switch`/`if` 级联分支在改动中反复出现。→ 改用多态，或抽成两处共享的一张映射表。
- **霰弹式修改（Shotgun Surgery）**：一个逻辑变更迫使你在 diff 里分散修改大量文件。→ 把总是一起变化的内容收拢到同一个模块。
- **发散式变化（Divergent Change）**：同一个文件或模块因为好几个互不相关的原因被同时修改。→ 拆分模块，使每个模块只因单一原因而变。
- **夸夸其谈通用性（Speculative Generality）**：为了 spec 根本没提的需求而预先添加的抽象、参数或钩子。→ 删掉并内联回去，直到真实需求出现再说。
- **过长的消息链（Message Chains）**：调用方不该依赖的冗长 `a.b().c().d()` 链式导航。→ 把这串遍历隐藏到第一个对象的一个方法背后。
- **中间人（Middle Man）**：一个类或函数绝大部分代码只是把调用原封不动转发给别人。→ 砍掉它，直接调用真正目标。
- **被拒绝的遗赠（Refused Bequest）**：子类或实现方忽略或覆写了所继承内容的大部分。→ 放弃继承，改用组合。

### 4. 并行拉起两个 sub-agent

在前台同时发起两个 sub-agent 调用，并汇总它们返回的报告。

**Standards sub-agent 提示词**应包含：

- 完整的 diff 命令与提交列表。
- 第 3 步找到的规范源文件列表，**加上完整粘贴的第 3 步异味基线**（sub-agent 没有其他途径看到它）。
- 任务简报：“按文件/hunk（在相关处）汇报：(a) diff 违反仓库成文规范的每一处：引用具体规范（文件 + 规则）；(b) 你发现的任何基线异味：点出异味名称并引用对应 hunk。明确区分硬性违规与判断性建议：成文规范违例可以是硬性问题，但基线异味永远只是判断性建议，且仓库成文规范优先于基线。跳过工具链已自动强制检查的内容。控制在 400 词以内。”

**Spec sub-agent 提示词**应包含：

- diff 命令与提交列表。
- spec 的路径或已拉取的完整内容。
- 任务简报：“汇报：(a) spec 要求但缺失或只实现了一部分的需求；(b) diff 中出现但 spec 未曾要求的行为（范围蔓延）；(c) 看起来已实现但实现方式疑似有误的需求。每条发现请引用对应的 spec 原文。控制在 400 词以内。”

如果缺失 spec，跳过 Spec sub-agent 并在最终报告中注明。

### 5. 汇总报告

在 `## Standards` 和 `## Spec` 两个标题下原样（或轻度整理后）呈现两份报告。**不要**合并或跨轴重排发现，因为双轴刻意保持分离（见*为什么分两条轴*）。

结尾用一句话总结：每条轴的发现总数，以及*各轴内部*最严重的问题（如有）。不要跨轴选出唯一“最大问题”：防止互相掩盖正是双轴分离存在的意义。

## 为什么分两条轴

一次改动完全可能一条轴通过而另一条轴失败：

- 代码严丝合缝遵守了所有规范，但实现错了需求 → **Standards 通过，Spec 失败。**
- 代码精确完成了 issue 要求的事，但破坏了项目约定 → **Spec 通过，Standards 失败。**

分别汇报可以防止一条轴的结论掩盖另一条轴的问题。
{%- endif %}
