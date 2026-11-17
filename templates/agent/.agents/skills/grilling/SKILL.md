---
name: grilling
description: >-
{%- if options["skill_lang"] == "en" %}
  Grill the user relentlessly about a plan, decision, or idea. Use when the user wants to stress-test their thinking, or uses any 'grill' trigger phrases.
{%- else %}
  围绕某个计划、决策或想法对用户进行持续追问。当用户希望压力测试自己的思考或使用任何 'grill' 触发短语时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
Interview the user relentlessly until you reach a shared understanding. Map this as a **design tree**: every decision branches into the decisions that hang off it.

Work the tree in **rounds**. The **frontier** is every decision whose prerequisites are already settled: the questions you can ask _now_ without guessing at answers you haven't heard yet. Ask the whole frontier in one round: number each question and give your recommended answer. Then wait for the user's answers before the next round.

Format a round like so:

```
❓ **Q1** - **<question title>**: <question body, might be multiple paragraphs, including multiple choices>

➡️ <your recommended answer>

---

❓ **Q2** - **<question title>**: <question body, might be multiple paragraphs, including multiple choices>

➡️ <your recommended answer>
```

Word each question so "yes" accepts your recommended answer.

Each round the user answers reshapes the tree: settled decisions push the frontier outward and unblock questions that depended on them. Recompute the frontier and ask the next round. A question whose answer depends on another question still open in this round belongs to a _later_ round, not this one.

Finding _facts_ is your job, never the user's. When a frontier question needs a fact from the environment (filesystem, tools, etc.), dispatch a sub-agent with a fresh context window to find it; don't ask the user for anything you could look up yourself. Don't block on it: a running exploration is an unsettled prerequisite, so only the questions downstream of it wait for the sub-agent to report; ask the rest of the frontier now. The _decisions_ are the user's: put each to them and wait.

The session is done when the frontier is empty: every branch of the design tree visited, nothing left silently assumed. Do not act on it until the user confirms you have reached a shared understanding.
{%- else -%}
对用户进行持续追问，直至双方达成共识。将其映射为一棵**设计树（design tree）**：每个决策都会分支出依赖于它的后续决策。

按**轮次（rounds）**推进这棵树。**前沿（frontier）**是指前提条件已经确定、无需猜测未听取答案即可*立即*提问的所有决策。在一轮中一次性提出整个前沿的问题：为每个问题编号，并给出你的推荐答案。然后等待用户回答，再进入下一轮。

每轮格式如下：

```
❓ **Q1** - **<问题标题>**：<问题正文，可包含多段以及多个选项>

➡️ <你的推荐答案>

---

❓ **Q2** - **<问题标题>**：<问题正文，可包含多段以及多个选项>

➡️ <你的推荐答案>
```

措辞上尽量让用户只需回复“是/同意”即可采纳你的推荐答案。

用户每回答一轮，设计树都会随之重塑：已落定的决策会将前沿向外推进，解锁原本依赖于它的后续问题。重新计算前沿并提出下一轮问题。凡是答案依赖于本轮尚待解决的其他问题的问题，都应留到*后续*轮次，不要放在本轮。

查明*事实*是你的职责，而不是用户的。当某个前沿问题需要从环境（文件系统、工具等）中获取事实时，派发一个全新上下文窗口的 sub-agent 去查找；不要向用户询问你自己能查到的信息。同时不要阻塞：正在进行的探索属于未定前提，只有依赖该结果的下游问题才需等待 sub-agent 返回，其余前沿问题立即在本轮提出。*决策*属于用户：把每个决策交由用户拍板并等待回复。

当前沿为空时，会话结束：设计树上的每条分支都已遍历，没有任何内容被悄然假设。在用户确认双方已达成共识之前，不要擅自行动。
{%- endif %}
