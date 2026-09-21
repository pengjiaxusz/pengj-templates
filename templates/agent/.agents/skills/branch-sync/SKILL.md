---
name: branch-sync
description: >-
{% if options["skill_lang"] == "en" %}
  Fast worktree-aware linear branch sync. Automates net-new commit identification via patch-id, rebase/ff or cherry-pick sync into integration branch, safe force-with-lease push, and verification with minimal tool calls and token usage. Use when merging branches, syncing worktrees, aligning branches, or consolidating parallel worktrees.
  Triggers: branch-sync, worktree sync, sync branch, merge branch, align branch, cherry-pick, rebase.
{% else %}
  极速且 Worktree 感知的线性化分支同步流程。自动化 patch-id 净贡献甄别、rebase/ff 合并与 cherry-pick 双路径、force-with-lease 推送与合后自检。内置脚本与复合单行流，大幅减少工具调用次数与 Token 消耗。当用户要求合并分支、同步分支、合入主分支、分支对齐、收编并行 worktree 时使用。
  Triggers: branch-sync, 分支同步, 同步分支, 合并分支, 合入主分支, worktree 同步, cherry-pick, rebase.
{% endif %}
---
{% if options["skill_lang"] == "en" %}
<!-- PENGJ_TEMPLATE_START -->
# Branch Sync — Fast Worktree-Aware Linear Sync

Sync a parallel feat branch (often held by a worktree) into the integration branch with **linear history, no merge commits, and `--force-with-lease` only**.

> Convention: `{{"{{ integration }}"}}` refers to the target integration branch (default `main`). Declare deviations (e.g. `dev`/`master`) once in the project-specific area below.

```
[Fast-Track: sync-branch.ps1] OR [Inspection (git cherry -v)] -> [Route A: Free / Route B: Occupied] -> [Push & Verify]
```

## ⚡ Fast-Track Workflow (Recommended: 1–2 Tool Calls)

Use the bundled script `.agents/skills/branch-sync/scripts/sync-branch.ps1` to automate topology detection, patch-level deduplication, branch alignment, safe push, and verification in a single run:

```powershell
# 1. Quick Dry Run: Check worktree topology & net-new commits in ~1s (read-only)
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x'

# 2. One-shot Execution: Rebase/ff or cherry-pick, align source branch, push, & verify
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x' -Apply
```

> **Efficiency Principle**: When the user requests a merge/sync and the source branch is known, run `-Apply` directly in **1 single tool call**. The script automatically verifies clean worktrees (respecting project-declared ignore regex), infers the integration branch (`origin/HEAD` -> `SKILL.md` declaration -> probe), identifies net commits, syncs both branches, and performs post-merge verification.

---

## 🛠️ Manual Fallback (Chained One-Liners)

If the script environment is unavailable, use chained compound commands. **Never execute git commands line-by-line across multiple tool rounds, and never print raw full git logs to manually compare subjects.**

### 1. Fast Topology & Net Contribution Inspection (1 Tool Call)

```powershell
git worktree list; git cherry -v main 'feat/x'
```

- **Topology decision**:
  - `git worktree list` has 1 entry or `feat/x` is not checked out elsewhere -> **Route A**;
  - `feat/x` is checked out in another worktree path -> **Route B**.
- **Net-contribution rules**:
  - Lines with `+ <hash>`: Truly net-new commits to be merged.
  - Lines with `- <hash>`: Already applied in `main` with an identical patch -> automatically ignored!

### 2. Chained Execution (1 Tool Call)

**Route A — Single-repo OR source branch is free:**
```powershell
git checkout 'feat/x' && git rebase main && git checkout main && git merge --ff-only 'feat/x' && git push origin main && git checkout 'feat/x' && git reset --hard main && git push --force-with-lease origin 'feat/x' && git checkout main
```

**Route B — Source branch is occupied by another worktree:**
```powershell
# 1) Main repo: cherry-pick net-new hashes in order and push
git checkout main && git cherry-pick <net-hash-1> <net-hash-2> && git push origin main

# 2) In occupied worktree: sync and push
git -C <worktree-path> fetch origin && git -C <worktree-path> reset --hard origin/main && git -C <worktree-path> push --force-with-lease origin 'feat/x'
```

### 3. Post-Merge Verification (1 Tool Call)

```powershell
git rev-parse HEAD origin/main origin/'feat/x'; git diff origin/main origin/'feat/x' --stat; git log --oneline --merges -n 5 origin/main; git status --short
```

Verification goals:
1. `HEAD`, `origin/main`, `origin/feat/x` all point to the same commit;
2. `git diff` is empty (source branch fully aligned);
3. No merge commits (`--merges` output is empty);
4. Workspace is clean.

Run the project build check **once** on the integration branch (e.g. `just ci`, `cargo test --workspace`).

## Guardrails & Traps
- **commitlint rejection**: Never create merge commits (`merge: ...`). Always use linear rebase/ff or cherry-pick.
- **Force push discipline**: Always use `--force-with-lease` after `git fetch`; never bare `-f` / `--force`.
- **Both branches aligned**: Always realign and push the source branch after merging so `origin/main` and `origin/feat/x` match.
- **Dirty worktree loss**: Never run `reset --hard` when uncommitted changes exist.
<!-- PENGJ_TEMPLATE_END -->

<!-- Project-specific area below -->
## Project-Specific Configuration & Verification

