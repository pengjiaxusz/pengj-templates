---
name: subrepo-sync
description: >-
{% if options["skill_lang"] == "en" %}
  Universal sub-repository / submodule synchronization, commit impact analysis, and adaptation workflow. Extract unapplied commits between baseline and target, cluster changes by semantic Conventional Commits and domain subsystems (motion, tokens, styles, components, breaking changes), safe checkout/update, guide host refactoring and replacement of handwritten wheels, and enforce host verification gates. Use when updating Git submodules, CMake FetchContent dependencies, embedded sub-repos, or syncing downstream repos with subrepo changes. Triggers: subrepo-sync, submodule-sync, subrepo, submodule, update-subrepo, upgrade-subrepo, sync-submodule.
{% else %}
  通用的子仓库与子模块依赖同步、变动语义聚类分析与宿主适配技能。自动化比对子仓库未应用提交（自旧指针到目标最新提交）、按约定式提交及领域子系统（破坏性变动、动效、高亮、样式、设计令牌、规范组件等）聚类分析影响面、执行安全更新签出、指导宿主代码重构替换与门禁核验。当需要升级 Git Submodule、CMake FetchContent 或独立子仓库依赖、同步子模块最新提交、排查子仓库变动影响面或适配新特性时使用。 Triggers: subrepo-sync, submodule-sync, subrepo, submodule, update-subrepo, upgrade-subrepo, sync-submodule.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# Sub-Repository Sync & Adaptation (Subrepo Sync)

Safely upgrade sub-repositories (Git submodules, CMake FetchContent targets, vendored repos), extract unapplied commit history, cluster impact across subsystems, refactor host code, and enforce project gates.

## Quick Start

Run the built-in detection script from repo root to inspect differences or update:

```powershell
# 1. Inspect unapplied commits and analyze impact clustering
pwsh .agents/skills/subrepo-sync/scripts/show-unapplied-commits.ps1 -SubrepoPath "<path/to/subrepo>"

# 2. Confirm and check out target commits directly
pwsh .agents/skills/subrepo-sync/scripts/show-unapplied-commits.ps1 -SubrepoPath "<path/to/subrepo>" -Update
```

## The Registry Is the Single Source of Truth

**Read `.pengj-templates/subrepo.toml` first.** It is the machine-readable registry of every
sub-repository this project depends on: one `[[subrepo]]` table per entry, carrying the name,
the relative path, the dependency mode, the upstream tracking branch and whether the
sub-repository ships its own workspace definition.

```toml
[[subrepo]]
name = "example-subrepo"
path = "submodules/example-subrepo"
mode = "submodule"        # submodule | fetch | local
upstream = "origin/main"
has_workspace = false
```

Two rules follow from it:

- **The registry decides which sub-repositories exist.** Do not work from a hand-maintained
  Markdown table: it cannot be parsed, so the skill and the health check would disagree about
  what the project depends on. The health check (`pengj-templates-cli subrepo check`) reads this
  very file, so the set of entries the skill suggests upgrading is the set the check verifies.
- **The dependency mode decides what to do.** `submodule` needs pointer and readiness checks;
  `fetch` is pinned by the host build system; `local` is a sibling directory on this machine.
  The mode is a closed enum — an unknown value is a hard error, never a silent skip.

List the registry in human-readable form with:

```powershell
pengj-templates-cli subrepo list --dir .
```

## Standard Upgrade Workflow

```
[0. Read the registry] ──► [1. Baseline & Target] ──► [2. Unapplied Commits & Clustering] ──► [3. Safe Checkout / Update] ──► [4. Host Refactor & Adapt] ──► [5. Gate Verification & Commit]
```

### Step 1: Baseline & Target Discovery
1. Read current baseline commit: `git -C <subrepo-path> rev-parse HEAD`.
2. Discover target commit source:
   - Sibling local dev worktree (`..\<name>`);
   - Environment variable `$env:<NAME>_DIR`;
   - Remote upstream branch (`origin/main` or `origin/master`).
