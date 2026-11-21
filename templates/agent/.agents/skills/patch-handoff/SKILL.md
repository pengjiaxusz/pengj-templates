---
name: patch-handoff
description: >-
{% if options["skill_lang"] == "en" %}
  Verify and land an offline patch handoff package (`HANDOFF.md` + `patches/NNNN-*.patch`): replay the series with `git am`, repair the build and tests, fold each repair back into the commit that introduced the defect, normalize authorship, force-push with lease, sync the worktree, and close the tracked issues. Every acceptance claim in the handoff document is an unverified hypothesis until you reproduce it yourself. Use when receiving a patch series or handoff zip from another agent, or when a delivered series does not compile as claimed. Triggers: patch-handoff, handoff package, handoff zip, patch series, git am, apply patches, verify handoff, fold fixes, offline handoff.
{% else %}
  验收并合入离线补丁交接包（`HANDOFF.md` + `patches/NNNN-*.patch`）：按序 `git am` 落提交、修编译与测试、把修复折回引入缺陷的提交、作者归一、强推、同步工作树并关闭对应 issue。交接文档里的每一条验收声明，在你亲手复现之前都是未验证假设。当收到别的 agent 产出的补丁集/交接 zip，或交付的补丁集并不像文档说的那样能编译时使用。 Triggers: patch-handoff, 交接包, handoff 包, handoff.zip, 补丁集, patches, git am, 打补丁验收, patch series, 补丁包验收, 校验交接文档.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" -%}
# Patch Handoff — Verify & Land an Offline Patch Series

Turn another agent's offline handoff package into commits that actually live in the repo, and prove it with the project's real gates instead of the document's claims.

**Assume the document is wrong until reproduced.** Handoff packages routinely claim `git am` / lint / tests all passed while the series does not compile and the recorded baseline SHA is off. Treat every claim as a hypothesis, reproduce it, and state the outcome in your report.

## Before Touching Anything: Pin Six Parameters

Read them out of the repo, never out of `HANDOFF.md`.

| Parameter | Where to get it | Why it matters |
| :--- | :--- | :--- |
| Base ref | `git rev-parse --abbrev-ref origin/HEAD`; compare with the SHA the doc claims | Most `git am` failures are a wrong base, not a bad patch |
| Gate chain | `justfile` / `Makefile` / `package.json` / `.github/workflows/*.yml` | Your locally chosen commands are not the CI gate |
| Format check | Whether CI runs `fmt --check` / `lint` as a separate step | A linter passing says nothing about formatting |
| Hook installation | `git config core.hooksPath`; fresh clones need the installer | Hooks are per-clone config and do not travel |
| Author identity | `git config user.email`, or the repo's stated convention | Package authors are usually synthetic placeholders |
| Issue tracker | The issue numbers the doc cites, plus repo and label names | Closing them is part of landing, not an afterthought |

## Quick Start

```bash
PKG=<handoff zip or unpacked dir>
BASE="origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|^origin/||')"
git status -sb; git rev-list --count "$BASE"..HEAD     # must be clean and 0

.agents/skills/patch-handoff/scripts/inspect-handoff-package.sh "$PKG"   # pre-flight audit
.agents/skills/patch-handoff/scripts/replay-patch-series.sh <dir>/patches <fix-spec-dir> "$BASE"

# then run the whole gate chain, no skipped steps
```

## Workflow

| Step | Action | Key point |
| :-- | :--- | :--- |
| S1 | Pre-flight | Inventory patches, authors, touched files; compare claimed baseline with reality |
| S2 | Replay | `git am patches/*.patch` in order; on failure diagnose the base first, then `--show-current-patch` |
| S3 | Repair | Run the gate chain top-down; syntax errors mask type errors, so re-run after each fix |
| S4 | Fold | Fold each repair into the commit that introduced it, keeping one commit per ticket |
| S5 | Author + push | Normalize authorship (keep author dates), then `push --force-with-lease` |
| S6 | Resync | After a force push: `fetch --prune --force` then `reset --hard`, never `git pull` |
| S7 | Issues | Close each implemented ticket with a per-ticket summary comment, then flip status labels |
| S8 | Cleanup | Delete only what you created: unpack dir, logs, throwaway clones |

## Iron Rules

1. `HANDOFF.md` and the handoff directory never enter the repo; only `patches/*.patch` may arrive via `git am`.
2. The claimed baseline SHA is not evidence: the real base is the commit where the series applies cleanly and every API reference lines up. Report the discrepancy.
3. One commit per ticket. Fold repairs back into the offending commit; do not append `fix(...)` commits that break the 1:1 mapping.
4. Authorship must be the repo's real identity; rewrite any synthetic author the package shipped with.
5. Before deleting anything, print the list and get confirmation. Delete only what you created. No wildcards.
6. A failing test is reported to the user, not silently accommodated by editing the assertion.
7. `git am`, `cherry-pick` and `rebase` trigger no `pre-commit` hook — only `pre-push` can still catch the damage. Install hooks in any throwaway clone before you work there.

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
| `scripts/normalize-authors.sh` | Rewrite authorship over a commit range, preserving author dates |
| `scripts/sync-after-force-push.sh` | Safe post-force-push worktree resync, refuses on dirty or unpushed state |

