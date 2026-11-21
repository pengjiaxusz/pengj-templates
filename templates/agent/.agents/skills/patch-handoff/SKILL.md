---
name: patch-handoff
description: >-
{% if options["skill_lang"] == "en" %}
  Verify and land an offline patch handoff package (`HANDOFF.md` + `patches/NNNN-*.patch`): push the issues the package carries up to GitHub first, replay the series with `git am`, repair the build and tests, fold each repair back into the commit that introduced the defect, normalize authorship to the repository owner's identity (auto-resolved, never hand-filled), force-push with lease, sync the worktree, close each mapped issue out after landing, and delete the unpacked temp content. Every acceptance claim in the handoff document is an unverified hypothesis until you reproduce it yourself. Use when receiving a patch series or handoff zip from another agent, or when a delivered series does not compile as claimed. Triggers: patch-handoff, handoff package, handoff zip, patch series, git am, apply patches, verify handoff, fold fixes, offline handoff, upload handoff issues, close handoff issues.
{% else %}
  验收并合入离线补丁交接包（`HANDOFF.md` + `patches/NNNN-*.patch`）：先把包里带的 issue 条目上传成真实 GitHub issue 并拿回编号，再按序 `git am` 落提交、修编译与测试、把修复折回引入缺陷的提交、作者自动归一为仓库拥有者身份（禁止手填）、强推、同步工作树；落地后按映射逐条收口 issue 状态，并删掉解包出来的临时内容。交接文档里的每一条验收声明，在你亲手复现之前都是未验证假设。当收到别的 agent 产出的补丁集/交接 zip，或交付的补丁集并不像文档说的那样能编译时使用。 Triggers: patch-handoff, 交接包, handoff 包, handoff.zip, 补丁集, patches, git am, 打补丁验收, patch series, 补丁包验收, 校验交接文档, 交接 issue 上传, issue 收口.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" -%}
# Patch Handoff — Verify & Land an Offline Patch Series

Turn another agent's offline handoff package into commits that actually live in the repo, and prove it with the project's real gates instead of the document's claims.

**Assume the document is wrong until reproduced.** Handoff packages routinely claim `git am` / lint / tests all passed while the series does not compile and the recorded baseline SHA is off. Treat every claim as a hypothesis, reproduce it, and state the outcome in your report.

## Work in Place — Do Not Clone the Project to Test

**Accept and land the patches directly in the user's existing worktree. Do NOT `git clone` the repo again, copy it to a scratch directory, or spin up a parallel build just to "test safely".** Re-cloning plus reinstalling dependencies and rebuilding a complex project costs tens of minutes to hours and buys nothing: the gates you must reproduce live in this worktree, and the commits must land here. A throwaway copy is a second, divergent state you would have to reconcile later — strictly more work and more risk.

Only clone when the user explicitly asks for isolation (e.g. the worktree must stay pristine for another in-flight task), and even then install hooks in the clone before working. Default: operate in place, keep the worktree clean enough to `git am`, and clean up only the unpack directory and logs you created (see S8).

## Before Touching Anything: Pin Seven Parameters

Read them out of the repo, never out of `HANDOFF.md`.

| Parameter | Where to get it | Why it matters |
| :--- | :--- | :--- |
| Base ref | `git rev-parse --abbrev-ref origin/HEAD`; compare with the SHA the doc claims | Most `git am` failures are a wrong base, not a bad patch |
| Gate chain | `justfile` / `Makefile` / `package.json` / `.github/workflows/*.yml` | Your locally chosen commands are not the CI gate |
| Format check | Whether CI runs `fmt --check` / `lint` as a separate step | A linter passing says nothing about formatting |
| Hook installation | `git config core.hooksPath`; run the repo's installer in the current worktree if unset | Hooks are per-worktree config and do not travel with a clone |
| Author identity | `normalize-authors.sh --print-identity` — owner from `origin` URL + `git config user.email`; NEVER hand-fill or read from the package | Package authors are synthetic placeholders; a hand-typed name is the leak |
| Issue source | What the package carries: `issues/*.md` (front matter), a ticket table inside the doc, or bare numbers | Without this inventory there is no patch↔issue mapping, and S7 has nothing to close |
| Import target | `gh repo view --json nameWithOwner`; `gh label list`; `gh issue view <n>` for any number the doc cites | On a fork `gh` defaults to the upstream repo; a label that does not exist fails the whole batch |

