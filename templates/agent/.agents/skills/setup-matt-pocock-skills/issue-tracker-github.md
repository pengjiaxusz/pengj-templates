{% if options["skill_lang"] == "en" -%}
# Issue tracker: GitHub

Issues and specs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --json number,title,body,labels,comments`.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` with appropriate `--label` and `--state` filters.
- **Make an issue a sub-issue of a parent**: `gh issue create --parent <parent> ...`, or `gh issue edit <parent> --add-sub-issue <child>` afterwards (`gh` 2.94+). Older `gh`: `gh api --method POST repos/<owner>/<repo>/issues/<parent>/sub_issues -F sub_issue_id=<child-db-id>` (database id, as in **Blocking** below). Without sub-issues, put `Part of #<parent>` at the top of the child body.
- **Blocking**: GitHub's **native issue dependencies**, the canonical, UI-visible representation. Add an edge with `gh issue create --blocked-by <n>,<n> ...` or `gh issue edit <child> --add-blocked-by <blocker>` (`gh` 2.94+). Older `gh`: `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`, where `<blocker-db-id>` is the blocker's numeric **database id** (`gh api repos/<owner>/<repo>/issues/<n> --jq .id`, _not_ the `#number` or `node_id`). Where dependencies aren't available, fall back to a `Blocked by: #<n>, #<n>` line at the top of the child body. A ticket is unblocked when every blocker is closed.
- **Count open blockers**: `gh api repos/<owner>/<repo>/issues/<n> --jq .issue_dependencies_summary.blocked_by` (open blockers only; 0 means startable). `gh issue view <n> --json blockedBy` also lists closed blockers, so filter it: `--jq '[.blockedBy.nodes[] | select(.state == "OPEN")] | length'`.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**: post the explanation with `gh issue comment <number> --body "..."`, then `gh issue close <number>`. On an already-closed issue, `gh issue close --comment` drops the comment and still exits 0.

Infer the repo from `git remote -v`; `gh` does this automatically when run inside a clone.

File issues only into the user's own repo. Never file into `mattpocock/skills` (the repo these skills come from) unless the user owns it. In a fork, `gh` can default to the upstream parent, so before the first `gh issue create` of a session, check the target with `gh repo view --json nameWithOwner` and confirm it with the user if it is not their repo.

## Pull requests as a triage surface

**PRs as a request surface: no.** _(Set to `yes` if this repo treats external PRs as feature requests; `/triage` reads this flag.)_

When set to `yes`, PRs run through the same labels and states as issues, using the `gh pr` equivalents:

- **Read a PR**: `gh pr view <number>`, then `gh pr view <number> --comments` (off a TTY, `--comments` prints only the comments, not the body), and `gh pr diff <number>` for the diff.
- **List external PRs for triage**: `gh api --paginate 'repos/{owner}/{repo}/pulls?state=open' --jq '.[] | select(.author_association | IN("OWNER","MEMBER","COLLABORATOR") | not) | {number, title, author: .user.login, author_association, labels: [.labels[].name]}'`.
- **Comment / label / close**: `gh pr comment`, `gh pr edit --add-label`/`--remove-label`, `gh pr close`.

GitHub shares one number space across issues and PRs, so a bare `#42` may be either: resolve with `gh pr view 42` and fall back to `gh issue view 42`.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Read it as in **Read an issue** above.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single issue with **child** issues as tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes / Decisions-so-far / Fog body. `gh issue create --label wayfinder:map`.
- **Child ticket**: an issue linked to the map as a GitHub sub-issue (see **Make an issue a sub-issue of a parent**). Where sub-issues aren't enabled, add the child to a task list in the map body and put `Part of #<map>` at the top of the child body. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: see **Blocking** under Conventions.
- **Frontier query**: list the map's open children (`gh issue list --state open`, scoped to the map's sub-issues / task list), drop any with an open blocker (see **Count open blockers**, or an open issue in the `Blocked by` line) or an assignee; first in map order wins.
- **Claim**: `gh issue edit <n> --add-assignee @me`, the session's first write.
- **Resolve**: `gh issue comment <n> --body "<answer>"`, then `gh issue close <n>`, then append a context pointer (gist + link) to the map's Decisions-so-far.
{%- else -%}
# Issue 跟踪器：GitHub

本仓库的 issue 与 spec 存放在 GitHub Issues 中，所有操作使用 `gh` CLI。

## 约定

