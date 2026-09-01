---
name: template-sync
description: >-
{% if options["skill_lang"] == "en" %}
  Template synchronization and project adoption workflow for pengj-templates. Use when applying/adopting templates to existing or new projects (adopt), updating managed projects from upstream templates (update), resolving update diffs and review notices (needs_review / conflicted / transition zones), or verifying template changes downstream.
  Triggers: template-sync, sync-template, update-template, apply-template, adopt-project, 模板同步, 更新模板, 应用模板, 纳管项目, 同步模板.
{% else %}
  基于 pengj-templates 的分层模板同步与项目纳管技能。在将模板应用/纳管到存量项目（adopt）、同步上游模板最新改动到已有项目（update）、裁决更新差异与接管过渡区（needs_review / conflicted / 纳管过渡区）、或验证下游项目对齐时使用。
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
cargo run -p pengj-templates-cli -- adopt <project-dir> --layers <layer1,layer2> [options]
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

### 2B. Adopt Mode (Apply Templates to Existing Project)

1. Determine required layers and options with the user (e.g. `agent,rust` or `agent,node,vite`):
   - Layer options: `--layers agent,rust`
   - Common flags: `--chinese` (Chinese programming), `--skills commit,template-sync,...`, `--skill-lang zh|en`, `--commit-and-push`
2. Run the adopt command:
   ```powershell
   cargo run -p pengj-templates-cli -- adopt <project-dir> --layers <layers> [options]
   ```
3. The engine initializes `.pengj-templates.json` baseline, injects managed blocks into matching files, unions configurations, and takes over legacy skills.

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

### 4. Verify & Commit

1. Switch to the target project directory and run test / lint / build checks matching its stack:
   ```powershell
   # e.g., for Rust
   cargo check && cargo test
   # e.g., for Frontend
   pnpm build
   ```
2. Trigger the commit workflow (`.agents/skills/commit`) to commit and push changes.
{% else %}
# 模板同步与项目纳管 (Template Sync)

用于将 `pengj-templates` 上游模板更新同步到下游项目，或将分层模板应用/纳管到存量现有项目的工作流。

## 快速开始

在 `pengj-templates` 仓库根目录运行，或确保 `pengj-templates-cli` 在 PATH 中 / 已设置 `PENGJ_TEMPLATES` 环境变量：

```powershell
# 1. 更新已纳管项目（已有 .pengj-templates.json）
cargo run -p pengj-templates-cli -- update --dir <目标项目目录>

# 2. 纳管/应用模板到存量项目（尚无 manifest）
cargo run -p pengj-templates-cli -- adopt <目标项目目录> --layers <层1,层2> [选项]
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

### 2B. Adopt 模式（应用模板到存量项目）

1. 与用户确认目标项目所需层与选项（如 `agent,rust` 或 `agent,node,vite`）：
   - 层参数：`--layers agent,rust`
   - 常用选项：`--chinese`（中文编程）、`--skills commit,template-sync,...`、`--skill-lang zh|en`、`--commit-and-push` 等
2. 执行纳管命令：
   ```powershell
   cargo run -p pengj-templates-cli -- adopt <目标项目目录> --layers <层列表> [选项]
   ```
3. 引擎将生成 `.pengj-templates.json` 基线、为对应文件注入受管块、并集合成配置、自动接管全自定义技能。

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

### 4. 验证与提交

1. 进入目标项目目录，执行对应技术栈的验证检查：
   ```powershell
   # 如 Rust 项目
   cargo check; cargo test
   # 如前端项目
   pnpm build
   ```
2. 按照阶段性完工要求，触发 `commit` 技能完成提交并推送。
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属同步配置与快捷方式

> 可在此登记本项目常用的上游模板地址、默认层组合、前后置同步脚本等信息。
