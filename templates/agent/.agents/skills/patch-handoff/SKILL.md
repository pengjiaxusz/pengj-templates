---
name: patch-handoff
description: >-
{% if options["skill_lang"] == "en" %}
  Verify and land an offline patch handoff package (`HANDOFF.md` + `patches/NNNN-*.patch`): replay the series with `git am`, repair the build and tests, fold each repair back into the commit that introduced the defect, normalize authorship to the repository owner's identity (auto-resolved, never hand-filled), force-push with lease, sync the worktree, file every problem found during acceptance as a GitHub issue, close out the referenced tickets plus the issues you filed, and delete the unpacked temp content. Every acceptance claim in the handoff document is an unverified hypothesis until you reproduce it yourself. Use when receiving a patch series or handoff zip from another agent, or when a delivered series does not compile as claimed. Triggers: patch-handoff, handoff package, handoff zip, patch series, git am, apply patches, verify handoff, fold fixes, offline handoff, file handoff issues, close handoff issues.
{% else %}
  验收并合入离线补丁交接包（`HANDOFF.md` + `patches/NNNN-*.patch`）：按序 `git am` 落提交、修编译与测试、把修复折回引入缺陷的提交、作者自动归一为仓库拥有者身份（禁止手填）、强推、同步工作树；验收中发现的问题当场上 GitHub issue，交接完成后逐条收口 issue 状态，并删掉解包出来的临时内容。交接文档里的每一条验收声明，在你亲手复现之前都是未验证假设。当收到别的 agent 产出的补丁集/交接 zip，或交付的补丁集并不像文档说的那样能编译时使用。 Triggers: patch-handoff, 交接包, handoff 包, handoff.zip, 补丁集, patches, git am, 打补丁验收, patch series, 补丁包验收, 校验交接文档, 交接建单, issue 收口.
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
| Issue tracker | The issue numbers the doc cites, plus the repo name | Closing them is part of landing, not an afterthought |
| Issue write target | `gh repo view --json nameWithOwner`; does each cited number exist (`gh issue view <n>`) | On a fork `gh` defaults to the upstream repo — filing there is filing in the wrong place |
| Issue status flow | `gh label list`; the project's in-progress / needs-decision labels | Miss the label and you close tickets without moving the board — the tracker silently rots |

## Quick Start

```bash
PKG=<handoff zip or unpacked dir>
BASE="origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|^origin/||')"
git status -sb; git rev-list --count "$BASE"..HEAD     # must be clean and 0

.agents/skills/patch-handoff/scripts/inspect-handoff-package.sh "$PKG"   # pre-flight audit
.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE" --print-identity  # confirm target identity
.agents/skills/patch-handoff/scripts/replay-patch-series.sh <dir>/patches <fix-spec-dir> "$BASE"

# then run the whole gate chain, no skipped steps
# after folding repairs, normalize authorship and then push:
.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE"   # no name/email: auto from repo
```

## Workflow

| Step | Action | Key point |
| :-- | :--- | :--- |
| S1 | Pre-flight | Inventory patches, authors, touched files; compare claimed baseline with reality; file an issue for every discrepancy you confirm (dedupe first) |
| S2 | Replay | `git am patches/*.patch` in order; on failure diagnose the base first, then `--show-current-patch` |
| S3 | Repair | Run the gate chain top-down; syntax errors mask type errors, so re-run after each fix; log anything you cannot fix as an issue |
| S4 | Fold | Fold each repair into the commit that introduced it, keeping one commit per ticket |
| S5 | Author + push | Run `normalize-authors.sh "$BASE"` **without name/email** so the identity is auto-resolved from the repo (keeping author dates); then `push --force-with-lease` |
| S6 | Resync | After a force push: `fetch --prune --force` then `reset --hard`, never `git pull` |
| S7 | Issues | Close out **both** lines: each implemented ticket (comment, then close, then labels) and every issue you filed during S1–S4 (closed with its fold-back SHA, or left open with a label and an owner) |
| S8 | Cleanup | Delete the unpacked temp content you created: unpack dir, fix-spec dir, logs. Verify it is gone. No throwaway clone should exist — do not clone to test |

## File Every Problem as an Issue — Not Just in the Report

Report prose is read once and then rots; an issue keeps a number somebody can point at. Anything you confirm while verifying gets a GitHub issue **in the same pass**, before you move on:

- The series does not compile as delivered, or the doc's claims do not reproduce.
- Scope gaps: a claimed feature is missing, or a cited ticket has no patch.
- Behavioural gaps the tests cannot see, while the user has not decided yet.
- Anything you deliberately leave unfixed.

