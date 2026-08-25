---
name: commit
description: >-
{% if options["skill_lang"] == "en" %}
  Conventional commit workflow. Write commit messages{% if options["commit_zh"] %} in Chinese (title/body, except type/scope){% else %} in English{% endif %}. Run the project's pre-commit check first (convention script > project checklist > generic diff scan), split unrelated changes, and push immediately.
  Triggers: commit, 提交, amend, push, commit message, conventional commits.
{% else %}
  约定式提交流程。提交信息{% if options["commit_zh"] %}用中文撰写（type/scope 除外）{% else %}用英文撰写{% endif %}。提交前先跑项目约定的检查（约定脚本 > 项目清单 > 兜底扫 diff）、拆分无关改动、提交后立即 push。
  Triggers: commit, 提交, amend, push, commit message, conventional commits.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
# Commit 提交流程

{% if options["skill_lang"] == "en" %}
Write conventional commit messages. Always push immediately after committing to avoid losing work.

## 1. Gather context (standard git, cross-platform)
Run from repo root:

`git status` · `git diff --stat` · `git diff --cached --stat` (add full diffs when needed)

## 2. Pre-commit completeness check — project-defined, resolved by priority
This step belongs to the PROJECT, not the template. Resolve in order and stop at the first hit:

1. **Convention script** `.agents/skills/commit/pre-commit-check.ps1` exists (any `pre-commit-check.*` in this skill dir) → MUST run it first; its output is the basis for judgment. The script defines what "complete" means for this repo.
2. **Project checklist** below the managed block ("Project-specific workflow & red lines") is filled in → follow it.
3. **Generic fallback** (nothing defined): scan the diff — if it touches build/deps, docs/AGENTS.md/config, or public naming, verify build / update docs / run formatter as appropriate before committing.

Do NOT invent repo-specific checks here; extend the script or checklist instead (see slots below).

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
按约定式提交写提交信息。提交后立即 push，防止丢失。

## 1. 收集提交上下文（标准 git 命令，跨平台）
仓库根运行：
`git status` · `git diff --stat` · `git diff --cached --stat`（必要时看完整 diff）

## 2. 提交前完整性检查 —— 项目自定义，按优先级取第一个命中
本步骤归**项目**所有，模板不硬编码具体检查。按顺序解析：

1. **约定脚本**：存在 `.agents/skills/commit/pre-commit-check.ps1`（或本技能目录下任意 `pre-commit-check.*`）→ MUST 先运行，以其输出（status/stat 概览等）为判定依据。脚本定义了本仓库「完整」的含义。
2. **项目清单**：下方托管块外的「项目专属提交流程与红线」已填写 → 按清单执行。
3. **兜底自检**（以上都没有）：扫 diff——改动构建/依赖、文档/AGENTS.md/配置、公开命名时对应校验构建、更新文档、跑格式化。

不要在本流程里发明仓库专属检查；需要扩展时改脚本或清单（见下方插槽）。

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
<!-- Below is the project-specific area: template updates replace only the managed block above; this area is preserved. -->
## Project-specific workflow & red lines

> This area sits outside the managed block above: template updates replace only the block, so everything below is preserved.

Two extension points feed step 2 of the flow above (keep at least one):

### Slot A: pre-commit check script (recommended)

Create `.agents/skills/commit/pre-commit-check.ps1`. It becomes step 2 automatically — no need to edit the flow text when your checks change, and template updates never touch it. Suggested output: worktree status + staged/unstaged stat overview (+ any repo-specific gates); avoid dumping full diffs to save context.

### Slot B: project checklist (editable)

- [ ] Custom pre-commit checks beyond the script (regenerate mocks, update golden files, run a specific test suite, ...)
- [ ] Domain principles / red lines (e.g. CLI `--help` must fully self-describe new args)
- [ ] Anything agents must never do when committing
{% else %}
<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域内容完整保留。 -->
## 项目专属提交流程与红线

> 本区域位于上方托管块之外：模板更新只替换托管块，下方内容完整保留。

两个扩展点共同支撑上文第 2 步（至少保留一个）：

### 插槽 A：pre-commit 检查脚本（推荐）

创建 `.agents/skills/commit/pre-commit-check.ps1`。它会自动成为第 2 步——之后调整项目检查只改脚本、不动流程文本，模板更新也永不覆盖它。建议输出：工作树状态 + 未暂存/已暂存 stat 概览（+ 本仓库特有门禁）；避免全量输出 diff 浪费上下文。

### 插槽 B：项目专属检查清单（可编辑）

- [ ] 脚本之外的提交前检查（重新生成 mock、更新 golden 文件、跑指定测试套件等）
- [ ] 领域原则 / 红线（如 CLI `--help` 必须完全自解释新参数）
- [ ] 提交时 agent 绝不能做的事项
{% endif %}