## Quick Start

```bash
PKG=<handoff zip or unpacked dir>
BASE="origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|^origin/||')"
git status -sb; git rev-list --count "$BASE"..HEAD     # must be clean and 0

.agents/skills/patch-handoff/scripts/inspect-handoff-package.sh "$PKG"   # pre-flight audit

# if the package carries issues: import them first and keep the key -> number mapping
pwsh .agents/skills/issue-sync/scripts/issue-push.ps1 -Repo <owner/name> \
    -InputDir <dir>/issues -Mapping ./issue-mapping.json -DryRun   # drop -DryRun to create

.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE" --print-identity  # confirm target identity
.agents/skills/patch-handoff/scripts/replay-patch-series.sh <dir>/patches <fix-spec-dir> "$BASE"

# then run the whole gate chain, no skipped steps
# after folding repairs, normalize authorship and then push:
.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE"   # no name/email: auto from repo
```

## Workflow

| Step | Action | Key point |
| :-- | :--- | :--- |
| S1 | Pre-flight | Inventory patches, authors, touched files; compare claimed baseline with reality; **import the package's issues to GitHub** (`-DryRun` first, then create) and build the patch↔issue mapping |
| S2 | Replay | `git am patches/*.patch` in order; on failure diagnose the base first, then `--show-current-patch` |
| S3 | Repair | Run the gate chain top-down; syntax errors mask type errors, so re-run after each fix |
| S4 | Fold | Fold each repair into the commit that introduced it, keeping one commit per ticket |
| S5 | Author + push | Run `normalize-authors.sh "$BASE"` **without name/email** so the identity is auto-resolved from the repo (keeping author dates); then `push --force-with-lease` |
| S6 | Resync | After a force push: `fetch --prune --force` then `reset --hard`, never `git pull` |
| S7 | Issue close-out | Walk the mapping: each landed ticket gets a comment with its fold-back SHA, then `close`, then its label flipped; a ticket with no landed patch stays open with that reason written down |
| S8 | Cleanup | Delete the unpacked temp content you created: unpack dir, fix-spec dir, import mapping, logs. Verify it is gone. No throwaway clone should exist — do not clone to test |

## The Package's Issues Go Up to GitHub First

A handoff package usually ships its tickets as files — `issues/*.md` with `key` / `title` / `labels` / `parent` / `blocked_by` front matter, sometimes just a ticket table inside `HANDOFF.md`. Those tickets are the plan behind the patch series: they have no business staying inside a zip. Push them to GitHub **before** replaying, so every claim in the doc points at a number somebody can open.

1. **Preview.** `issue-push.ps1 -DryRun` prints every create, parent link and `blocked_by` edge without touching the remote.
2. **Create.** Drop `-DryRun`. The script is idempotent — re-running only fills gaps — and writes `mapping.json` (local `key` → real number).
3. **Map.** Build the `patch NNNN ↔ #number` table from that mapping (and from the numbers the doc already cites, cross-checked with `gh issue view`). The mapping is what S7 walks; a patch with no number is a ticket that is not on GitHub yet.
4. **Backfill.** Push writes `number` and `url` back into the issue files — keep that so the next handoff can match without guessing.
5. **No `issue-sync` skill in this project** (it is not a baseline skill): create them one by one with `gh issue create --title ... --body-file ...`, in dependency order, and write the same mapping by hand. Slower, identical contract.
6. **Never invent the mapping.** If a patch title and an issue title do not obviously correspond, stop and ask the user — a guessed number closes the wrong ticket, which is worse than a ticket left open.

Filing is not the same as reporting: problems **you** discover while verifying are not new tickets. They go into the report, and — if the user wants them tracked — as comments on the corresponding ticket. Do not open your own issues unless the user asks for them.

## Close-Out — After Landing, Walk the Mapping

After the push (S5–S6), every ticket in the mapping gets a state you can name:

1. **Landed.** `gh issue comment <n>` with the landing summary — folded commit SHA, gate commands, test counts — **then** `gh issue close <n>`, **then** flip labels (add the new label before removing the old one, so no issue is ever label-less). Do not use `gh issue close --comment`: on an already-closed issue it drops the comment and still exits 0.
2. **Not landed.** A ticket whose patch was dropped, folded elsewhere, or failed the gates stays **open**, with a comment saying exactly which patch and why. Silently closing it makes the tracker lie about what shipped.
3. **Self-check.** End the report with the table you actually walked: `#101 → 0001 → 3f2a1c9 closed`, `#104 → 0004 → open (patch dropped, behaviour gap)`, plus the total `closed / open`.