Rules for filing:

1. **Deduplicate first.** `gh issue list --state all --search '"<symptom>"'`; if a twin exists, extend it with a comment instead of opening another one.
2. **One issue per distinct defect.** Title `[handoff] <package>: <symptom in one line>`; body carries package + patch number, claim vs reality, the command that reproduces it, and what "fixed" looks like.
3. **Reuse the project's labels** (`gh label list`); do not invent a taxonomy for one handoff.
4. **Back-reference from the report.** Every open-item row names its `#number` — a row without a number is an issue you forgot to file.
5. **No `gh`, no write access, or `gh` pointing at a fork's upstream:** say so in the report and print the command you would have run. Never quietly degrade back to prose.

Filed issues are not decoration: S7 has to give each one a terminal state.

## Issue Close-out — After Landing, Nothing Stays Dangling

Run two lines, both of which must end with a state you can name:

1. **Ticket line** (the tickets the doc maps to): for each one, `gh issue comment <n>` with the landing summary — folded commit SHA, gate commands, test counts — **then** `gh issue close <n>`, **then** flip labels (add the new label before removing the old one, so no issue is ever label-less). Do not use `gh issue close --comment`: on an already-closed issue it drops the comment and still exits 0.
2. **Filed-issue line** (what you opened in S1–S4): fixed in this pass → comment with the fold-back SHA and close it; not fixed, or waiting on the user → keep it open, add the project's decision/blocked label, assign or @-mention whoever moves next, and say so in the report.
3. **Self-check.** `gh issue list --state open --search '[handoff]'` must leave only issues you can justify. End the report with three lines: `closed #…` / `filed #…` / `still open #…, why`.

## Cleanup — the Unpacked Temp Content Must Go

Only `patches/*.patch` has lasting value in a handoff package; `HANDOFF.md`, the doc's side files and your `fix-spec/` tree are scaffolding. Once the handoff is landed, delete them:

- Unpack dir (`handoff-*/`, a `mktemp -d` path), fix-spec dir, logs, exported intermediates.
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
9. **A problem you found but did not file does not exist.** Every confirmed defect goes to GitHub as an issue in the pass that found it — deduped, with the reproducing command — and the report cites its number.
10. **Nothing in scope stays dangling after landing.** Tickets and filed issues both end closed (with the fold-back SHA) or open with a label and a named owner. `gh issue close --comment` is not a substitute for comment-then-close.
11. **The unpacked temp content is not an artifact.** Unpack dir, fix-spec dir and logs are deleted at S8, and you verify they are gone rather than assuming.

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
- [references/issue-closeout.md](references/issue-closeout.md) — when to file, the issue body template, dedupe, the state machine, `gh` failure modes
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
| issue 仓库 | 文档引用的编号 + 仓库名 | 关单是落地的一部分，不是收尾补充 |
| issue 写入目标 | `gh repo view --json nameWithOwner`；编号在本仓库是否真存在（`gh issue view <n>`） | fork 里 `gh` 默认指向上游仓库，发错仓等于白发 |
| issue 状态流 | `gh label list`；项目现有的在办 / 待定标签 | 不知道翻哪个标签，结果就是只关单不翻标签，看板从此失真 |

## 快速开始

```bash
PKG=<交接 zip 或已解包目录>
BASE="origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|^origin/||')"
git status -sb; git rev-list --count "$BASE"..HEAD     # 必须干净且为 0

.agents/skills/patch-handoff/scripts/inspect-handoff-package.sh "$PKG"   # 落库前体检
.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE" --print-identity  # 确认归一目标身份
.agents/skills/patch-handoff/scripts/replay-patch-series.sh <dir>/patches <fix-spec-dir> "$BASE"

# 然后整条门禁链全跑，不跳步
# 折回修复后归一作者，再推送：
.agents/skills/patch-handoff/scripts/normalize-authors.sh "$BASE"   # 不传名字/邮箱：自动从仓库解析
```

## 工作流

