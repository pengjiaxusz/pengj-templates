---
name: retro
description: >-
{%- if options["skill_lang"] == "en" %}
  Conduct a retrospective on a coding session.
{%- else %}
  对一次编码会话进行复盘（retrospective）。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
The user has asked for a **retrospective**. You are suggesting improvements to the coding agent's **environment** to improve future runs.

## Steps

1. Call the Skill tool with `writing-for-agents` for the writing style guide.

2. Read the primary sources for the session the user specifies. This may mean searching through session logs on this machine. If the user doesn't specify a session, default to the current one.

3. Look for candidates for improvement in these categories.

- **Navigation**: how easy was it for the agent to find the right files? Are there hidden dependencies between files? Would a **navigation pointer** make it easier? _Use when_ the session took a long time to find a piece of information.
- **Automated checks**: are there automated checks that could catch errors the agent made? Linting, typing, tests, filesystem linters? Read the repo's own check command first (its `package.json`/build-tool `lint`/`check` scripts, its CI workflow), so a check that already exists but sits unwired or silently broken is the finding, not a reinvention. A repo with no **guardrail** (no pre-commit hook and no CI job running its lint/typecheck/test command) is itself a finding: an un-linted repo is a standing missed opportunity, not a neutral default. _Use when_ the agent made a mistake an automated check could have caught, or the repo has no guardrail at all.
- **Coding standards**: should the **reviewer agent** be given a new rule to enforce? Should an existing rule be removed or clarified? Classify the violation first: a **mechanical** one (a fixed syntactic pattern, a banned API, an import shape, a file-location rule) gets a deterministic check, full stop: a custom rule in the repo's own linter, a new pre-commit hook, or a new CI job, whichever the repo's language and existing guardrail make cheapest. Default to building the check over writing the rule. Reserve `CODING_STANDARDS.md` for genuine **judgement calls** (cross-file consistency, "matches the surrounding style," anything no guardrail could ever substitute for). _Use when_ the reviewer agent failed to catch a mistake.
- **Global AGENTS.md**: are there any steering instructions that should be moved to coding standards (or automated checks) instead? _Use when_ the AGENTS.md file is particularly large - in the repo OR the user's global scope.
- **Tool economy**: did the agent make expensive tool calls that could be streamlined? Is there any custom tooling (CLI's, MCP's) that is particularly token-inefficient? _Use when_ the agent made an expensive tool call.
- **No-ops**: look for instructions in steering files that don't modify the agent's behavior. _Use when_ the steering files are large and unwieldy.
- **Information access**: look for opportunities to increase the agent's access to information. Teeing dev server logs, readonly access to third-party services. _Use when_ a crucial piece of information was not available to the agent.

4. Present these candidates to the user, in order of severity.

## Reference

### Implementation vs Review

Remember that all work goes through two stages: implementation and review. The implementation agent has the most **context pressure**. They are responsible for exploration, writing code, and debugging failures.

The review agent has the least context pressure - it receives a diff, so no exploration needed. It often does not need to write code or debug.

This means that the review agent should be responsible for imposing coding standards, not the implementation agent.

### Files

You have access to several files in the repo:

- `CLAUDE.md`/`AGENTS.md`: these files are pushed to the context window of any agent working in this repo. They should be used incredibly sparingly, usually only for **navigation pointers** to other files.
- `CODING_STANDARDS.md`: this file is read during review, not implementation. Add **navigation pointers** to docs folders if the standards file gets more than 1,000 lines long.
- Docs: use docs as references files, pointed to by other files. Look for existing docs before writing new ones.
- Skills: use skills for docs (since their description goes into the agent's context window), or for user-invoked commands. Follow the advice in the `writing-for-agents` skill.
{%- else -%}
用户要求进行一次**复盘（retrospective）**。你的任务是为编码 Agent 的**环境**提出改进建议，让未来的运行表现更好。

## 步骤

1. 调用 Skill 工具执行 `writing-for-agents` 获取写作风格指南。

2. 阅读用户指定会话的一手来源（可能需要检索本机的会话日志）。如果用户未指定会话，默认复盘当前会话。

3. 按以下类别寻找改进点：

- **导航（Navigation）**：Agent 找到正确文件的难度如何？文件之间是否存在隐蔽依赖？加一条**导航指针**会不会更容易？_适用时机_：会话花了很长时间才找到某份信息。
- **自动化检查（Automated checks）**：有没有能拦住 Agent 所犯错误的自动化检查？Lint、类型检查、测试、目录结构检查？先读仓库自己的检查命令（`package.json` / 构建工具里的 `lint`/`check` 脚本、CI 工作流），因为“已经存在但没接上或静默失效的检查”本身就是发现，不要重复造轮子。一个没有任何**护栏**的仓库（没有 pre-commit hook 也没有跑 lint/typecheck/test 的 CI 任务）本身就是重大发现：没有 lint 的仓库是持续错失的机会，而不是中性默认状态。_适用时机_：Agent 犯了自动化检查本可拦住的错误，或仓库根本没有护栏。
- **编码规范（Coding standards）**：是否应给 **reviewer agent** 增加一条新规则？或删掉/澄清某条已有规则？先对违规性质分类：**机械性**违规（固定语法模式、禁用 API、import 形态、文件存放位置规则）一律做成确定性检查——写入仓库自己的 linter 自定义规则、新增 pre-commit hook 或 CI 任务，按仓库语言与现有护栏成本最低的方式选；优先做自动化检查而非只写文字规则。把 `CODING_STANDARDS.md` 留给真正的**主观判断**（跨文件一致性、“与周边风格协调”等任何工具都无法替代的规范）。_适用时机_：reviewer agent 漏掉了某类错误。
- **全局 AGENTS.md**：是否有本该下沉到编码规范（或自动化检查）的引导指令？_适用时机_：仓库内或用户全局的 `AGENTS.md` 过于庞大。
- **工具经济性（Tool economy）**：Agent 是否发起了本可精简的高开销工具调用？有没有哪些自定义工具（CLI、MCP）特别费 token？_适用时机_：Agent 发起了昂贵的工具调用。
- **空操作（No-ops）**：检查引导文件里是否存在根本不改变 Agent 行为的废话指令。_适用时机_：引导文件冗长难维护。
- **信息可达性（Information access）**：寻找提升 Agent 获取信息能力的机会，例如把开发服务器日志 tee 到文件、开放第三方服务的只读访问等。_适用时机_：某项关键信息当时对 Agent 不可见。

4. 按严重程度从高到低向用户呈现这些改进建议。

## 参考

### 实现 vs 评审

记住所有工作都经过两个阶段：实现与评审。实现 Agent 承受最大的**上下文压力**，它要负责探索、写代码和排查失败。

评审 Agent 承受的上下文压力最小——它直接拿到 diff，不需要探索，通常也不需要写代码或调试。

这意味着应当由评审 Agent 来把关编码规范，而不是全压在实现 Agent 身上。

### 文件分工

仓库里通常有以下几类文件：

- `CLAUDE.md` / `AGENTS.md`：只要有 Agent 在仓库工作，这些文件就会被塞进上下文窗口。因此必须极其克制地使用，通常只放指向其他文件的**导航指针**。
- `CODING_STANDARDS.md`：在评审阶段读取，不在实现阶段读取。如果规范文件超过 1000 行，改为放指向文档目录的**导航指针**。
- Docs：把文档作为被其他文件指针引用的参考资料。写新文档前先找有没有现成的。
- Skills：用技能承载文档（因为它们的 description 会进入上下文窗口）或用户触发命令。写作规范遵循 `writing-for-agents` 技能。
{%- endif %}
