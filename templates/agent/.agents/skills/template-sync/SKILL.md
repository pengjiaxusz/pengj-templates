---
name: template-sync
description: >-
{% if options["skill_lang"] == "en" %}
  Template synchronization and project adoption workflow for pengj-templates. Automatically diagnoses tech stack, paradigms, and commit habits to recommend tailored profiles and consult users when adopting existing/new projects (adopt); updates managed projects from upstream templates (update); resolves review notices (needs_review / conflicted / transition zones) and verifies downstream alignment.
  Triggers: template-sync, sync-template, update-template, apply-template, adopt-project, 模板同步, 更新模板, 应用模板, 纳管项目, 同步模板.
{% else %}
  基于 pengj-templates 的分层模板同步与项目纳管技能。针对存量或新项目纳管（adopt），自动诊断技术栈、编程范式与提交习惯并提供推荐方案咨询用户；同步上游模板最新改动到已有项目（update）；裁决更新差异与接管过渡区（needs_review / conflicted / 纳管过渡区）或验证下游项目对齐。
  Triggers: template-sync, 模板同步, 更新模板, 应用模板, 纳管项目, 同步模板, update-template, apply-template, adopt-project, 模板更新.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# Template Sync (Update & Adopt)

Workflow for updating downstream projects from upstream `pengj-templates` or adopting templates into existing repositories.

## Quick Start

Run from `pengj-templates` repo or ensure `pengj-templates-cli` is in PATH / `PENGJ_TEMPLATES` is set:

```powershell
# 1. Update an already managed project (has .pengj-templates.json)
cargo run -p pengj-templates-cli -- update --dir <project-dir>

# 2. Adopt / Apply templates to an existing project (no manifest yet)
cargo run -p pengj-templates-cli -- adopt --dir <project-dir> --layers <layer1,layer2> [options]
```

## Workflow

### 1. Diagnose & Select Action

Check whether the target project has `.pengj-templates.json`:

- **Manifest exists**: The project is already managed. Proceed to **Update Mode** (Step 2A).
- **Manifest does NOT exist**: The project is not yet managed. Proceed to **Adopt Mode** (Step 2B).

### 2A. Update Mode (Sync Upstream Changes)

1. Run the update command:
   ```powershell
   cargo run -p pengj-templates-cli -- update --dir <project-dir>
   ```
2. The engine reads `.pengj-templates.json` to reuse original options (`edition`, `skills`, `skill_lang`, etc.) and performs baseline-driven sync:
   - **Unmodified files**: Overwritten cleanly with the latest template.
   - **Files with managed blocks (`PENGJ_TEMPLATE_START/END`)**: Managed block is replaced in place; user-owned area outside is preserved.
   - **TOML / JSON (`.cargo/config.toml`, `package.json`)**: Structured union merge.
   - **Unmanaged modified files**: Flagged as `conflicted` and skipped (never silently overwritten).

### 2B. Adopt Mode (Adoption & Recommendation for New/Existing Projects)

When adopting an unmanaged new or existing repository, **never run adopt blindly without user alignment**. The AI must first perform an automated project diagnosis, generate a recommended profile, and actively consult the user:

#### 1. Automated Tech Stack & Feature Diagnosis (Infer Recommended Profile)

Inspect the target project workspace across the following dimensions:
- **Tech Stack & Layer Compatibility**:
  - Check project files: If `Cargo.toml` exists, it matches the Rust stack — recommend `rust` or `rust-workspace`. **Strictly avoid Rust layers for non-Rust repositories**.
  - Check toolchains: Does the repo have Node.js / frontend assets (`package.json`)? If yes, or if local Git Hook commit enforcement is desired, recommend `lefthook`. For pure script or lightweight projects without Node, default to excluding `lefthook` (avoiding extra npm/pnpm tooling), but present it as an optional enhancement.
  - Editor settings: Recommend `vscode` to inject file nesting patterns (`explorer.fileNesting`); existing `.vscode` configs merge automatically.
  - `common` and `agent` are foundational and core layers for all projects (always required).
