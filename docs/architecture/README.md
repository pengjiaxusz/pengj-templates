# 项目架构与核心规范索引

> **AI / 开发者查阅指南**：本文件是项目的**唯一权威架构主索引**。
> 处理新需求、跨模块改动或定位复杂问题前，请根据当前任务涉及的领域查阅对应文档。
> 标注【权威规范】的必须严格遵守，【设计实现】用于指导具体编码。

---

## 1. 全局系统架构与核心骨架

| 权威架构文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`系统架构总览.md`](./系统架构总览.md) | 分层模板工具三件套（引擎 Rust + CLI clap + 提交门禁 Rust + templates 运行时目录）与整体生命周期（发现→依赖排序→合并渲染→生成/纳管/更新→manifest 驱动同步） | `crates/core/src/lib.rs`, `crates/core/src/layer.rs`, `crates/cli/src/main.rs`, `crates/gate/src/cli.rs`, `templates/*/layer.toml` | 【权威规范】 |
| [`对外契约与模板定位.md`](./对外契约与模板定位.md) | 统一模板根定位（`PENGJ_TEMPLATES` > cwd `templates/` > 可执行文件旁 `templates/`，运行时读取无需重编译）与对外契约（CLI 命令 `list-layers/create/update/adopt/add-layer/audit/check-lang/mcp/gate` + `RenderContext {project_name,project_slug,year,layers,options}`） | `crates/core/src/context.rs`, `crates/core/src/engine.rs::default_templates_dir`, `crates/cli/src/main.rs::load_templates` | 【权威规范】 |

## 2. 核心子系统与领域模块

| 权威架构文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`模板合并与渲染管线.md`](./模板合并与渲染管线.md) | `FileMap {normal,concat}` 两桶分流 + `executable` 可执行位集合、依赖拓扑排序与层覆盖语义、`ACCUMULATE_FILES` 累加拼接、minijinja 渲染与二进制直通 | `crates/core/src/engine.rs::FileMap/build_file_map/render_file_map/apply_executable_bits`, `crates/core/src/render.rs`, `templates/` 各层文件 | 【权威规范】 |
| [`受管块与TOML结构化合并.md`](./受管块与TOML结构化合并.md) | 三风格受管块 `Html/Hash/Slash` 的提取/原位替换/追加语义与 TOML 受管合并（`.cargo/config.toml` 表级并集、键级去重、同键异值冲突跳过） | `crates/core/src/block.rs`, `crates/core/src/toml_merge.rs`, `templates/rust/.cargo/config.toml` | 【权威规范】 |
| [`Manifest与同步更新.md`](./Manifest与同步更新.md) | `.pengj-templates/manifest.json` 持久化（层/选项/文件 sha256 基线）驱动的 `generate` 空目录生成、`adopt` 存量纳管与 `update` 冲突判定（未改→覆盖，已改无块→冲突跳过，有块→合并） | `crates/core/src/manifest.rs`, `crates/core/src/engine.rs::generate/adopt_project/update_project` | 【权威规范】 |
| [`技能体系与双语渲染.md`](./技能体系与双语渲染.md) | `options["skills"]` 过滤与 `skill_name_of` 路由、`SKILL.md` frontmatter 解析、`list_skills` 双语渲染与 legacy 全自定义技能 `take_over_legacy_skill` 接管（过渡区双流程） | `crates/core/src/engine.rs::selected_skills/skill_name_of/is_skill_doc/take_over_legacy_skill`, `templates/agent/.agents/skills/*/SKILL.md` | 【设计实现】 |
| [`排队层与增量加层.md`](./排队层与增量加层.md) | `just` 层唯一根 `justfile` + 托管块框架区/项目区契约、`gate`/`subrepo`/`worktree` 层 `just/*.just` 条件 import、`bque` 纯声明层（不产出配方，重型命令直调 `bque run --`）、Windows 平台边界、层-技能联动与 `add-layer` 存量迁移路径 | `crates/core/src/engine.rs::ensure_layer_skills/add_layers_to_project/audit_project`, `templates/just/justfile`, `templates/bque/layer.toml`, `templates/gate/.pengj-templates/just/gate.just` | 【权威规范】 |
| [`集成配置与工作区同步.md`](./集成配置与工作区同步.md) | `.vscode/settings.json` 与 `*.code-workspace` 的 fileNesting 增量合并 | `crates/core/src/engine.rs::sync_workspace_file/sync_settings_file`, `templates/vscode/.vscode/settings.json` | 【设计实现】 |
| [`语言变体文件.md`](./语言变体文件.md) | 代码类产物按语言拆文件（`<stem>.<lang>.<ext>` 跟 `skill_lang`、`<stem>.commit.<lang>.<ext>` 跟 `commit_zh`），引擎二选一写盘、产物去标记 | `crates/core/src/engine.rs::resolve_language_variant/build_file_map` | 【设计实现】 |

## 3. 决策记录与规格（Decision Records & Specs）

| 文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`GLOSSARY.md`](../../GLOSSARY.md) | 模板引擎领域术语表：层、托管块、更新黑名单、结构化合并、迁移提示、子仓库、登记表、共享编译目录、产物覆盖 | — | 【权威规范】 |
| [`docs/adr/0001-shared-build-dir-via-env-injection.md`](../adr/0001-shared-build-dir-via-env-injection.md) | 共享编译目录用 `CARGO_TARGET_DIR` 注入而非目录链接 / `target-dir`：链接状态无法被自动校验，且手工方案已静默退化 | — | 【权威规范】 |
| [`docs/adr/0002-subrepo-and-worktree-as-separate-layers.md`](../adr/0002-subrepo-and-worktree-as-separate-layers.md) | 子仓库能力与多工作树能力拆成两个独立层：机制正交，交汇点交由仓库初始化入口承接 | — | 【权威规范】 |
| [`docs/adr/0003-machine-readable-subrepo-registry.md`](../adr/0003-machine-readable-subrepo-registry.md) | 子仓库登记表用机器可读格式而非 Markdown 表格：表格方案无法被可靠解析且失效是静默的 | — | 【权威规范】 |
| [`docs/adr/0004-shared-target-is-main-worktree-target.md`](../adr/0004-shared-target-is-main-worktree-target.md) | 共享编译目录落在**主工作树的 `target/`**：产物跟着项目走、主工作树裸构建天然命中；代价是 `cargo clean` 波及所有工作树（取代 ADR 0001 的落点结论） | `crates/core/src/worktree.rs::shared_target_dir` | 【权威规范】 |
| [`docs/spec/subrepo-worktree-layers.md`](../spec/subrepo-worktree-layers.md) | `subrepo` 与 `worktree` 两个新层的规格：问题陈述、用户故事、实现决策、测试决策、范围之外 | — | 【规格】 |

## 4. 关联设计草稿与探索（非权威，仅供参考）

- 暂无 `docs/design/` 或 `docs/设计/` 草稿目录。后续实验性设计可置于 `docs/design/<topic>/`，成熟后晋升为 `docs/architecture/` 权威文档并回填本索引。
