<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# {{ project_name }} Coding Standards

## Workflow Invariants
{%- if 'bque' in layers or (options["skills"] is defined and 'bque' in options["skills"]) %}
- **Build and test**: Route all builds, checks and tests through `just` (`just check`, `just test`); do not run raw build commands.
{%- endif %}
{%- if options["commit_and_push"] %}
- **Commit before reply**: After changes pass verification, commit and push before replying. Skip only for pure Q&A or when reporting failures.
{%- endif %}
- **Conventional commits**: Subject must match `type(scope): subject`, type from `.pengj-templates/gate.toml`, {% if options["commit_zh"] %}subject must contain at least 2 Chinese characters{% else %}subject must be pure English (no Han characters){% endif %}, no trailing period, length <= 100 chars (see `git-workflow` skill).

## Code and Comment Languages
- **Code comments**: All doc comments (`//!`, `///`) and inline comments (`//`, `/* */`) must be written in English.
{%- if 'gate' in layers %}
{%- if not options["chinese_programming"] %}
- **Code-language guard (English programming)**:
  - **English identifiers only**: Function, type, field and variable names use ASCII only.
  - **Default English strings**: Assertions, error messages and fixtures must be English.
  - **Exemptions**: Line exemption via `lang-ok: <reason>`; whole file via `lang-allow: <reason>` (e.g. data fixtures). Full audit: {% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}.
{%- else %}
- **Code-language guard (Chinese programming)**: Human-facing copy checked against `.pengj-templates/gate.toml`. Exemptions via `lang-ok: <reason>` / `lang-allow: <reason>`. Full audit: {% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}.
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

## 工作流红线
{%- if 'bque' in layers or (options["skills"] is defined and 'bque' in options["skills"]) %}
- **构建与测试**：编译、检查与测试统一走 `just`（`just check`、`just test`），不裸跑底层构建工具。
{%- endif %}
{%- if options["commit_and_push"] %}
- **回复前提交**：完成修改并通过验证后，先 commit & push 再回复；纯问答或构建/测试失败排查除外。
{%- endif %}
- **约定式提交**：提交标题必须匹配 `type(scope): 摘要`，type 取自 `.pengj-templates/gate.toml` 的 `commit.types`，{% if options["commit_zh"] %}摘要至少 2 个汉字{% else %}摘要不含汉字{% endif %}，且不以句号结尾，标题长度 ≤ 100 字符（详见 `git-workflow` 技能）。

## 代码与注释语言规范
- **代码注释必须使用中文**：所有源码文件中的文档注释（`//!`、`///`）与行内注释（`//`、`/* */`）必须使用中文编写；API 标识符、符号名与专有名词（如 `Vulkan`、`SlotMap`）可保留英文原文。
{%- if 'gate' in layers %}
{%- if not options["chinese_programming"] %}
- **代码语言守卫（英文编程）**：
  - **标识符一律英文**：函数名、类型名、字段名、局部变量、测试函数名只使用 ASCII；中文只出现在注释里。
  - **字符串文案默认英文**：测试断言、错误消息、示例数据一律英文。
  - **例外认证**：单行例外行尾注释写 `lang-ok: 理由`；整文件放行文件头注释写 `lang-allow: 理由`（仅限词典、测试样本等数据文件）。全量自检：{% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}。
{%- else %}
- **代码语言守卫（中文编程）**：按 `.pengj-templates/gate.toml` 校验面向人的文案（机器字面量除外）；例外通过 `lang-ok` / `lang-allow` 认证。全量自检：{% if 'just' in layers %}`just gate-check`{% else %}`pengj-templates-cli gate check`{% endif %}。
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