## Cleanup — the Unpacked Temp Content Must Go

Only `patches/*.patch` has lasting value in a handoff package; `HANDOFF.md`, the doc's side files, the `issues/` tree and your `fix-spec/` are scaffolding. Once the handoff is landed and closed out, delete them:

- Unpack dir (`handoff-*/`, a `mktemp -d` path), fix-spec dir, the `mapping.json` left in the worktree, logs, exported intermediates.
- List before deleting (iron rule 5), delete only what you created, no wildcards, nothing in shared locations such as `~/.cache`.
- Exception: a worktree left half-applied by a stopped `git am` is evidence — keep it until the user has seen it, then delete it.
- Verify: `git status --porcelain -uall` is empty and the unpack dir no longer exists. A leftover unpack dir is how the next handoff gets confused with the last one.

## Iron Rules

1. `HANDOFF.md` and the handoff directory never enter the repo; only `patches/*.patch` may arrive via `git am`.
2. The claimed baseline SHA is not evidence: the real base is the commit where the series applies cleanly and every API reference lines up. Report the discrepancy.
3. One commit per ticket. Fold repairs back into the offending commit; do not append `fix(...)` commits that break the 1:1 mapping.
4. Authorship must be the repository owner's identity. Run `normalize-authors.sh` **without** name/email so it resolves the owner from `origin` and the email from `git config user.email`; never hand-fill them or copy them from the package.
5. Before deleting anything, print the list and get confirmation. Delete only what you created. No wildcards.
6. A failing test is reported to the user, not silently accommodated by editing the assertion.
7. `git am`, `cherry-pick` and `rebase` trigger no `pre-commit` hook — only `pre-push` can still catch the damage. Confirm hooks are installed in the current worktree before you work; if you were explicitly told to isolate in a clone, install hooks there first.
8. **Work in the user's worktree.** Never clone or copy the project just to verify or land a patch. Complex projects lose tens of minutes to hours on a fresh clone plus dependency setup, for zero gain.
9. **The package's issues belong on GitHub, not in the zip.** Import them (`-DryRun` first) before replaying, build the `patch ↔ #number` mapping from the resulting `mapping.json`, and never guess a number that does not correspond.
10. **Nothing in scope stays dangling after landing.** Every mapped ticket ends closed with its fold-back SHA, or open with a comment naming the patch and the reason. `gh issue close --comment` is not a substitute for comment-then-close.
11. **The unpacked temp content is not an artifact.** Unpack dir, `issues/` tree, fix-spec dir, `mapping.json` and logs are deleted at S8, and you verify they are gone rather than assuming.

## How to Write Acceptance Criteria

Build a two-column table — command, passing signature — from the project's real gate chain (fill it in the project-specific area below or in a reference file). Then respect the boundary:

- A gate passing locally means the commands you ran passed; it does not mean CI passes. CI usually runs a superset (formatting, language guards, title checks).
- For tests, read the summary line (`failed=0`, total count) rather than the absence of red text.
- For newly added behaviour, grep the log for the specific test name to confirm it actually executed instead of being skipped or unnamed.

## What Tests Cannot Catch

Green build plus green tests is not feature completeness. Shape or snapshot assertions that count elements cannot tell "the element is there" from "the element draws nothing" — a real case had a popup reserve capacity for its menu labels and never push them, so every row rendered blank while all tests passed. Sample the new output paths by hand (text, icons, separators, side effects) and put what you find in the report for the user to decide on.

## Scripts

| Script | Purpose |
| :--- | :--- |
| `scripts/inspect-handoff-package.sh` | Pre-flight: patch inventory, authors, touched files, claimed-vs-actual baseline |
| `scripts/replay-patch-series.sh` | Ordered `git am` plus fold-back of repair files into their commits |
| `scripts/normalize-authors.sh` | Rewrite authorship over a commit range to the auto-resolved owner identity (dates kept); `--print-identity` previews the target |
| `scripts/sync-after-force-push.sh` | Safe post-force-push worktree resync, refuses on dirty or unpushed state |

## References

