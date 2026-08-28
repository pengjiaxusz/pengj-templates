# AGENTS.md — pengj-templates 编码规范 / Coding Standards

> 供 AI 编码助手在修改本仓库时遵循。本文件内容与工具能力相关部分可能过时，以代码为准。
> For AI coding assistants editing this repository. When in doubt, trust the code.
>
> 提交规范请以 `.agents/skills/commit` 技能为准（含提交前完整性检查与 pre-release 切发流程）。

## 项目概览 / Overview

分层模板生成与同步更新工具：把仓库模板按「层（layer）」组织，勾选所需层即可生成新仓库；模板更新后，一键同步到所有基于它生成的项目。

- **引擎**（Rust）：`crates/core` — 层发现、依赖排序、合并渲染、生成/更新、`.pengj-templates.json` manifest
- **CLI**（Rust/clap）：`crates/cli`，命令 `pengj-templates-cli`
- **GUI**（Tauri 2 + React + Vite + TypeScript + Tailwind v4 + shadcn/ui）：`crates/app`（`src-tauri/` 为 Rust 壳，`src/` 为前端）
- **模板**：`templates/<layer>/`，运行时读取（改模板无需重编译，直接 re-run 生效）

## 提交规范 / Commits

- 约定式提交：`type(scope): 中文标题`（type/scope 英文、标题正文中文）。
- type：`feat fix docs style refactor perf test build ci chore revert`。
- scope 白名单见 `commitlint.config.js`：`agent app cli core ci templates main`；无合适 scope 时省略；需要新 scope 时按 commit 技能流程更新白名单。
- 提交前跑「提交前完整性检查」（构建验证 / 文档同步 / 格式与命名）；无关改动拆分多次提交；**提交后立即 push**。
- 提交由 lefthook + commitlint 校验，格式不符会被拦截。

> EN: Conventional commits, Chinese subject. Type/scope whitelisted (see commitlint.config.js). Split unrelated changes; push immediately after commit; hooks enforce format.

## Rust 规范 / Rust (crates/core, crates/cli, src-tauri)

- 格式化：`cargo fmt`；静态检查：`cargo clippy --workspace --all-targets -- -D warnings`（warnings 视为错误）。
- 编译检查：`cargo check --workspace`；测试：`cargo test --workspace`。
- 命名：变量/函数/模块 `snake_case`，类型/特征/枚举 `CamelCase`，常量 `SCREAMING_SNAKE_CASE`；文档注释用中文。
- 错误处理：`anyhow::Result` / `CoreError`（`thiserror`），禁止裸 `unwrap`/`panic`、禁止 `as any` 式逃生门。
- CLI 参数用 clap 派生；核心库的公开类型加 `#[derive(Debug, Clone, serde::Serialize)]` 供 CLI/GUI 共用。

> EN: rustfmt + clippy -D warnings required; workspace-level checks; Chinese doc comments; typed errors, no unwrap/panic.

## 前端规范 / Frontend (crates/app/src)

- 构建：`pnpm --dir crates/app build`（tsc 类型检查 + vite 产物）。
- React + TypeScript + Tailwind v4 + shadcn/ui；组件放 `src/components/ui/`，页面逻辑放 `src/App.tsx` 或 `src/components/`。
- UI 文案用中文；与 Tauri 通信走 `invoke("cmd_xxx", ...)`，命令定义在 `crates/app/src-tauri/src/lib.rs`。
- 新增后端命令时：`#[tauri::command]` + `invoke_handler` 注册 + 前端 `invoke` 三处同步。

> EN: tsc + vite build; shadcn/ui components; Chinese UI copy; Tauri commands defined in src-tauri/src/lib.rs.

## 模板层规范 / Template layers (templates/)

