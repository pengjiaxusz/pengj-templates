---
name: branch-sync
description: >-
{% if options["skill_lang"] == "en" %}
  Linear branch/worktree sync with clean history. Identify net-new commits, sync parallel feat branches into the integration branch via rebase/ff-merge or cherry-pick + force-with-lease, and verify no merge commits. Use when merging branches, syncing worktrees, aligning branches, or consolidating parallel worktrees.
  Triggers: branch-sync, worktree sync, sync branch, merge branch, align branch, cherry-pick, rebase, 合并分支, 分支同步.
{% else %}
  多 worktree 并行分支线性化合入集成分支、保证提交记录干净的同步流程。含净贡献甄别、rebase/ff 合并与 cherry-pick 双路径、force-with-lease 推送与合后校验。当用户要求合并分支、同步分支、合入主分支、分支对齐、收编并行 worktree 时使用。
  Triggers: branch-sync, 分支同步, 同步分支, 合并分支, 合入主分支, worktree 同步, cherry-pick, rebase.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# Branch Sync — Worktree-Aware Linear Sync

Sync a parallel feat branch (often held by a worktree) into the integration branch with **linear history, no merge commits, and `--force-with-lease` only**. Failsafe by default: inspect first, deduplicate duplicate commits, then merge via the route that matches the checkout state.

> Convention: this template calls the target `{{"{{ integration }}"}}` (default `main`). Replace it with your project's actual integration branch (`main`/`dev`/`master`). Declare it once in the project-specific area below — the managed block below never hard-codes it.

```
[Pre-check] -> [Net contribution filtering] -> [Route A: no-worktree / free branch | Route B: worktree-occupied] -> [Push] -> [Verify]
```

## 1. Pre-check (see clearly before touching)

Run from repo root:

```powershell
git worktree list          # 1 entry = single-repo (no worktree); >1 = multi-worktree mode
git branch -a
git status --short
# integration = your integration branch (main/dev); source = parallel branch (feat/x)
git log --oneline 'feat/x' '^main'          # exclusive commits on source not in integration
# only if worktree exists:
git -C <worktree-path> status --short       # must be clean before any reset --hard
```

Rules:

- Determine the integration branch first: ask the user if ambiguous; default `main` if `origin/main` exists, otherwise ask explicitly and record it in the project-specific area.
- Decide mode from `git worktree list`: single entry -> **no-worktree (single-repo) mode** -> skip all `git -C <worktree-path>` checks and go directly to Route A; multiple entries -> multi-worktree mode -> check each worktree's cleanliness.
- Quote branch names in PowerShell: `git log --oneline 'feat/x' '^main'` — bare `^`/`..` is parsed by PowerShell.
- Never `reset --hard` or `--force` a dirty worktree — `status --short` must be empty; stash or commit first.

## 2. Net contribution filtering (critical — avoid pulling duplicate commits)

Parallel branches often contain **same title, different hash** commits (both cherry-picked shared fixes). Merge only the truly net-new commits:

1. `git log --oneline 'feat/x' '^main'` — candidate exclusives.
2. Cross-check titles against `git log --oneline main` — drop any candidate whose subject already exists on integration (same change, no need to reintroduce).
3. For remaining candidates, `git show --stat <hash>` to confirm touched files are the intended contribution.
4. Reconcile: `git diff main 'feat/x' --stat` should match the combined `--stat` of the kept candidates — that set is the net contribution. If it doesn't, you missed a duplicate or an unrelated commit.
5. Record the final hash list in order (oldest-first) for cherry-pick.

## 3. Route selection (exactly one)

**Route A — No worktree / single-repo OR source branch is NOT held by a worktree (free to checkout):**

Use this for **both**: (a) you don't use `git worktree` at all (single-repo, `git worktree list` shows 1 entry), and (b) you use worktrees but the source branch is currently free. It is the only route needed when there is no worktree.

```powershell
git checkout 'feat/x'
git rebase main
git checkout main
git merge --ff-only 'feat/x'   # linear fast-forward, no merge commit
git push origin main
# Sync merged branch to latest (make feat/x identical to main)
git checkout 'feat/x'
git reset --hard main
git push --force-with-lease origin 'feat/x'
```

`rebase` keeps periodic sync the same: `git checkout 'feat/x' && git rebase main`. If conflicts arise, resolve, `git rebase --continue`, then continue.

**Route B — Source branch IS held by a worktree (multi-worktree only):**

```powershell
# 1) From main repo: cherry-pick net contributions in order (oldest -> newest)
git checkout main
git cherry-pick <hash-1> <hash-2>   # only the filtered net-new hashes
git push origin main

# 2) In the worktree that holds feat/x:
git fetch origin
git reset --hard origin/main
git push --force-with-lease origin 'feat/x'
```

Never use `git merge <branch>` on the integration branch — `merge:` is not a valid `commitlint` type and will be rejected by the `commit-msg` hook. Linear paths above avoid it by construction.