## References

- [references/handoff-audit.md](references/handoff-audit.md) — classifying claims, the usual lies, report format
- [references/git-mechanics.md](references/git-mechanics.md) — hook trigger matrix, `git am` failure triage, fold preconditions, author rewrite
- [references/field-notes.md](references/field-notes.md) — real error-to-API table from one landed series
{%- else -%}
# 补丁交接包验收与合入（Patch Handoff）

把别的 agent 产出的离线交接包，变成真正落在仓库里的提交，并用项目真实门禁证明它成立——而不是用文档的声称。

**默认文档不可信，直到你复现它。** 这类包常声称 `git am` / lint / 测试全部通过，实测常见「打完根本编译不过」且基线 SHA 写错。文档里的每条声明都当假设，逐条复现，把结论写进报告。

## 动手前先钉六个参数

参数一律从仓库读，不从 `HANDOFF.md` 读。

| 参数 | 取值处 | 为什么必须钉 |
| :--- | :--- | :--- |
| 基线 ref | `git rev-parse --abbrev-ref origin/HEAD`，并与文档声称的 SHA 对照 | `git am` 失败多数是基线不对，不是补丁坏 |
| 验收命令链 | `justfile` / `Makefile` / `package.json` / `.github/workflows/*.yml` | 你自选的命令不等于 CI 门禁 |
| 格式检查位置 | CI 是否有独立的 `fmt --check` / `lint` 步骤 | linter 过了不代表格式过 |
| 钩子安装 | `git config core.hooksPath`；新克隆要跑仓库的安装脚本 | 钩子是每克隆自己的配置，不随 clone 传播 |
| 作者身份 | `git config user.email` 或仓库明文约定 | 包里的作者多为合成占位身份 |
| issue 仓库 | 文档引用的编号 + 仓库名 + 状态标签 | 关单是落地的一部分，不是收尾补充 |

## 快速开始

```bash
PKG=<交接 zip 或已解包目录>
BASE="origin/$(git rev-parse --abbrev-ref origin/HEAD | sed 's|^origin/||')"
git status -sb; git rev-list --count "$BASE"..HEAD     # 必须干净且为 0

.agents/skills/patch-handoff/scripts/inspect-handoff-package.sh "$PKG"   # 落库前体检
.agents/skills/patch-handoff/scripts/replay-patch-series.sh <dir>/patches <fix-spec-dir> "$BASE"

# 然后整条门禁链全跑，不跳步
```

## 工作流

| 步 | 动作 | 要点 |
| :-- | :--- | :--- |
| S1 | 体检 | 清点补丁、作者、涉及文件；把文档声称的基线与真实基线对照 |
| S2 | 落提交 | 按序 `git am patches/*.patch`；失败先判基线，再 `--show-current-patch` |
| S3 | 修编译 | 自上而下跑门禁链；语法错会掩盖类型错，每修完一类必须重跑 |
| S4 | 折回 | 把修复折进引入缺陷的那条提交，保持一单一提交 |
| S5 | 作者与推送 | 归一作者（保留 author date），再 `push --force-with-lease` |
| S6 | 同步 | 强推后 `fetch --prune --force` 再 `reset --hard`，**不要** `git pull` |
| S7 | issue | 逐条关闭已实现工单并写单条摘要评论，再翻状态标签 |
| S8 | 清理 | 只删自己创建的：解包目录、日志、临时克隆 |

## 铁律

1. `HANDOFF.md` 与交接目录**永不入库**，只有 `patches/*.patch` 可经 `git am` 进入仓库。
2. 文档里的基线 SHA 不是证据：真实基线是「补丁能干净应用且 API 全对得上」的那个提交，差异要写进报告。
3. 一张子工单对应一个提交；修复折回引入缺陷的提交，不追加 `fix(...)` 提交破坏 1:1 映射。
4. 作者必须是仓库的真实身份，包内自带的合成身份必须重写。
5. 删除任何路径前先输出清单并获得确认，只删自己创建的，禁止通配符。
6. 测试失败先反馈用户，不擅自改断言把失败糊过去。
7. `git am` / `cherry-pick` / `rebase` 都不触发 `pre-commit`，只有 `pre-push` 拦得住；临时克隆里先装钩子再干活。

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
| `scripts/normalize-authors.sh` | 在提交区间内重写作者，保留 author date |
| `scripts/sync-after-force-push.sh` | 强推后安全同步工作树，脏树或有未推送提交时拒绝执行 |

## 参考

- [references/handoff-audit.md](references/handoff-audit.md) —— 声明分类、常见谎报、报告格式
- [references/git-mechanics.md](references/git-mechanics.md) —— 钩子触发面矩阵、`git am` 失败诊断、折叠前提、作者重写
- [references/field-notes.md](references/field-notes.md) —— 一次真实落地的「报错 → 真实 API」对照
{%- endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- Project-specific area left empty by default (fill in the gate chain, author identity, issue repo and repository conventions here) -->
{% else %}
<!-- 项目专属区域默认留空（在此填写本项目的门禁链、作者身份、issue 仓库与仓库约定） -->
{% endif %}
