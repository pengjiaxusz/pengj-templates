# 模板层说明

本目录按「层」组织模板内容，每个子目录是一个层：

- `layer.toml`：层元数据（名称、描述、依赖的其他层），不会写入生成的项目
- 其余文件/目录：模板内容，支持 minijinja 占位符（如 `{{ project_name }}`、`{{ project_slug }}`）

## 层合并规则

- 生成时按依赖顺序合并：先 `common`，再依赖它的各层
- 后合并的层覆盖先合并层的同名文件（`toolchain.toml` 等仅存在于 rust 层的文件只发给 rust 项目）
- `.gitignore`、`.gitattributes` 例外：按层**累积拼接**而不是覆盖——每个层各贡献一段规则，生成的项目拿到合并后的完整清单（各段带来源层注释）
- `package.json` 按层做 JSON 并集；更新/纳管时依赖类字段同名包版本一律以模板为准，脚本与其余字段以用户为准、模板只补缺失
- `.cargo/config.toml` 的受管块在更新/纳管时走 TOML 结构化合并（表级并集、等值去重、冲突跳过并上报）——模板里**不要**把项目可能自定义的表写到受管块之外
- 含受管块（`PENGJ_TEMPLATE_START/END`）的文本文件更新时只替换块区间，块外内容归用户所有
- 生成后会在项目根写入 `.pengj-templates.json` manifest，记录使用的层与模板托管文件的哈希，供 `update` 使用

## just / bque 排队装配

- `just` 层产出**唯一的**根 `justfile`：框架区在 `# PENGJ_TEMPLATE_START/END` 托管块内（`set shell`、`default`、按层条件 `import`），块外的「项目专属区」归项目所有，`update` 只替换块内，不再整页覆盖。
- `bque` 层产出 `just/bque.just`（`queue-*` / `queue-doctor` / `q-run` / `q-bench`）；`rust` 与 `rust-workspace` 层分别产出 `just/rust.just` / `just/rust-workspace.just`。主 justfile 按 `layers` 条件 import，三层的 `depends` 都含 `just` 以保证顺序。
- `options["just_recipes"]`：`"rust"` / `"workspace"` 显式指定语言配方；`"none"` 关闭语言配方导入（已有自定义编排的项目只接入排队基础设施）；未设置时按所选层自动推断。
- 勾 `bque` 层时会自动把 `bque` 技能并入 `options["skills"]`，避免「有配方没文档」的半接入；`audit` 对半接入、以及缺少 justfile 托管块的项目给出配置提示。
- 存量项目加层走 `add-layer --dir <项目> --layers just,bque`：只合并层清单并重跑 `update`，不做全量重纳管。
- bque 是 Windows 全局锁；`[unix]` 分支的同一批配方直接透传，CI 与 Unix 裸跑属预期行为。
