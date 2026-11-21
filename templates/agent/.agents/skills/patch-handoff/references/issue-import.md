{% if options["skill_lang"] == "en" -%}
# Importing the Package's Issues, Then Closing Them Out

A handoff package carries tickets as often as it carries patches, and a ticket inside a zip is a ticket nobody can open. Two moments: import before you replay, close out after you push.

## 1. What a package carries

| Shape | Looks like | How to import |
| :--- | :--- | :--- |
| Issue bundle | `issues/NNNN-*.md` with `key` / `title` / `labels` / `parent` / `blocked_by` front matter | `issue-push.ps1` — the bundle format this skill's own scripts read and write |
| Table in the doc | a markdown table mapping patch → ticket → title | turn each row into an issue file with a `key`, then push the directory |
| Bare numbers | the doc just cites `#102–#113` | nothing to import; verify each with `gh issue view <n>` and reuse the numbers |

The bundle form is the one to prefer, because it carries the dependency wiring (`parent`, `blocked_by`) that a plain `gh issue create` loop loses.

## 2. Three checks before importing

1. **Target repo.** `gh repo view --json nameWithOwner` — in a fork `gh` defaults to the upstream repo. Pass `-Repo <owner/name>` explicitly and compare.
2. **Labels exist.** A front-matter label that is not on the repo fails the whole batch. `gh label list` first; create the missing ones deliberately, or strip them from the files and say so in the report.
3. **Duplicates.** If the doc's numbers already exist, do not recreate them. If you are re-importing a set that was imported before, the script is idempotent by `key` — but a **regenerated** key set (`T-01` vs `S1-01`) creates duplicates; `gh issue list --search '"<title>"'` before pushing.

## 3. Import

```powershell
# preview: every create, parent link and blocked_by edge, no remote writes
pwsh .agents/skills/patch-handoff/scripts/issue-push.ps1 -Repo <owner/name> `
    -InputDir <pkg>/issues -Mapping ./issue-mapping.json -DryRun

# create; add -Resume if it stops on its time budget
pwsh .agents/skills/patch-handoff/scripts/issue-push.ps1 -Repo <owner/name> `
    -InputDir <pkg>/issues -Mapping ./issue-mapping.json
```

It writes `issue-mapping.json` (`key` → real number), backfills `number` / `url` into the issue files, links parents as sub-issues and dependencies as native `blocked_by`.

The script is part of this skill (see [issue-sync.md](./issue-sync.md)); there is no extra skill to install.

## 4. The mapping table

Write this into the report before you close anything:

```
patch   issue   #     landed
0001    T-01    #101  yes (3f2a1c9)
0004    T-04    #104  no  (dropped: duplicate of #101)
```

Sources, in order of reliability: the doc's own patch↔ticket table → the numbers already cited in issue subjects → `issue-mapping.json` keys. Match only on explicit identifiers (a key embedded in the patch subject, or a number the doc states). **If a patch and a ticket do not obviously correspond, stop and ask the user.** A guessed number closes somebody else's ticket.

## 5. Close-out by mapping

| State | Action |
| :--- | :--- |
| Landed, folded into commit `abc1234` | `gh issue comment <n>` (SHA, gates, counts) → `gh issue close <n>` → labels: add new, then remove old |
| Patch dropped / folded elsewhere | keep **open**, comment with the patch number and the reason |
| Was already closed before this handoff | leave it closed; add the landing comment only, do not reopen |

`gh issue close <n> --comment "..."` on an already-closed issue discards the comment and still exits 0 — comment first, then close, then read the output.

## 6. Failure modes

- **Missing label** — the batch aborts mid-way; some issues exist, some do not. Re-run after fixing the labels: the script resumes by `key`.
- **`gh` older than 2.94** — `gh issue` has no sub-issue or dependency subcommands; those edges need `gh api` REST calls, which the script already does. Do not hand-roll GraphQL to work around it.
- **Time budget hit** — the state file is written; pass `-Resume`, do not start over.
- **Unauthenticated `gh`** — the report must say `issues not imported: gh not authenticated` **plus the exact command**. Never paraphrase the ticket list into prose and call it tracked.
- **Numbers from another repo** — a doc written against an upstream project cites upstream numbers; they mean nothing in your fork. Re-import under new numbers and say so.

## 7. Temp artifacts to delete at S8

`issue-mapping.json`, the unpacked `issues/` tree, the unpack dir itself, `fix-spec/`, logs. List them, confirm, delete — then verify the unpack dir is gone and `git status --porcelain -uall` is empty.
{% else %}
# 交接包的 issue 导入，以及落地后的收口