- [references/handoff-audit.md](references/handoff-audit.md) — classifying claims, the usual lies, report format
- [references/git-mechanics.md](references/git-mechanics.md) — hook trigger matrix, `git am` failure triage, fold preconditions, author rewrite
- [references/field-notes.md](references/field-notes.md) — real error-to-API table from one landed series
- [references/issue-import.md](references/issue-import.md) — importing the package's issues, the mapping table, close-out by mapping, `gh` failure modes
{%- else -%}
# 补丁交接包验收与合入（Patch Handoff）

把别的 agent 产出的离线交接包，变成真正落在仓库里的提交，并用项目真实门禁证明它成立——而不是用文档的声称。

**默认文档不可信，直到你复现它。** 这类包常声称 `git am` / lint / 测试全部通过，实测常见「打完根本编译不过」且基线 SHA 写错。文档里的每条声明都当假设，逐条复现，把结论写进报告。

## 就地验收，不要克隆项目来测试

**直接在用户现有的工作区里验收并落补丁，不要为了「安全测试」再 `git clone` 一份、拷到临时目录或另起一套并行构建。** 复杂项目重新克隆加重装依赖动辄几十分钟到几小时，收益为零：要复现的门禁就在这个工作区里，提交也必须落在本地这份仓库。另起一份只会多出一个需要事后对齐的分叉状态，徒增工作量与风险。

只有用户明确要求隔离时（例如工作区要为另一个进行中的任务保持原样）才另起克隆，且即便如此也要先在克隆里装好钩子再干活。默认就是：在当前工作区操作、保持工作树干净到能 `git am`、只清理自己创建的解包目录和日志（见 S8）。

## 动手前先钉七个参数

参数一律从仓库读，不从 `HANDOFF.md` 读。

| 参数 | 取值处 | 为什么必须钉 |
| :--- | :--- | :--- |
| 基线 ref | `git rev-parse --abbrev-ref origin/HEAD`，并与文档声称的 SHA 对照 | `git am` 失败多数是基线不对，不是补丁坏 |
| 验收命令链 | `justfile` / `Makefile` / `package.json` / `.github/workflows/*.yml` | 你自选的命令不等于 CI 门禁 |
| 格式检查位置 | CI 是否有独立的 `fmt --check` / `lint` 步骤 | linter 过了不代表格式过 |
| 钩子安装 | `git config core.hooksPath`；在当前工作区跑仓库的安装脚本（若未装） | 钩子是每个工作区自己的配置，不随 clone 传播 |
| 作者身份 | `normalize-authors.sh --print-identity` —— owner 取自 `origin` URL + `git config user.email`；**禁止手填或从包里读** | 包里的作者多为合成占位身份，手填名字就是乱填的源头 |
| issue 条目来源 | 包里带了什么：`issues/*.md`（front matter）、文档里的工单表、或只写了编号 | 没有清单就没有 patch↔issue 映射，S7 无从关单 |
| 导入目标 | `gh repo view --json nameWithOwner`；`gh label list`；文档引用的编号逐个 `gh issue view <n>` | fork 里 `gh` 默认打上游仓库；标签不存在会让整批建单失败 |

## 快速开始

```bash
PKG=<交接 zip 或已解包目录>
BASE="origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|^origin/||')"
git status -sb; git rev-list --count "$BASE"..HEAD     # 必须干净且为 0

.agents/skills/patch-handoff/scripts/inspect-handoff-package.sh "$PKG"   # 落库前体检

# 包里带 issue 条目时：先导入成真实 issue，留住 key -> 编号 的映射
pwsh .agents/skills/issue-sync/scripts/issue-push.ps1 -Repo <owner/name> \
    -InputDir <dir>/issues -Mapping ./issue-mapping.json -DryRun   # 去掉 -DryRun 才真正建单

.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE" --print-identity  # 确认归一目标身份
.agents/skills/patch-handoff/scripts/replay-patch-series.sh <dir>/patches <fix-spec-dir> "$BASE"

# 然后整条门禁链全跑，不跳步
# 折回修复后归一作者，再推送：
.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE"   # 不传名字/邮箱：自动从仓库解析
```

## 工作流