- **Programming Paradigm Detection (`--chinese` Chinese Programming)**:
  - Check for Chinese directory names (e.g. `源码/`, `配置/`) or prevalent Chinese identifiers (classes, methods, variables).
  - If detected, **strongly recommend `--chinese`** to inject Chinese programming guidelines and bilingual naming anchors.
- **Commit Conventions & Language Detection (`--commit-zh` / `--commit-and-push`)**:
  - Run `git log -n 5 --oneline` to inspect commit styles.
  - If commits use Chinese subjects/bodies, keep `--commit-zh` enabled (default true).
  - **Strongly recommend `--commit-and-push`** to enforce the "Pre-Response Commit Gate", ensuring the AI commits and pushes verified changes before concluding turns.
- **Existing Rules & Asset Assessment**:
  - If `AGENTS.md` exists, inform the user that managed blocks will be prepended at the top, cleanly preserving existing project-specific rules below.
  - If legacy custom skills exist, note that they will be taken over into transition zones (`needs_review`).
- **Skill Selection (`--skills`)**:
  - Core baseline: `commit` (Conventional Commits), `template-sync` (upstream updates).
  - Architecture & Collaboration: `arch-align` (architecture alignment), `branch-sync` (linear branch/worktree sync).
  - Utilities: `caveman` (token compression), `grill-me` (design grilling), `subrepo-sync` (submodule sync), `write-a-skill` (authoring skills). Default to all skills or tailored subsets.

#### 2. Present Diagnosis & Consult the User (Provide a Recommended Profile)

Present a structured diagnostic report to the user:
- **Project Diagnosis Summary**: Tech stack, Chinese programming style, commit conventions, existing asset safety;
- **Primary Recommendation (Marked `(Recommended)`)**: Specific layers (e.g. `--layers common,agent,vscode`), options (e.g. `--chinese`, `--commit-and-push`), and the technical rationale;
- **Alternative / Add-on Profiles**: Feasible variations (e.g. adding `lefthook` with its trade-offs regarding Node dependencies);
- **Command Preview**: Exact CLI command preview;
- **Consultation**: Ask the user via structured choice (e.g. `ask_question` or interactive prompt) for confirmation or adjustments.

#### 3. Execute Adopt Command

Upon user confirmation, execute:
```powershell
cargo run -p pengj-templates-cli -- adopt --dir <project-dir> --layers <layer1,layer2> [options]
```
*(Note: `--dir` specifies the target project directory; do not pass bare positional arguments).*

### 3. Post-Processing & Conflict Resolution

Inspect the CLI execution report:

- **`needs_review` — Transition Zone Merge**:
  - If a legacy custom skill was taken over, the template body is applied and the original custom content moves into `<!-- === PENGJ_ADOPT_TRANSITION_ZONE === -->`.
  - Assist the user in moving domain-specific checks from the transition zone into the project-specific area outside the managed block.
  - **Delete the transition zone comment and content** after merging.
- **`conflicted` — Unmanaged Modified Files**:
  - Review `path` and `reason`. Compare diffs and manually apply upstream template updates if needed.
- **Toolchain / Workspace Wiring**:
  - Verify `commitlint.base.js` wiring, `.vscode/settings.json`, or `.cargo/config.toml` merged as expected.

### 4. Verify & Selective Commit

1. Switch to the target project directory and run verification checks matching its stack:
   ```powershell
   # e.g., for Rust
   cargo check && cargo test
   # e.g., for Frontend
   pnpm build
   ```
2. **Commit Policy & Selective Staging**:
   - **No auto-commit by default**: Running `update` or `adopt` only updates the working tree for review. Do NOT automatically commit unless the user explicitly requested it.
   - **Strict selective commit when requested (Red line)**: When the user explicitly asks to commit, **stage ONLY the files modified or created by this template sync/adoption** (matching the update report paths). NEVER use indiscriminate `git add .` or `git commit -a` which might accidentally sweep in unrelated dirty or pre-existing working tree changes.
   - Commit message: `chore(templates): sync upstream template updates` or `chore(templates): adopt project layered templates`, then push.
{% else %}
# 模板同步与项目纳管 (Template Sync)

