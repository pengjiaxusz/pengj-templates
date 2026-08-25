---
name: commit
description: >-

  约定式提交流程。提交信息用中文撰写（type/scope 除外）。提交前先跑项目约定的检查（约定脚本 > 项目清单 > 兜底扫 diff）、拆分无关改动、提交后立即 push。
  Triggers: commit, 提交, amend, push, commit message, conventional commits.

---

<!-- PENGJ_TEMPLATE_START -->
# Commit 提交流程


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

- 标题用中文短句、不加句号。
- type/scope 用英文：`feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`。
- scope 参考项目 `commitlint.config.js` 的 scope-enum 白名单，无合适则省略。


## 5. 提交与 Push（PowerShell 兼容）
`git commit -m "<type>: 标题"` 后立即 `git push`；远端有更新先 `git pull --rebase` 再 push。

## 6. Amend
仅当用户明确要求、且是刚提交未 push、无人依赖时。Amend 前仍跑完整性检查，完成后立即 push。

<!-- PENGJ_TEMPLATE_END -->


<!-- 以下为项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属提交流程与红线

> 本节归**项目**所有：模板更新只维护上方托管块，这里可以随意改写，不会丢失。
> 与上文第 2 步的关系：脚本（若存在）提供检查输出，**本节**定义「怎么判定、命中了要补什么」。

### 领域完整性检查（按仓库领域改写本节）

把下面的示例骨架替换为本仓库真实的三问与判定表：

| # | 原则问题 | 命中 → 要查 |
| --- | --- | --- |
| 1 | （示例）是否触及核心模块契约？ | 对应 `AGENTS.md` / `docs/` 是否同步 |
| 2 | （示例）是否新增对外接口/参数？ | 文档与 `--help` 是否完全自解释 |
| 3 | （示例）是否改动构建/依赖？ | 构建是否验证通过 |

判定表速查：

| 改动 | 命中的检查 |
| --- | --- |
| （示例）`src/core/**` | 核心契约文档 |
| 纯测试、纯格式化、修 bug 不改行为 | 无（快速路径） |

### 红线（agent 绝不能做）

- （示例）跳过完整性检查直接提交
- （示例）把多个领域混进一次提交

### 可选：提交前检查脚本

需要可执行门禁时，创建 `.agents/skills/commit/pre-commit-check.ps1`（或任意 `pre-commit-check.*`）——它会自动成为上文第 2 步的输入源，模板更新永不覆盖。
