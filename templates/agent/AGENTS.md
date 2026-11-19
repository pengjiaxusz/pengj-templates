<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# {{ project_name }} Coding Standards

{%- if 'bque' in layers or (options["skills"] is defined and 'bque' in options["skills"]) %}
- **Build and test**: Run `just check` / `just test`.
{%- endif %}
{%- if options["commit_and_push"] %}
- **Commit before reply**: Commit & push verified changes before replying (conventional commit: `type(scope): subject`).
{%- endif %}
- **Code comments**: Comments must be in English.
{%- if 'gate' in layers %}
{%- if not options["chinese_programming"] %}
- **Code-language guard**: English code and strings only (no foreign script); exceptions use `lang-ok` / `lang-allow`, audit with {% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}.
{%- else %}
- **Code-language guard**: Chinese programming mode; see conventions below.
{%- endif %}
{%- endif %}
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

{%- if 'bque' in layers or (options["skills"] is defined and 'bque' in options["skills"]) %}
- **构建与测试**：统一走 `just check` / `just test`。
{%- endif %}
{%- if options["commit_and_push"] %}
- **回复前提交**：验证通过后先 commit & push 再回复（约定式提交：`type(scope): {% if options["commit_zh"] %}中文摘要{% else %}英文摘要{% endif %}`）。
{%- endif %}
- **代码注释**：注释强制中文，专有名词保留英文。
{%- if 'gate' in layers %}
{%- if not options["chinese_programming"] %}
- **代码语言守卫**：代码与字符串一律英文（中文仅限注释）；例外标 `lang-ok` / `lang-allow`，自检跑 {% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}。
{%- else %}
- **代码语言守卫**：中文编程模式，见下方约定；自检跑 {% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}。
{%- endif %}
{%- endif %}
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