## 4. Push discipline

- `git push origin main` for the integration branch (fast-forward only after Route A/B).
- `git push --force-with-lease origin 'feat/x'` for the **merged source branch after realignment** — never bare `-f`/`--force`. This applies to **both** no-worktree (Route A) and worktree (Route B) modes: the merged branch must always be fast-forwarded/reset to the integration tip and pushed so `origin/main` and `origin/feat/x` become identical.
- If push is rejected (non-fast-forward): `git fetch` first; if remote hasn't moved -> `--force-with-lease` is safe; if remote has moved -> fetch + rebase/cherry-pick again, then push.

## 5. Post-merge verification (one-shot checklist)

```powershell
git rev-parse HEAD origin/main origin/'feat/x'   # all three should match (local HEAD must be on main)
git diff origin/main origin/'feat/x' --stat       # empty = content identical (proves merged branch synced)
git log --oneline --merges origin/main            # empty = no merge commits
git worktree list                                 # 1 entry = no-worktree; >1 = worktree mode
git status --short                                # clean (single-repo)
# only if worktree exists:
git -C <worktree-path> status --short             # clean
```

Also run the project's single post-merge build/check once on the integration branch (declare the command in the project-specific area; e.g. `cargo test --workspace`, `just ci`, `pnpm build`). Do not run full builds on multiple branches in parallel.

## 6. Traps

- **commitlint has no `merge` type**: don't create merge commits to "fix" a failed merge; re-linearize instead.
- **Chinese/non-ASCII paths**: wrap pathspecs in quotes in PowerShell: `git add "docs/开发规范/file.md"`.
- **Dirty worktree loss**: `reset --hard` discards uncommitted changes with no recovery — verify `status --short` is clean.
- **Stale origin**: always `git fetch origin` before `--force-with-lease`; a stale view makes the lease check meaningless.
- **Rebase vs cherry-pick confusion**: if the source branch is occupied, don't `checkout` it in the main repo — you will get "already checked out" errors; use cherry-pick.

{% else %}
# 分支同步 — Worktree 感知的线性化同步

将并行特性分支（常被某个 worktree 占用）线性化合入集成分支，**历史必须线性、无 merge 提交、force 推送一律 `--force-with-lease`**。先看清、再去重、再按占用状态走对应路径，万无一失。

> 约定：本文以 `{{"{{ 集成分支 }}"}}` 指代目标分支（本项目默认 `main`）。若你的项目以 `dev`/`master` 为统一分支，请先确认并在下方项目专属区声明一次——托管块内不硬编码具体分支名。

```
[前置检查] -> [净贡献甄别] -> [路径 A：无worktree/分支空闲 | 路径 B：被 worktree 占用] -> [推送] -> [校验]
```

## 1. 前置检查（先看清楚再动手）

仓库根执行：

```powershell
git worktree list          # 1 条 = 单仓库（无 worktree）；>1 条 = 多 worktree 模式
git branch -a
git status --short
# 集成分支 = 统一分支（main/dev）；源分支 = 并行分支（feat/x）
git log --oneline 'feat/x' '^main'          # 源分支独有提交
# 仅在存在 worktree 时执行：
git -C <worktree路径> status --short       # 必须干净，reset --hard 前必检
```

规则：

- 先确定集成分支：不确定就问用户；若 `origin/main` 存在则默认 `main`，否则显式询问并记录到项目专属区。
- 由 `git worktree list` 判定模式：1 条记录 -> **无 worktree（单仓库）模式** -> 跳过所有 `git -C <worktree路径>` 检查，直接走路径 A；多条记录 -> 多 worktree 模式 -> 逐个检查 worktree 是否干净。
- PowerShell 下分支名加引号：`git log --oneline 'feat/x' '^main'` — 裸 `^`/`..` 会被 PowerShell 解析。
- 脏 worktree 绝不直接 `reset --hard` / `--force` — 必须 `status --short` 为空，否则先提交或 `stash`。

## 2. 净贡献甄别（关键步骤，避免把重复提交带进集成分支）

并行分支历史常有与集成分支**同名但不同 hash** 的提交（各自 cherry-pick 了公共改动）。合并前必须只取真正净新增的：

1. `git log --oneline 'feat/x' '^main'` 列出候选独有提交；
2. 与 `git log --oneline main` 对照标题，**删掉同名重复项**（内容相同，无需再引入）；
3. 对剩余候选 `git show --stat <hash>` 确认改动文件是否符合预期贡献；
4. 复核：`git diff main 'feat/x' --stat` 的结果应与候选提交的 `--stat` 合计一致 → 该集合即全部净贡献；
5. 按时间正序记录最终 hash 列表，供后续 `cherry-pick` 按序使用。

## 3. 路径选择（二选一）

**路径 A — 无 worktree / 单仓库 或 源分支未被 worktree 占用（可自由 checkout）：**

