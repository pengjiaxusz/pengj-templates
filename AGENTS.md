# AGENTS.md — pengj-templates 编码规范 / Coding Standards

> 供 AI 编码助手在修改本仓库时遵循。本文件内容与工具能力相关部分可能过时，以代码为准。
> For AI coding assistants editing this repository. When in doubt, trust the code.
>
> 提交规范请以 `.agents/skills/git-workflow` 技能为准（含提交前完整性检查与 pre-release 切发流程）。

## 项目概览 / Overview

分层模板生成与同步更新工具：把仓库模板按「层（layer）」组织，勾选所需层即可生成新仓库；模板更新后，一键同步到所有基于它生成的项目。

- **引擎**（Rust）：`crates/core` — 层发现、依赖排序、合并渲染、生成/更新、`.pengj-templates.json` manifest
- **CLI**（Rust/clap）：`crates/cli`，命令 `pengj-templates-cli`
- **MCP 安装**（Rust）：`crates/mcp` — 配方目录（catalog）、五家客户端适配、stdio 握手巡检；命令面挂在 `crates/cli` 的 `mcp` 子命令。它作用于**机器上的客户端配置**（`~/.workbuddy/mcp.json`、`~/.claude.json` 等），与项目模板无关，因此不依赖 `--templates`
- **提交门禁**（Rust）：`crates/gate` — 约定式提交校验、代码语言守卫、rustfmt 暂存格式化；零第三方依赖，已收编为内部库，唯一对外入口是 `pengj-templates-cli gate ...`
- **模板**：`templates/<layer>/`，运行时读取（改模板无需重编译，直接 re-run 生效）

## 提交规范 / Commits

- 约定式提交：`type(scope): 中文标题`（type/scope 英文、标题正文中文）。
- type：`feat fix docs style refactor perf test build ci chore revert`。
- **回复前提交门禁（硬性拦截，禁止堆积）**：凡本轮对话有任何代码/文档文件的创建、修改或删除，且通过了构建与测试验证，**在停止工具调用并向用户输出最终回复前，必须将运行提交流程（检查、拆分、提交并 git push）作为最后一个工具调用步骤**。严禁在工作区留存已验证修改直接结束回复；严禁在多轮对话中拖延堆积或等待用户催促 `/commit`；在 `/plan` 等规划模式下生成 `walkthrough.md` 后，必须紧随其后执行提交与推送，不可中途停顿。
- 提交前跑「提交前完整性检查」（构建验证 / 文档同步 / 格式与命名）；无关改动拆分多次提交；**提交后立即 push**。
- 提交由 `crates/gate` 校验（Rust 实现，无 Node 依赖），格式不符会被拦截；规则见 `.gate.toml`。

> EN: Conventional commits, Chinese subject. Type/scope whitelisted (see `.gate.toml`). Pre-response commit gate: whenever files are modified and verified in a turn, commit & push must be the final tool-calling step before outputting the final reply to user. Split unrelated changes; push immediately after commit; hooks enforce format.

## Rust 规范 / Rust (crates/core, crates/cli, crates/mcp, crates/gate)

- 编译检查：`cargo check --workspace`；测试：`cargo test --workspace`。
- 命名：变量/函数/模块 `snake_case`，类型/特征/枚举 `CamelCase`，常量 `SCREAMING_SNAKE_CASE`；文档注释用中文。
- 错误处理：`anyhow::Result` / `CoreError`（`thiserror`），禁止裸 `unwrap`/`panic`、禁止 `as any` 式逃生门。
- CLI 参数用 clap 派生；核心库的公开类型加 `#[derive(Debug, Clone, serde::Serialize)]` 供 CLI 复用。

> EN: workspace-level checks; Chinese doc comments; typed errors, no unwrap/panic.

## 提交门禁规范 / Commit gate (crates/gate)

