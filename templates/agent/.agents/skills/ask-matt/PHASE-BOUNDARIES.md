{% if options["skill_lang"] == "en" -%}
# Phase boundaries

A **phase** is a chunk of work inside a session: the grilling, the implementation, the QA. The definition is fuzzy on purpose: a phase ends when you think *"ok, we're done with that"*.

The **phase boundary** is the gap between two phases, and it is the only place this decision belongs. Mid-phase there is no decision to make: continue, or split the work that's left into subagents. Compacting mid-phase makes the agent lose the thread.

## The five options

| Option       | What it does                                                    |
| ------------ | --------------------------------------------------------------- |
| **Continue** | Stay in the session. No context switch at all.                    |
| **`/clear`** | Empty the context window and start from nothing.                  |
| **`/handoff`** | Write a portable markdown file and seed a session anywhere with it. |
| **Subagent** | Send the task to its own context window and get a report back.     |
| **`/compact`** | Compress this context and seed a fresh session with the summary.  |

## The tree

Work top to bottom at the boundary. The first **yes** wins.

**1. Can you continue in this session?** Two things make the answer yes: the next phase needs this phase as a **primary source**, or you have enough [smart zone](https://www.aihero.dev/ai-coding-dictionary/smart-zone) left (~150k tokens) for the next phase to fit. Grilling → implementation is the standard yes: the implementation wants the reasoning verbatim, not a summary of it. Continue costs nothing and loses nothing, so rule it out before anything else.

**2. Is the context irrelevant to what comes next?** Is everything in this session (the exploration, the decisions, the dead ends) disposable? If so, **`/clear`**. It is the cheapest move on the board: it takes no time and hands back the whole window. `/clear` also isn't terminal: the old session stays resumable.

The cost of getting this wrong is one-way. Clear a *relevant* context and you lose the **why** behind what you built, and no amount of reading the diff back gets it returned.

**3. Do you need to hand off?** `/handoff` is narrow. You need it only when you are:

- swapping to a **new harness** (Claude → Codex),
- moving to a **new directory** or repo,
- sending the work to a **colleague**,
- or forking a side task you found **mid-phase** without derailing what you're doing.

That list is the whole clause. What `/handoff` buys is **portability**: a file that travels. If nothing is travelling, you don't need it.

**4. Can the task be done AFK?** Is it scoped tightly enough to run with you away from the keyboard, no steering? Then send it to a **subagent** and leave this session untouched. Automated review is the standard case: the agent reads the diff and reports, and you aren't needed while it does.

**5. Otherwise, `/compact`.** Relevant context, same harness, same directory, and you need to stay in the loop: this is where the tree lands, and it lands here often. Pass it an instruction (`/compact we're going to QA this area`) so the summary keeps what the next phase needs.

`/compact` is the **default, not the first reach**. It sits at the bottom because the four questions above it are all cheaper or more precise. The failure mode when people start here is a fresh session that is confidently wrong about a decision the summary flattened.

## Primary and secondary sources

Every move except **Continue** turns a **primary source** into a **secondary source**: the session as it happened, replaced by a summary of it. The trade is always the same shape:

| Source                            | Information | Noise | Room to move |
| --------------------------------- | ----------- | ----- | ------------ |
| Primary (Continue)                | Full        | Lots  | Little       |
| Secondary (`/compact`, `/handoff`) | Lossy       | Less  | Lots         |

This is why question 1 comes first. You only pay the lossiness when staying costs more than it saves.

## These are judgement calls

The questions are not objective: each has taste in it, and the same boundary can go two ways on two days. The value is in asking them **in order**, at the boundary rather than in the middle of the work.
{%- else -%}
# 阶段边界（Phase boundaries）

一个**阶段（phase）**是指会话中的一块工作单元：需求追问（grilling）、功能实现（implementation）、质量验收（QA）。这个定义刻意保持模糊：当你心里觉得*“好，这部分搞定了”*时，一个阶段就结束了。

**阶段边界（phase boundary）**是两个阶段之间的衔接点，也是唯一需要做上下文切换决策的时刻。在阶段中途没有决策可做：要么继续做，要么把剩下的任务拆给 sub-agent。在阶段中途执行 compact 会让 Agent 丢失上下文主线。

## 五个选项

| 选项 | 作用 |
| :--- | :--- |
| **Continue（继续）** | 留在当前会话。完全不切换上下文。 |
| **`/clear`** | 清空上下文窗口，从零开始。 |
| **`/handoff`** | 写出一份可移植的 Markdown 交接文件，并用它开启任意新会话。 |
| **Subagent** | 把任务发到独立的上下文窗口执行，并收回一份汇报。 |
| **`/compact`** | 压缩当前上下文，并用摘要开启一个新会话。 |

## 决策树

在阶段边界从上到下依次判断。第一个回答为**是**的选项胜出。

**1. 能在当前会话直接继续吗？** 满足以下两条之一即可选是：下一阶段需要把当前阶段作为**一手来源（primary source）**，或者你还有足够的 [smart zone](https://www.aihero.dev/ai-coding-dictionary/smart-zone) 余量（约 150k token）来容纳下一阶段。Grilling → 实现就是典型的“是”：实现阶段需要保留原始推理过程，而不是压缩后的摘要。继续执行零成本、零信息损失，因此必须最先排除它。

**2. 当前上下文对接下来要做的事完全无关吗？** 本轮会话里的所有内容（探索过程、决策、走过的弯路）都可以丢弃吗？如果是，用 **`/clear`**。这是成本最低的操作：不花时间，直接归还整个上下文窗口。而且 `/clear` 并非不可逆：旧会话依然可以恢复。

这里判断失误的代价是单向的。清空了*相关*上下文，就会丢失代码背后的**为什么**，事后看再多 diff 也找不回来。

**3. 需要跨边界交接吗？** `/handoff` 的适用面很窄。只有当你准备做以下事情之一时才需要它：

- 切换到**新的工具/运行环境（harness）**（Claude → Codex），
- 移动到**新的目录**或仓库，
- 把工作转交给**同事**，
- 或在**阶段中途**分叉出一个支线任务，同时不想打乱当前主线。

这张清单就是全部适用范围。`/handoff` 买来的是**可移植性（portability）**：一份可以随处携带的文件。如果工作不需要迁移，就不需要它。

**4. 这项任务能离线（AFK）完成吗？** 它的范围是否足够收敛，可以让你离开键盘、无需中途纠偏就跑完？如果是，把它交给 **subagent**，让当前会话保持干净。自动化代码评审就是典型场景：Agent 读完 diff 并汇报结果，期间不需要你介入。

**5. 否则，用 `/compact`。** 上下文相关、同一个工具、同一个目录，且你需要继续留在回路里：决策树最终往往会落在这里，而且很常见。记得给它附带一条指令（例如 `/compact 接下来我们要对这一块做 QA`），让摘要保留下一阶段需要的信息。

`/compact` 是**兜底默认项，而不是首选**。它排在最末，是因为前面四个问题要么更便宜、要么更精确。人们从这里直接起手时常见的翻车模式是：新会话带着一份把关键决策压扁了的摘要，极其自信地往错的方向跑。

## 一手来源与二手来源

除了 **Continue** 之外，其他所有操作都会把**一手来源（primary source）**变成**二手来源（secondary source）**：原本真实发生过的完整会话，被替换成一份摘要。权衡结构永远是相同的：

| 来源 | 信息完整度 | 噪音 | 剩余空间 |
| :--- | :--- | :--- | :--- |
| 一手（Continue） | 完整 | 较多 | 较小 |
| 二手（`/compact`、`/handoff`） | 有损 | 较少 | 较大 |

这就是为什么第 1 题排在最前面。只有当继续留在原会话的代价高于收益时，你才去支付信息有损的成本。

## 这些都是主观判断

这些问题没有纯客观的标准答案：每一条都带有工程品味，同一道边界在不同日子里也可能走向不同的分支。它的价值在于在阶段边界**按顺序**发问，而不是在干活干到一半时临时起意。
{%- endif %}
