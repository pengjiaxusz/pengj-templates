---
name: ask-matt
description: >-
{%- if options["skill_lang"] == "en" %}
  Ask which skill or flow fits your situation. A router over the skills in this repo.
{%- else %}
  询问当前场景适合使用哪个技能或工作流。本仓库技能体系的路由指南。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
# Ask Matt

You don't remember every skill, so ask.

Before stating what a skill does or recommending a step be skipped, read that skill's SKILL.md: the summaries here are for orientation only.

A **flow** is a path through the skills. Most paths run along one **main flow**, and two **on-ramps** merge onto it. Everything else is standalone, or a vocabulary layer that runs underneath.

## The main flow: idea → ship

The route most work travels. You have an idea and want it built.

1. **`/grill-with-docs`** sharpens the idea by interview. Start here whenever you are **working in a working directory**: it's stateful, retaining what it learns in `GLOSSARY.md` and ADRs. (No working directory? Use `/grill-me` instead, covered under Standalone. Both run the same `/grilling` primitive; `grill-with-docs` is the one that leaves a paper trail, which makes it the better of the two whenever a repo is there to leave it in.)
2. **Branch: can you settle every question in conversation?** If a question needs a runnable answer (state, business logic, a UI you have to see), detour through a prototype, bridged by **`/handoff`** in both directions (a prototype lives in its own directory, which is exactly what `/handoff` is for; see Phase boundaries):
   - **`/handoff`** out, then open a fresh session against that file,
   - **`/prototype`** to answer the question with throwaway code,
   - **`/handoff`** back what you learned, and reference it from the original idea thread.
3. **Branch: is this a multi-session build?**
   - **Yes** → **`/to-spec`** (turn the thread into a spec), then **`/to-tickets`** to split it into tracer-bullet tickets, each declaring its **blocking edges**. Then work the tickets one of two ways:
     - **`/implement`** per ticket, **`/clear`ing context between each one**. On a local tracker that's one file per ticket under `.scratch/<feature>/issues/`, worked blockers-first by hand; on a real tracker the edges become native blocking links, so any ticket whose blockers are done can be grabbed. Each ticket is self-contained, so the last one's context is disposable.
     - **`/implement-spec`** for the whole spec in one run. It reads the tickets as a **task graph**, runs implementer subagents across the ready **frontier** in parallel, and lands everything on one **integration branch**. Reach for it when you'd rather orchestrate the build than drive each ticket yourself.
   - **No** → **`/implement`** right here, in the same context window.

   Either way, the code gets built by driving **`/tdd`** (one red-green slice at a time) and closes out with **`/code-review`**, a two-axis review (Standards + Spec) of the diff. `/implement` runs both per ticket; `/implement-spec`'s implementers each drive `/tdd`, and it runs one `/code-review` over the integration branch. Reach for **`/tdd`** on its own when you just want to build a concrete behaviour test-first without a full spec, and **`/code-review`** on its own whenever you want to review a branch or PR against a fixed point.

   When the work goes up as a pull request, **`/pr`** shapes the body: the smallest visual that shows the change, before/after evidence that it works, and a one-way or two-way door call. It's model-invoked, so the agent reaches for it whenever it writes a PR.

4. **`/retro`** closes the loop. After a build, and especially one that went sideways, it looks back over the session and suggests changes to the agent's **environment**, not the code: navigation pointers, automated checks, the coding standards `/code-review` enforces, steering files, tooling. Mechanical mistakes become deterministic checks; judgement calls become coding standards. The next build then starts from a better environment.

### Context hygiene