- **新增层** = `templates/<layer>/` 目录 + `layer.toml`（`name` / `description` / `depends` / `update_ignore`），自动发现、无需注册。
- **合并规则**：按依赖顺序合并、后层覆盖前层；`.gitignore`、`.gitattributes` 按层累加拼接（`ACCUMULATE_FILES`）；`package.json` 结构化并集（`MERGE_JSON_FILES`，更新/纳管时依赖类字段同名包版本一律以模板为准，脚本与其余字段用户优先、键序保持）；`.cargo/config.toml` 走 TOML 结构化受管合并（表级并集、等值去重、冲突跳过并上报，见 `crates/core/src/toml_merge.rs`）。
- **占位符**：minijinja（`{{ project_name }}`、`{{ project_slug }}`、`{{ year }}`、`layers`、`options`），渲染逻辑在 `crates/core/src/render.rs`。
- **选项**：生成时用户选项（edition、skills、skill_lang 等）固化进 `.pengj-templates.json` manifest；`update` 按同一批选项重渲染。
- **技能过滤**：`options["skills"]`（字符串数组）决定生成哪些技能文件；缺失时包含全部（向后兼容），过滤逻辑在 `crates/core/src/engine.rs`（`skill_name_of` / `selected_skills`）。
- 层内「种子文件、后续归用户」的文件在 `layer.toml` 的 `update_ignore` 声明，`update` 时跳过。

> EN: a layer = directory + layer.toml, auto-discovered. Merge by dependency order, later overrides earlier; .gitignore accumulates; package.json unions. minijinja placeholders; options persisted in manifest and reused by update.

## 技能规范 / Skills

- 技能模板放 `templates/agent/.agents/skills/<name>/SKILL.md`，**自动发现无需注册**；生成后落到项目的 `.agents/skills/<name>/SKILL.md`。
- 技能列表由 `Templates::list_skills()` 提供（渲染后解析 frontmatter 的 `description` 供 UI/CLI 展示）。
- 新技能目录名即技能名；`SKILL.md` 必须含 frontmatter（`name` + 双语 `description`，含 Triggers）。
- **技能扩展规范**：SKILL.md = 托管框架块（模板更新原位替换）+ 块外项目专属区（归用户，文档型定制与提交前检查的定义都写这里）；可执行门禁由项目自定形式并在块外声明，框架不预设；存量全自定义技能（无托管块）adopt/update 自动接管——模板整页（含 description）覆盖、原正文下移为纳管过渡区待用户合并（见 `engine.rs` 的 `take_over_legacy_skill`）。
- 本仓库自己的 `.agents/skills/` 与模板 `templates/agent/.agents/skills/` 各自维护：前者是当前仓库生效的渲染版，后者是生成给新项目的模板版。

> EN: skills live at templates/agent/.agents/skills/<name>/SKILL.md, auto-discovered (no registration). Frontmatter requires name + bilingual description with Triggers. Extension convention: managed framework block + user-owned area outside it (document-type customization and check definitions live there; executable gates are project-defined and declared in that area); legacy fully-custom skills are taken over automatically — the template page (incl. description) wins and the original body moves into a transition zone for manual merging (see take_over_legacy_skill in engine.rs). The repo's own .agents/skills/ holds rendered copies; templates/agent/ holds the template versions.

## 多语言要求 / i18n（agent 层，当前支持 zh/en）

- **agent 层全部内容必须中英双语**：`templates/agent/AGENTS.md` 与每个技能 `SKILL.md`。
- 双语写法：`{% if options["skill_lang"] == "en" %}` … `{% else %}` … `{% endif %}` 分支（zh 为默认）；frontmatter 的 `description` 也要双语分支（供 UI 勾选列表展示）。
- 新增技能必须照抄现有技能（如 `commit`）的双语模板结构；新增语言时同步扩展 `skill_lang` 校验（CLI / GUI / core）。
- 三个中文概念互相独立：**中文编程**（代码标识符，`chinese_programming`）、**技能用中文写**（文档语言，`skill_lang`）、**提交信息是中文**（提交信息语言，`commit_zh`）。