- 零第三方依赖：只允许标准库，新增依赖要显式讨论。
- 规则集中在 `crates/gate/src/config.rs`（`.gate.toml` 解析）与 `crates/gate/src/commit.rs`（约定式提交校验），不要把规则散落到别处。
- 钩子安装走 `core.hooksPath=.githooks` + `git update-index --chmod=+x`——Linux 克隆会静默跳过没有可执行位的钩子，改钩子文件时必须确认模式位。
- 钩子文件必须含 marker `gate hook`；`install` 遇到非 gate 写的钩子会拒绝覆盖，并提示 lefthook / husky 与门禁争抢 `core.hooksPath`。
- 语料来自真实 commitlint 的裁决（`crates/gate/tests/commit_corpus.jsonl`）：门禁不能比它替换掉的工具更松，改规则前先看语料。
- 门禁只以 `crates/gate` 内部库的形式存在，唯一对外入口是 `pengj-templates-cli gate ...`（参数解析集中在 `crates/gate/src/cli.rs` 的 `run()`）。不再维护独立 `gate` 二进制——装一个 `pengj-templates-cli` 就同时拿到模板引擎、MCP 安装器与提交门禁。
- 钩子与 `just/gate.just` 统一直接调 `pengj-templates-cli gate ...`，不再做 `$PENGJ_GATE` / 独立 `gate` / 仓库 URL 的多级回退探测；仅本仓库自身（开发态）允许 `cargo run -p pengj-templates-cli -- gate ...` 兜底。
- **门禁只有一套实现**：规则全在 `crates/gate`（`commit.rs` / `lang_guard.rs` / `scanner.rs`），钩子与 `just/gate.just` 只负责找到 CLI 并把活交给它。模板**不产出任何兜底脚本**——原 `gate-fallback.sh` 已删除：没装 CLI 时 `commit-msg` 与 `pre-commit` 都直接拦下提交并打印安装命令，不放行、也不用第二套更宽松的 shell 规则顶替。所以改 `commit.rs` 的规则时不需要同步任何脚本。
- 服务端强制力在 `ci` 层：`gate` 任务跑的是同一个 CLI，`rulesets/gate.json` 把它设成必需状态检查。CI 里没有「降级检查」这条路径，本地与远端永远是同一套规则。
- `ci` 层的 workflow 里 job 名固定为 `gate`：必需状态检查引用的就是这个名字，不能因为 `skill_lang` 切换而失效。

> EN: the commit gate has zero third-party dependencies; rules live in config.rs / commit.rs; hooks need `core.hooksPath=.githooks` plus the executable bit recorded in the index; hook files carry the `gate hook` marker; the corpus pins the gate to commitlint's verdicts; the gate is an internal library reached only through `pengj-templates-cli gate ...` — one binary covers the template engine, the MCP installer and the commit gate. There is exactly one implementation: the hooks only locate the CLI and hand over, and block the commit with install instructions when it is missing (the old shell fallback is gone), while the `ci` job runs the same binary behind a required status check.

## 模板层规范 / Template layers (templates/)

- **新增层** = `templates/<layer>/` 目录 + `layer.toml`（`name` / `description` / `depends` / `update_ignore`），自动发现、无需注册。
- **合并规则**：按依赖顺序合并、后层覆盖前层；`.gitignore`、`.gitattributes` 按层累加拼接（`ACCUMULATE_FILES`）；`.cargo/config.toml` 走 TOML 结构化受管合并（表级并集、等值去重、冲突跳过并上报，见 `crates/core/src/toml_merge.rs`）。模板源文件带可执行位时（钩子脚本），写盘后由 `apply_executable_bits` 补回——`std::fs::write` 不复制权限位，缺位会让 git 在 Unix 上静默跳过钩子。
- **占位符**：minijinja（`{{ project_name }}`、`{{ project_slug }}`、`{{ year }}`、`layers`、`options`），渲染逻辑在 `crates/core/src/render.rs`。
- **选项**：生成时用户选项（edition、skills、skill_lang 等）固化进 `.pengj-templates.json` manifest；`update` 按同一批选项重渲染。
- **技能过滤**：`options["skills"]`（字符串数组）决定生成哪些技能文件；缺失时包含全部（向后兼容），过滤逻辑在 `crates/core/src/engine.rs`（`skill_name_of` / `selected_skills`）。
- 层内「种子文件、后续归用户」的文件在 `layer.toml` 的 `update_ignore` 声明，`update` 时跳过。
- **just / bque / gate 装配**：根 `justfile` 只由 `just` 层产出，框架区在 `# PENGJ_TEMPLATE_START/END` 托管块内（按 `layers` / `options["just_recipes"]` 条件 `import`），块外项目专属区归用户所有；`bque` / `gate` / `rust` / `rust-workspace` 层产出 `just/*.just` 模块并依赖 `just`。`justfile` **不得**列入 `update_ignore`，否则框架区无法随模板演进。
- **层-技能联动**：勾 `bque` 层时自动把 `bque` 技能并入 `options["skills"]`（`generate` / `adopt` / `update` / `add-layer` 四处一致），避免「有配方没文档」的半接入；`audit` 对半接入与缺少 justfile 托管块给出非阻断配置提示。
- **增量加层**：`add-layer --dir <项目> --layers <层>` 只合并层清单并重跑 `update`，禁止用 `adopt --force` 全量重纳管；已有自定义编排的项目设 `options["just_recipes"] = "none"` 保留自己的 recipe。
- **环境自检与钩子激活（唯一初始化入口）**：`common` 层产出 `scripts/doctor.sh` / `doctor.ps1`，只检查已选层/技能实际用到的工具（git、just/pwsh、rust、bque、CLI、node）与 gate 钩子激活状态；`--fix` / `-Fix` 通过 `core.hooksPath=.githooks` 激活钩子（只调用 gate 层的 `.githooks/install.sh`，不再保留 CLI / 裸 `git config` 的重复回退），并在 agent 层项目里调用 `pengj-templates-cli repo init` 创建工作区链接、补齐仓库级 Git 配置与未初始化子模块。生成项目统一用 `just doctor --fix` 初始化，不提供独立的 `setup` 配方；用户扩展检查与修复放 `scripts/doctor.d/*.sh|*.ps1`，doctor 按退出码汇总。CLI 安装器不再写全局 `core.hooksPath`；存量全局分发由 doctor 检测并提示 `git config --global --unset core.hooksPath`。
- **仓库初始化能力（CLI）**：`crates/core/src/repo.rs` 提供 `repo_status` / `repo_init`，CLI 暴露 `pengj-templates-cli repo status|init`，不依赖模板目录。默认链接 `.trae` → `.agents`、`.claude/skills` → `.agents/skills`（仅当项目含 `.agents`），可用 `--link <link>=<target>` 追加；不变式是只补缺失、不覆盖真实目录、目标必须未跟踪且被 `.gitignore` 忽略、子模块只初始化未初始化的。
- **gate→ci 联动**：`resolve_layers_with_linkage` 在 generate / adopt / update / add-layer 四处把 `ci` 并入 `gate` 的层清单（`layer.toml` 的 `depends` 会成环，不能靠它）；`audit` 走 `resolve_layers`，按 manifest 实际层巡检，避免存量项目首次 update 前误报缺少 ci 文件。

