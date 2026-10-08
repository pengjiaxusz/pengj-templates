---
name: branch-sync
description: >-
  极速且 Worktree 感知的线性化分支同步流程。自动化 patch-id 净贡献甄别、rebase/ff 合并与 cherry-pick 双路径、force-with-lease 推送与合后自检。内置脚本与复合单行流，大幅减少工具调用次数与 Token 消耗。当用户要求合并分支、同步分支、合入主分支、分支对齐、收编并行 worktree 时使用。
  Triggers: branch-sync, 分支同步, 同步分支, 合并分支, 合入主分支, worktree 同步, cherry-pick, rebase.
---

<!-- PENGJ_TEMPLATE_START -->
# 分支同步 — Worktree 感知的极速线性化全分支同步

将所有分支或指定特性分支（支持单仓库或由 Worktree 占用）线性化合入集成分支，内置 **当前目录自适应集成分支推导、全局时间序智能合流、持久化快照兜底（refs/sync-backup/）、Tree-Diff Guard 树级改动保全校验、批量 Worktree 对齐与项目门禁 1-Shot 闭环**。
历史必须严格线性、零 merge 提交、强制推送一律 `--force-with-lease`。

> **集成分支智能推导**：默认自动选取当前目录检出/激活的分支作为集成分支（即跑测试与验证的主工作区）；同时支持 Detached HEAD 工作树自动关联、SKILL.md 项目登记、远端/默认分支多层降级，绝不硬编码死静态 `HEAD`。

```
[一键执行 1-Shot: sync-branch.ps1 -Apply] 
  ├── 1. 动态自适应集成分支判定 (优先当前目录激活分支)
  ├── 2. 自动全分支发现与远端双向快进 (防漏远端提交)
  ├── 3. 全局时间序提交提取 (按 committerdate 升序规避时序冲突)
  ├── 4. 自动建立持久化安全快照 (refs/sync-backup/ 永久防丢)
  ├── 5. 严格线性合入 (变基快进 / 按序 cherry-pick / 冲突自动持久化)
  ├── 6. Tree-Diff Guard 树级防漏审计 (未 100% 合入绝不重置源分支)
  ├── 7. 批量 Worktree 感知对齐 & --force-with-lease 安全推送
  └── 8. 自动运行项目合后门禁 (如 cargo test) -> 输出最终看板
```

## ⚡ 极速自动化通道（强制推荐：单次工具调用 1-Shot）

当用户要求合并分支、同步分支或分支对齐时，**直接执行带 `-Apply` 的单次调用**。脚本自动化闭环全流程，杜绝提交遗漏与误覆盖，大幅节省 Token 与调用耗时：

```powershell
# 1. 推荐：默认一键将所有分支同步、对齐并推送到当前激活分支（1 次调用闭环）
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -Apply

# 2. 指定单个源分支一键合并、推送与验证
pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch 'feat/x' -Apply

# 3. 只读拓扑巡检（预览所有分支状态与净贡献，不修改任何分支）
pwsh .agents/skills/branch-sync/scripts/show-branch-topology.ps1
```

> **极速与低智力防呆准则（硬性红线）**：
> 1. **单次调用直达终态**：不要先跑 Dry Run 再跑 Apply 再跑构建！直接运行 `-Apply`，脚本内部会自动安全预检、拦截脏工作区、按全局时间序完成合并并在末尾自动执行项目的合后门禁命令（如 `cargo test`）。
> 2. **严禁手动拼接 Git 原生命令**：严禁自行执行 `git merge`（产生 merge 提交破坏规范）、严禁手动 `git reset --hard`（极易造成未合入提交永久丢失）。所有操作必须且仅需通过 `sync-branch.ps1` 托管执行。
> 3. **看板判定即交差**：当脚本输出 `STATUS: COMPLETED_READY_TO_REPORT` 时，代表分支同步、推送与项目门禁已全部通过，无需追加任何工具调用，直接向用户汇报即可。

---

## 🧰 工具箱全景速查 (.agents/skills/branch-sync/scripts/)

| 工具脚本 | 职责与定位 | 典型调用方式 |
|---|---|---|
| `sync-branch.ps1` | **核心一键同步引擎**（默认全分支，支持单分支） | `pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -Apply` |
| `show-branch-topology.ps1` | 只读拓扑与净贡献巡检器（展示所有分支 Ahead/Behind/Net） | `pwsh .agents/skills/branch-sync/scripts/show-branch-topology.ps1 -Detailed` |
| `align-branches.ps1` | 批量分支与 Worktree 重置对齐及 safe push 工具 | `pwsh .agents/skills/branch-sync/scripts/align-branches.ps1 -Apply` |
| `continue-sync.ps1` | 冲突解决后 1-Shot 续接合流或安全中止回滚工具 | `pwsh .agents/skills/branch-sync/scripts/continue-sync.ps1 -Continue` |
| `manage-sync-backups.ps1` | `refs/sync-backup/` 安全快照清单、还原与过期清理 | `pwsh .agents/skills/branch-sync/scripts/manage-sync-backups.ps1 -List` |

