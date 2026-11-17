---
name: prototype
description: >-
{%- if options["skill_lang"] == "en" %}
  Build a throwaway prototype to answer a design question. Use when the user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.
{%- else %}
  构建一次性原型来回答单个设计问题。当用户想要验证某个状态模型或逻辑是否顺手，或探索某个 UI 应该长什么样时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
# Prototype

A prototype is **throwaway code that answers a question**. The question decides the shape.

## Pick a branch

Identify which question is being answered, using the user's prompt, the surrounding code, or by asking if the user is around:

- **"Does this logic / state model feel right?"** → [LOGIC.md](LOGIC.md). Build a single shareable HTML file (free-play buttons plus tabbed guided walkthroughs) that pushes the state machine through cases that are hard to reason about on paper, and that a non-developer can drive.
- **"What should this look like?"** → [UI.md](UI.md). Generate several radically different UI variations on a single route, switchable via a URL search param and a floating bottom bar.

The two branches produce very different artifacts, so getting this wrong wastes the whole prototype. If the question is genuinely ambiguous and the user isn't reachable, default to whichever branch better matches the surrounding code (a backend module → logic; a page or component → UI) and state the assumption at the top of the prototype.

## Rules that apply to both

1. **Throwaway from day one, and clearly marked as such.** Locate the prototype code close to where it will actually be used (next to the module or page it's prototyping for) so context is obvious, but name it so a casual reader can see it's a prototype, not production. For throwaway UI routes, obey whatever routing convention the project already uses; don't invent a new top-level structure.
2. **Trivial to run.** A UI prototype starts from one command in the project's task runner: `pnpm <name>`, `python <path>`, `bun <path>`, etc. A logic demo is a single HTML file the user double-clicks. Either way, no thinking required to start it.
3. **No persistence by default.** State lives in memory. Persistence is the thing the prototype is _checking_, not something it should depend on. If the question explicitly involves a database, hit a scratch DB or a local file with a clear "PROTOTYPE, wipe me" name.
4. **Skip the polish.** No tests, no error handling beyond what makes the prototype _runnable_, no abstractions. The point is to learn something fast.
5. **Surface the state.** After every action (logic) or on every variant switch (UI), print or render the full relevant state so the user can see what changed.
6. **Capture it when done.** Fold any validated decision into the real code, then capture the prototype itself as a **primary source**: commit it to a throwaway branch, out of main, and leave a context pointer to that branch on the implementation issue. Capture the answer too (the verdict and the question it settled) in the issue or a commit. The main branch keeps only the validated decision.
{%- else -%}
# 原型（Prototype）

原型是**用来回答一个问题的一次性代码**。问题本身决定了原型的形态。

## 选择分支

根据用户提示词、周边代码，或在用户在线时直接询问，判断当前要回答哪类问题：

- **“这套逻辑 / 状态模型对劲吗？”** → [LOGIC.md](LOGIC.md)。构建单个可分享的 HTML 文件（自由试玩按钮 + 分标签页的引导式演练），把状态机推过纸面上难以推演的边界情况，并让非开发人员也能直接操作。
- **“这个界面应该长什么样？”** → [UI.md](UI.md)。在同一个路由上生成几套截然不同的 UI 变体，通过 URL 查询参数和底部悬浮栏切换。

两条分支产出的产物截然不同，选错分支等于整个原型白做。如果问题确实模糊且用户暂时不在，默认选择更贴合周边代码的分支（后端模块 → 逻辑；页面或组件 → UI），并在原型顶部写明这一假设。

## 两条分支共同遵守的规则

1. **从第一天起就是一次性的，并明确标记出来。** 把原型代码放在离未来真实使用位置很近的地方（紧邻它要验证的模块或页面），让上下文一目了然；但命名要让随手路过的读者一眼看出它是原型、不是生产代码。对于一次性 UI 路由，遵守项目现有的路由约定，不要发明新的顶层结构。
2. **极其容易运行。** UI 原型能从项目任务运行器的一条命令启动（`pnpm <name>`、`python <path>`、`bun <path>` 等）；逻辑演示页则是用户双击即开的单个 HTML 文件。无论哪种，启动都不需要动脑子。
3. **默认不做持久化。** 状态只存在内存里。持久化往往正是原型要*检验*的对象，而不是原型该依赖的东西。如果问题明确涉及数据库，使用临时 scratch DB 或带有明确 `"PROTOTYPE, wipe me"` 命名的本地文件。
4. **跳过打磨。** 不写测试、不写超出“能跑起来”之外的错误处理、不做多余抽象。重点是快速学到结论。
5. **把状态摊在明面上。** 逻辑原型每次动作后、UI 原型每次切换变体时，都打印或渲染出完整的相关状态，让用户直观看到什么变了。
6. **完成后妥善沉淀。** 把验证出的决策吸收进正式代码，然后将原型本身作为**一手来源**保存下来：提交到一个脱离 main 的一次性分支，并在实现工单上留下指向该分支的上下文指针。把结论（最终拍板的方案以及它回答的问题）也记录在工单或提交里。main 分支只保留已验证的决策。
{%- endif %}