| 步 | 动作 | 要点 |
| :-- | :--- | :--- |
| S1 | 体检 | 清点补丁、作者、涉及文件；把文档声称的基线与真实基线对照；确认的每处不符当场上 issue（先查重） |
| S2 | 落提交 | 按序 `git am patches/*.patch`；失败先判基线，再 `--show-current-patch` |
| S3 | 修编译 | 自上而下跑门禁链；语法错会掩盖类型错，每修完一类必须重跑；修不掉的立刻建单 |
| S4 | 折回 | 把修复折进引入缺陷的那条提交，保持一单一提交 |
| S5 | 作者与推送 | **不传名字/邮箱**运行 `normalize-authors.sh "$BASE"`，身份自动从仓库解析（保留 author date），再 `push --force-with-lease` |
| S6 | 同步 | 强推后 `fetch --prune --force` 再 `reset --hard`，**不要** `git pull` |
| S7 | issue 收口 | **两条线都要收**：逐条关闭已实现工单（先评论、再关闭、后翻标签），以及 S1–S4 建的问题单（已修的贴折回 SHA 关闭，未修的带标签留开并指明下一步归谁） |
| S8 | 清理 | 删掉自己创建的临时内容：解包目录、fix-spec 目录、日志，逐个验证已消失。不应存在临时克隆——不要为了验收而克隆项目 |

## 交接时发现的问题，直接落成 issue

报告里的文字只会被读一遍，然后烂掉；issue 有个编号，谁都能指着它说话。验收过程中确认的每一处问题，**在发现它的那一轮就建单**，不要攒：

- 交付的补丁集编译不过，或文档声称的结论复现不出来。
- 范围缺口：声称实现了但补丁里没有，或文档点名的工单没有对应补丁。
- 测试看不见的行为缺口，而用户还没拍板。
- 你主动决定不修的任何东西。

建单规矩：

1. **先查重。** `gh issue list --state all --search '"<症状>"'`；已有同因的单就补一条评论，不要开第二张。
2. **一个缺陷一张单。** 标题 `[handoff] <包名>: <一句话症状>`；正文写清：包名 + 补丁编号、声称 vs 实测、能复现的命令、什么算修好。
3. **用项目已有标签**（`gh label list`），不要为一次交接自造一套标签体系。
4. **报告里回指编号。** 每条遗留行都要带 `#编号`——没有编号的那一行，就是你忘了建的单。
5. **没有 `gh`、没有写权限、或 `gh` 指向 fork 的上游仓库**：在报告里明说，并把你本该执行的命令原样贴出来。绝不允许静默退回成纯文字。

建单不是装饰：S7 必须给每一张单一个终态。

## issue 收口 —— 交接完成后，一条都不许悬着

两条线都跑，每条都要落到一个你说得出的状态：

1. **工单线**（文档映射的 ticket）：逐条 `gh issue comment <n>` 写落地摘要——折回提交 SHA、验收命令、测试通过数——**然后** `gh issue close <n>`，**最后**翻标签（先加新标签再撤旧标签，中间不留无标签态）。不要用 `gh issue close --comment`：对已关闭的 issue 它会丢掉评论却仍返回 0。
2. **问题线**（你在 S1–S4 建的单）：本轮已修掉的 → 评论贴折回 SHA 再关闭；没修、或在等用户决定的 → 保持 open，补上项目的待定/阻塞标签，指派或 @ 到下一步的负责人，并在报告里写明。
3. **自检。** `gh issue list --state open --search '[handoff]'` 剩下的必须只剩你解释得清的那几张。报告收尾固定三行：`已关闭 #…` / `新建 #…` / `仍开 #…，原因`。

## 清理 —— 解包出来的临时内容必须消失

交接包里只有 `patches/*.patch` 有长期价值；`HANDOFF.md`、文档附带文件、你自己的 `fix-spec/` 都是脚手架。交接落完之后删干净：

- 解包目录（`handoff-*/`、`mktemp -d` 出来的路径）、fix-spec 目录、日志、导出中间文件。
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
9. **发现了却没建单的问题等于不存在。** 每处确认的缺陷都在发现它的那一轮进 GitHub issue（先查重、带复现命令），报告里引用编号。
10. **落地后范围内的 issue 不许悬着。** 工单与你建的单，一律以「带折回 SHA 的关闭」或「带标签 + 明确责任人的留开」收尾；`gh issue close --comment` 不能替代「先评论再关闭」。
11. **解包临时内容不是产物。** 解包目录、fix-spec、日志在 S8 删除，并且要验证已消失，而不是假设。

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
- [references/issue-closeout.md](references/issue-closeout.md) —— 何时建单、正文模板、查重、状态机、`gh` 的失败模式
{%- endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- Project-specific area left empty by default (fill in the gate chain, issue repo and repository conventions here; author identity is auto-resolved, do not hand-fill it) -->
{% else %}
<!-- 项目专属区域默认留空（在此填写本项目的门禁链、issue 仓库与仓库约定；作者身份自动解析，不要手填） -->
{% endif %}