---

## 🛡️ 防漏提交与防覆盖硬核机制

1. **自动持久化快照 (Safety Backup Ref)**：
   每次执行 `-Apply` 前，自动在本地写入快照指针：
   `refs/sync-backup/<分支名>/<时间戳>-<SHA>`
   若有任何人为中止或异常，原分支所有提交永远有 ref 保护，绝不沦为悬空提交，随时可通过 `manage-sync-backups.ps1 -RestoreBranch <分支名> -BackupRef <快照Ref>` 毫秒级无损还原。
2. **全局提交时序自动编排 (Global Chronological Ordering)**：
   多分支并发开发时，自动提取所有未合入净提交，并严格依据 `committerdate`（提交时间戳）全局升序排列再逐个 cherry-pick，消除 80% 以上因时序倒置引发的合并冲突。
3. **冲突现场持久化与一键续接 (Conflict Persistence & Resume)**：
   若遇到代码重叠冲突，脚本自动将剩余队列与快照保存至 `.git/branch-sync-state.json` 并高亮冲突文件。开发者或 Agent 解决冲突后，仅需单次运行 `continue-sync.ps1 -Continue` 即可自动继续后续所有提交流程，无需重新手写原生 Git 命令。
4. **树级改动保全校验 (Tree-Diff Guard)**：
   在向源分支执行重置前，脚本硬核核算集成分支与所有源分支的净提交映射：
   **集成分支未 100% 涵盖各源分支净贡献前，严禁重置与强推源分支！** 若校验不通过，自动回滚集成分支并保留现场。

## 🤖 智能体环境适配（沙箱 / 工具约束）

在受沙箱约束的智能体环境里，`git` 的行为与手敲终端**并不一致**。以下三条为硬性操作纪律，
完整命令模板与原理见 `REFERENCE.md` §1：

1. **git 写操作走脚本进程，不要逐条 shell 调用**：部分沙箱会**静默虚拟化 `refs/remotes/**` 的写入**
   ——`git fetch` 正常打印 `old..new` 却写不进去，随后 `rev-parse origin/<分支>`、`branch -r -v`
   读到陈旧值，会误判成「远端分支被推坏 / 提交丢了」。请用宿主语言的进程调用
   （如 Python `subprocess.run(['git', *args], cwd=...)`）把连续操作串起来。
2. **远端真值只信 `git ls-remote`**；本地侧做净贡献甄别时用本地分支对象而非 `origin/*`：
   `git cherry -v <集成分支 tip> <源分支>`。
3. **长 git 操作必须给足超时**：调用超时会向 git 发 SIGTERM，把大 worktree 的 `reset --hard`
   杀在中途，留下 `index.lock` 与成片半删除文件；被中断后先删锁再重跑。

## 红线与避坑
- **禁止 merge 提交**：commitlint 无 `merge:` 类型，必须走严格线性 fast-forward 或 cherry-pick。
- **强制推送纪律**：一律先 fetch 后 `--force-with-lease`，**且必须用显式形式**
  `--force-with-lease=refs/heads/<分支>:<刚取到的远端sha>`（隐式形式在部分环境会报 `stale info`）；严禁裸 `-f`。
- **强推前当场复核远端净贡献（硬性）**：`--force-with-lease` 只保证「远端自你上次观察后没被改」，
  **不保证远端没有你没看到的净贡献**——lease 会取到新值、判定一致并放行，把别人的提交抹掉。
  每次强推前**重新** `ls-remote` 取 lease（禁止复用几分钟前的观察值），再用该 sha 算一次
  `git cherry -v <集成分支> <远端sha>`；有 `+` 说明远端有新活，**先合入再推**。
- **批量合入按时间序**：多分支合入按 `committerdate` 全局升序逐个 cherry-pick，
  **不要按分支分组**；合入后做文件级 diff 完整性校验，并以「增删行比对」判别是否已合入。
- **冲突取值约定**：默认取源分支（待合入）一侧；若两侧是**不同维度**的改动（非同一处的两种写法），
  必须两侧都保留——先看完整 diff 理解语义再动手。
- **工作区防丢**：必须保持工作区干净，严禁在有未暂存修改时执行任何重置。

## 📚 深入参考（渐进式披露）

动手前按需查阅 [`REFERENCE.md`](REFERENCE.md)：

- 智能体环境 git 语义（引用写入虚拟化、跟踪引用丢失 `[gone]` 修复、缺 coreutils 等）；
- 强推前的净贡献复核与**误覆盖后的抢救流程**；
- 分叉场景（Tree-Diff Guard 判 Diverged）的人工处置；
- 多分支批量合入方法论与「已合入」快速判别法；
- 生成物 / 资源文件（翻译包、清单等）的实体级冲突合并；
- 合后失败用例的真基线判定（含常见误判来源）。
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
cargo test --workspace
```
