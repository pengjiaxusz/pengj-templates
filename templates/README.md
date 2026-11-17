# 模板层说明

本目录按「层」组织模板内容，每个子目录是一个层：

- `layer.toml`：层元数据（名称、描述、依赖的其他层），不会写入生成的项目
- 其余文件/目录：模板内容，支持 minijinja 占位符（如 `{{ project_name }}`、`{{ project_slug }}`）

## 层合并规则

- 生成时按依赖顺序合并：先 `common`，再依赖它的各层
- 后合并的层覆盖先合并层的同名文件（`toolchain.toml` 等仅存在于 rust 层的文件只发给 rust 项目）
- `.gitignore`、`.gitattributes` 例外：按层**累积拼接**而不是覆盖——每个层各贡献一段规则，生成的项目拿到合并后的完整清单（各段带来源层注释）
- 模板不产出 Node 清单文件（`package.json` 等），存量项目里已有的这类文件既不被合并也不被删除，原样归用户所有
- `.cargo/config.toml` 的受管块在更新/纳管时走 TOML 结构化合并（表级并集、等值去重、冲突跳过并上报）——模板里**不要**把项目可能自定义的表写到受管块之外
- 含受管块（`PENGJ_TEMPLATE_START/END`）的文本文件更新时只替换块区间，块外内容归用户所有
- 生成后会在项目根写入 `.pengj-templates.json` manifest，记录使用的层与模板托管文件的哈希，供 `update` 使用

## just / bque 排队装配

- `just` 层产出**唯一的**根 `justfile`：框架区在 `# PENGJ_TEMPLATE_START/END` 托管块内（`set shell`、`default`、按层条件 `import`），块外的「项目专属区」归项目所有，`update` 只替换块内，不再整页覆盖。
- `bque` 层产出 `just/bque.just`（`queue-*` / `queue-doctor` / `q-run` / `q-bench`）；`gate` 层产出 `just/gate.just`（`gate-check` / `gate-hooks` / `gate-selftest`）；`rust` 与 `rust-workspace` 层分别产出 `just/rust.just` / `just/rust-workspace.just`。主 justfile 按 `layers` 条件 import，各层的 `depends` 都含 `just` 以保证顺序。
- `options["just_recipes"]`：`"rust"` / `"workspace"` 显式指定语言配方；`"none"` 关闭语言配方导入（已有自定义编排的项目只接入排队基础设施）；未设置时按所选层自动推断。
- 勾 `bque` 层时会自动把 `bque` 技能并入 `options["skills"]`，避免「有配方没文档」的半接入；`audit` 对半接入、以及缺少 justfile 托管块的项目给出配置提示。
- 存量项目加层走 `add-layer --dir <项目> --layers just,bque`：只合并层清单并重跑 `update`，不做全量重纳管。
- bque 是 Windows 全局锁；`[unix]` 分支的同一批配方直接透传，CI 与 Unix 裸跑属预期行为。

## 语言轴 / Language axes

- `skill_lang`（`zh` / `en`）决定**所有层**面向人的文案：注释、报错提示、脚本日志、`.gitignore` 拼接段标题以及 agent 层文档。模板统一用 `{% if options["skill_lang"] == "en" %}…{% else %}…{% endif %}` **行内分支**（不引入额外空行），中文分支渲染结果必须与改动前逐字节一致，避免存量中文项目漂移。
- `commit_zh` 独立控制门禁的提交主题语言规则（`zh` / `en`）与提交规范技能文档。
- **代码类产物按语言拆文件，而不是在脚本里写双语分支**：同一份逻辑各写一版完整文件，
  命名 `<stem>.<lang>.<ext>`（跟随 `skill_lang`）或 `<stem>.commit.<lang>.<ext>`（跟随 `commit_zh`，
  用于提交校验类产物）。生成时只写命中那份，产物路径去掉语言标记，另一版不会落到项目里，
  因此英文项目的脚本里不会残留中文校验逻辑，反之亦然。
- 全英文项目用 `--skill-lang en --no-commit-zh`；生成后所有文本文件零汉字（`crates/core` 有端到端单测保障）。`.agents/skills/*/scripts/` 恒为英文，不受语言轴影响。
- **代码语言守卫**：`gate` 层产出 `.gate.toml`（配置）与 `.githooks/pre-commit`（钩子），pre-commit 扫描暂存的 `.rs`。判定方向来自 `.gate.toml` 的 `mode`（`english` 查非拉丁文字、`chinese` 查非中文），按文件算占比：字符串违规 ≤ `lang.max_ratio`（默认 1%）、标识符违规 ≤ `lang.max_ident_ratio`（默认 0）。中文模式只判定面向人的文案，机器字面量（参数、路径、协议字段、内嵌源码）不判违规但仍计入分母。真实例外用行尾 `lang-ok: 理由` 或文件头 `lang-allow: 理由` 认证。门禁是纯 Rust 实现、零 Node 依赖，已收编进 `pengj-templates-cli`（唯一入口 `pengj-templates-cli gate ...`），不另有独立二进制。
- **提交门禁装配**：`gate` 层依赖 `common`，产出 `.gate.toml`（首次生成后归用户所有，列入 `update_ignore`）与 `.githooks/pre-commit` / `.githooks/commit-msg`（模板托管，随更新同步）。钩子脚本在写盘后补可执行位——git 在 Unix 上靠它决定跑不跑钩子，缺位会静默跳过。生成项目后需执行一次 `just gate-hooks`：它把 `core.hooksPath` 指向 `.githooks` 并把模式位写进 git 索引（`core.hooksPath` 存在 `.git/config` 里，不会随克隆传播）。钩子直接调 `pengj-templates-cli gate hook ...`，没有 `$PENGJ_GATE` / 独立 `gate` / 仓库 URL 之类的多级回退分支；机器上没装 CLI 时钩子会报错并给出一键安装命令。
- **历史层名迁移**：老的 `lefthook` 层已被 `gate` 层取代。存量项目的 manifest 里仍写着 `lefthook`，引擎按 `LEGACY_LAYER_ALIASES` 映射到 `gate`，不会被当成「已移除的层」静默丢弃。
