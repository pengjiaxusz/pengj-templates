<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# {{ project_name }} Coding Standards

## General conventions
{%- if 'bque' in layers or (options["skills"] is defined and 'bque' in options["skills"]) %}
- Route all builds, checks, tests and benchmarks through `just` (`just check`, `just test`, `just q-run <cmd>`); never run raw `cargo`/`ninja`/`cmake`. See `.agents/skills/bque/SKILL.md`.
{%- endif %}
{%- if options["commit_and_push"] %}
- After changes pass verification, commit and push before replying. Skip only for pure Q&A or when reporting failing builds/tests.
{%- endif %}
- After cloning, run `just doctor --fix` (or `sh scripts/doctor.sh --fix` / `pwsh -NoProfile -File scripts/doctor.ps1 -Fix` when `just` is unavailable): it checks the toolchain, activates the commit hooks, creates the agent-client workspace links (`.trae` / `.claude/skills`), and runs the project fixes in `scripts/doctor.d/`.
{%- if options["commit_zh"] %}
- Commit subjects MUST be Chinese (at least 2 CJK characters); type/scope stay English. Every commit entry point (including /commit commands) follows this rule — the commit-msg hook rejects pure-English subjects.
{%- else %}
- Commit subjects MUST be pure English (no CJK characters); type/scope stay English. Every commit entry point follows this rule — the commit-msg hook rejects Chinese subjects.
{%- endif %}
{%- if 'gate' in layers %}
- The commit gate runs from `.githooks/`; activate it with `just doctor --fix` (or `sh scripts/doctor.sh --fix`) — the doctor calls `.githooks/install.sh`, which needs only git and a POSIX shell, so no CLI install is required first. The gate layer implies the `ci` layer, so machines without the CLI are covered by the server-side CI.
- Without the CLI, the commit message is still checked by the reduced rule set in `.githooks/gate-fallback.sh`; the `pre-commit` hook warns and lets the commit through instead of blocking on checks it cannot run. The full rule set stays with the server-side CI: the `gate` job is a required status check, and its ruleset lives in `.github/rulesets/gate.json`.
- Check the subject before committing: it must match `type(scope): subject`, the type must be one of `commit.types` in `.gate.toml`, {% if options["commit_zh"] %}the subject needs at least 2 Han characters{% else %}the subject must carry no Han characters{% endif %}, it must not end with a full stop, and the header stays within 100 characters. The gate enforces all of it, but a self-check is faster than a rejection.
- Code-language guard: every commit checks identifiers and string copy against `.gate.toml` (`mode`) — English programming rejects non-English, Chinese programming rejects non-Chinese. Chinese mode judges human-facing copy only: command-line flags, paths, protocol field names and embedded source are machine text, not violations (they still count toward the denominator so the budget is not diluted). Each file tolerates a small ratio of string exceptions (default 1%); genuine ones must be certified in source with `lang-ok: <reason>` on the line or `lang-allow: <reason>` near the top. Full audit: {% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}.
{%- endif %}
- If a change touches module boundaries, public APIs, data flow, lifecycle, or adds a module/subsystem, update the matching architecture/design docs before committing.
{%- if options["chinese_programming"] %}

### Chinese programming conventions

- Prefer Chinese file/variable names; keep short common English terms. Types use "English anchor + Chinese role" (`HttpRequest快照`).
- No literal jargon translation (plane→通路, edge→依赖, node→条目); keep spec proper nouns (Win32, Vulkan) in English.
{%- endif %}
{%- if ('rust' in layers or 'rust-workspace' in layers) and options["chinese_programming"] %}

### Rust Chinese-programming specifics

- Keep entry files (`main.rs`/`lib.rs`/`mod.rs`) ASCII; map Chinese-named files via `#[path]`.
- Chinese identifiers need no snake_case; pure-ASCII ones follow Rust conventions. Prefer derive macros (e.g. `derive_more`).
{%- endif %}
{%- else %}
# {{ project_name }} 编码规范