> EN: ALL agent-layer content (AGENTS.md template + every skill) must be bilingual zh/en via `{% if options["skill_lang"] == "en" %}` branches. Frontmatter descriptions too (shown in the GUI skill picker). Copy the commit skill's structure for new skills.

## 验证 / Verification

改动提交前至少跑与改动面相关的检查：

- Rust / 模板 / 构建配置：`cargo fmt` → `cargo clippy --workspace --all-targets -- -D warnings` → `cargo test --workspace`。
- 前端（`crates/app/src` 等）：`pnpm --dir crates/app build`。
- 模板改动：临时目录端到端验证，如 `cargo run -q -p pengj-templates-cli -- create demo --layers agent --skills commit,caveman --output <临时目录>`，检查生成文件与 AGENTS.md 渲染结果。
- 全量 CI 校验：`just ci`（fmt check + 编译 + clippy + 测试 + 前端构建）。

> EN: run the checks matching your change surface before committing (see above). Template changes: verify with an end-to-end CLI create into a temp dir. Full check: `just ci`.

## 文档 / Docs

- `README.md` 为中文主文档；agent 层内容双语。
- CI 对纯文档改动（根目录 `*.md`、`docs/**`、`.agents/**`）自动跳过；模板（`templates/**`）改动必触发构建。
- 版本号与发布走 release-please（conventional commits 驱动），见 `.github/workflows/release.yml` 与 commit 技能 §5b。

<!-- PENGJ_TEMPLATE_START -->

# pengj-templates 编码规范

> 供 AI 编码助手在修改本仓库时遵循。由 pengj-templates 的 `agent` 层按所选层与选项生成。

## 通用约定

- 提交信息遵守约定式提交：`type(scope): 标题`，type 用英文、标题正文用中文（见 `.agents/skills/commit`）。
- 改动遵循最小化与可读性，先对齐仓库现有风格；能复用不新造。
- 涉及构建/依赖、文档、公开命名时，改完先自检再收尾。


### 启用的技能


- `commit` —— 见 `.agents/skills/commit/SKILL.md`

- `arch-align` —— 见 `.agents/skills/arch-align/SKILL.md`

- `branch-sync` —— 见 `.agents/skills/branch-sync/SKILL.md`

- `caveman` —— 见 `.agents/skills/caveman/SKILL.md`

- `grill-me` —— 见 `.agents/skills/grill-me/SKILL.md`

- `write-a-skill` —— 见 `.agents/skills/write-a-skill/SKILL.md`


### 技能扩展规范

- 技能文件 = 托管框架 + 项目专属区：`SKILL.md` 中 `PENGJ_TEMPLATE_START/END` 托管块内是模板框架（更新时原位替换）；**块外内容归项目所有**，模板更新永不触碰。
- 文档型定制（领域检查清单、判定表、红线）直接写在托管块外的项目专属区，保证单次读取即可获得全部约定，不要另建外部文件。
- 可执行门禁的形式与位置由项目自定（任意语言脚本、task runner 或纯清单），并在项目专属区写明调用方式；框架不预设实现。
- 存量的全自定义技能（无托管块）在纳管/更新时自动接管：以模板整页为准（frontmatter 含 description 一并覆盖），原正文下移为「纳管过渡区」（暂时双流程），由用户把领域差异合并进项目专属区后删除过渡区。





## Rust 编码规范

- 格式化：`cargo fmt`；CI 用 `just fmt`（`cargo fmt --check`）。
- 静态检查：`cargo clippy --all-targets --all-features -- -D warnings`（`just check`），保持 warnings 清零。
- 命名：变量/函数/模块 `snake_case`，类型/特征/枚举 `CamelCase`，常量 `SCREAMING_SNAKE_CASE`。
- 《`Cargo.toml`、`.cargo/config.toml`、`rust-toolchain.toml` 由模板托管：改编译选项、工具链需同步模板（`pengj-templates-cli update`）。`src/main.rs` 归用户所有，模板更新不覆盖。



<!-- PENGJ_TEMPLATE_END -->