3. **When several pending commits each bumped the pin, settle the ordering before picking a target**:
   the new pin must be a **descendant** of the old one, otherwise the features you merge in
   **silently fail** (surfacing as missing properties or symbols at the API/QML layer).
   Read each commit's pin (`git show <sha>:<lockfile> | grep <PIN variable>`), run
   `git fetch origin <sha>` inside the subrepo first (the object is usually absent locally —
   **the fetch is mandatory**), then check `git merge-base --is-ancestor <old> <new>`;
   finally confirm the target capability really exists in the source tree actually used for builds.

### Step 2: Unapplied Commits & Semantic Impact Clustering
Extract commits via `git log <old>..<new> --oneline --no-merges` and cluster:
- **💥 Breaking Changes**: `feat!:`, `fix!:`, `BREAKING CHANGE:` ➔ Check signature & breaking API changes.
- **🎬 Motion & Transitions**: `motion`, `easing`, `animate` ➔ Verify exit transitions, easing curves, and duration collapse.
- **🎨 Tokens & Themes**: `token`, `theme`, `palette`, `color` ➔ Ensure foreground/background color coherence.
- **📐 Styles & Layers**: `styles`, `cascade-layer`, `layer`, `css` ➔ Ensure cascade layer order is preserved.
- **🧩 Canonical Components**: `component`, `feat(...)` ➔ Replace host handwritten wheels with upstream components.
- **⚡ Performance & 🐛 Fixes**: `perf:`, `fix:` ➔ Clean up temporary workarounds in the host.

### Step 3: Safe Checkout & Pointer Update
- Verify host and subrepo working trees are clean (`git status --short`).
- Checkout target commit in the subrepo (`git checkout <target-hash>` or `git submodule update`).

### Step 4: Host Refactoring & Adaptation
1. Replace duplicate host implementations with canonical components.
2. Comply with project architecture red lines (declared in the project-specific area below).

### ⚠️ Rebuild the Test Target First (Stale Artifact Trap)

A normal build usually **produces only the main executable, not the test executables**; the dependency's
modules (static libraries, resources, QML) are compiled into each test executable. If you run test gates
right after bumping the dependency pin, **the old executables load the new source tree's resources with
the old dependency modules**, producing a string of errors that look self-inflicted:

- `Cannot assign to non-existent property "<newly added property>"`;
- `Type <host component> unavailable`;
- If host code was changed in parallel, unrelated errors of the same shape appear too
  (same cause: the executable predates that property).

**These are stale-artifact false positives, not regressions** — rebuild/relink the test target and they
disappear. Before investigating, confirm the test executables were built *after* the dependency update.

### Step 5: Verification Gates & Conventional Commit
1. Run host compilation, static checks, and unit tests.
2. Run project hygiene and UI verification gates.
3. Commit host changes using Conventional Commits: `chore(deps): update <subrepo> to <short-hash> and adapt <changes>`.

## Progressive Disclosure & Reference

- [Detailed Architecture & Integration Modes](REFERENCE.md)
- [Real-World Upgrade & Refactoring Examples](EXAMPLES.md)
{% else %}
# 子仓库同步与升级 (Subrepo Sync)

用于在宿主项目中受控升级外部依赖子仓库（Git Submodule、CMake FetchContent、独立子仓库等），提取未应用提交、按子系统聚类分析影响面、指导业务组件重构与门禁回归。

## 快速开始

在宿主仓库根目录下执行内置脚本，自动化分析提交差异与影响面：

```powershell
# 1. 查看未应用提交记录并聚类分析影响面
pwsh .agents/skills/subrepo-sync/scripts/show-unapplied-commits.ps1 -SubrepoPath "<子仓库路径>"

# 2. 确认升级，附加 -Update 一键签出最新目标提交
pwsh .agents/skills/subrepo-sync/scripts/show-unapplied-commits.ps1 -SubrepoPath "<子仓库路径>" -Update
```

## 登记表是唯一事实源