用于将 `pengj-templates` 上游模板更新同步到下游项目，或将分层模板应用/纳管到存量现有项目的工作流。

## 快速开始

在 `pengj-templates` 仓库根目录运行，或确保 `pengj-templates-cli` 在 PATH 中 / 已设置 `PENGJ_TEMPLATES` 环境变量：

```powershell
# 1. 更新已纳管项目（已有 .pengj-templates.json）
cargo run -p pengj-templates-cli -- update --dir <目标项目目录>

# 2. 纳管/应用模板到存量项目（尚无 manifest）
cargo run -p pengj-templates-cli -- adopt --dir <目标项目目录> --layers <层1,层2> [选项]
```

## 工作流

### 1. 状态诊断与模式判定

检查目标项目根目录是否存在 `.pengj-templates.json`：

- **存在 manifest**：说明项目已纳管，直接进入 **Update 模式（步骤 2A）**。
- **不存在 manifest**：说明是存量或未受管项目，进入 **Adopt 模式（步骤 2B）**。

### 2A. Update 模式（同步上游更新）

1. 执行更新命令：
   ```powershell
   cargo run -p pengj-templates-cli -- update --dir <目标项目目录>
   ```
2. 引擎自动从 manifest 读取固化的选项（`edition`、`skills`、`skill_lang` 等）并重渲染比对：
   - **磁盘未改动文件**：直接覆盖为最新模板；
   - **含受管块文本（`PENGJ_TEMPLATE_START/END`）**：受管块内原位替换，块外项目专属内容完整保留；
   - **TOML / JSON（`.cargo/config.toml`、`package.json`）**：结构化并集合并；
   - **无受管块且用户已修改的文件**：标记为 `conflicted` 并跳过（绝不静默覆盖用户改动）。

### 2B. Adopt 模式（存量/新项目纳管与推荐）

面对未受管的存量或新项目，**严禁盲目直接执行 adopt**。AI 必须先自动进行项目深度诊断、生成首选推荐方案，并主动咨询用户进行确认与选择：

#### 1. 自动化技术栈与特征诊断（推导推荐方案）

AI 应首先巡检目标项目的工作区，按以下维度完成自动化诊断：
- **技术栈与适用层判定**：
  - 检查项目根目录与源码：包含 `Cargo.toml` 则命中 Rust 技术栈，推荐 `rust` 或 `rust-workspace` 层；**非 Rust 项目严禁引入 Rust 相关层**。
  - 检查工具链：是否包含 Node.js / 前端生态（`package.json`）？若有或用户希望本地 Git Hooks 物理拦截提交，可推荐 `lefthook`；若为纯脚本/无 Node 生态轻量项目，默认不选 `lefthook`（避免引入 node 依赖），作为可选附加项提示。
  - 编辑器配置：推荐引入 `vscode` 层，提供文件折叠嵌套规则（`explorer.fileNesting`），现有 `.vscode` 配置会自动并集合并。
  - `common` 与 `agent` 为所有项目的基础与核心层（必选）。
- **编程范式检测（`--chinese` 中文编程）**：
  - 检查是否存在中文目录（如 `源码/`、`配置/`、`动作/`）或代码中大量使用中文类名/函数/变量标识符。
  - 命中时必须**强烈推荐开启 `--chinese`**，为项目注入中文化命名与中英拼接规范。
- **提交习惯检测（`--commit-zh` / `--commit-and-push`）**：
  - 运行 `git log -n 5 --oneline` 检查历史提交信息。
  - 提交使用中文标题与正文时，保持 `--commit-zh`（默认开启）。
  - **强烈推荐开启 `--commit-and-push`**，落实「回复前提交门禁（硬性拦截，禁止堆积）」原则，确保 AI 助手任务完工后自动提交并推送，杜绝改动堆积。
