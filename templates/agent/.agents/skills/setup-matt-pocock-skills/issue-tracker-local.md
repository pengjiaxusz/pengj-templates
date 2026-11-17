{% if options["skill_lang"] == "en" -%}
# Issue tracker: Local Markdown

Issues and specs for this repo live as markdown files in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01`, never a single combined tickets file
- Triage state is recorded as a `Status:` line near the top of each issue file (see `triage-labels.md` for the role strings)
- Comments and conversation history append to the bottom of the file under a `## Comments` heading

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` (the Notes / Decisions-so-far / Fog body).
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.
{%- else -%}
# Issue 跟踪器：本地 Markdown

本仓库的 issue 与 spec 以 Markdown 文件形式存放在 `.scratch/` 下。

## 约定

- 每个特性一个目录：`.scratch/<feature-slug>/`
- spec 为 `.scratch/<feature-slug>/spec.md`
- 实现工单为每个工单一个文件：`.scratch/<feature-slug>/issues/<NN>-<slug>.md`，从 `01` 开始编号，绝不把所有工单塞进同一个文件
- 分诊状态记录在每个工单文件顶部附近的 `Status:` 行（角色字符串见 `triage-labels.md`）
- 评论与讨论历史追加在文件底部的 `## Comments` 标题下

## 当技能说“发布到 issue tracker”时

在 `.scratch/<feature-slug>/` 下创建新文件（必要时先创建目录）。

## 当技能说“拉取相关工单”时

读取所引用路径的文件。用户通常会直接提供路径或工单编号。

## Wayfinding 操作

供 `/wayfinder` 使用。**地图（map）**是一个文件，每个子工单对应一个独立文件。

- **地图**：`.scratch/<effort>/map.md`（包含 Notes / Decisions-so-far / Fog）。
- **子工单**：`.scratch/<effort>/issues/NN-<slug>.md`，从 `01` 开始编号，正文写待解决的问题。`Type:` 行记录工单类型（`research`/`prototype`/`grilling`/`task`）；`Status:` 行记录 `claimed`/`resolved`。
- **阻塞关系**：顶部附近写一行 `Blocked by: NN, NN`。当列出的每一个文件都变为 `resolved` 时，该工单解除阻塞。
- **前沿查询**：扫描 `.scratch/<effort>/issues/` 中状态既不是 `resolved` 也未被 `claimed`、且无未解决阻塞者的文件；按编号取第一个。
- **认领**：在开始任何工作前先设置 `Status: claimed` 并保存。
- **解决**：在 `## Answer` 标题下追加答案，将状态设为 `Status: resolved`，然后向 `map.md` 的 Decisions-so-far 追加一条上下文指针（要点摘要 + 链接）。
{%- endif %}