| 步 | 动作 | 要点 |
| :-- | :--- | :--- |
| S1 | 体检 | 清点补丁、作者、涉及文件；把文档声称的基线与真实基线对照；**把包里的 issue 条目导入 GitHub**（先 `-DryRun` 再真建），建立 patch↔issue 映射 |
| S2 | 落提交 | 按序 `git am patches/*.patch`；失败先判基线，再 `--show-current-patch` |
| S3 | 修编译 | 自上而下跑门禁链；语法错会掩盖类型错，每修完一类必须重跑 |
| S4 | 折回 | 把修复折进引入缺陷的那条提交，保持一单一提交 |
| S5 | 作者与推送 | **不传名字/邮箱**运行 `normalize-authors.sh "$BASE"`，身份自动从仓库解析（保留 author date），再 `push --force-with-lease` |
| S6 | 同步 | 强推后 `fetch --prune --force` 再 `reset --hard`，**不要** `git pull` |
| S7 | issue 收口 | 按映射逐条走：已落地的工单评论贴折回 SHA → `close` → 翻标签；没有对应补丁落地的工单保持 open，并把这个原因写进评论 |
| S8 | 清理 | 删掉自己创建的临时内容：解包目录、fix-spec 目录、导入产生的 mapping、日志，逐个验证已消失。不应存在临时克隆——不要为了验收而克隆项目 |

## 包里的 issue，先上传到 GitHub

交接包通常把工单也一起带过来——`issues/*.md`（front matter 带 `key` / `title` / `labels` / `parent` / `blocked_by`），有时只是 `HANDOFF.md` 里的一张工单表。这些工单是补丁序列背后的计划，不该留在 zip 里。**落补丁之前**先把它们推上 GitHub，文档里每一条声称才对得上一个能点开的编号。

1. **预演。** `issue-push.ps1 -DryRun` 只打印将要创建的 issue、父子链接与 `blocked_by` 边，不碰远端。
2. **建单。** 去掉 `-DryRun` 真跑。脚本幂等——重跑只补缺口——并产出 `mapping.json`（本地 `key` → 真实编号）。
3. **建映射。** 用 mapping 拼出 `补丁 NNNN ↔ #编号` 表（文档原本就写了编号的，逐个 `gh issue view` 核对）。S7 就按这张表走；没有编号的补丁，意味着那张工单还没上 GitHub。
4. **回填。** push 会把 `number` 与 `url` 写回 issue 文件——留着它，下一份交接包就不必靠猜来对齐。
5. **项目里没有 `issue-sync` 技能**（它不是基线技能）：按依赖顺序逐条 `gh issue create --title ... --body-file ...`，映射表手工记。慢一点，契约一样。
6. **映射绝不许猜。** 补丁标题与工单标题对不上就直接停手问用户——猜错编号关掉的是别人的工单，比留一张开着更糟。

注意「上传交接包的 issue」与「自己发现的问题」是两件事：你验收中新发现的缺陷**不开新单**，写进报告；用户要留痕的话，就评论在对应工单下面。要建单先问用户。

## issue 收口 —— 交接完成后按映射逐条走

推送完（S5–S6）之后，映射表里每一张工单都要落到一个你说得出的状态：

1. **已落地。** `gh issue comment <n>` 写落地摘要——折回提交 SHA、验收命令、测试通过数——**然后** `gh issue close <n>`，**最后**翻标签（先加新标签再撤旧标签，中间不留无标签态）。不要用 `gh issue close --comment`：对已关闭的 issue 它会丢掉评论却仍返回 0。
2. **没落地。** 补丁被丢弃、折到别处、或过不了门禁的工单保持 **open**，评论里写清是哪一条补丁、为什么。静悄悄关掉，等于让看板谎报发版内容。
3. **自检。** 报告收尾贴出你实际走过的那张表：`#101 → 0001 → 3f2a1c9 已关闭`、`#104 → 0004 → 仍开（补丁丢弃，行为缺口）`，再给一行合计 `已关 / 仍开`。

## 清理 —— 解包出来的临时内容必须消失

交接包里只有 `patches/*.patch` 有长期价值；`HANDOFF.md`、文档附带文件、`issues/` 目录、你自己的 `fix-spec/` 都是脚手架。交接落地并收口之后删干净：

- 解包目录（`handoff-*/`、`mktemp -d` 出来的路径）、fix-spec 目录、落在工作树里的 `mapping.json`、日志、导出中间文件。
- 删之前先 `ls` 列清单（铁律 5），只删自己创建的，禁通配符，不碰 `~/.cache` 这类共享位置。
- 例外：`git am` 中途停下留下的半应用工作树是证据——留着交用户看，用户确认后再删。
- 收尾校验：`git status --porcelain -uall` 为空，解包目录确认已不存在。留一个解包目录，下一份交接包就会和上一份搅在一起。

