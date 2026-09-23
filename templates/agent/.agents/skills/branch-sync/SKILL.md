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

Sync a feature branch (single-repo or worktree-occupied) into the integration branch with **safety backup references (refs/sync-backup/), Tree-Diff Guard revision preservation, remote-alignment anti-loss checks, and 1-shot closed-loop verification**.
History must be strictly linear, zero merge commits, and force pushes must use `--force-with-lease`.

> Convention: `{{"{{ integration }}"}}` refers to the target integration branch (default `main`). Declare deviations (e.g. `dev`/`master`) once in the project-specific area below.

```
[1-Shot One-Line Execution: sync-branch.ps1 -Apply] 
  ├── 1. Adaptive topology & branch auto-detection
  ├── 2. Remote alignment check (fast-forward remote commits, prevent drops)
  ├── 3. Safety snapshot reference (refs/sync-backup/ permanent protection)
  ├── 4. Linear merge (Route A: rebase-ff / Route B: ordered cherry-pick)
  ├── 5. Tree-Diff Guard (strictly verifies 100% changes preserved before resetting source)
  ├── 6. Source branch realignment & safe push (--force-with-lease)
  └── 7. Automated post-merge test command (e.g. cargo test) -> Final status dashboard
```

## ⚡ Fast-Track Workflow (Recommended: 1 Single Tool Call)

When requested to merge, sync, or align branches, **directly execute the script with `-Apply` in 1 single tool call**. The script handles end-to-end safety checks, sync, push, and verification:

```powershell
# 1. Recommended: 1-Shot merge, push, and test in a single tool call
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x' -Apply

# 2. Auto-inference: Automatically detects current feature branch when omitted
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -Apply

# 3. Dry-run only (read-only preview of topology and net commits)
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x'
```

> **Efficiency & Low-Intelligence Guardrails (Hard Rules)**:
> 1. **1-Shot to Completion**: Do NOT split into multiple tool calls (dry-run -> apply -> build check). Run `-Apply` directly; the script inspects worktrees, prevents dirty overwrites, merges, pushes, and automatically runs the project's verification test (e.g. `cargo test`).
> 2. **Never Hand-Craft Raw Git Commands**: Do NOT attempt manual `git merge` (violates commitlint) or manual `reset --hard` (causes irrevocable commit drops). Everything must go through `sync-branch.ps1`.
> 3. **Dashboard Decides Completion**: When the script reports `STATUS: COMPLETED_READY_TO_REPORT`, all synchronization, pushes, and test checks have succeeded. Immediately report completion to the user without redundant tool calls.

---

## 🛡️ Anti-Drop & Anti-Overwrite Safeguards

1. **Safety Backup Reference (refs/sync-backup/)**:
   Before performing any destructive operation, the script snapshots branches to `refs/sync-backup/<branch>/<timestamp>-<sha>`. Any interrupted or failed operation can be restored instantly via `git branch -f <branch> <backup-ref>`.
2. **Remote Alignment Guard**:
   Compares local and `origin/<branch>` before merging: auto-fast-forwards if remote is ahead (preventing dropped remote commits) and blocks immediately if diverged.
3. **Tree-Diff Guard**:
   Before resetting the source branch, the script rigorously audits the integration branch:
   **If any net commit from the source branch is missing, the source branch is NEVER reset or pushed**, and integration is rolled back automatically.

---

## 🛠️ Emergency Fallback (Only when PowerShell script execution is impossible)

If operating in a restricted environment without PowerShell:

```powershell
# 1. Create safety snapshot
git update-ref refs/sync-backup/feat_x/temp HEAD

# 2. Route A (free branch): rebase -> merge ff -> push -> realign source -> push source
git checkout 'feat/x' && git rebase main && git checkout main && git merge --ff-only 'feat/x' && git push origin main && git checkout 'feat/x' && git reset --hard main && git push --force-with-lease origin 'feat/x' && git checkout main

# 3. Post-merge validation
cargo test --workspace
```

## Guardrails & Traps
- **No merge commits**: commitlint rejects `merge:`. Always use linear rebase/ff or cherry-pick.
- **Force push discipline**: Always use `--force-with-lease` after `git fetch`; never bare `-f`.
- **Workspace cleanliness**: Never run resets when uncommitted changes exist.
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

将特性分支（支持单仓库或由 Worktree 占用）线性化合入集成分支，内置 **持久化快照兜底（refs/sync-backup/）、Tree-Diff Guard 树级改动保全校验、双端对齐防漏核查与项目门禁 1-Shot 闭环**。
历史必须严格线性、零 merge 提交、强制推送一律 `--force-with-lease`。