> EN: a layer = directory + layer.toml, auto-discovered. Merge by dependency order, later overrides earlier; .gitignore accumulates; the executable bit of hook scripts is preserved on write. minijinja placeholders; options persisted in manifest and reused by update. The root justfile is owned by the `just` layer via a managed block; queue/gate/language layers emit `just/*.just` modules and depend on it. Selecting the bque layer auto-selects the bque skill; `add-layer` appends layers without full re-adoption. The `common` layer ships `scripts/doctor.sh|ps1` — `just doctor --fix` is the single bootstrap entry point (there is no separate `setup` recipe); it checks only the tools the selected layers/skills use and activates the per-repo hooks through `.githooks/install.sh` only (no duplicated CLI / plain-`git config` fallbacks). In agent-layer projects it also calls `pengj-templates-cli repo init`, which owns the workspace links, repo-level Git config and submodule fill (`crates/core/src/repo.rs`); default links are `.trae` → `.agents` and `.claude/skills` → `.agents/skills`, and the invariants are: only fill missing entries, never overwrite real directories, require the link path to be untracked and gitignored, and initialize only uninitialized submodules. Custom checks/fixes live in `scripts/doctor.d/`. `resolve_layers_with_linkage` makes the gate layer imply `ci` (audit deliberately skips the linkage); the installers no longer write a global `core.hooksPath`.

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
- 新增技能必须照抄现有技能（如 `git-workflow`）的双语模板结构；新增语言时同步扩展 `skill_lang` 校验（CLI / core）。
- **只有文档双语，代码不双语**：`.agents/skills/*/scripts/` 下的脚本一律英文——标识符、结构化字段名（如 `type` / `source` / `entry`）、注释、日志输出都用英文，不随 `skill_lang` 变化。脚本是被 agent、CI 与其他工具执行的共享基础设施，中文标识符在编码、diff、shell 传参与工具链交接上都容易出问题；中文含义写在 `SKILL.md`，不写进脚本。存量中文脚本在改造时一并英文化（参考 `mcp-bootstrap/scripts/ensure-mcp.mjs`）。
- 四条语言轴互相独立：**中文编程**（目标项目代码的标识符，`chinese_programming`）、**技能文档语言**（`skill_lang`）、**提交信息语言**（`commit_zh`）、**技能脚本语言**（恒为英文，无选项）。
- **`skill_lang` 是全部模板层「面向人的文案」总开关**：除 agent 层文档外，`just` / `bque` / `gate` / `rust*` / `common` 各层的注释、报错提示、日志、`.gitignore` 拼接段标题都按 `{% if options["skill_lang"] == "en" %}` 分支渲染，`en` 时生成的所有文本文件不得出现汉字。新增模板文案必须同时写中英两支，且**中文分支渲染结果要与改动前逐字节一致**（否则存量中文项目整体漂移）。`commit_zh` 只控制门禁的提交主题语言规则与提交规范文档；全英文项目须同时设 `skill_lang=en` 与 `commit_zh=false`。技能脚本恒为英文，不随语言轴变化。
- **代码类产物按语言拆文件，不在脚本里写双语分支**：脚本、配置、YAML 等「另一种语言的逻辑会变成死代码」的文件，用**语言变体文件**各写一版完整文件，引擎按语言轴二选一写盘（输出时去掉语言标记，另一份完全不落到项目里）：
  - `<stem>.<lang>.<ext>`（如 `foo.zh.js` / `foo.en.js`）跟随 `skill_lang`；
  - `<stem>.commit.<lang>.<ext>` 跟随 `commit_zh`，用于提交校验类产物——`commit_zh` 与 `skill_lang` 是独立轴，「中文文档 + 英文提交」时按提交轴选版。
  - 解析实现：`crates/core/src/engine.rs::resolve_language_variant`；`update` 语义不变（产物路径无语言标记，manifest 照常记录）。
  - **例外：`.githooks/` 下的脚本用行内双语分支**。`commit-msg` 与 `pre-commit` 没有扩展名，拆不了；`install.sh` 与它们是同一套钩子的三个入口，拆成两份完整副本会让修复只落到一半。渲染结果照样满足「英文项目零汉字」——每个中文字符串都在 `{% if %}` 分支里，`crates/core` 的端到端单测会守住这条。
