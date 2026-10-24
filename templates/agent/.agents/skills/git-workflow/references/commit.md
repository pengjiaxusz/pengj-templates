{% if options["skill_lang"] == "en" %}
# Commit & Push Workflow

Conventional commit workflow with pre-response commit gates, semantic splitting, and immediate push protection.

---

## 1. Trigger Conditions (Mandatory Proactive Triggers)

- **Code modification & milestone completion (Mandatory barrier — no accumulation)**:
  Whenever files are modified and verified in the current turn, you MUST run this commit workflow (pre-commit checks, split commits, and `git push`) before reporting back to the user or proceeding to the next step. Never leave verified changes sitting in the working directory before yielding control; writing `walkthrough.md` in planning modes (`/plan` / `/boost`) does NOT conclude the task — you must commit and push immediately after.
- **Task conclusion (Mandatory)**:
  Before concluding any development or maintenance task or delivering final responses, ensure the working tree is clean and all commits are pushed to the remote repository.
- **Explicit user instruction**:
  When user invokes `/commit`, "commit", "commit & push", etc.

---

## 2. Gather Context (Standard Cross-Platform Git)

Run from repository root:

```bash
git status
git --no-pager diff --stat
git --no-pager diff --cached --stat
```

---

## 3. Pre-Commit Completeness Check

This step belongs to the **PROJECT**, not the template:
Follow whatever the project-specific area in `SKILL.md` defines (commands, test suites, checklists). If the project-specific area defines no checks, use the generic fallback:
- Scan the diff: if it touches build/deps, docs/AGENTS.md/config, or public naming, verify build / update docs / run formatter as appropriate before committing.{% if options["skills"] is undefined or 'arch-align' in options["skills"] %}
- **Architecture doc consistency check**: If this change touches architecture (new/adjusted module boundaries, data flow, lifecycle, system topology, core invariants, etc.), verify `docs/architecture/README.md` and the corresponding domain docs are in sync; see `.agents/skills/arch-align` for details.{% endif %}

Do NOT invent arbitrary checks on the fly; reference the project-specific area.

---

## 4. Semantic Splitting

Separate unrelated areas into distinct commits (e.g., do not mix documentation updates with core feature implementations or build script refactorings).

---

## 5. Commit Message Format

Format: `type(scope): subject`

{% if options["commit_zh"] %}
- **Subject in Chinese**, short, no trailing punctuation (mandatory: at least 2 Chinese characters; strictly forbidden to write pure English subjects).
  - ❌ Forbidden: `feat(cli): add new flags`
  - ✅ Correct: `feat(cli): 新增参数支持`
- **Type/Scope in English**:
  - Types: `feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`
  - Scope: from the project's `commitlint.config.js` scope-enum whitelist when present.
{% else %}
- **Subject in English** (imperative, short, no trailing period; strictly forbidden to contain Chinese characters).
  - ❌ Forbidden: `feat(cli): 新增参数支持`
  - ✅ Correct: `feat(cli): add new flags`
- **Type/Scope in English**:
  - Types: `feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`
  - Scope: from the project's `commitlint.config.js` scope-enum whitelist when present.
{% endif %}

---

## 6. Commit & Push Execution

```bash
# Non-interactive commit
git commit -m "<type>(<scope>): <subject>"

# Push immediately
git push

# If remote diverged, rebase silently and push
git pull --rebase --autostash
git push
```

---

## 7. Amend (Amend Guardrail)

Amend is ONLY permitted when explicitly requested by the user, and ONLY for the most recent, unpushed commit that nobody else depends on:

```bash
# MUST supply --no-edit or explicit -m to avoid popping up editors
git commit --amend --no-edit
git push
```
{% else %}
# Commit 提交流程与规范

约定式提交流程、回复前硬性门禁、语义拆分与立即推送安全规范。

---

## 1. 触发时机（主动触发规则）

- **代码修改与阶段性完工（回复前硬性门禁，禁止堆积）**：
  凡本轮对话存在已通过构建/测试验证的代码修改，在向用户输出最终汇报前，**必须将运行提交流程（执行检查、拆分提交并 `git push`）作为最后一个工具调用步骤**。严禁在工作区留存已验证修改直接结束回复；在 `/plan` 等规划模式下生成 `walkthrough.md` 后，亦必须紧随其后执行提交与推送，不可中途停顿。
- **任务收尾（硬性触发）**：
  任何开发或维护任务结束、向用户做最终交付前，必须确保工作区干净且已推送到远端，不留未提交修改。
- **用户显式指令**：
  用户发送 `/commit`、"提交"、"commit & push" 等。

---

## 2. 收集提交上下文（跨平台标准命令）

在仓库根目录执行：

```bash
git status
git --no-pager diff --stat
git --no-pager diff --cached --stat
```

---

## 3. 提交前完整性检查

本步骤归**项目**所有，模板不硬编码具体检查：
按主入口 `SKILL.md` 托管块外「项目专属提交流程与红线」的定义执行——跑什么命令、查哪些项，由项目自行声明。项目专属区未定义时，执行兜底自检：
- 扫 diff：改动构建/依赖、文档/AGENTS.md/配置、公开命名时对应校验构建、更新文档、跑格式化；{% if options["skills"] is undefined or 'arch-align' in options["skills"] %}
- **架构文档一致性检查**：触及架构（新增/调整模块边界、数据流、生命周期、系统拓扑、核心不变式等）时，检查 `docs/architecture/README.md` 与对应领域文档是否已同步，详情参见 `.agents/skills/arch-align`。{% endif %}

---

## 4. 拆分无关改动

不相关领域的改动必须拆分多次提交（文档与功能、构建配置与业务代码分开提交）。

---

## 5. 提交信息格式

格式：`type(scope): 标题`

{% if options["commit_zh"] %}
- **标题用中文短句**、不加句号（硬性要求：必须包含至少 2 个汉字，严禁写纯英文）。
  - ❌ 严禁：`feat(cli): add new flags`
  - ✅ 必须：`feat(cli): 新增参数支持`
- **type/scope 用英文**：
  - 常用 type：`feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`
  - scope 参考项目 `commitlint.config.js` 的 scope-enum 白名单，无合适则省略。
{% else %}
- **标题用英文短句**（祈使式），不加句号（硬性要求：必须纯英文，严禁包含汉字）。
  - ❌ 严禁：`feat(cli): 新增参数支持`
  - ✅ 必须：`feat(cli): add new flags`
- **type/scope 用英文**：
  - 常用 type：`feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore` `revert`
  - scope 参考项目 `commitlint.config.js` 的 scope-enum 白名单，无合适则省略。
{% endif %}

---

## 6. 提交与 Push 执行（非交互标准）

```bash
# 非交互式提交（必须携带 -m）
git commit -m "<type>(<scope>): <标题>"

# 提交后立即 push，防止工作丢失
git push

# 远端若有分叉，变基拉取后再 push
git pull --rebase --autostash
git push
```

---

## 7. Amend（修正红线）

仅当用户明确要求、且是刚提交未 push、无人依赖的最新提交时方可执行：

```bash
# 必须带 --no-edit 或显式 -m，严禁弹出编辑器
git commit --amend --no-edit
git push
```
{% endif %}
