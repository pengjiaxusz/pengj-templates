---
name: commit
description: >-
{% if options["skill_lang"] == "en" %}
  Conventional commit workflow. Write commit messages{% if options["commit_zh"] %} in Chinese (title/body, except type/scope){% else %} in English{% endif %}. Run the project's pre-commit check first (convention script > project checklist > generic diff scan), split unrelated changes, and push immediately.
  Triggers: commit, amend, push, commit message, conventional commits.
{% else %}
  约定式提交流程。提交信息{% if options["commit_zh"] %}用中文撰写（type/scope 除外）{% else %}用英文撰写{% endif %}。提交前先跑项目约定的检查（约定脚本 > 项目清单 > 兜底扫 diff）、拆分无关改动、提交后立即 push。
  Triggers: commit, 提交, amend, push, commit message, conventional commits.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# Commit

Write conventional commit messages. Always push immediately after committing to avoid losing work.

## 1. Gather context (standard git, cross-platform)
Run from repo root:

`git status` · `git diff --stat` · `git diff --cached --stat` (add full diffs when needed)

## 2. Pre-commit completeness check — project-defined
This step belongs to the PROJECT, not the template: follow whatever the project-specific area below the managed block defines (which commands to run, what to verify — all written there, any language or tool). If the project-specific area defines no checks, use the generic fallback: scan the diff — if it touches build/deps, docs/AGENTS.md/config, or public naming, verify build / update docs / run formatter as appropriate before committing.{% if options["skills"] is undefined or 'arch-align' in options["skills"] %} If this change touches architecture (new/adjusted module boundaries, data flow, lifecycle, system topology, core invariants, etc.), also verify `docs/architecture/README.md` and the corresponding domain docs are in sync; see `.agents/skills/arch-align` for details.{% endif %}

Do NOT invent repo-specific checks here; the project owns them (see the project-specific area below).
{% if options["skills"] is undefined or 'arch-align' in options["skills"] %}
### Architecture doc consistency check (active when `arch-align` is enabled)

When `arch-align` is enabled, add the following gate to step 2:

- **Trigger question:** Does this change touch architecture — new/adjusted module boundaries, system topology, data flow, lifecycle, cross-layer contracts, core invariants?
- **If hit → verify:** `docs/architecture/README.md` index and the corresponding `docs/architecture/<domain>.md` are in sync with the code change. See `.agents/skills/arch-align` for deep alignment.
- **Fast path (no doc update needed):** pure tests, pure formatting, single-function bugfixes with no behavior/contract change.

Judgment addition:

| Change seen | Check hit |
| --- | --- |
| Touched architecture (new/adjusted modules, data flow, lifecycle, cross-module contracts) | `docs/architecture/README.md` index & domain docs in sync |
{% endif %}

## 3. Split unrelated changes
Separate unrelated areas into distinct commits (e.g. do not mix docs with feature code).

## 4. Commit message format
`type(scope): subject`
{% if options["commit_zh"] %}
- Subject in Chinese, short, no trailing punctuation.
- Type/scope in English (see type list below): `feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`.
- Scope from the project's `commitlint.config.js` scope-enum whitelist when it exists.
{% else %}
- Subject in English (imperative, short, no trailing period).
- Type/scope in English: `feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`.
- Scope from the project's `commitlint.config.js` scope-enum whitelist when it exists.
{% endif %}

## 5. Commit & push (PowerShell compatible)
`git commit -m "<type>: <subject>"` then `git push`. On remote divergence, `git pull --rebase` then push.

## 6. Amend
Only when explicitly asked, for the just-made, unpushed commit with no dependency from others. Run the completeness check again; push after amend.
{% else %}
# Commit 提交流程
按约定式提交写提交信息。提交后立即 push，防止丢失。

## 1. 收集提交上下文（标准 git 命令，跨平台）
仓库根运行：
`git status` · `git diff --stat` · `git diff --cached --stat`（必要时看完整 diff）

## 2. 提交前完整性检查 —— 项目自定义
本步骤归**项目**所有，模板不硬编码具体检查：按下方托管块外「项目专属提交流程与红线」的定义执行——跑什么命令、查哪些项、用什么语言写，全部由项目在那里自行声明。

项目专属区未定义任何检查时，使用兜底自检：扫 diff——改动构建/依赖、文档/AGENTS.md/配置、公开命名时对应校验构建、更新文档、跑格式化{% if options["skills"] is undefined or 'arch-align' in options["skills"] %}；触及架构（新增/调整模块边界、数据流、生命周期、系统拓扑、核心不变式等）时检查 `docs/architecture/README.md` 与对应领域文档是否已同步，详情见 `.agents/skills/arch-align`{% endif %}。

不要在本流程里发明仓库专属检查；它们归项目所有（见下方项目专属区）。
{% if options["skills"] is undefined or 'arch-align' in options["skills"] %}
### 架构文档一致性检查（`arch-align` 已启用时本节生效）

本项目已启用 `arch-align` 时，第 2 步需追加以下判定：

- **触发判断：** 是否触及架构（新增模块/改拓扑/改数据流/改不变式/改跨层契约）？
- **命中 → 核验：** `docs/architecture/README.md` 索引与对应 `docs/architecture/<domain>.md` 是否已同步。需深度对齐时执行 `.agents/skills/arch-align` 流程。
- **快速路径（无需更新文档）：** 纯测试、纯格式化、单函数 bugfix 且不改行为/契约。

判定表追加：

| 改动 | 命中的检查 |
| --- | --- |
| 触及架构（新增/调整模块、数据流、生命周期、跨模块契约） | `docs/architecture/README.md` 索引与对应领域文档是否已同步 |
{% endif %}

## 3. 拆分无关改动
不相关领域拆多次提交（文档与功能、构建与业务分开）。

## 4. 提交信息格式
`type(scope): 标题`
{% if options["commit_zh"] %}
- 标题用中文短句、不加句号。
- type/scope 用英文：`feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`。
- scope 参考项目 `commitlint.config.js` 的 scope-enum 白名单，无合适则省略。
{% else %}
- 标题用英文短句（祈使式），不加句号。
- type/scope 用英文：`feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`。
- scope 参考项目 `commitlint.config.js` 的 scope-enum 白名单，无合适则省略。
{% endif %}

## 5. 提交与 Push（PowerShell 兼容）
`git commit -m "<type>: 标题"` 后立即 `git push`；远端有更新先 `git pull --rebase` 再 push。

## 6. Amend
仅当用户明确要求、且是刚提交未 push、无人依赖时。Amend 前仍跑完整性检查，完成后立即 push。
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
<!-- Project-specific area left empty by default (no commit skill → no content; with commit skill, add your checks here) -->
{% else %}
<!-- 项目专属区域默认留空（无 commit 技能时为空；有 commit 技能时在此添加检查） -->
{% endif %}