- **代码语言守卫（gate 层）**：门禁在 pre-commit 扫描暂存 `.rs`，按 `.gate.toml` 的 `mode` 判定——`english` 时标识符必须 ASCII、字符串文案必须英文；`chinese` 时字符串文案必须中文（标识符允许英文锚点，符合中文编程规范）。中文模式只判定面向人的文案：命令行参数、路径、协议字段名、内嵌源码/着色器代码等机器字面量不算违规，但仍计入分母，避免额度被稀释。判定按文件算占比：字符串违规数 ÷ 字符串总数 ≤ `lang.max_ratio`（默认 1%），标识符违规数 ÷ 标识符总数 ≤ `lang.max_ident_ratio`（默认 0），允许额度向下取整。真实例外在源码里认证：行尾 `lang-ok: 理由` 免该行，文件头 `lang-allow: 理由` 免整文件（i18n 词表、CJK 测试样本）。`.gate.toml` 属 `update_ignore`（首次生成后归用户所有），但 `.githooks/*` 由模板托管、随更新同步；存量项目从 `lefthook` 层迁过来时，层名由 `lefthook` 自动映射为 `gate`（`LEGACY_LAYER_ALIASES`），门禁不会被静默丢弃。模板仓库自身的中文 CLI 文案是 `skill_lang=zh` 的文案，不是英文编程项目的代码，因此本仓库在 `.gate.toml` 的 `project.disabled` 里关掉了 `lang-guard`。
- **文档语言门禁（双向）**：`crates/core/src/i18n_audit.rs` 提供 `audit_generated` / `audit_project_lang`，对**全部生成文本文件**做双向校验——英文项目零汉字、中文项目面向人的文案必须含汉字（纯英文模板即违规，典型是漏写双语分支的 `doctor.d/README.md`）。机器产物（`.json`、`.agents/skills/*/scripts/*`、`Cargo.toml`、`rust-toolchain.toml`、`src/main.rs`）走窄白名单，白名单条目要有理由，新增前先确认它不承载面向人文案。CLI 入口 `pengj-templates-cli check-lang --dir <项目>`（可 `--lang zh|en` 覆盖、`--json`），只审计 manifest 记录的模板托管文件；`gate` 层产出 `just gate-lang` 配方。模板仓库的 `skill_lang_drives_every_generated_file_language` 端到端测试已收口到这个审计器，CI（`cargo test --workspace`）会在模板漂移时拦截。

