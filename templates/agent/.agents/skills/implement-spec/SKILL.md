---
name: implement-spec
description: >-
{%- if options["skill_lang"] == "en" %}
  Implement the result of /to-spec and /to-tickets in code.
{%- else %}
  将 /to-spec 与 /to-tickets 的产出完整实现为代码。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
You have been provided a spec. This spec should have tickets associated with it, describing how to implement the spec.

The issue tracker should have been provided to you. If not, tell the user to run `/setup-matt-pocock-skills`.

The goal is the entire spec implemented on a single **integration branch**, with every ticket resolved the way the issue tracker closes work.

The tickets are not a list of steps. They are a **task graph** with blocking relationships between them. This means there is always a **frontier** of tickets which are ready to be grabbed.

Start every subagent with a fresh context window.

Communication to and from subagents should be sparse. Communicate primarily through **context pointers**: to the spec, tickets, research notes, and previous commits. Don't duplicate information already available via pointers.

**Implementer subagents** should be run in the background where possible for maximum concurrency.

## Steps

1. Read the spec and tickets to understand the task graph.

2. (optional) Use an **exploration subagent** to conduct any exploration required by the tickets - relevant codebase files or external documentation. Ensure the exploration subagent can save files - it should save its markdown notes in a directory outside the repo, accessible by all future subagents. This lets **implementer subagents** focus on implementation rather than exploration.

3. Create the integration branch. If the issue tracker closes work through PRs, or the user asks for one, open a draft PR after the first merge in step 5 (a branch with no commits ahead of main can't open one), marked as closing the spec and tickets.

4. Use a fresh **implementer subagent** for each ticket, each in its own worktree on its own branch. Each implementer subagent:
   - confirms its worktree is based on the integration branch before starting, and resets onto it if not;
   - calls the Skill tool with `tdd` to build the ticket;
   - merges the integration branch tip into its own branch before reporting done

5. Once an **implementer subagent** completes, merge its work to the integration branch with a **merger subagent**.

6. If this changes the **frontier** of available tickets, kick off more **implementer subagents** to work on the new tickets. This allows for maximum concurrency.

7. Once all tickets are complete, call the Skill tool with `code-review` on the integration branch. Fix all issues raised by the code review in a single fresh **implementer subagent**.

8. If a draft PR exists, mark it ready for review. Otherwise, resolve each ticket the way the issue tracker closes work, and report the integration branch.

9. Clean up all **implementer subagent** worktrees.
{%- else -%}
你拿到了一份 spec。这份 spec 应当已经关联了对应的工单，描述了如何实现它。

issue tracker 的配置应当已经提供给你；如果没有，请提示用户先运行 `/setup-matt-pocock-skills`。

目标是把整份 spec 完整实现到同一条**集成分支（integration branch）**上，并按照 issue tracker 的关闭流程结清每个工单。

这些工单不是线性的步骤清单，而是一张带有阻塞关系的**任务图（task graph）**。这意味着图上始终存在一批已就绪、可立即认领的**前沿（frontier）**工单。

每个 subagent 都必须从全新的上下文窗口启动。

与 subagent 之间的通信应当精简。主要通过**上下文指针（context pointers）**通信：指向 spec、工单、调研笔记和先前的提交记录。不要重复复制通过指针即可读取的信息。

**实现者 subagent（Implementer subagents）**应尽量在后台并发运行，以实现最大并发度。

## 步骤

1. 阅读 spec 与工单，理清任务图依赖关系。

2. （可选）使用一个**探索 subagent** 完成工单所需的背景探索——相关代码文件或外部文档。确保探索 subagent 能够落盘文件：它应把 Markdown 笔记保存在仓库外、且所有后续 subagent 都能访问的目录中。这样**实现者 subagent**就能专注于实现，而不必重复探索。

3. 创建集成分支。如果 issue tracker 是通过 PR 关闭工单的，或者用户明确要求开 PR，在第 5 步完成第一次合并后立即打开一个草稿 PR（还没有领先 main 的提交时无法创建 PR），并标记为关闭对应的 spec 与工单。

4. 为每个工单拉起一个全新的**实现者 subagent**，各自运行在独立的 worktree 与独立分支上。每个实现者 subagent 需做到：
   - 动工前确认自己的 worktree 基于当前集成分支创建，若不是则 reset 到集成分支；
   - 调用 Skill 工具执行 `tdd` 来完成该工单；
   - 在汇报完成前，先把集成分支最新 tip 合并回自己的分支。

5. 一旦某个**实现者 subagent** 完成，通过一个**合并者 subagent（merger subagent）**将其工作合并到集成分支。

6. 如果这次合并推进了可用工单的**前沿**，立刻拉起更多**实现者 subagent** 处理新解锁的工单，保持最大并发。

7. 所有工单完成后，在集成分支上调用 Skill 工具执行 `code-review`。用一个全新的**实现者 subagent** 集中修复 code review 提出的所有问题。

8. 如果存在草稿 PR，将其标记为 Ready for Review；否则按 issue tracker 的关闭方式逐一关闭工单，并汇报集成分支情况。

9. 清理所有**实现者 subagent** 的 worktree。
{%- endif %}