> 约定：本文以 `{{"{{ 集成分支 }}"}}` 指代目标分支（本项目默认 `main`）。若项目以 `dev`/`master` 为统一分支，请在下方项目专属区声明。

```
[一键执行 1-Shot: sync-branch.ps1 -Apply] 
  ├── 1. 拓扑与分支自适应探测
  ├── 2. 远端双向快进与对齐检测 (防漏远端提交)
  ├── 3. 自动建立持久化安全快照 (refs/sync-backup/ 永久防丢)
  ├── 4. 线性合入 (Route A: 变基快进 / Route B: 按序 cherry-pick)
  ├── 5. Tree-Diff Guard 树级防漏审计 (未 100% 合入绝不重置源分支)
  ├── 6. 同步对齐源分支 & --force-with-lease 安全推送
  └── 7. 自动运行项目合后门禁 (如 cargo test) -> 输出最终看板
```

## ⚡ 极速自动化通道（强制推荐：单次工具调用 1-Shot）

当用户要求合并分支、同步分支或分支对齐时，**直接执行带 `-Apply` 的单次调用**。脚本自动化闭环全流程，杜绝提交遗漏与误覆盖，大幅节省 Token 与调用耗时：

```powershell
# 1. 推荐：指定源分支一键合并、推送与验证（1 次调用闭环）
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x' -Apply

# 2. 智能探测：当前已位于特性分支时，可直接省略分支名
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -Apply

# 3. 仅预检（只读预览拓扑与净贡献，不修改任何分支）
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x'
```

> **极速与低智力防呆准则（硬性红线）**：
> 1. **单次调用直达终态**：不要先跑 Dry Run 再跑 Apply 再跑构建！直接运行 `-Apply`，脚本内部会自动安全预检、拦截脏工作区、完成合并并在末尾自动执行项目的合后门禁命令（如 `cargo test`）。
> 2. **严禁手动拼接 Git 原生命令**：严禁自行执行 `git merge`（产生 merge 提交破坏规范）、严禁手动 `git reset --hard`（极易造成未合入提交永久丢失）。所有操作必须且仅需通过 `sync-branch.ps1` 托管执行。
> 3. **看板判定即交差**：当脚本输出 `STATUS: COMPLETED_READY_TO_REPORT` 时，代表分支同步、推送与项目门禁已全部通过，无需追加任何工具调用，直接向用户汇报即可。

---

## 🛡️ 防漏提交与防覆盖硬核机制

1. **自动持久化快照 (Safety Backup Ref)**：
   每次执行 `-Apply` 前，自动在本地写入快照指针：
   `refs/sync-backup/<分支名>/<时间戳>-<SHA>`
   若有任何人为中止或异常，原分支所有提交永远有 ref 保护，绝不沦为悬空提交，随时可通过 `git branch -f <分支名> <快照Ref>` 毫秒级无损还原。
2. **双端对齐检测 (Remote vs Local Guard)**：
   自动检测本地分支与远端 `origin/<分支>`：远端落后则提示推，远端领先则自动快进本地，杜绝本地以旧基线比对而漏掉远端他人新提交的隐患；分叉冲突则硬性拦截。
3. **树级改动保全校验 (Tree-Diff Guard)**：
   在向源分支执行 `reset --hard` 重置前，脚本硬核核算集成分支与源分支的变动映射：
   **集成分支未 100% 涵盖源分支净贡献前，严禁重置与强推源分支！** 若校验不通过，自动回滚集成分支并保留现场。

---

## 🛠️ 应急备用参考（仅限脚本执行环境彻底缺失时）

若在无 PowerShell 环境且无法运行脚本的极端受限环境下，方可参考以下防御性单行：

```powershell
# 1. 建立安全快照
git update-ref refs/sync-backup/feat_x/temp HEAD

# 2. Route A（自由分支）：变基快进合入 -> 确认零 merge -> 推送集成 -> 对齐源分支
git checkout 'feat/x' && git rebase main && git checkout main && git merge --ff-only 'feat/x' && git push origin main && git checkout 'feat/x' && git reset --hard main && git push --force-with-lease origin 'feat/x' && git checkout main

# 3. 运行合后验证
cargo test --workspace
```

## 红线与避坑
- **禁止 merge 提交**：commitlint 无 `merge:` 类型，必须走严格线性 fast-forward 或 cherry-pick。
- **强制推送纪律**：一律先 fetch 后 `--force-with-lease`，严禁裸 `-f`。
- **工作区防丢**：必须保持工作区干净，严禁在有未暂存修改时执行任何重置。
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
