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
- After cloning, run `just doctor --fix` (or `sh scripts/doctor.sh --fix` / `pwsh -NoProfile -File scripts/doctor.ps1 -Fix` when `just` is unavailable): it checks the toolchain, activates the commit hooks, creates the agent-client workspace links (`.trae` / `.claude/skills` / `.workbuddy/skills`), and runs the project fixes in `scripts/doctor.d/`. A missing `pengj-templates-cli` is an error, not a warning — the gate and the links both need it — and `--install-cli` is the only step that touches the network (`--fix` never does).
{%- if options["commit_zh"] %}
- Commit subjects MUST be Chinese (at least 2 CJK characters); type/scope stay English. Every commit entry point (including /commit commands) follows this rule — the commit-msg hook rejects pure-English subjects.
{%- else %}
- Commit subjects MUST be pure English (no CJK characters); type/scope stay English. Every commit entry point follows this rule — the commit-msg hook rejects Chinese subjects.
{%- endif %}
{%- if 'gate' in layers %}
- The commit gate runs from `.githooks/`; activate it with `just doctor --fix` (or `sh scripts/doctor.sh --fix`) — the doctor calls `.githooks/install.sh`, which needs only git and a POSIX shell, so activating the hooks does not need the CLI. The gate itself runs from `pengj-templates-cli`: without it the hooks block the commit and print the install command. The gate layer implies the `ci` layer, which runs the same binary as a required status check.
- There is exactly one rule set: the hooks only locate the CLI and hand over, and the template ships no fallback script. A machine without the CLI cannot commit locally until it installs the CLI — never a weaker local check that might disagree with CI. The `gate` job in `.github/workflows/gate.yml` and its ruleset in `.github/rulesets/gate.json` enforce the same rules server-side.
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
- 克隆后跑 `just doctor --fix`（没有 just 时用 `sh scripts/doctor.sh --fix` / `pwsh -NoProfile -File scripts/doctor.ps1 -Fix`）：检查工具链、激活提交钩子、创建 agent 客户端工作区链接（`.trae` / `.claude/skills` / `.workbuddy/skills`），并执行 `scripts/doctor.d/` 下的项目自检与修复。缺 `pengj-templates-cli` 是**错误**而不是警告（门禁与链接都依赖它），唯一的联网入口是 `--install-cli`，`--fix` 永不联网。
{%- if options["commit_zh"] %}
- 提交标题必须用中文（至少 2 个汉字），type/scope 用英文；任何提交入口（含 /commit 命令）都不得写纯英文标题，commit-msg 钩子会拦截。
{%- else %}
- 提交标题必须用纯英文（不得含汉字），type/scope 用英文；任何提交入口都不得写中文标题，commit-msg 钩子会拦截。
{%- endif %}
{%- if 'gate' in layers %}
- {% if options["skill_lang"] | default("zh") == "en" %}The commit gate runs from `.githooks/`; activate it with `just doctor --fix` (or `sh scripts/doctor.sh --fix`) — the doctor calls `.githooks/install.sh`, which needs only git and a POSIX shell, so activating the hooks does not need the CLI. The gate itself runs from `pengj-templates-cli`: without it the hooks block the commit and print the install command. The gate layer implies the `ci` layer, which runs the same binary as a required status check.{% else %}提交门禁由 `.githooks/` 执行，激活入口是 `just doctor --fix`（或 `sh scripts/doctor.sh --fix`）：doctor 会调用 `.githooks/install.sh`，后者只需要 git 与一个 POSIX shell，所以激活钩子不需要 CLI。门禁本体在 `pengj-templates-cli` 里：没装它时钩子会拦下提交并打印安装命令。gate 层会自动带上 `ci` 层，服务端跑的是同一个二进制，并把它设成必需状态检查。{% endif %}
- {% if options["skill_lang"] | default("zh") == "en" %}There is exactly one rule set: the hooks only locate the CLI and hand over, and the template ships no fallback script. A machine without the CLI cannot commit locally until it installs the CLI — never a weaker local check that might disagree with CI. The `gate` job in `.github/workflows/gate.yml` and its ruleset in `.github/rulesets/gate.json` enforce the same rules server-side.{% else %}规则只有一套：钩子只负责找到 CLI 并把活交给它，模板不带兜底脚本。没装 CLI 的机器在装好之前无法在本地提交，刻意不提供一套可能与 CI 判决不一致的更宽松检查。同样的规则由 `.github/workflows/gate.yml` 的 `gate` 任务与 `.github/rulesets/gate.json` 的必需状态检查在服务端执行。{% endif %}
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