## 铁律

1. `HANDOFF.md` 与交接目录**永不入库**，只有 `patches/*.patch` 可经 `git am` 进入仓库。
2. 文档里的基线 SHA 不是证据：真实基线是「补丁能干净应用且 API 全对得上」的那个提交，差异要写进报告。
3. 一张子工单对应一个提交；修复折回引入缺陷的提交，不追加 `fix(...)` 提交破坏 1:1 映射。
4. 作者必须是仓库拥有者的身份：**不传名字/邮箱**运行 `normalize-authors.sh`，由脚本从 `origin` 解析 owner、从 `git config user.email` 取邮箱；绝不手填，也不从交接包里抄。
5. 删除任何路径前先输出清单并获得确认，只删自己创建的，禁止通配符。
6. 测试失败先反馈用户，不擅自改断言把失败糊过去。
7. `git am` / `cherry-pick` / `rebase` 都不触发 `pre-commit`，只有 `pre-push` 拦得住；干活前先确认当前工作区钩子已装（若被明确要求隔离到克隆，则先在克隆里装钩子）。
8. **在用户的工作区里干活。** 绝不为验收或落补丁而克隆、复制项目——复杂项目重新克隆加重装依赖要耗掉几十分钟到几小时，收益为零。
9. **包里的 issue 属于 GitHub，不属于 zip。** 落补丁前先导入（先 `-DryRun`），用产出的 `mapping.json` 建立 `补丁 ↔ #编号` 映射；对不上的绝不猜编号。
10. **落地后映射里的工单不许悬着。** 每张单要么带折回 SHA 关闭，要么保持 open 并在评论里写清是哪条补丁、为什么没落地；`gh issue close --comment` 不能替代「先评论再关闭」。
11. **解包临时内容不是产物。** 解包目录、`issues/` 目录、fix-spec、`mapping.json`、日志在 S8 删除，并且要验证已消失，而不是假设。

## 验收判据怎么写

按项目真实门禁链列一张两列表——命令 / 通过标志（在下方项目专属区或引用文件里填）。然后守住三条边界：

- 本地门禁通过只说明你跑的那几条命令过了，不等于 CI 过。CI 通常是超集（格式、语言守卫、提交标题）。
- 测试看汇总行（`failed=0` 与通过数），不要只看没有红字。
- 新增行为要在日志里 grep 到具体测试名，确认它真被执行，而不是被跳过或没命名。

## 测试覆盖不到的地方

编译与测试全绿 ≠ 功能完整。只数元素个数的形状/快照断言，分不清「元素在」和「元素什么都没画」——真实案例里某弹窗给菜单文案预留了容量却从不 push，整列视觉上是空行，测试全过。除跑测试外须亲手抽查新增输出路径（文字、图标、分隔符、副作用），写进报告交用户决定。

## 脚本

| 脚本 | 用途 |
| :--- | :--- |
| `scripts/inspect-handoff-package.sh` | 落库前体检：补丁清单、作者、涉及文件、声称基线与实际基线 |
| `scripts/replay-patch-series.sh` | 按序 `git am`，并把修复文件折回对应提交 |
| `scripts/normalize-authors.sh` | 在提交区间内重写作者为自动解析的仓库拥有者身份（保留 author date）；`--print-identity` 预览目标身份 |
| `scripts/sync-after-force-push.sh` | 强推后安全同步工作树，脏树或有未推送提交时拒绝执行 |

## 参考

- [references/handoff-audit.md](references/handoff-audit.md) —— 声明分类、常见谎报、报告格式
- [references/git-mechanics.md](references/git-mechanics.md) —— 钩子触发面矩阵、`git am` 失败诊断、折叠前提、作者重写
- [references/field-notes.md](references/field-notes.md) —— 一次真实落地的「报错 → 真实 API」对照
- [references/issue-import.md](references/issue-import.md) —— 交接包 issue 的导入、映射表、按映射收口、`gh` 的失败模式
{%- endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- Project-specific area left empty by default (fill in the gate chain, issue repo and repository conventions here; author identity is auto-resolved, do not hand-fill it) -->
{% else %}
<!-- 项目专属区域默认留空（在此填写本项目的门禁链、issue 仓库与仓库约定；作者身份自动解析，不要手填） -->
{% endif %}
