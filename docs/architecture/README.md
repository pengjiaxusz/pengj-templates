# 项目架构与核心规范索引

> **AI / 开发者查阅指南**：本文件是项目的**唯一权威架构主索引**。
> 处理新需求、跨模块改动或定位复杂问题前，请根据当前任务涉及的领域查阅对应文档。
> 标注【权威规范】的必须严格遵守，【设计实现】用于指导具体编码。

---

## 1. 全局系统架构与核心骨架

| 权威架构文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`系统架构总览.md`](./系统架构总览.md) | 分层模板工具四件套（引擎 Rust + CLI clap + GUI Tauri 2/React + templates 运行时目录）与整体生命周期（发现→依赖排序→合并渲染→生成/纳管/更新→manifest 驱动同步） | `crates/core/src/lib.rs`, `crates/core/src/layer.rs`, `crates/cli/src/main.rs`, `crates/app/src-tauri/src/lib.rs`, `templates/*/layer.toml` | 【权威规范】 |
| [`对外契约与模板定位.md`](./对外契约与模板定位.md) | 统一模板根定位（`PENGJ_TEMPLATES` > 可执行文件旁 `templates/` > cwd `templates/`，运行时读取无需重编译）与对外契约（CLI 四命令 `list-layers/create/update/adopt` + Tauri `invoke("cmd_*")` 6 命令 + `RenderContext {project_name,project_slug,year,layers,options}`） | `crates/core/src/context.rs`, `crates/core/src/engine.rs::default_templates_dir`, `crates/cli/src/main.rs::load_templates`, `crates/app/src-tauri/src/lib.rs::resolve_templates` | 【权威规范】 |

## 2. 核心子系统与领域模块

| 权威架构文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`模板合并与渲染管线.md`](./模板合并与渲染管线.md) | `FileMap {normal,concat,json}` 三桶分流、依赖拓扑排序与层覆盖语义、`ACCUMULATE_FILES` 累加拼接与 `MERGE_JSON_FILES` 并集合并、minijinja 渲染与二进制直通 | `crates/core/src/engine.rs::FileMap/build_file_map/render_file_map`, `crates/core/src/render.rs`, `templates/` 各层文件 | 【权威规范】 |
| [`受管块与TOML结构化合并.md`](./受管块与TOML结构化合并.md) | 三风格受管块 `Html/Hash/Slash` 的提取/原位替换/追加语义与 TOML 受管合并（`.cargo/config.toml` 表级并集、键级去重、同键异值冲突跳过） | `crates/core/src/block.rs`, `crates/core/src/toml_merge.rs`, `templates/rust/.cargo/config.toml` | 【权威规范】 |
| [`Manifest与同步更新.md`](./Manifest与同步更新.md) | `.pengj-templates.json` 持久化（层/选项/文件 sha256 基线）驱动的 `generate` 空目录生成、`adopt` 存量纳管与 `update` 冲突判定（未改→覆盖，已改无块→冲突跳过，有块→合并） | `crates/core/src/manifest.rs`, `crates/core/src/engine.rs::generate/adopt_project/update_project` | 【权威规范】 |
| [`技能体系与双语渲染.md`](./技能体系与双语渲染.md) | `options["skills"]` 过滤与 `skill_name_of` 路由、`SKILL.md` frontmatter 解析、`list_skills` 双语渲染与 legacy 全自定义技能 `take_over_legacy_skill` 接管（过渡区双流程） | `crates/core/src/engine.rs::selected_skills/skill_name_of/is_skill_doc/take_over_legacy_skill`, `templates/agent/.agents/skills/*/SKILL.md` | 【设计实现】 |
| [`集成配置与工作区同步.md`](./集成配置与工作区同步.md) | `package.json` 并集（依赖版本模板为准、脚本用户优先）、lefthook/commitlint `base` 自动接线与去重、`.vscode/settings.json` 与 `*.code-workspace` fileNesting 增量合并 | `crates/core/src/engine.rs::merge_json/sync_workspace_file/sync_settings_file`, `templates/lefthook/commitlint.base.js`, `templates/vscode/.vscode/settings.json` | 【设计实现】 |

## 3. 关联设计草稿与探索（非权威，仅作参考）

- 暂无 `docs/design/` 或 `docs/设计/` 草稿目录。后续实验性设计可置于 `docs/design/<topic>/`，成熟后晋升为 `docs/architecture/` 权威文档并回填本索引。
