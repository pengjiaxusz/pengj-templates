{% if options["skill_lang"] == "en" -%}
# Issue Tooling — Clone the Backlog Out, Push a Bundle Back

The two PowerShell scripts bundled with this skill move issues between GitHub and a portable Markdown
bundle: one command each way, no third-party binary, no local mirror committed to the repo. The push
path is what a handoff package needs (see [issue-import.md](./issue-import.md)); the pull path exists
so an export can be read, diffed or handed to another agent without GitHub access.

Both ship inside this skill — there is no separate `issue-sync` skill to install, and no dependency
on a skill the project may not have generated.

## Quick Start

```powershell
# Pull: open issues -> ./issue-export/ (+ optional zip)
pwsh .agents/skills/patch-handoff/scripts/issue-pull.ps1 -Repo owner/name -Out ./issue-export -Zip

# Push: local issue files -> real issues, then wire parents and dependencies
pwsh .agents/skills/patch-handoff/scripts/issue-push.ps1 -Repo owner/name -InputDir ./issue-plan -DryRun
```

Always run `-DryRun` first on push. It prints every create/link without touching the remote.

## Read Path

`issue-pull.ps1` calls `gh issue list --json` once (plus `--label` / `--search` / `--state` / `--limit`
pass-through), normalizes the GraphQL connection shapes, then renders:

- `README.md` — index table + dependency topology tree
- `issues/NNNN-<kind>-<slug>.md` — front matter metadata, body, comments
- `data/issues.json`, `data/relations.json` — untouched API payloads

Layout is driven by the templates in `templates/` next to `scripts/`, so restyling an export never
touches code.

## Write Path

`issue-push.ps1` reads a directory of Markdown files whose front matter carries `key`, `title`,
`labels`, optional `parent` and `blocked_by` (keys, not numbers). It creates missing issues, links
parents as sub-issues, then links dependencies as native `blocked_by`. The output is a `mapping.json`
from local key to real number.

Three invariants it enforces:

1. **Idempotent** — existing edges are read before writing, so re-running only fills gaps.
2. **Chunked** — one invocation stays under a wall-clock budget (default 110s) and leaves a resumable
   state file; pass `-Resume` to continue.
3. **Retried** — every API call retries on transport errors.

## Boundaries

- Read and write both require `gh` (>= 2.94) and an authenticated account; nothing else.
- No two-way conflict merge, no `.issues/` mirror committed to the repo, no hand-written GraphQL.
- `gh issue` has no sub-issue or dependency subcommands — those two links go through `gh api` REST
  endpoints. See [issue-api-notes.md](./issue-api-notes.md).
- Output format and front matter fields: [issue-export-format.md](./issue-export-format.md).
{% else %}
# Issue 工具 —— 把积压拉出来，把包推回去

本技能自带的两个 PowerShell 脚本在 GitHub 与可搬运的 Markdown 包之间搬运 issue：一进一出各一条
命令，不引第三方二进制，不在仓库里留镜像。交接包用的是推送那一侧（见
[issue-import.md](./issue-import.md)）；拉取一侧用来把积压导出成可读、可 diff、可交给别的 agent
的包，让对方不必访问 GitHub。

两个脚本随本技能一起下发——不存在独立的 `issue-sync` 技能要装，也不依赖项目可能没生成的技能。

## 快速开始

```powershell
# 读：open issue -> ./issue-export/（可额外打包 zip）
pwsh .agents/skills/patch-handoff/scripts/issue-pull.ps1 -Repo owner/name -Out ./issue-export -Zip

# 写：本地 issue 文件 -> 真实 issue，并挂父子与依赖
pwsh .agents/skills/patch-handoff/scripts/issue-push.ps1 -Repo owner/name -InputDir ./issue-plan -DryRun
```

推送侧先跑 `-DryRun`：它只打印将要执行的创建与连边，不碰远端。

## 读侧

`issue-pull.ps1` 只调一次 `gh issue list --json`（`--label` / `--search` / `--state` / `--limit`
直通），把 GraphQL connection 形状归一化后渲染出：

- `README.md` —— 索引表 + 依赖拓扑树
- `issues/NNNN-<kind>-<slug>.md` —— front matter 元数据 + 正文 + 评论
- `data/issues.json`、`data/relations.json` —— API 原始载荷

版式由 `scripts/` 同级的 `templates/` 下的模板驱动，换排版不用改代码。

## 写侧

`issue-push.ps1` 读一个目录下的 Markdown 文件，front matter 带 `key`、`title`、`labels`，可选
`parent` 与 `blocked_by`（填 key 而不是编号）。它先建缺失的 issue，再把 parent 挂成 sub-issue，
最后连原生 `blocked_by`，产出 `mapping.json`（本地 key → 真实编号）。

它保证三件事：

1. **幂等** —— 写之前先读现有边，重跑只补缺口，不会产生重复。
2. **分片** —— 单次运行受墙钟预算约束（默认 110s），超时落可续传的状态文件，加 `-Resume` 继续。
3. **重试** —— 每次 API 调用都对传输层错误重试。

## 边界

- 读写都只需要 `gh`（>= 2.94）与已登录账号，别的都不要。
- 不做双向冲突合并，不把 `.issues/` 镜像提交进仓库，不手写 GraphQL。
- `gh issue` 没有子工单与依赖的子命令，这两类边只能走 `gh api` 的 REST 端点，见
  [issue-api-notes.md](./issue-api-notes.md)。
- 产物格式与 front matter 字段见 [issue-export-format.md](./issue-export-format.md)。
{% endif %}
