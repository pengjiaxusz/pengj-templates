---
name: wayfinder
description: >-
{%- if options["skill_lang"] == "en" %}
  Plan a huge chunk of work (more than one agent session can hold) as a shared map of decision tickets on your issue tracker, and resolve them one at a time until the way to the destination is clear.
{%- else %}
  将庞大复杂的工作（单次 Agent 会话装不下）规划为 issue tracker 上的一张决策工单共享地图，每次解决一个，直到通往终点的路径清晰为止。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
A loose idea has arrived, too big for one agent session, and wrapped in fog: the way from here to the **destination** isn't visible yet. Wayfinding is about finding that way, not charging at the destination. This skill charts the way as a **shared map** on the repo's issue tracker, then works its **decision tickets** (questions whose resolution is a decision, not slices of a build to execute) one at a time until the route is clear.

The destination varies per effort, and naming it is the first act of charting: it shapes every ticket. It might be a spec to hand off and iterate on, a decision to lock before planning starts, or a change made in place like a data-structure migration. The map is domain-agnostic: engineering work, course content, whatever fits the shape.

## Plan, don't do

Wayfinder is **planning** by default: each ticket resolves a decision, and the map is done when the way is clear, with nothing left to decide before someone goes and does the thing. The pull to just do the work is usually the signal you've reached the edge of the map and it's time to hand off. An effort can override this in its **Notes**, carrying execution into the map itself, but absent that, produce decisions, not deliverables.

## Refer by name