**先读 `.pengj-templates/subrepo.toml`。** 它是本项目全部外部子仓库的机器可读登记表：
每个子仓库一张 `[[subrepo]]` 表，记着名称、相对路径、依赖模式、上游跟踪分支，
以及该子仓库是否自带工作区定义。

```toml
[[subrepo]]
name = "example-subrepo"
path = "submodules/example-subrepo"
mode = "submodule"        # submodule | fetch | local
upstream = "origin/main"
has_workspace = false
```

由此有两条规矩：

- **登记表决定有哪些子仓库。** 不要依据手工维护的 Markdown 表格行事：那张表无法被程序解析，
  技能与体检命令会对「本项目依赖哪些子仓库」产生分歧。体检命令
  （`pengj-templates-cli subrepo check`）读的就是这份文件，因此技能建议升级的对象
  与体检检查的对象永远是同一批。
- **依赖模式决定该做什么。** `submodule` 要查指针与就绪状态；`fetch` 的版本锁在宿主构建系统里；
  `local` 是本机的同级目录。模式是封闭枚举，写出三种之外的值是硬错误，绝不静默跳过。

用下面这条命令把登记表渲染成人类可读的清单：

```powershell
pengj-templates-cli subrepo list --dir .
```

## 标准升级工作流

```
[0. 读取登记表] ──► [1. 识别基线与目标] ──► [2. 提取未应用提交与聚类] ──► [3. 安全签出与更新] ──► [4. 宿主适配与重构] ──► [5. 门禁验证与提交]
```

### 步骤 1：识别基线与目标
1. 读取当前子仓库锁定指针：`git -C <子仓库路径> rev-parse HEAD`。
2. 探测目标更新源：
   - 本地同级联调目录（`..\<subrepo>`）；
   - 环境变量指定目录（`$env:<NAME>_DIR`）；
   - 远程主干分支（`origin/main` 或 `origin/master`）。
3. **多个待合入提交各自 bump 过指针时，先判先后再定目标**：新指针必须是旧指针的**后代**才安全，
   否则合入的功能会**静默失效**（QML/接口层面表现为属性或符号不存在）。
   取各提交里的指针值（`git show <sha>:<锁文件> | grep <PIN 变量>`），在子仓库内
   `git fetch origin <sha>` 之后（本地常缺该对象，**必须先 fetch**）用
   `git merge-base --is-ancestor <旧> <新>` 判定；最后再对照实际构建用的源码目录确认目标能力确实存在。

### 步骤 2：提取未应用提交并进行语义聚类
提取区间提交（`git log <旧Commit>..<新Commit> --oneline --no-merges`），按子系统聚类：
- **💥 破坏性变动 (Breaking Changes)**：`feat!:`, `fix!:`, `BREAKING CHANGE:` ➔ 重点研判 API 破坏与类型变动；
- **🎬 动效与过渡 (Motion)**：`motion`, `easing`, `animate` ➔ 检查退出动画、缓动曲线与时钟坍缩契约；
- **🎨 设计令牌与色彩 (Tokens & Themes)**：`token`, `theme`, `palette` ➔ 遵循背景/前景色成对自洽律；
- **📐 样式与层叠 (Styles & Layers)**：`styles`, `cascade-layer`, `css` ➔ 确保样式层叠优先级不被覆盖；
- **🧩 规范组件与接口 (Components)**：`component`, `feat(...)` ➔ 遵循组件自举律，替换业务自研轮子；
- **⚡ 性能与 🐛 缺陷修复 (Perf & Fixes)**：`perf:`, `fix:` ➔ 清理宿主内临时规避代码。

### 步骤 3：安全签出与指针更新
- 确认子仓库与宿主工作区干净无脏改动（`git status --short`）；
- 签出目标提交（`git checkout <新Commit>` 或执行项目专用更新命令）。

### 步骤 4：宿主代码适配与重构
1. 检索宿主项目中是否有对应功能的自研手写实现，将其重构替换为规范组件；
2. 严格遵循宿主架构红线（见托管块外的项目专属区）。

