---
name: commit
description: >-
{% if options["skill_lang"] == "en" %}
  Conventional commit workflow. Write commit messages{% if options["commit_zh"] %} in Chinese (title/body, except type/scope){% else %} in English{% endif %}. Check diff before committing, split unrelated changes, and push immediately.
  Triggers: commit, 提交, amend, push, commit message, conventional commits.
{% else %}
  约定式提交流程。提交信息{% if options["commit_zh"] %}用中文撰写（type/scope 除外）{% else %}用英文撰写{% endif %}。提交前扫 diff 自检、拆分无关改动、提交后立即 push。
  Triggers: commit, 提交, amend, push, commit message, conventional commits.
{% endif %}
---

# Commit 提交流程

{% if options["skill_lang"] == "en" %}
Write conventional commit messages. Always push immediately after committing to avoid losing work.

## 1. Gather context (standard git, cross-platform)
Run from repo root:

`git status` · `git diff --stat` · `git diff --cached --stat` (add full diffs when needed)

## 2. Check completeness before committing
Scan the diff; if it touches build/deps, docs/AGENTS.md/config, or public naming — verify build, update docs, run `cargo fmt` as appropriate before committing.

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

## 2. 提交前完整性检查
扫 diff：若改动构建/依赖、文档/AGENTS.md/配置、公开命名 —— 对应校验构建、更新文档、跑 `cargo fmt`，全部补齐再提交。

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