Keep steps 1–3 in **one unbroken context window** (don't compact or clear until after `/to-tickets`) so the grilling, spec, and tickets all build on the same thinking. Each `/implement` then starts fresh, working from the ticket. Run `/retro` in the session it's looking back on, before you clear; after clearing, point it at that session's log instead.

The limit on this is the **[smart zone](https://www.aihero.dev/ai-coding-dictionary/smart-zone)**: the window (~150k tokens on state-of-the-art models) within which the model still reasons sharply. If a session approaches it before `/to-tickets`, don't push on degraded; `/compact` at the nearest phase boundary and carry on (see Phase boundaries).

## On-ramps

A starting situation that generates work, then merges onto the main flow.

- **Bugs and requests piling up** → **`/triage`**. It moves issues through triage roles and produces agent-ready issues, which **`/implement`** later picks up.

  Triage is only for issues **you didn't create**: bug reports, incoming feature requests, anything that arrives raw. Tickets that `/to-tickets` produced are already agent-ready, so **don't triage them**.

- **Something's broken** → **`/diagnosing-bugs`**. For the hard ones: the bug that resists a first glance, the intermittent flake, the regression that crept in between two known-good states. It refuses to theorise until it has a **tight feedback loop** (one command that already goes red on *this* bug), then fixes with a regression test. Once the fix is in, run **`/retro`** in the same session to ask what would have prevented the bug; where the real finding is that there's no good seam to lock it down, that's a job for **`/improve-codebase-architecture`**.

- **A huge, foggy effort: a greenfield project or a huge feature build, too big for one session** → **`/wayfinder`**, the most cognitively demanding flow here. When the way from here to the destination isn't visible yet, it charts a **shared map** of **decision tickets** on the issue tracker and resolves them one at a time, producing **decisions, not deliverables**, until the fog is pushed back and the way is clear. Where **`/grill-with-docs`** sharpens an idea you can hold in one session, wayfinder is for the idea you can't, and it's slower and denser, so save it for exactly that, never a well-scoped feature.

  When the map clears, **it hands off, it doesn't build**: merge onto the main flow at **`/to-spec`**, which collapses the map's linked decisions into a buildable plan, then `/to-tickets` and `/implement` as usual. Looping the map straight into `/implement` skips that collapse and throws the linked detail away, so go straight to `/implement` only when the effort turned out genuinely small.

## Codebase health

Not feature work, just upkeep.

- **`/improve-codebase-architecture`** runs whenever you have a spare moment to keep the codebase good for agents to operate in. It surfaces **deepening opportunities**; picking one _generates an idea_ you can take into the main flow at `/grill-with-docs`. It's the survey that finds the candidates; **`/codebase-design`** (below) is the bench you design the chosen one on.

## Vocabulary underneath

Two model-invoked references that run *beneath* the other skills, each the single source of truth for its vocabulary. Reach for them directly when the **words**, not the process, are the problem; or let the skills above pull them in.

- **`/domain-modeling`**: sharpen the project's *domain* language: challenge a fuzzy term, resolve an overloaded word ("account" doing three jobs), record a hard-to-reverse decision as an ADR. It's the active discipline `/grill-with-docs` drives to keep `GLOSSARY.md` a clean glossary.
- **`/codebase-design`** is the deep-module vocabulary (module, interface, depth, seam, adapter, leverage, locality) for designing a module's *shape*: a lot of behaviour behind a small interface at a clean seam. `/tdd` and `/improve-codebase-architecture` both speak it.

## Phase boundaries

A **phase** is a chunk of work inside a session: the grilling, the implementation, the QA. At the **boundary** between two of them you have five options, and picking between them is the fuzziest decision in this whole map:

- **Continue**: stay put. Costs nothing, loses nothing.
- **`/clear`**: empty the window, when nothing here matters to what's next.
- **`/handoff`** writes a portable markdown file. Narrow: only for a **new harness**, a **new directory**, a **colleague**, or forking a side task **mid-phase**. What it buys is portability.
- **Subagent**: send a tightly-scoped task to its own window and get a report back.
- **`/compact`** compresses this context and seeds a fresh session with it. The **default**, at the bottom of the tree rather than the first reach.

Read [PHASE-BOUNDARIES.md](PHASE-BOUNDARIES.md) for the ordered tree: the five questions, the reasoning behind each branch, and why the primary-source cost makes **Continue** the one to rule out first. Make the decision **at** a boundary; mid-phase, continue or split the rest into subagents.

## Standalone

Off the main flow entirely.

- **`/grill-me`**: the same relentless interview as `/grill-with-docs`, but **stateless**: it saves nothing locally and builds no `GLOSSARY.md`. Reach for it when you are **not working in a working directory** (sharpening a plan, a design, a piece of writing, anything with no repo under it). If you are in a working directory, use `/grill-with-docs` instead: it runs the same interview and leaves a paper trail, so it is strictly the better one.
- **`/grilling`** is the interview primitive itself: rounds, the frontier, facts are the agent's job and decisions are yours. `/grill-me` and `/grill-with-docs` are the two named ways in, and `/triage`, `/wayfinder` and `/improve-codebase-architecture` all run it internally. Reach for it directly only when you want the interview with no wrapper around it.
- **`/prototype`** is a small, throwaway program that answers one design question: does this state model feel right, or what should this UI look like. Throwaway is a constraint on how the code is written, not a promise to destroy it: the answer folds into the real code, and the prototype itself is kept as a **primary source** on a `prototype/<name>` branch out of main, pointed at from the implementation issue. It's the detour in step 2 of the main flow, but reach for it any time a design question is hard to settle on paper.
- **`/research`**: delegate reading legwork to a **background agent**: it investigates a question against **primary sources**, then leaves a cited Markdown file in the repo. Keep working while it reads. The file it produces is something to take *into* the main flow at `/grill-with-docs`, since research feeds the thinking rather than replacing it.
- **`/to-questionnaire`** comes in when the thing blocking you isn't in your head or the codebase but in **someone else's**, and it writes them a questionnaire to fill in. It's the inverse of `/grill-me`: instead of interviewing you about the subject, it interviews you about the **send** (who it's going to, what you need back) and aims the questions at the gap. What comes back is material for `/grill-with-docs` or `/to-spec`.
- **`/wizard`** is for the steps only a **human** can take: provisioning infrastructure, setting up credentials or CI secrets, clicking through an unfamiliar third-party dashboard, running a one-off migration or cutover. It generates an interactive bash script that opens each URL, captures each value, and writes it into `.env` and GitHub secrets, so the procedure stops being something you re-explain to an agent every time. Model-invoked, so the agent reaches for it the moment it hits a wall only you can pass. If the agent could just do it itself, it should; this is for where a human is genuinely in the loop.
- **`/wait-what`** is the corrective for a message that didn't land. Use it mid-conversation, inside any other skill, and the agent re-pitches what it just said with the context you were missing, in plain English, using the `GLOSSARY.md` vocabulary. It works after the fact; `/grill-with-docs` is the upfront cure, because a shared language agreed early is what stops the jargon arriving at all.
- **`/teach`**: learn a concept over multiple sessions, using the current directory as a stateful workspace.
- **`/writing-for-agents`** is the reference for writing documents agents consume: skills, AGENTS.md, pointed-at docs.

## Precondition

**`/setup-matt-pocock-skills`**: run before your first engineering flow to configure the issue tracker, triage labels, and doc layout the other skills assume. Custom issue trackers also work.
{%- else -%}
# Ask Matt（技能与工作流路由）

你记不住每一个技能，直接问就好。

在说明某个技能做什么或建议跳过某一步之前，先读取该技能的 `SKILL.md`：这里的摘要仅用于定位导航。

一条**工作流（flow）**是指串联多个技能的路径。大多数工作沿着一条**主干流（main flow）**推进，另有两条**入口匝道（on-ramps）**汇入主干。其余技能要么独立使用，要么作为底层词汇层服务于上层流程。

## 主干流：想法 → 上线（idea → ship）

绝大多数开发任务走的路径：你有一个想法，想把它做出来。

1. **`/grill-with-docs`** 通过持续追问打磨想法。只要你在**工作目录中作业**，就从这里开始：它是有状态的，会把学到的内容沉淀到 `GLOSSARY.md` 和 ADR 中。（不在工作目录里？改用“独立技能”里的 `/grill-me`。两者底层跑的都是 `/grilling` 原语；`grill-with-docs` 会留下文档轨迹，因此只要有仓库可以落盘，它就严格更优。）
2. **分支：所有问题都能在对话里拍板吗？** 如果某个问题需要可运行的验证才能回答（状态流转、业务逻辑、必须亲眼看到的 UI），先通过原型绕道验证，并双向使用 **`/handoff`** 衔接（原型通常建在独立目录中，这正是 `/handoff` 的用武之地；参见「阶段边界」）：
   - 先 **`/handoff`** 出去，对着该文件开启新会话，
   - 用 **`/prototype`** 写一次性代码验证问题，
   - 再 **`/handoff`** 把学到的结论带回来，并在原想法线程中引用它。
3. **分支：这需要跨多个会话来构建吗？**
   - **是** → 先用 **`/to-spec`**（把讨论线程整理成 spec），再用 **`/to-tickets`** 拆分成带**阻塞边（blocking edges）**的曳光弹（tracer-bullet）工单。然后有两种推进方式：
     - 逐个工单跑 **`/implement`**，**每个工单之间 `/clear` 清空上下文**。若使用本地跟踪器，`.scratch/<feature>/issues/` 下每个工单对应一个文件，按先解决阻塞项的顺序手动推进；若使用真实 issue tracker，阻塞边会变成原生的阻塞关系，任何前置阻塞已完成的工单都可直接认领。每个工单都是自包含的，因此上一个工单的上下文可以直接丢弃。
     - 用 **`/implement-spec`** 一次性跑完整个 spec。它把工单读取为一张**任务图（task graph）**，并行拉起多个实现者 sub-agent 处理已就绪的**前沿（frontier）**工单，并最终汇入同一条**集成分支（integration branch）**。当你更希望编排整个构建而不是亲自逐个驱动工单时使用它。
   - **否** → 直接在当前上下文窗口运行 **`/implement`**。

   无论哪种方式，代码都会通过驱动 **`/tdd`**（一次一个红-绿切片）来构建，并以 **`/code-review`** 收尾——沿双轴（规范 Standards + 需求 Spec）评审 diff。`/implement` 会在每个工单结束时跑这两项；`/implement-spec` 的每个实现者各自跑 `/tdd`，并在集成分支上统一跑一次 `/code-review`。当你只想不写完整 spec、直接测试先行地实现一个具体行为时，可单独调用 **`/tdd`**；想基于某个固定基准点评审分支或 PR 时，可单独调用 **`/code-review`**。

   当改动准备提成 Pull Request 时，**`/pr`** 负责塑造 PR 正文：能说明改动的最小化可视化表达、改动前后有效的证据，以及单向门/双向门风险评估。它是模型自动触发的技能，Agent 在撰写 PR 时会自动调用它。

4. **`/retro`** 完成闭环。一次构建结束后（尤其是过程曲折的那种），它回顾整场会话并建议改进 Agent 的**环境**而非代码本身：导航指针、自动化检查、`/code-review` 强制执行的编码规范、引导文件与工具链。机械性失误变成确定性检查；判断性失误变成编码规范。下一次构建便能从更好的环境出发。

### 上下文卫生（Context hygiene）

将第 1–3 步保持在**同一个连续的上下文窗口**中（在 `/to-tickets` 完成前不要 compact 或 clear），这样追问、spec 和工单才能建立在同一套完整思考之上。随后每个 `/implement` 从干净状态启动，直接依据工单施工。在清空之前、于当前会话内运行 `/retro`；如果已经清空了，则把上次会话的日志指给它看。

这里的上限是 **[smart zone](https://www.aihero.dev/ai-coding-dictionary/smart-zone)**：主流前沿模型仍能保持敏锐推理的窗口上限（约 150k token）。如果会话在 `/to-tickets` 之前就接近上限，不要在退化状态下硬撑；在最近的阶段边界执行 `/compact` 然后继续（见「阶段边界」）。

## 入口匝道（On-ramps）

从特定起点产生工作项，随后汇入主干流。

- **Bug 和需求越积越多** → **`/triage`**。它把 issue 推进过分诊角色，产出可供 Agent 直接接手的就绪 issue，之后由 **`/implement`** 领走。

  分诊只针对**不是你亲手创建的** issue：bug 报告、外部需求、任何原始涌入的条目。由 `/to-tickets` 生成的工单本身已经是 Agent 就绪状态，**不要再对它们做分诊**。

- **有东西坏掉了** → **`/diagnosing-bugs`**。专治硬骨头：第一眼看不出来的 bug、偶发抖动（flake）、在两个已知正常状态之间悄悄引入的回归。在拿到**紧凑反馈回路（tight feedback loop）**（一条稳定能在*这个* bug 上亮红灯的命令）之前，它拒绝空想理论；拿到后再带着回归测试修复。修完后在同一会话跑一次 **`/retro`**，追问当初怎么做能防住这个 bug；如果真正的根因是代码里缺乏合适的接缝（seam）来锁死它，那就是 **`/improve-codebase-architecture`** 的活。

- **庞大且迷雾重重的工程：全新绿地项目或大到单次会话装不下的巨型特性** → **`/wayfinder`**，这里认知负荷最高的工作流。当从当前位置通往终点的路还看不清时，它在 issue tracker 上绘制一张由**决策工单（decision tickets）**构成的**共享地图**，逐个敲定，产出的是**决策而不是交付物**，直到迷雾散尽、路径清晰。**`/grill-with-docs`** 用于打磨单次会话装得下的想法，而 wayfinder 专治装不下的想法——它更慢、密度更高，所以只留给这种场景，绝不用于边界清晰的小特性。

  当迷雾散尽、地图明朗时，**它负责交接而非直接施工**：在 **`/to-spec`** 处汇入主干流，把地图上彼此关联的决策收束成可构建的计划，接着按常规走 `/to-tickets` 和 `/implement`。如果把地图直接丢进 `/implement` 会跳过这次收束并丢失关联细节，只有当实际工作量被证明确实很小时才可直连 `/implement`。

## 代码库健康度

不是新特性开发，而是日常养护。

- **`/improve-codebase-architecture`** 可以在你有空时随时运行，保持代码库对 Agent 友好。它会挖掘**深化机会（deepening opportunities）**；选中其中之一就会*催生一个想法*，你可以把它带进主干流的 `/grill-with-docs`。它是找出候选点的勘测仪；下面的 **`/codebase-design`** 则是用来设计所选模块的工作台。

## 底层词汇表（Vocabulary underneath）

两个在其余技能*下方*运行的模型自动触发参考技能，各自是对应词汇体系的唯一事实来源。当问题出在**用词**而不是流程时可直接调用它们，也可让上层技能自动引入。

- **`/domain-modeling`**：打磨项目的*领域*语言——质疑模糊术语、拆解过载词汇（比如把做三件事的 "account" 拆清）、把难以反悔的决策记成 ADR。它是 `/grill-with-docs` 背后驱动 `GLOSSARY.md` 保持干净的主动纪律。
- **`/codebase-design`**：深度模块设计词汇表（module、interface、depth、seam、adapter、leverage、locality），用于塑造模块的*形态*：在一个干净的接缝处，用极小的接口封装大量行为。`/tdd` 和 `/improve-codebase-architecture` 都说这套语言。

## 阶段边界（Phase boundaries）

**阶段**是会话内的一块工作。在两个阶段之间的**边界**处你有五个选择，而在这五个之间做取舍是整张地图里最需要判断力的地方：

- **Continue**：留在原处。零成本、零损失。
- **`/clear`**：清空窗口，当当前一切对下一步毫无意义时使用。
- **`/handoff`**：写一份可移植的 Markdown 文件。适用范围极窄：仅用于**新环境/harness**、**新目录**、**转交同事**，或**阶段中途**分叉支线任务。它换来的是可移植性。
- **Subagent**：把边界严格的任务丢进独立窗口执行并拿回报告。
- **`/compact`**：压缩当前上下文并以此开启新会话。它是**兜底默认项**，位于决策树底端而非第一反应。

阅读 [PHASE-BOUNDARIES.md](PHASE-BOUNDARIES.md) 查看有序决策树：五个问题、每条分支的推理逻辑，以及为什么一手来源的损耗决定了应最先排除 **Continue**。只在边界处做此决策；阶段中途请继续或将剩余部分拆给 subagent。

## 独立技能（Standalone）

完全独立于主干流之外的技能。

- **`/grill-me`**：与 `/grill-with-docs` 同款毫不留情的追问，但是**无状态**的——不在本地保存任何内容，也不构建 `GLOSSARY.md`。当你**不在工作目录中作业**时使用（打磨计划、设计、文章或任何没有仓库托底的事情）。只要你在工作目录里，请改用 `/grill-with-docs`：它跑同样的追问且能留下文档轨迹，因此严格更优。
- **`/grilling`**：追问原语本身——按轮次推进、计算前沿、Agent 负责查事实、你负责拍板决策。`/grill-me` 和 `/grill-with-docs` 是它的两个具名入口，而 `/triage`、`/wayfinder` 和 `/improve-codebase-architecture` 内部也都调用它。仅当你想要不带任何外层包装的纯粹追问时才直接调用它。
- **`/prototype`**：一个小型、一次性的程序，用来回答单个设计问题：这个状态模型顺手吗？这个 UI 应该长什么样？“一次性”是对代码写法的约束，而不是非要销毁它的承诺：验证出来的答案会吸收到正式代码里，原型本身则作为**一手来源**保留在基于 main 拉出的 `prototype/<name>` 分支上，并由实现工单链接引用。它是主干流第 2 步的绕道方案，但只要某个设计问题在纸面上很难拍板，随时都可以用它。
- **`/research`**：把繁重的阅读跑腿活委托给**后台 Agent**：它对照**一手来源**调研问题，并在仓库里留下一份带引用的 Markdown 文件。它读书时你可以继续干别的。它产出的文件适合带入主干流的 `/grill-with-docs`，因为调研服务于思考，而非替代思考。
- **`/to-questionnaire`**：当卡住你的信息既不在你脑子里也不在代码库里、而是在**别人脑子里**时使用，它会帮你写一份发给对方填写的问卷。它是 `/grill-me` 的逆操作：不追问你主题细节，而是追问**发送本身**（发给谁、需要对方回什么），问题直指信息差。收到的回复可继续喂给 `/grill-with-docs` 或 `/to-spec`。
- **`/wizard`**：专用于只有**人类**才能完成的步骤：开通基础设施、配置凭据或 CI 密钥、在陌生的第三方控制台点选配置、执行一次性迁移或割接。它会生成一个交互式 bash 脚本，依次打开每个 URL、采集每个值并写入 `.env` 和 GitHub secrets，免得每次都要重新向 Agent 解释一遍操作步骤。它是模型自动触发的，Agent 一旦撞上只有你能越过的墙就会调起它。只要 Agent 自己能做到的就让它自己做；这只留给真正需要人在回路里的环节。
- **`/wait-what`**：当某条消息没讲明白时的纠偏工具。在对话中途、任何其他技能内部都可以调用，Agent 会补上你缺失的上下文，用简明技术语言并严格使用 `GLOSSARY.md` 词汇重新讲一遍。它是事后补救；`/grill-with-docs` 才是事前根治，因为早早对齐共同语言就能从源头消灭黑话。
- **`/teach`**：把当前目录当作有状态的工作区，跨多次会话系统学习一个概念。
- **`/writing-for-agents`**：编写供 Agent 消费的文档（技能、`AGENTS.md`、指针指向的文档）的权威参考。

## 前置条件

**`/setup-matt-pocock-skills`**：在首次运行工程流之前先跑一次，用来配置其他技能所依赖的 issue tracker、分诊标签体系和文档布局。也支持自定义 issue tracker。
{%- endif %}