- **存量规范与资产评估**：
  - 若已有 `AGENTS.md` 或 `GEMINI.md`，引擎将以置顶插入（Prepend）方式注入托管块，现有专属规范完整保留在块外，绝不覆盖。
  - 若已有自定义技能（无托管块），引擎会自动接管并将原内容下移至纳管过渡区（`needs_review`）。
- **技能集精选（`--skills`）**：
  - 核心基线：`commit`（约定式提交）、`template-sync`（模板同步更新）。
  - 架构与协作：`arch-align`（架构对齐）、`branch-sync`（高效分支/worktree 线性同步）。
  - 辅助增强：`caveman`（token 压缩）、`grill-me`（方案质询）、`subrepo-sync`（子模块/子仓库同步）、`write-a-skill`（技能创建）。默认全选或按需勾选。

#### 2. 向用户呈现诊断报告并咨询选择（必须给推荐方案）

必须以结构化格式向用户呈现诊断结论并提供选择方案：
- **项目特征摘要**：技术栈、中文编程范式、提交规范、现有规范保护说明；
- **首选推荐方案（标记为 `(Recommended)`）**：列出最适合该项目的层组合（如 `--layers common,agent,vscode`）、选项参数（如 `--chinese`、`--commit-and-push`）及选型理由；
- **备选/增选方案**：列出其他可行组合（如追加 `lefthook` 强化本地 Git 拦截，并说明需引入 Node 依赖的利弊）；
- **完整 CLI 命令预览**：给出对应的执行命令示例；
- **咨询与确认**：通过交互式选择（如 `ask_question` 或结构化提问）征询用户意见，获得用户确认或自定义调整。

#### 3. 执行纳管命令

在用户确认后，执行纳管命令：
```powershell
cargo run -p pengj-templates-cli -- adopt --dir <目标项目目录> --layers <层1,层2> [选项]
```
*(注意：`--dir` 用于指定目标项目目录，请勿使用裸路径位置参数)*

### 3. 结果解析与后处理（关键步骤）

检查 CLI 输出报告中的各项分类：

- **`needs_review`（过渡区合并与审核）**：
  - 若存量自定义技能被模板整页接管，原正文会移至 `<!-- === PENGJ_ADOPT_TRANSITION_ZONE === -->`（纳管过渡区）。
  - 协助用户将过渡区中独有的领域检查项合并入托管块外的「项目专属区」。
  - 合并完成后，**删除过渡区注释及内容**。
- **`conflicted`（无受管块冲突）**：
  - 查看冲突路径与原因，按需手动将模板变更合入对应文件。
- **配置与工具链接线核验**：
  - 检查 `commitlint.base.js` 自动接线、`.vscode/settings.json`、`.cargo/config.toml` 合并是否符合预期。

### 4. 验证与按需精准提交

1. 进入目标项目目录，执行对应技术栈的验证检查：
   ```powershell
   # 如 Rust 项目
   cargo check; cargo test
   # 如前端项目
   pnpm build
   ```
2. **提交策略与精准提交流程**：
   - **默认不提交**：执行 `update` 或 `adopt` 后，默认仅在工作区完成更新与核验，**不要自动提交**，等待用户确认或显式指令。
   - **用户要求提交时的精准提交（红线）**：当用户明确要求提交时，**只能单独 `git add <file>` 本次模板同步/纳管所修改或新增的文件**（严格对照 update 报告清单），**严禁 `git add .` 或 `git commit -a`**，防止误将工作区既有的无关改动或脏文件一并带入提交。
   - 提交信息采用约定式提交（如 `chore(templates): 同步上游模板更新` 或 `chore(templates): 纳管项目分层模板`），并在提交后推送。
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属同步配置与快捷方式

> 可在此登记本项目常用的上游模板地址、默认层组合、前后置同步脚本等信息。
