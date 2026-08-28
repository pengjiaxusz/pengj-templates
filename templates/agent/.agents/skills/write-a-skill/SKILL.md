---
name: write-a-skill
description: >-
{% if options["skill_lang"] == "en" %}
  Create new Agent skills with correct structure, progressive disclosure, and bundled resources. Use when the user wants to create, write, or build a new skill.
  Triggers: write-a-skill, new skill, create skill, skill template, 编写技能, 创建技能.
{% else %}
  以正确的结构、渐进式披露和捆绑资源创建新的 Agent 技能。当用户想要创建、编写或构建新技能时使用。
  Triggers: write-a-skill, 编写技能, 创建技能, new skill, skill template.
{% endif %}
---
{% if options["skill_lang"] == "en" %}
# Write a Skill

Create a new Agent skill that follows the repository's skill conventions (auto-discovery, bilingual, managed block + project-specific area).

## Workflow

### 1. Gather requirements

Ask the user:

- What task/domain does the skill cover?
- Which concrete use cases must it handle?
- Does it need executable scripts, or just instructions?
- Does it need bundled reference material?

### 2. Draft the skill

Create:

- `SKILL.md` with concise instructions
- Additional reference files if the body exceeds ~500 lines
- Utility scripts if deterministic operations are needed

### 3. Review with the user

Show the draft and ask:

- Does it cover your use cases?
- Anything missing or unclear?
- Any section that should be more detailed or more concise?

## Skill Structure

```
skill-name/
├── SKILL.md           # Main instructions (required)
├── REFERENCE.md       # Detailed docs (as needed)
├── EXAMPLES.md        # Usage examples (as needed)
└── scripts/           # Helper scripts (as needed)
    └── helper.js
```

In this repo a skill lives at `templates/agent/.agents/skills/<name>/SKILL.md` (auto-discovered by `Templates::list_skills()`, no registration). It is rendered to the generated project at `.agents/skills/<name>/SKILL.md` and filtered by `options["skills"]` (`skill_name_of` / `selected_skills` in `crates/core/src/engine.rs`; missing option means include all).

## SKILL.md Template

```md
---
name: skill-name
description: Brief capability description. Use when [specific trigger condition].
---

# Skill Name

## Quick Start

[Minimal runnable example]

## Workflow

[Step-by-step flow with checklists for complex tasks]

## Advanced

[Link to separate file: See [REFERENCE.md](REFERENCE.md)]
```

## Description Requirements

The description is the **only content the Agent sees when choosing which skill to load**. It is shown together with all other installed skills in the system prompt.

Goal — tell the Agent:

1. What capability this skill provides
2. When/why to trigger it (concrete keywords, contexts, file types)

Format:

- Max 1024 characters
- Written in third person
- Sentence 1: what it does
- Sentence 2: "Use when [specific trigger condition]"

Good:

```
Extract text and tables from PDF files, fill forms, and merge documents. Use when handling PDF files or when the user mentions PDF, forms, or document extraction.
```

Bad:

```
Help with documents.
```

## When to Add Scripts

Add utility scripts when:

- The operation is deterministic (validation, formatting)
- The same code would be generated repeatedly
- Errors need explicit handling

Scripts save tokens and improve reliability compared to generated code.

## When to Split Files

Split into separate files when:

- `SKILL.md` exceeds ~100 lines
- Content spans different domains (e.g. finance vs sales modes)
- Advanced features are rarely needed

Keep `SKILL.md` lean; move infrequent detail to `REFERENCE.md` / `EXAMPLES.md` and link to it (one level deep only).

## Review Checklist

After drafting, verify:

- [ ] Description contains trigger condition ("Use when ...")
- [ ] `SKILL.md` is under ~100 lines (or split)
- [ ] No time-sensitive information
- [ ] Consistent terminology
- [ ] Concrete examples included
- [ ] References are at most one level deep
- [ ] Bilingual requirements met (see below)

## Bilingual & Framework Requirements (this repo)

- All agent-layer content (this `SKILL.md` + `AGENTS.md`) must be bilingual via `{% raw %}{% if options["skill_lang"] == "en" %}{% endraw %}` ... `{% raw %}{% else %}{% endraw %}` ... `{% raw %}{% endif %}{% endraw %}` branches (zh is default); frontmatter `description` must also branch.
- Copy the structure of the `commit` skill for new skills; when adding a language, extend the `skill_lang` validation in CLI / GUI / core.
- `SKILL.md` = managed framework block (`PENGJ_TEMPLATE_START/END`, replaced in place on `update`) + project-specific area **outside** the block (never overwritten). Document-style customization and commit-gate definitions live outside the block; executable gate form is project-defined.