同时适用于：(a) 完全未使用 `git worktree`（单仓库，`git worktree list` 仅 1 条），(b) 使用 worktree 但源分支当前空闲。无 worktree 时**只走此路径**。

```powershell
git checkout 'feat/x'
git rebase main
git checkout main
git merge --ff-only 'feat/x'   # 线性 fast-forward，无 merge 提交
git push origin main
# 同步被合并分支到最新（使 feat/x 与 main 完全一致）
git checkout 'feat/x'
git reset --hard main
git push --force-with-lease origin 'feat/x'
```

并行分支日常同步也用同一条：`git checkout 'feat/x' && git rebase main`；冲突时解决后 `git rebase --continue` 再继续。

**路径 B — 源分支被 worktree 占用（仅多 worktree 模式）：**

```powershell
# 1) 主仓库：在集成分支上按序 cherry-pick 净贡献（线性，无 merge 提交）
git checkout main
git cherry-pick <净贡献hash-1> <净贡献hash-2>
git push origin main

# 2) 占用该分支的 worktree 目录内：
git fetch origin
git reset --hard origin/main
git push --force-with-lease origin 'feat/x'
```

禁止在集成分支上 `git merge <分支>` 产生 merge 提交 — `merge:` 不在 `commitlint` 的 type 白名单中，会被 `commit-msg` hook 拒绝。上述线性化路径天然规避。

## 4. 推送纪律

- 集成分支：`git push origin main`（路径 A/B 后应为 fast-forward）；
- 源分支对齐后：`git push --force-with-lease origin 'feat/x'` — 绝不裸 `-f`/`--force`；**无论是否使用 worktree，被合并分支都必须 reset 到集成分支最新并推送**，使 `origin/main` 与 `origin/feat/x` 完全一致；
- 推送被拒（non-fast-forward）：先 `git fetch` 看远端是否被他人更新；远端未动 → `--force-with-lease` 安全；远端已动 → 先 fetch + rebase/cherry-pick 再推。

## 5. 合后校验（一条组合替代临场拼凑）

```powershell
git rev-parse HEAD origin/main origin/'feat/x'   # 三处一致（HEAD 应在 main 上）
git diff origin/main origin/'feat/x' --stat       # 为空 = 内容相同（证明被合并分支已同步）
git log --oneline --merges origin/main            # 为空 = 无 merge 提交
git worktree list                                 # 1 条 = 无 worktree；>1 条 = 多 worktree
git status --short                                # 干净（单仓库）
# 仅在存在 worktree 时执行：
git -C <worktree路径> status --short             # 干净
```

并在集成分支上**只跑一次**项目校验（在项目专属区声明命令，如 `cargo test --workspace` / `just ci` / `pnpm build`），不要在多分支上重复全量构建。

## 6. 陷阱

- **commitlint 无 `merge` type**：不要为了解决冲突而补一个 `merge:` / `chore:` 的 merge 提交，应改走线性化。
- **中文文件名路径**：PowerShell 下 `git add` / `git diff` 涉及中文路径时用引号包裹，否则 pathspec 匹配失败。
- **裸 `-f` 丢失他人提交**：一律 `--force-with-lease`，且先 `git fetch`。
- **reset --hard 前必须确认干净**：`git -C <路径> status --short` 有未提交改动会直接丢失。
- **PowerShell 对 `^` 的转义**：`'feat/x' '^main'` 加单引号，否则被当作转义符。
- **构建只跑一次**：合后在集成分支跑一次完整校验即可；多 worktree 重复全量构建浪费且易产生并发产物污染。
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- Below is the project-specific area: template updates replace only the managed block above; this area belongs to the project and is fully preserved. -->

## Project-specific sync policy

> This section belongs to the **project**: template updates only touch the managed block above. Declare your project's concrete choices here.

### Integration branch

- Integration branch for this project: `main` (default). If your project uses `dev`, change this line.
- Remote: `origin`.

### Post-merge verification command (run once on integration branch)

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# or: just ci
```

Adjust to your stack (Rust / frontend / docs) and keep it to a single run on the integration branch after sync.

### Red lines (must never)

- Create a merge commit on the integration branch.
- `git push -f` / `--force` without `--force-with-lease`.
- `git reset --hard` on a dirty worktree.
- Sync without filtering net contributions first.

{% else %}
<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->

## 项目专属同步策略

> 本节归**项目**所有：模板更新只维护上方托管块，这里可按项目实际声明。

### 集成分支

- 本项目统一分支：`main`（默认）。若项目以 `dev` 为统一分支，请改此行。
- 远端：`origin`。

### 合后校验命令（仅在集成分支跑一次）

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# 或：just ci
```

按技术栈调整（Rust / 前端 / 文档），合后仅在集成分支跑一次即可。

### 红线（绝不能做）

- 在集成分支上产生 merge 提交。
- 裸 `git push -f` / `--force`（必须 `--force-with-lease`）。
- 在脏 worktree 上 `git reset --hard`。
- 未甄别净贡献就直接合入。
{% endif %}