- **创建 issue**：`gh issue create --title "..." --body "..."`。多行正文使用 heredoc。
- **读取 issue**：`gh issue view <number> --json number,title,body,labels,comments`。
- **列出 issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，按需搭配 `--label` 与 `--state` 过滤。
- **设为父 issue 的子 issue**：`gh issue create --parent <parent> ...`，或事后执行 `gh issue edit <parent> --add-sub-issue <child>`（`gh` 2.94+）。旧版 `gh` 用：`gh api --method POST repos/<owner>/<repo>/issues/<parent>/sub_issues -F sub_issue_id=<child-db-id>`（数据库 ID，见下文**阻塞关系**）。若平台不支持 sub-issues，在子 issue 正文顶部写 `Part of #<parent>`。
- **阻塞关系（Blocking）**：使用 GitHub **原生 issue 依赖关系**，这是 UI 可见的权威表达。添加依赖边：`gh issue create --blocked-by <n>,<n> ...` 或 `gh issue edit <child> --add-blocked-by <blocker>`（`gh` 2.94+）。旧版 `gh`：`gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`，其中 `<blocker-db-id>` 是阻塞者的数字**数据库 ID**（`gh api repos/<owner>/<repo>/issues/<n> --jq .id`，*不是* `#number` 或 `node_id`）。若原生依赖不可用，退回到在子 issue 正文顶部写一行 `Blocked by: #<n>, #<n>`。当所有阻塞者均已关闭时，工单解除阻塞。
- **统计未关闭的阻塞者数量**：`gh api repos/<owner>/<repo>/issues/<n> --jq .issue_dependencies_summary.blocked_by`（仅统计 open 阻塞者；为 0 表示可开工）。`gh issue view <n> --json blockedBy` 也会列出已关闭的阻塞者，需过滤：`--jq '[.blockedBy.nodes[] | select(.state == "OPEN")] | length'`。
- **评论 issue**：`gh issue comment <number> --body "..."`
- **添加 / 移除标签**：`gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **关闭 issue**：先用 `gh issue comment <number> --body "..."` 发说明，再执行 `gh issue close <number>`。对于已关闭的 issue，`gh issue close --comment` 会丢弃评论并仍返回 0。

通过 `git remote -v` 推断仓库；在克隆目录中运行 `gh` 时会自动识别。

只向用户自己的仓库提交 issue。除非用户是仓库所有者，否则绝不要把 issue 发到 `mattpocock/skills`（本套技能来源仓库）。在 fork 仓库中，`gh` 可能默认指向游父仓库，因此在每次会话首次执行 `gh issue create` 前，先用 `gh repo view --json nameWithOwner` 检查目标，若不是用户自己的仓库，先向用户确认。

## 将 Pull Request 作为分诊入口

**PRs as a request surface: no.** _（若本仓库把外部 PR 当作特性请求处理，改为 `yes`；`/triage` 会读取此标记。）_

设为 `yes` 时，PR 与 issue 走相同的标签与状态流转，使用对应的 `gh pr` 命令：

- **读取 PR**：`gh pr view <number>`，再 `gh pr view <number> --comments`（非 TTY 下 `--comments` 只打印评论不打印正文），以及 `gh pr diff <number>` 查看 diff。
- **列出待分诊的外部 PR**：`gh api --paginate 'repos/{owner}/{repo}/pulls?state=open' --jq '.[] | select(.author_association | IN("OWNER","MEMBER","COLLABORATOR") | not) | {number, title, author: .user.login, author_association, labels: [.labels[].name]}'`。
- **评论 / 打标签 / 关闭**：`gh pr comment`、`gh pr edit --add-label`/`--remove-label`、`gh pr close`。

GitHub 的 issue 与 PR 共用同一套编号空间，因此裸写的 `#42` 可能是其中任意一种：先用 `gh pr view 42` 解析，失败再回退到 `gh issue view 42`。

## 当技能说“发布到 issue tracker”时

创建一个 GitHub issue。

## 当技能说“拉取相关工单”时

按上文**读取 issue** 的方式读取。

## Wayfinding 操作

供 `/wayfinder` 使用。**地图（map）**是一个单独的 issue，**子 issue** 作为各决策工单。

- **地图**：带 `wayfinder:map` 标签的单个 issue，正文包含 Notes / Decisions-so-far / Fog。命令：`gh issue create --label wayfinder:map`。
- **子工单**：作为 GitHub sub-issue 关联到地图的 issue（见**设为父 issue 的子 issue**）。若未开启 sub-issues，把子工单加入地图正文的任务列表，并在子工单正文顶部写 `Part of #<map>`。标签为 `wayfinder:<type>`（`research`/`prototype`/`grilling`/`task`）。认领后指派给推动该任务的开发者。
- **阻塞关系**：见上文约定中的**阻塞关系**。
- **前沿查询**：列出地图下所有 open 子工单（`gh issue list --state open`，限定在地图的 sub-issues / 任务列表），排除仍有未关闭阻塞者（见**统计未关闭的阻塞者数量**，或 `Blocked by` 行中仍有 open issue）或已有指派人的工单；按地图顺序取第一个。
- **认领**：`gh issue edit <n> --add-assignee @me`，作为会话的第一次写操作。
- **解决**：`gh issue comment <n> --body "<answer>"`，然后 `gh issue close <n>`，再向地图的 Decisions-so-far 追加一条上下文指针（要点摘要 + 链接）。
{%- endif %}