{% else %}
# 编写技能

以符合本仓库约定的结构创建新的 Agent 技能（自动发现、双语、托管块 + 项目专属区）。

## 工作流

### 1. 收集需求

向用户询问：

- 该技能覆盖什么任务/领域？
- 它应处理哪些具体用例？
- 是否需要可执行脚本，还是仅需指令？
- 是否需要包含参考资料？

### 2. 起草技能

创建：

- `SKILL.md`，包含简洁的指令
- 若内容超过约 500 行，添加额外的参考文件
- 若需要确定性操作，添加实用脚本

### 3. 与用户评审

展示草稿并询问：

- 是否覆盖了你的用例？
- 有遗漏或不清晰的地方吗？
- 是否有章节需要更详细或更精简？

## 技能结构

```
skill-name/
├── SKILL.md           # 主指令（必需）
├── REFERENCE.md       # 详细文档（按需）
├── EXAMPLES.md        # 使用示例（按需）
└── scripts/           # 实用脚本（按需）
    └── helper.js
```

在本仓库中，技能位于 `templates/agent/.agents/skills/<name>/SKILL.md`，由 `Templates::list_skills()` 自动发现，无需注册。生成后落到项目的 `.agents/skills/<name>/SKILL.md`，受 `options["skills"]` 过滤（`crates/core/src/engine.rs` 的 `skill_name_of` / `selected_skills`；选项缺失时包含全部，向后兼容）。

## SKILL.md 模板

```md
---
name: skill-name
description: 能力的简要描述。当 [具体触发条件] 时使用。
---

# 技能名称

## 快速开始

[最小可运行示例]

## 工作流

[复杂任务的分步流程及检查清单]

## 高级功能

[链接到单独文件：参见 [REFERENCE.md](REFERENCE.md)]
```

## 描述要求

描述是 **Agent 在选择加载哪个技能时唯一看到的内容**。它会与所有其他已安装技能一起展示在系统提示中。

目标：让 Agent 了解：

1. 此技能提供什么能力
2. 何时/为何触发（具体的触发关键词、上下文、文件类型）

格式：

- 最多 1024 个字符
- 使用第三人称书写
- 第一句：它做什么
- 第二句："当 [具体触发条件] 时使用"

好的示例：

```
从 PDF 文件中提取文本和表格，填写表单，合并文档。当处理 PDF 文件或用户提及 PDF、表单或文档提取时使用。
```

不好的示例：

```
帮助处理文档。
```

## 何时添加脚本

在以下情况下添加实用脚本：

- 操作是确定性的（校验、格式化）
- 相同的代码会被反复生成
- 错误需要显式处理

与生成代码相比，脚本可以节省 token 并提高可靠性。

## 何时拆分文件

在以下情况下拆分为单独的文件：

- `SKILL.md` 超过约 100 行
- 内容涉及不同的领域（如财务 vs 销售模式）
- 高级功能很少需要用到

保持 `SKILL.md` 精简；把低频细节移到 `REFERENCE.md` / `EXAMPLES.md` 并用一层链接引用。

## 评审检查清单

起草完成后，验证：

- [ ] 描述包含触发条件（"当……时使用" / "Use when ..."）
- [ ] `SKILL.md` 不超过约 100 行（否则已拆分）
- [ ] 无时间敏感信息
- [ ] 术语一致
- [ ] 包含具体示例
- [ ] 引用仅一层深度
- [ ] 已满足双语要求（见下）

## 双语与框架要求（本仓库）

- agent 层全部内容（本 `SKILL.md` 与 `AGENTS.md`）必须中英双语，通过 `{% raw %}{% if options["skill_lang"] == "en" %}{% endraw %}` … `{% raw %}{% else %}{% endraw %}` … `{% raw %}{% endif %}{% endraw %}` 分支实现（zh 为默认）；frontmatter 的 `description` 也要双语分支（供 UI 勾选列表展示）。
- 新增技能照抄 `commit` 技能的双语模板结构；新增语言时同步扩展 `skill_lang` 校验（CLI / GUI / core）。
- `SKILL.md` = 托管框架块（`PENGJ_TEMPLATE_START/END`，`update` 时原位替换）+ 块外项目专属区（归用户、永不覆盖）。文档型定制与提交前检查的定义都写在块外；可执行门禁的形式与位置由项目自定，框架不预设。
{% endif %}