交接包带工单的频率和带补丁差不多，而 zip 里的工单等于没人能点开的工单。两个时刻：落补丁前导入，推送后收口。

## 一、包里可能带的几种形态

| 形态 | 长什么样 | 怎么导入 |
| :--- | :--- | :--- |
| issue 包 | `issues/NNNN-*.md`，front matter 带 `key` / `title` / `labels` / `parent` / `blocked_by` | `issue-push.ps1` —— 本技能自带脚本读写的同一套格式 |
| 文档里的表 | 一张 markdown 表：补丁 → 工单 → 标题 | 每行转成一个带 `key` 的 issue 文件，再推整个目录 |
| 只有编号 | 文档直接写 `#102–#113` | 无需导入；逐个 `gh issue view <n>` 核实并沿用编号 |

优先用 bundle 形态：它带着 `parent` 与 `blocked_by` 的依赖接线，而手写 `gh issue create` 循环会把这些关系丢掉。

## 二、导入前三个检查

1. **目标仓库。** `gh repo view --json nameWithOwner` —— fork 里 `gh` 默认打上游仓库。显式传 `-Repo <owner/name>` 并核对。
2. **标签存在。** front matter 里写了仓库没有的标签，整批会失败。先 `gh label list`；缺的标签要么有意创建，要么从文件里去掉并在报告里说明。
3. **重复。** 文档引用的编号已存在就别重建。若这批之前导过一次，脚本按 `key` 幂等——但**重新生成过的 key**（`T-01` 变成 `S1-01`）会造出重复单；推之前先 `gh issue list --search '"<标题>"'`。

## 三、导入

```powershell
# 预演：打印每次创建、父子链接与 blocked_by 边，不写远端
pwsh .agents/skills/patch-handoff/scripts/issue-push.ps1 -Repo <owner/name> `
    -InputDir <pkg>/issues -Mapping ./issue-mapping.json -DryRun

# 真建；撞到时间预算就加 -Resume 续跑
pwsh .agents/skills/patch-handoff/scripts/issue-push.ps1 -Repo <owner/name> `
    -InputDir <pkg>/issues -Mapping ./issue-mapping.json
```

它产出 `issue-mapping.json`（`key` → 真实编号），把 `number` / `url` 回填进 issue 文件，并把 parent 挂成 sub-issue、依赖连成原生 `blocked_by`。

脚本属于本技能（见 [issue-sync.md](./issue-sync.md)），不需要额外安装任何技能。

## 四、映射表

关任何单之前，先把这张表写进报告：

```
补丁    工单     #     是否落地
0001    T-01    #101  是（3f2a1c9）
0004    T-04    #104  否（已丢弃：#101 已覆盖）
```

取值优先级：文档自己的 补丁↔工单 表 → issue 标题里已写明的编号 → `issue-mapping.json` 的 key。只按显式标识匹配（补丁标题里嵌的 key，或文档明写的编号）。**补丁与工单对不上就停手问用户。** 猜一个编号，关掉的是别人的工单。

## 五、按映射收口

| 状态 | 动作 |
| :--- | :--- |
| 已落地、折进提交 `abc1234` | `gh issue comment <n>`（SHA、门禁、通过数）→ `gh issue close <n>` → 标签：先加新、再撤旧 |
| 补丁丢弃 / 折到别处 | 保持 **open**，评论写清是哪条补丁、为什么 |
| 交接前就已关闭 | 保持关闭，只补一条落地评论，不要重新打开 |

`gh issue close <n> --comment "..."` 对已关闭的 issue 会丢评论并仍返回 0 —— 先评论、再关闭、然后读输出。

## 六、失败模式

- **标签缺失** —— 批次中途中断，有的单建了有的没建。补好标签后重跑，脚本按 `key` 续上。
- **`gh` 低于 2.94** —— `gh issue` 没有子工单与依赖的子命令，这两类边要走 `gh api` 的 REST 端点，脚本里已经这么做了；不要为了绕开它去手写 GraphQL。
- **撞时间预算** —— 状态文件已落盘，加 `-Resume` 续跑，不要从头再来。
- **`gh` 未登录** —— 报告必须写 `issue 未导入：gh 未登录` **外加原样命令**，不许把工单清单改写成一段文字然后声称已跟踪。
- **编号来自别的仓库** —— 对着上游项目写的文档引的是上游编号，在你的 fork 里毫无意义。重新导入拿新编号，并在报告里说明。

## 七、S8 要删的临时产物

`issue-mapping.json`、解包出来的 `issues/` 目录、解包目录本身、`fix-spec/`、日志。列清单、确认、删除——然后验证解包目录已不存在且 `git status --porcelain -uall` 为空。
{% endif %}