### ⚠️ 升级后先重建测试目标（陈旧产物陷阱）

常规构建命令通常**只产出主程序，不产出测试可执行文件**；而依赖模块（静态库、资源、QML 等）
是编译进各个测试可执行文件的。升级依赖指针后若直接跑测试门禁，**旧可执行文件会用旧依赖模块去加载
新源码树里的资源**，报出一串看似自伤的错：

- `Cannot assign to non-existent property "<新增属性>"`；
- `Type <宿主组件> unavailable`；
- 若同时并行改过宿主自身的代码，还会冒出毫不相关的同类报错（同因：可执行文件早于该属性入库）。

**这些是陈旧产物的假阳性，不是回归**——先重建（重链）测试目标即可消除。
动手排查前，先确认测试可执行文件的构建时间是否晚于本次依赖更新。

### 步骤 5：门禁验证与规范提交
1. 执行宿主技术栈的编译、类型检查与单元测试；
2. 运行项目专属的卫生门禁与界面真实环境核验；
3. 撰写规范提交信息（如 `chore(deps): 更新 <subrepo> 至 <hash> 并适配组件规范`），提交后立即 push。

## 详细指南与实战案例（渐进式披露）

- [子仓库系统架构与依赖模式详解](REFERENCE.md)
- [历史真实升级与组件重构案例](EXAMPLES.md)
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- The following is the project-specific area: template updates only replace the managed block above; this area is owned by the project and fully preserved. -->
## Project-Specific Subrepo Configuration & Gate Contracts

> Declare project-specific clustering keywords, host verification gates, and architectural invariants here.

### 1. Subrepo Registry (optional derived view)

The authoritative registry is `.pengj-templates/subrepo.toml` — read it, and keep it up to date
whenever a sub-repository is added, removed or re-pointed. The table below is an **optional
human-readable snapshot** derived from that file; it is never the source of truth and it is
never parsed by any tool. If it drifts from the registry, the registry wins.

| Subrepo Name | Relative Path | Dependency Mode (Submodule / FetchContent / Local) | Upstream Tracking Branch |
| :--- | :--- | :--- | :--- |
| `example-subrepo` | `submodules/example-subrepo` | Git Submodule | `origin/main` |

### 2. Host Invariants & Architectural Boundaries
- **Single Source of Truth**: Never edit or fork inside dependency cache directories directly; all shared components and contracts must originate from upstream.
- **Component Bootstrap Law**: Once upstream provides official standard components, timely refactor and delete host handwritten duplicates.

### 3. Project Verification Gates
```powershell
# Lint & build check
# cargo check --workspace
# pnpm build

# Automated test & hygiene gates
# cargo test --workspace
# pwsh tools/hygiene.ps1
```
{% else %}
<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属子仓库配置与门禁契约

> 在此处声明领域专属聚类关键词、宿主验证门禁与架构红线。

### 1. 子仓库登记（可选展示）

权威登记在 `.pengj-templates/subrepo.toml` —— 读它，并且增删子仓库、改指向时同步维护它。
下表是**由登记表派生的可选人类可读快照**，不是事实源，也没有任何工具会解析它。
若它与登记表不一致，以登记表为准。

| 子仓库名称 | 相对路径 | 依赖模式 (Submodule / FetchContent / 本地) | 上游跟踪分支 |
| :--- | :--- | :--- | :--- |
| `example-subrepo` | `submodules/example-subrepo` | Git Submodule | `origin/main` |

### 2. 宿主架构红线
- **单真相源驱动**：严禁在依赖缓存目录中直接修改或分叉；所有通用组件与契约必须源自 upstream。
- **组件自举律**：上游一旦推出规范控件，必须及时重构并移除宿主自研的重复手写实现。

### 3. 项目专属验证门禁
```powershell
# 静态与编译检查
# cargo check --workspace
# pnpm build

# 自动化测试与卫生门禁
# cargo test --workspace
# pwsh tools/hygiene.ps1
```
{% endif %}
