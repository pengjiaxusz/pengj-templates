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
{%- if options["commit_zh"] %}
- Commit subjects MUST be Chinese (at least 2 CJK characters); type/scope stay English. Every commit entry point (including /commit commands) follows this rule — the commit-msg hook rejects pure-English subjects.
{%- else %}
- Commit subjects MUST be pure English (no CJK characters); type/scope stay English. Every commit entry point follows this rule — the commit-msg hook rejects Chinese subjects.
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
{%- if options["commit_zh"] %}
- 提交标题必须用中文（至少 2 个汉字），type/scope 用英文；任何提交入口（含 /commit 命令）都不得写纯英文标题，commit-msg 钩子会拦截。
{%- else %}
- 提交标题必须用纯英文（不得含汉字），type/scope 用英文；任何提交入口都不得写中文标题，commit-msg 钩子会拦截。
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