Every map and ticket is an issue, so it has a **name**: its title. In everything the human reads (narration, the map's Decisions-so-far), refer to it by that name, never by a bare id, number, or slug. A wall of `#42, #43, #44` is illegible; names read at a glance. The id and URL don't vanish; a name wraps its link, but they ride _inside_ the name, never stand in for it.

## The Map

The map is a single issue on this repo's issue tracker, labelled `wayfinder:map`, the canonical artifact. Its tickets are child issues of the map.

The map is an **index**, not a store. It lists the decisions made and points at the tickets that hold their detail; a decision lives in exactly one place, its ticket, so the map never restates it, only gists it and links.

**Where the map, its child tickets, blocking, and frontier queries physically live is tracker-specific.** The issue tracker should have been provided to you. If not, tell the user to run `/setup-matt-pocock-skills`. Consult the tracker doc's "Wayfinding operations" section for how _this_ repo expresses them. If no tracker has been provided, default to the local-markdown tracker.

### The map body

The whole map at low resolution, loaded once per session. Open tickets are **not** listed: they are open child issues, found by query.

```markdown
## Destination

<what reaching the end of this map looks like: the spec, decision, or change this effort is finding its way to. One or two lines; every session orients to it before choosing a ticket.>

## Notes

<domain; skills every session should consult; standing preferences for this effort>

## Decisions so far

<!-- the index: one line per closed ticket, enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [<closed ticket title>](link): <one-line gist of the answer>

## Not yet specified

<!-- see "Fog of war": in-scope fog you can't ticket yet; graduates as the frontier advances -->

## Out of scope

<!-- see "Out of scope": work ruled beyond the destination; closed, never graduates -->
```

### Tickets

Each ticket is a **child issue** of the map; the tracker's issue id is its identity. Its body is the question, sized to one 100K token agent session:

```markdown
## Question

<the decision or investigation this ticket resolves>
```

Each ticket carries a `wayfinder:<type>` label, one of `research`, `prototype`, `grilling`, `task` (see [Ticket Types](#ticket-types)). `wayfinder:` labels are the only labels a map and its tickets carry, never a triage label like `ready-for-agent`: they are decisions, not implementation work.

A session **claims** a ticket by assigning it to the dev driving the map, **first**, before any work, so concurrent sessions skip it. That assignee _is_ the claim: an open, unassigned ticket is unclaimed.

Blocking uses the tracker's **native** dependency relationship: essential because it renders the frontier _visually_ in the tracker's own UI, so the human sees what's takeable without opening the map. Only a tracker that lacks native blocking falls back to a body convention. A ticket is **unblocked** when every ticket blocking it is closed; the **frontier** is the open, unblocked, unclaimed children, the edge of the known.

The answer isn't part of the body; it's recorded on resolution (see [Work through the map](#work-through-the-map)). Assets created while resolving a ticket are linked from the issue, not pasted in.

## Ticket Types

Every ticket is either **HITL** (human in the loop, worked _with_ a human who speaks for themselves) or **AFK**, driven by the agent alone. A HITL ticket only resolves through that live exchange; the agent never stands in for the human's side of it (a grilling agent that answers its own questions has broken this).

- **Research** (AFK): Reading documentation, third-party APIs, or local resources like knowledge bases to surface a fact a decision waits on. Resolved by a subagent that calls the Skill tool with "research". Use when knowledge outside the current working directory is required.
- **Prototype** (HITL): Raise the fidelity of the discussion by making a cheap, rough, concrete artifact to react to (an outline, a rough take, a stub, or UI/logic code) by calling the Skill tool with "prototype". Links the prototype as an asset. Use when "how should it look" or "how should it behave" is the key question.
- **Grilling** (HITL): Conversation. The default case. Always call the Skill tool twice, for "grilling" and "domain-modeling".
- **Task** (HITL or AFK): Manual work that must happen before a _decision_ can be made: nothing to decide, prototype, or research, but the discussion is blocked until it's done. Signing up for a service so its API can be judged, provisioning access, moving data so its shape can be seen. This is the one type that _does_ rather than decides, and it earns its place by unblocking a decision, not by delivering the destination. The agent drives it alone where it can (AFK); otherwise it hands the human a precise checklist (HITL). Resolved when the work is done; the answer records what was done and any resulting facts (credentials location, new URLs, row counts) later tickets depend on.

## Fog of war

The map is _deliberately_ incomplete: don't chart what you can't yet see. Beyond the live tickets lies the **fog of war**: the dim view of decisions and investigations you can tell are coming but can't yet pin down, because they hang on questions still open. Resolving a ticket clears the fog ahead of it, graduating whatever's now specifiable into fresh tickets, one at a time, until the way to the destination is clear and no tickets remain.

The map's **Not yet specified** section is where that dim view is written down: the suspected question, the area to revisit later. It's the undiscovered frontier _toward_ the destination: everything here is in scope, just not sharp enough to ticket. Write as loosely or as fully as the view allows; it doubles as a signpost for collaborators reading where the effort is headed.

**Fog or ticket?** The test is whether you can state the question precisely now, _not_ whether you can answer it now.

- **Ticket when** the question is already sharp, even if it's blocked and you can't act on it yet.
- **Not yet specified when** you can't yet phrase it that sharply. Don't pre-slice the fog into ticket-sized pieces: it's coarser than a ticket, and one patch may graduate into several tickets, or none, once the frontier reaches it.

**Not yet specified** excludes what's already decided (Decisions so far), what's already a live ticket, and what's out of scope (the next section).

## Out of scope

Fog only ever gathers _toward_ the destination. The destination fixes the scope, so work beyond it is **out of scope**: it isn't fog, and it doesn't belong in **Not yet specified**. It gets its own **Out of scope** section on the map: work you've consciously ruled out of _this_ effort. Scope, not sharpness, lands it here.

Out-of-scope work never graduates (the frontier stops at the destination), so it returns only if the destination is redrawn, and then as a fresh effort, not a resumption.

Ruling something out of scope is a scoping act, not a step on the route. When a ticket that already exists turns out to sit past the destination (mis-scoped in while charting, or exposed by a resolution), **close it** (a closed ticket is unambiguously off the frontier) and leave one line in the **Out of scope** section: the gist plus why it's out of scope, linking the closed ticket. It stays out of **Decisions so far**, which records the route actually walked; a scope boundary isn't a step on it.

## Invocation

Two modes. Either way, **never resolve more than one ticket per session**, with the exception of research tickets.

### Chart the map

User invokes with a loose idea.

1. **Name the destination.** Call the Skill tool twice, for "grilling" and "domain-modeling", to pin down what this map is finding its way to: the spec, decision, or change. The destination fixes the scope, so it's settled first.
2. **Map the frontier.** Grill again, **breadth-first** this time: fan out across the whole space rather than deep on any one thread, surfacing the open decisions and the first steps takeable now. **If this surfaces no fog** (the way to the destination is already clear, the whole journey small enough for one session), you don't need a map. Stop and ask the user how they'd like to proceed.
3. **Create the map** (label `wayfinder:map`): Destination and Notes filled in, Decisions-so-far empty, the fog sketched into **Not yet specified**.
4. **Create the tickets you can specify now** as child issues of the map, then wire blocking edges in a **second pass** (issues need ids before they can reference each other). Write cross-references in that pass too, with real ids: a placeholder `#<n>` auto-links to an unrelated issue. Wiring sorts them into the frontier and the blocked; everything you can't yet specify stays in the fog: the **Not yet specified** section.
5. **Fire the research subagents.** For each `research` ticket you just created, spin up a subagent with a fresh context window that calls the Skill tool with "research" to resolve it in parallel, capturing its findings on a throwaway `research/<name>` branch with a context pointer from the ticket. Push the branch but open no PR: it is never merged.
6. Stop: charting is one session's work; it hand-resolves nothing.

### Work through the map

User invokes with a map (URL or number). A ticket is **optional**: without one, you pick the next decision, not the user.

1. Load the **map**: the low-res view, not every ticket body.
2. Choose the ticket. If the user named one, use it. Otherwise take the first frontier ticket in order. **Claim it**: assign it to yourself before any work.
3. Resolve it as the type its `wayfinder:<type>` label names (see [Ticket Types](#ticket-types)). Read the label, not just the body: the body never states the type. **Zoom as needed**: fetch the full body of any related or closed ticket on demand; call the Skill tool for whichever skills the `## Notes` block names. If in doubt, call the Skill tool twice, for "grilling" and "domain-modeling".
4. Record the resolution: post the answer as a **resolution comment**, **close** the issue, and **append a context pointer** to the map's Decisions-so-far.
5. Add newly-surfaced tickets (create-then-wire); graduate any fog the answer has made specifiable, clearing each graduated patch from **Not yet specified** so it lives only as its new ticket. If the answer reveals that a ticket (this one or another) sits beyond the destination, **rule it out of scope** rather than resolving it on the route. If the decision invalidates other parts of the map, update or delete those tickets.

The user may run unblocked tickets in parallel, so expect other sessions to be editing the tracker concurrently.
{%- else -%}
一个庞大的想法摆在眼前：大到单次 Agent 会话装不下，而且笼罩在迷雾中——从这里通往**终点（destination）**的路还看不清。Wayfinding（寻路）的任务就是找到那条路，而不是盲目朝终点猛冲。本技能在仓库的 issue tracker 上绘制一张**共享地图（shared map）**，然后每次推进一个**决策工单（decision tickets）**（解决后产出的是一个决策，而不是待执行的代码切片），直到路线彻底清晰。

不同项目的终点各不相同，而命名终点正是制图的第一步：它决定了每一张工单的形状。终点可能是一份待交接迭代的 spec、一个在动工规划前必须锁定的架构决策，或是一次就地进行的改动（如数据结构迁移）。地图与领域无关：工程开发、课程内容，凡是符合这种形态的都适用。

## 只做规划，不直接施工

Wayfinder 默认是**规划**：每张工单解决一个决策；当道路彻底清晰、在真正动工前已没有任何待拍板事项时，地图即告完成。当你觉得“真想直接把活干了”时，这通常正说明你已经走到了地图边缘，该交接出去了。某项工作也可以在 **Notes** 里显式覆盖这一规则、把执行任务纳入地图本身；但除非有此覆盖，否则产出的是**决策，而不是交付物**。

## 用名称指代，不要用裸编号

每张地图和每张工单都是一个 issue，因此都有**名称（name）**：它的标题。在一切给人看的内容里（过程叙述、地图的 Decisions-so-far），一律用名称指代，绝不用裸 ID、裸编号或 slug。满屏 `#42, #43, #44` 根本无法阅读；名称一眼就能看懂。ID 和 URL 并没有消失——名称本身包着链接，但它们藏在名称*里面*，绝不替代名称。

## 地图（The Map）

地图是仓库 issue tracker 上带 `wayfinder:map` 标签的单个 issue，是权威主产物；它的工单则是该地图的子 issue。

地图是**索引**，不是仓库。它列出已做出的决策并指向承载细节的工单；每项决策只活在一个地方——它自己的工单里，所以地图绝不复述细节，只写一句话摘要并附上链接。

**地图、子工单、阻塞关系以及前沿查询在物理上如何存放，取决于具体的 tracker。** issue tracker 的配置应当已经提供给你；如果没有，请提示用户先运行 `/setup-matt-pocock-skills`。查阅对应 tracker 文档中的“Wayfinding 操作”一节以了解本仓库的表达方式；若未配置 tracker，默认使用本地 Markdown tracker。

### 地图正文

整张地图的低分辨率总览，每次会话加载一次。未关闭的工单**不要**手写列在正文里：它们是 open 状态的子 issue，通过查询动态获取。

```markdown
## Destination

<走到这张地图尽头时是什么样子：本次寻路要抵达的 spec、决策或变更。一两行即可；每次会话在挑工单前先对齐它。>

## Notes

<所属领域；每次会话都应查阅的技能；本次工作的常驻偏好>

## Decisions so far

<!-- 索引：每张已关闭工单占一行，足够判断相关性即可，需要细节时点进链接看原工单 -->

- [<已关闭工单标题>](link): <一句话答案摘要>

## Not yet specified

<!-- 见「战争迷雾」：在范围内但尚无法拆成工单的模糊区域；随前沿推进逐步毕业 -->

## Out of scope

<!-- 见「范围之外」：明确划出终点之外的工作；直接关闭，永不毕业 -->
```

### 工单（Tickets）

每张工单都是地图的**子 issue**，以 tracker 的 issue ID 作为身份标识。正文就是待回答的问题，体量控制在单次 100K token Agent 会话可完成之内：

```markdown
## Question

<本工单要解决的决策或调研问题>
```

每张工单携带一个 `wayfinder:<type>` 标签，取自 `research`、`prototype`、`grilling`、`task` 四者之一（见[工单类型](#工单类型ticket-types)）。`wayfinder:` 标签是地图及其工单唯一使用的标签，绝不贴 `ready-for-agent` 这类分诊标签：它们是决策，不是实现工单。

会话在动工**之前**必须**先认领（claim）**工单——把它指派给当前推动地图的开发者，这样并发会话就会跳过它。指派人本身*就是*认领标记：open 且未指派的工单即为未认领。

阻塞关系使用 tracker 的**原生**依赖关系：这至关重要，因为它能在 tracker 自己的 UI 里*直观*渲染出前沿，让人不点开地图就知道哪些工单可领。只有缺乏原生阻塞功能的 tracker 才退回到正文约定。当阻塞某张工单的所有工单都已关闭时，该工单即**解除阻塞**；**前沿（frontier）**就是所有 open、已解除阻塞且未认领的子工单——已知世界的边缘。

答案不写在工单原本的正文里，而是在解决时记录下来（见[推进地图](#推进地图work-through-the-map)）。解决工单过程中产出的资产从 issue 链接过去，不要整段贴进正文。

## 工单类型（Ticket Types）

每张工单要么是 **HITL**（人在回路，必须与人类实时协作，由人类亲自表达意愿），要么是 **AFK**（由 Agent 独立完成）。HITL 工单只能通过真实互动来解决，Agent 绝不能替人类代答（一个自问自答的 grilling Agent 就彻底违背了这条规则）。

- **Research**（AFK）：阅读文档、第三方 API 或本地知识库等资源，查明某个决策等待依赖的事实。由拉起的 subagent 调用 Skill 工具执行 `"research"` 解决。当需要当前工作目录之外的知识时使用。
- **Prototype**（HITL）：调用 Skill 工具执行 `"prototype"`，做出一个低成本、粗糙但具体的可交互产物（大纲、粗稿、桩代码或 UI/逻辑原型），提高讨论的保真度，并将原型作为资产链接到工单。当核心问题是“它该长什么样”或“它表现起来如何”时使用。
- **Grilling**（HITL）：对话追问，最常见的默认类型。务必先后调用两次 Skill 工具，分别执行 `"grilling"` 与 `"domain-modeling"`。
- **Task**（HITL 或 AFK）：在做出*决策*之前必须先完成的手工前置事务——本身没什么可决策、做原型或调研的，但这件事不干完讨论就卡住（例如注册服务以评估其 API、开通权限、搬运数据以观察其形状）。这是唯一一种“动手做事”而非“做决策”的类型，它存在的正当性在于*解锁某项决策*，而不是交付最终产物。Agent 能自己干的就独立完成（AFK）；否则给人类一份精确的检查清单（HITL）。事情做完即告解决，答案中记录做了什么以及后续工单依赖的产出事实（凭据位置、新 URL、行数等）。

## 战争迷雾（Fog of war）

地图是*刻意*留白的：不要去规划你现在还看不清的东西。在活跃工单之外是**战争迷雾**——你隐约知道后面还会有决策和调研，但现在还说不准具体是什么，因为它们取决于眼前还没敲定的问题。每解决一张工单，就会吹散它前方的迷雾，把现在已经能说清楚的部分逐步毕业成新工单，一次一张，直到通往终点的路径彻底明朗、不再有剩余工单。

地图的 **Not yet specified** 小节就是记下这片模糊地带的地方：隐约察觉的问题、日后需要回访的区域。它是朝向终点的未探索前沿——这里的一切都在范围内，只是还不够锋利到能写成工单。能写多粗写多粗、能写多细写多细，它同时也为协作者指明整个工作的大致走向。

**进迷雾还是建工单？** 检验标准在于你现在能不能**精确表述这个问题**，*而不在于*你现在能不能回答它：

- **建工单**：问题本身已经足够锋利明确，哪怕它目前被阻塞、还不能立刻动手。
- **留进 Not yet specified**：问题还无法表述到那么锋利。不要提前把迷雾硬切成工单大小的碎片：迷雾的颗粒度比工单粗，等前沿推进到这里时，一片迷雾可能会毕业成好几张工单，也可能一张都不用建。

**Not yet specified** 不包含已决事项（Decisions so far）、已建出的活跃工单，以及范围之外的事项（下一节）。

## 范围之外（Out of scope）

迷雾永远只聚拢在*通往终点*的方向上。终点划定了范围，因此超出终点的事情就是**范围之外**：它不是迷雾，也不属于 **Not yet specified**。它在地图上有自己专属的 **Out of scope** 小节：你有意排除在*本次*工作之外的内容。决定它进这里的不是清晰度，而是范围。

范围之外的工作永远不会毕业（前沿止步于终点），只有当终点被重新划定时它才会回来，而且是以一次全新工作的形式，而不是续接。

把某件事划出范围是一次定界动作，不是路线上的推进步骤。如果一张已经建出来的工单后来发现其实在终点之外（制图时误划入，或某个决策后暴露出它超范围），**直接关闭它**（已关闭的工单明确不在前沿上），并在 **Out of scope** 小节留一行：一句话摘要 + 为什么超范围 + 指向该已关闭工单的链接。不要放进 **Decisions so far**——那里只记录实际走过的路线，范围边界不是路线上的节点。

## 调用方式

两种模式。无论哪种模式，**每次会话绝不解决超过一张工单**（`research` 类工单除外）。

### 绘制地图（Chart the map）

用户带着一个模糊的大想法调用。

1. **命名终点。** 先后调用两次 Skill 工具（`"grilling"` 与 `"domain-modeling"`），把这张地图究竟要通往哪里钉死：是 spec、决策还是变更。终点决定了范围，必须最先落定。
2. **测绘前沿。** 再次进行追问，这次采用**广度优先**：在整个空间横向展开，而不是在某一条线上钻太深，把所有待决决策以及眼下就能迈出的第一步浮出来。**如果这一步没有发现任何迷雾**（通往终点的路已经清晰，整件事小到一次会话就能装下），那你不需要地图——停下来询问用户接下来希望怎么推进。
3. **创建地图**（标签 `wayfinder:map`）：填好 Destination 和 Notes，Decisions-so-far 留空，把迷雾草拟进 **Not yet specified**。
4. **建出眼下就能明确表述的工单**作为地图的子 issue，然后在**第二遍**里连上阻塞边（issue 必须先有 ID 才能互相引用）。这一遍里也顺便把真实 ID 的交叉引用写上——占位符 `#<n>` 会误链接到无关 issue。连好线后自然就分出了前沿和被阻塞工单；其余尚无法明确表述的部分留在迷雾中，即 **Not yet specified** 小节。
5. **拉起 research subagent。** 为刚刚建出的每张 `research` 工单各自拉起一个全新上下文窗口的 subagent，调用 Skill 工具执行 `"research"` 并行解决它们，把结果保存在一次性 `research/<name>` 分支上，并在工单里留下上下文指针。把分支推到远端，但**不要开 PR**：它永远不会被合并。
6. 停下：制图本身就是一次会话的完整工作，制图阶段不顺手解决任何工单。

### 推进地图（Work through the map）

用户带着地图（URL 或编号）调用。具体工单是**可选参数**：没指定时，由你挑下一张决策工单，而不是问用户挑哪张。

1. 加载**地图**：看低分辨率总览，不要一口气读完所有工单正文。
2. 挑选工单。若用户指定了就用指定的；否则按顺序取前沿里的第一张。**认领它**：在动工前先指派给自己。
3. 按其 `wayfinder:<type>` 标签的类型去解决（见[工单类型](#工单类型ticket-types)）。务必看标签，不要只看正文——正文里从不写类型。**按需放大**：仅在需要时拉取相关工单或已关闭工单的完整正文；调用 `## Notes` 块中点名的技能。如有拿不准的，先后调用两次 Skill 工具（`"grilling"` 与 `"domain-modeling"`）。
4. 记录解决结果：把答案发成一条**解决评论（resolution comment）**，**关闭**该 issue，并向地图的 Decisions-so-far **追加一条上下文指针**。
5. 补建新浮现的工单（先建工单再连阻塞边）；把因为这次答案而变得清晰可拆的迷雾毕业成工单，并从 **Not yet specified** 中清掉已毕业的区块，使其只作为新工单存在。如果答案揭示某张工单（本张或另一张）已经超出终点，**将它划出范围**，不要在路线上硬解。如果该决策使地图的其他部分失效，同步更新或删除对应工单。

用户可能会并行推进互不阻塞的工单，因此要预料到同一时间可能有其他会话也在并发修改 tracker。
{%- endif %}