## 通用约定
{%- if 'bque' in layers or (options["skills"] is defined and 'bque' in options["skills"]) %}
- 编译/检查/测试/跑分统一走 `just`（`just check`、`just test`、`just q-run <cmd>`），不裸跑 `cargo`/`ninja`/`cmake`；详见 `.agents/skills/bque/SKILL.md`。
{%- endif %}
{%- if options["commit_and_push"] %}
- 完成修改并通过验证后，先 commit & push 再回复；纯问答或构建/测试失败待排查时除外。
{%- endif %}
- 克隆后跑 `just doctor --fix`（没有 just 时用 `sh scripts/doctor.sh --fix` / `pwsh -NoProfile -File scripts/doctor.ps1 -Fix`）：检查工具链、激活提交钩子、创建 agent 客户端工作区链接（`.trae` / `.claude/skills`），并执行 `scripts/doctor.d/` 下的项目自检与修复。
{%- if options["commit_zh"] %}
- 提交标题必须用中文（至少 2 个汉字），type/scope 用英文；任何提交入口（含 /commit 命令）都不得写纯英文标题，commit-msg 钩子会拦截。
{%- else %}
- 提交标题必须用纯英文（不得含汉字），type/scope 用英文；任何提交入口都不得写中文标题，commit-msg 钩子会拦截。
{%- endif %}
{%- if 'gate' in layers %}
- {% if options["skill_lang"] | default("zh") == "en" %}The commit gate runs from `.githooks/`; activate it with `just doctor --fix` (or `sh scripts/doctor.sh --fix`) — the doctor calls `.githooks/install.sh`, which needs only git and a POSIX shell, so no CLI install is required first. The gate layer implies the `ci` layer, so machines without the CLI are covered by the server-side CI.{% else %}提交门禁由 `.githooks/` 执行，激活入口是 `just doctor --fix`（或 `sh scripts/doctor.sh --fix`）：doctor 会调用 `.githooks/install.sh`，后者只需要 git 与一个 POSIX shell，不需要先装 CLI。gate 层会自动带上 `ci` 层，未装 CLI 的机器由服务端 CI 执行同样的规则。{% endif %}
- {% if options["skill_lang"] | default("zh") == "en" %}Without the CLI, the commit message is still checked by the reduced rule set in `.githooks/gate-fallback.sh`; the `pre-commit` hook warns and lets the commit through instead of blocking on checks it cannot run. The full rule set stays with the server-side CI: the `gate` job is a required status check, and its ruleset lives in `.github/rulesets/gate.json`.{% else %}没装 CLI 时，提交标题仍由 `.githooks/gate-fallback.sh` 的缩减规则集检查；`pre-commit` 里的 rustfmt 与语言守卫跑不了，钩子会告警放行而不是堵死提交。完整规则集仍归服务端 CI：`gate` 任务是必需状态检查，ruleset 配置在 `.github/rulesets/gate.json`。{% endif %}
- {% if options["skill_lang"] | default("zh") == "en" %}Check the subject before committing: it must match `type(scope): subject`, the type must be one of `commit.types` in `.gate.toml`, {% if options["commit_zh"] %}the subject needs at least 2 Han characters{% else %}the subject must carry no Han characters{% endif %}, it must not end with a full stop, and the header stays within 100 characters. The gate enforces all of it, but a self-check is faster than a rejection.{% else %}提交前自查标题：必须匹配 `type(scope): 摘要`，type 取自 `.gate.toml` 的 `commit.types`，{% if options["commit_zh"] %}摘要至少 2 个汉字{% else %}摘要不含汉字{% endif %}，且不以句号结尾，标题不超过 100 字符。门禁会兜住全部，但先自查比被拦下更快。{% endif %}
- 代码语言守卫：每次提交按 `.gate.toml` 的 `mode` 校验标识符与字符串文案——英文编程拒绝非英文，中文编程拒绝非中文。中文模式只判定面向人的文案，命令行参数、路径、协议字段名、内嵌源码等机器字面量不算违规（仍计入分母，避免额度被稀释）。每个文件容忍小比例（默认 1%）字符串例外；真实例外必须在源码里认证：行尾 `lang-ok: 理由`，或文件头附近 `lang-allow: 理由`。全量自检：{% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}。
{%- endif %}
- 改动涉及模块边界、公共 API、数据流、生命周期或新增模块/子系统时，同步更新对应的架构/设计文档再提交。
{%- if options["chinese_programming"] %}

### 中文编程规范

- 文件名/变量名优先中文，简短通用英文保留；类型用「英文锚点 + 中文角色」（如 `HttpRequest快照`）。
- 禁止英文黑话直译：plane→通路、edge→依赖、node→条目；规范专名（Win32/Vulkan 等）保留英文。
{%- endif %}
{%- if ('rust' in layers or 'rust-workspace' in layers) and options["chinese_programming"] %}

### Rust 中文编程特定规范

- 入口文件（`main.rs`/`lib.rs`/`mod.rs`）保持 ASCII；中文源文件用 `#[path]` 映射模块。
- 含中文标识符不强制蛇形，纯英文标识符仍遵循 Rust 惯例；优先用派生宏（如 `derive_more`）减少样板。
{%- endif %}
{%- endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- Project-specific rules go below (outside the block; preserved on update). -->
{% else %}
<!-- 项目专属规范写在下方（块外，模板更新时保留）。 -->
{% endif %}