> This section belongs to the **project**. Template updates will only replace the managed block above.

### Integration Branch Declaration
- Integration branch: `main`

### Untracked Path Ignore Regex (Optional)
Declare regex patterns for untracked/generated directories to ignore during workspace dirty checks:
- Dirty ignore regex: ``

### Post-Merge Validation Command
Declare the single post-merge verification command to run on integration:
```powershell
# cargo test --workspace
# just ci
```
{% else %}
<!-- PENGJ_TEMPLATE_START -->
# 分支同步 — Worktree 感知的极速线性化同步

将并行特性分支（常被某个 worktree 占用）线性化合入集成分支，**历史必须严格线性、零 merge 提交、强制推送一律 `--force-with-lease`**。

> 约定：本文以 `{{"{{ 集成分支 }}"}}` 指代目标分支（本项目默认 `main`）。若项目以 `dev`/`master` 为统一分支，请在下方项目专属区声明。

```
[极速通道: sync-branch.ps1] 或 [原生甄别 (git cherry -v)] -> [路径 A: 自由分支 / 路径 B: Worktree 占用] -> [推送与校验]
```

## ⚡ 极速自动化通道（推荐：1~2 次工具调用）

优先运行内置自动化脚本 `.agents/skills/branch-sync/scripts/sync-branch.ps1`，毫秒级完成拓扑探测、patch-id 净贡献甄别、双向安全合入对齐、推送与合后自检：

```powershell
# 1. 快速预检：1 秒内看清工作区拓扑、占用状态与净新增提交（只读预览）
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x'

# 2. 一键执行：线性变基/快进或 cherry-pick、安全同步被合并分支、--force-with-lease 推送并自检
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x' -Apply
```

> **提速与省 Token 准则**：在源分支明确且用户要求合并同步时，Agent 可直接执行带 `-Apply` 的单次调用。脚本会自动拦截脏工作区（支持项目忽略正则）、自适应探测集成分支（`origin/HEAD` -> `SKILL.md` 登记 -> 常见分支嗅探）、自动甄别净贡献、自动对齐双端分支并完成合后校验。

---

## 🛠️ 手动精简流水线（复合单行命令）

若无法运行脚本，使用以下**复合单行命令**执行。**严禁拆成数十次零散工具调用，严禁打印全文 git log 让大模型肉眼比对提交标题**！

### 1. 拓扑与净贡献一键甄别（1 次调用）

```powershell
git worktree list; git cherry -v main 'feat/x'
```

- **拓扑判断**：
  - `git worktree list` 仅 1 条或源分支未被其他 worktree 检出 ➔ **路径 A**；
  - 源分支已被其他 worktree 检出 ➔ **路径 B**（记录其路径）。
- **净贡献法则（基于 patch-id 自动比对）**：
  - `+ <hash>`：真正净新增的独有提交；
  - `- <hash>`：上游已存在相同 patch-id 的等价改动，**自动忽略**。

### 2. 复合执行流水线（1 次调用）

**路径 A — 单仓库 或 源分支未被占用（自由 checkout）：**
```powershell
git checkout 'feat/x' && git rebase main && git checkout main && git merge --ff-only 'feat/x' && git push origin main && git checkout 'feat/x' && git reset --hard main && git push --force-with-lease origin 'feat/x' && git checkout main
```

**路径 B — 源分支被其他 worktree 占用：**
```powershell
# 1) 主仓库：只 cherry-pick 真正净新增的 hash 列表并推送
git checkout main && git cherry-pick <净贡献hash-1> <净贡献hash-2> && git push origin main

# 2) 占用源分支的 worktree 目录：拉取、对齐并安全推送
git -C <worktree路径> fetch origin && git -C <worktree路径> reset --hard origin/main && git -C <worktree路径> push --force-with-lease origin 'feat/x'
```

### 3. 一键合后校验（1 次调用）

```powershell
git rev-parse HEAD origin/main origin/'feat/x'; git diff origin/main origin/'feat/x' --stat; git log --oneline --merges -n 5 origin/main; git status --short
```

校验合格标准：
1. `HEAD`、`origin/main`、`origin/feat/x` 三处 commit hash 完全一致；
2. 两分支 diff 为空（内容完全对齐）；
3. 无 merge 提交（`--merges` 为空）；
4. 工作区干净。

在集成分支上**只跑一次**项目构建校验（如 `just ci` / `cargo test --workspace` / `pnpm build`）。

## 红线与避坑
- **禁止 merge 提交**：commitlint 无 `merge:` 类型，必须走线性 fast-forward 或 cherry-pick。
- **强制推送纪律**：一律先 fetch 后 `--force-with-lease`，严禁裸 `-f`。
- **双端必须对齐**：被合并分支无论是否在 worktree 中，都必须 reset 到集成分支最新并推送，保持双端一致。
- **工作区防丢**：执行 `reset --hard` 前必须确认 `status --short` 无未提交改动。
<!-- PENGJ_TEMPLATE_END -->

<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属分支配置与验证

> 本节归**项目**所有：模板更新只维护上方托管块，这里声明具体分支名与项目门禁。

### 集成分支登记
- 集成分支：`main`

### 忽略未追踪路径正则（可选）
声明工作区脏检查时需忽略的本地未追踪/生成目录正则：
- 忽略正则：``

### 合后验证命令
在集成分支运行一次项目专属验证：
```powershell
# cargo test --workspace
# just ci
```
{% endif %}