> EN: ALL agent-layer content (AGENTS.md template + every skill) must be bilingual zh/en via `{% if options["skill_lang"] == "en" %}` branches. Frontmatter descriptions too. Copy the `git-workflow` skill's structure for new skills. Docs are bilingual, code is not: scripts under `.agents/skills/*/scripts/` are always English (identifiers, structured field names, comments, log lines) regardless of `skill_lang` — they are shared infrastructure run by agents, CI and other tools, so Chinese identifiers hurt encoding, diffing and shell hand-off. Put the Chinese meaning in `SKILL.md`. Four independent language axes: `chinese_programming` (identifiers in the target project), `skill_lang` (all human-facing text in every layer), `commit_zh` (the gate's commit-subject language rule + commit guidelines), and skill scripts (always English, no option). A fully English project needs `skill_lang=en` together with `commit_zh=false`; inline `{% if %}` branches must keep the zh output byte-identical. Code-like artifacts must NOT carry bilingual runtime branches: split them into language-variant files — `<stem>.<lang>.<ext>` follows `skill_lang`, `<stem>.commit.<lang>.<ext>` follows `commit_zh` — and the engine writes only the selected one under the unmarked name (`resolve_language_variant` in `crates/core/src/engine.rs`). The gate layer ships the code-language guard: a pre-commit check that reads `mode` from `.gate.toml` and, per file, caps violating strings at `lang.max_ratio` (default 1%) and violating identifiers at `lang.max_ident_ratio` (default 0, rounded down). Chinese mode judges human-facing copy only — flags, paths, protocol field names and embedded source are machine text, not violations, yet they stay in the denominator so the budget is not diluted. Genuine exceptions are certified in source with `lang-ok: <reason>` (line) or `lang-allow: <reason>` (file: i18n tables, CJK fixtures). `.gate.toml` is in `update_ignore` (it belongs to the project after the first generation), while `.githooks/*` stays template-managed; projects migrating from the old `lefthook` layer get their layer name mapped to `gate` automatically (`LEGACY_LAYER_ALIASES`), so the gate is never silently dropped. This repository's own Chinese CLI copy belongs to the `skill_lang=zh` axis, not to English-programming code, so the language guard is disabled here through `project.disabled`. A separate **bidirectional documentation-language gate** lives in `crates/core/src/i18n_audit.rs`: every generated text file must have zero Han in an English project and human-facing copy must contain Han in a Chinese project (an English-only prose file means a missing bilingual branch). Machine artifacts (`.json`, `.agents/skills/*/scripts/*`, `Cargo.toml`, `rust-toolchain.toml`, `src/main.rs`) use a narrow, documented allowlist. CLI: `pengj-templates-cli check-lang --dir <project>` (`--lang zh|en`, `--json`), auditing only manifest-managed files; the `gate` layer ships a `just gate-lang` recipe. The template repo's end-to-end `skill_lang_drives_every_generated_file_language` test uses this auditor, so CI (`cargo test --workspace`) blocks drift.

## 验证 / Verification

改动提交前至少跑与改动面相关的检查：

- Rust / 模板 / 构建配置：按 `.agents/skills/git-workflow` 的提交前检查执行，再跑 `cargo test --workspace`。
- 模板改动：临时目录端到端验证，如 `cargo run -q -p pengj-templates-cli -- create demo --layers agent --skills git-workflow,caveman --output <临时目录>`，检查生成文件与 AGENTS.md 渲染结果。
- 全量 CI 校验：`just ci`（fmt check + 编译 + 静态检查 + 测试）。

> EN: run the checks matching your change surface before committing (see above). Template changes: verify with an end-to-end CLI create into a temp dir. Full check: `just ci`.

## 文档 / Docs

- 根 `README.md` 为**英文**主文档（GitHub 默认展示），`README.zh-CN.md` 为中文版，两版顶部互相挂语言切换；两版内容保持 1:1（同样的章节、命令、参数与标识符），改动任一侧都要同步另一侧。`docs/architecture/**` 与 `templates/README.md` 保持中文。agent 层内容双语。
- CI 对纯文档改动（根目录 `*.md`、`docs/**`、`.agents/**`）自动跳过；模板（`templates/**`）改动必触发构建。
- 版本号与发布走 release-please（conventional commits 驱动），见 `.github/workflows/release.yml`。

<!-- PENGJ_TEMPLATE_START -->

# pengj-templates 编码规范

## 通用约定
- 编译/检查/测试/跑分统一走 `just`（`just check`、`just test`、`just q-run <cmd>`），不裸跑 `cargo`/`ninja`/`cmake`；详见 `.agents/skills/bque/SKILL.md`。
- 完成修改并通过验证后，先 commit & push 再回复；纯问答或构建/测试失败待排查时除外。
- 提交标题必须用中文（至少 2 个汉字），type/scope 用英文；任何提交入口（含 /commit 命令）都不得写纯英文标题，commit-msg 钩子会拦截。
- 改动涉及模块边界、公共 API、数据流、生命周期或新增模块/子系统时，同步更新对应的架构/设计文档再提交。
<!-- PENGJ_TEMPLATE_END -->
