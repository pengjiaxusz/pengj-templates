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

## 2. 提交前完整性检查 —— 项目自定义
本步骤归**项目**所有，模板不硬编码具体检查：按下方托管块外「项目专属提交流程与红线」的定义执行——跑什么命令、查哪些项、用什么语言写，全部由项目在那里自行声明。

项目专属区未定义任何检查时，使用兜底自检：扫 diff——改动构建/依赖、文档/AGENTS.md/配置、公开命名时对应校验构建、更新文档、跑格式化。

不要在本流程里发明仓库专属检查；它们归项目所有（见下方项目专属区）。

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
> 本节就是上文第 2 步的定义——在这里写明提交前要执行的检查与判定标准。

### 提交前检查（项目自定义）

每次提交前按顺序执行（PowerShell 兼容，仓库根运行）：

```powershell
# 1. 收集上下文
git status; git diff --stat; git diff --cached --stat
# 2. 构建与静态检查（按改动面选择，改 Rust/模板/配置必跑）
cargo fmt --check 2>$null; cargo clippy --workspace --all-targets -- -D warnings 2>$null
# 3. 架构文档一致性判断（见下方“架构文档一致性检查”）
# 4. 前端改动时：pnpm --dir crates/app build
# 5. 模板改动时：cargo run -q -p pengj-templates-cli -- create <tmp> --layers agent --skills commit,arch-align --output <临时目录> 校验渲染
```

> 可执行门禁以本清单为准，未配置独立脚本前按清单手工逐项勾选；后续若新增 `pre-commit-check.ps1` / `just` 任务，需在本节登记调用方式。

### 领域完整性检查

| # | 原则问题 | 命中 → 要查 |
| --- | --- | --- |
| 1 | 是否触及架构（新增/调整模块边界、数据流、生命周期、系统拓扑、跨层契约、核心不变式）？ | `docs/architecture/README.md` 索引与对应 `docs/architecture/<domain>.md` 是否已同步；必要时走 `.agents/skills/arch-align` 深度对齐 |
| 2 | 是否改动模板层/引擎核心契约（`crates/core` 合并渲染、manifest、层依赖、受管块/TOML 合并）？ | `AGENTS.md` / `templates/README.md` / 层 `layer.toml` 说明是否同步；是否跑通端到端 `create`/`update` 验证 |
| 3 | 是否新增对外接口/参数（CLI args、GUI invoke、生成选项 `options`、技能列表）？ | `README.md` / `--help` / GUI 文案是否完全自解释；`commitlint.config.js` scope 白名单是否更新 |
| 4 | 是否改动构建/依赖/工具链（Cargo/pnpm/lefthook/CI）？ | `cargo check --workspace` / `cargo test --workspace` / `pnpm --dir crates/app build` / `just ci` 是否通过 |

判定表速查：

| 改动 | 命中的检查 |
| --- | --- |
| `crates/core/**`、`templates/**/layer.toml`、`crates/core/src/render.rs`/`toml_merge.rs`/`engine.rs` | 架构文档 + 模板文档 + 端到端验证 |
| `templates/agent/**`、`.agents/skills/**`、技能 `SKILL.md` / `AGENTS.md` | 技能双语分支与托管块完整性；架构技能关联时检查架构索引是否登记 |
| `crates/cli/**`、`crates/app/src-tauri/**`、`crates/app/src/**` | 对外接口文档与构建验证 |
| `docs/architecture/**` | 索引 `docs/architecture/README.md` 路由是否可达（渐进式披露不堆细节） |
| 纯测试、纯格式化、单函数修 bug 且不改行为/契约 | 无（快速路径） |

### 架构文档一致性检查

> 本仓库启用 `arch-align`，每次提交必须显式回答“本次是否需更新架构文档”，禁止跳过。

**触发判断（任一命中即视为触及架构）：**
- 新增/删除/重命名层或技能（`templates/<layer>/`、`templates/agent/.agents/skills/<name>/`）
- 调整模块边界或依赖方向（`layer.toml` depends、`crates/core` 分层合并/渲染逻辑）
- 改变核心数据流或生命周期（生成/更新/纳管流程、`FileMap`/`manifest`/`merge_managed_block`）
- 修改跨层/跨进程契约或核心不变式（受管块协议、`update_ignore`、TOML/JSON 合并规则、`docs/architecture` 已记载的约束）
- 变更系统拓扑（CLI/GUI/引擎三件套交互路径、Tauri invoke 注册）

**命中 → 核验：**
1. `docs/architecture/README.md` 索引是否已登记/更新对应领域文档条目（权威级别、关联模块路径、职责一句话）。
2. 对应 `docs/architecture/<domain>.md` 是否已按 `references/GENERATE-DOC.md` 四段式（边界/数据流/源码映射/红线）同步。
3. 若 `docs/architecture/README.md` 尚不存在或领域缺檔，按 `arch-align` 技能询问用户后生成并回写索引。

**快速路径（无需更新架构文档）：**
- 纯测试、纯格式化、注释/文案微调
- 单函数内部修 bug 且不改模块边界/数据流/对外契约

**提交信息要求：** 触及架构的提交需在正文或 footer 注明已同步的架构文档路径（例：`同步 docs/architecture/README.md + docs/architecture/core-engine.md`）。

### 红线（agent 绝不能做）

- 跳过完整性检查直接提交（含跳过架构文档一致性判断）
- 触及架构却未同步 `docs/architecture/README.md` 与领域文档就提交
- 把多个领域（架构+功能+文档+构建）混进一次提交
- 为过检查而删除/伪造架构文档索引条目
