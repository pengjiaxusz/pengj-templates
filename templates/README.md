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
- 生成后会在项目根写入 `.pengj-templates/manifest.json` manifest，记录使用的层与模板托管文件的哈希，供 `update` 使用
- **模板产物统一收在 `.pengj-templates/` 下**：`manifest.json`、`gate.toml`、`scripts/`（doctor 本体与 `doctor.d/` 扩展点）、`just/`（各层配方模块）。这是硬约定，新增层不要往根的 `scripts/`、`just/` 或`.xxx.toml` 里落文件——那两个名字留给项目自己用。唯一例外是根 `justfile`：just 只认项目根的 `justfile`，且它含受管块需要原位更新，必须留在根上。

## just / bque 排队装配

- `just` 层产出**唯一的**根 `justfile`：框架区在 `# PENGJ_TEMPLATE_START/END` 托管块内（`set shell`、`default`、按层条件 `import`），块外的「项目专属区」归项目所有，`update` 只替换块内，不再整页覆盖。
- `bque` 层产出 `.pengj-templates/just/bque.just`（`queue-*` / `queue-doctor` / `q-run` / `q-bench`）；`gate` 层产出 `.pengj-templates/just/gate.just`（`gate-check` / `gate-hooks-local` / `gate-selftest`）；`rust` 与 `rust-workspace` 层分别产出 `.pengj-templates/just/rust.just` / `.pengj-templates/just/rust-workspace.just`。主 justfile 在选中 `just` 层时按 `layers` 条件 import；`bque` / `rust` / `rust-workspace` 的 `depends` 含 `just`，`gate` 只依赖 `common`——未选 just 时它的模块不被 import，门禁仍由 `.githooks` 与 `.pengj-templates/scripts/doctor.sh` 提供。
- `common` 层产出 `.pengj-templates/scripts/doctor.sh` / `.pengj-templates/scripts/doctor.ps1`，`just` 层在根 justfile 里提供 `doctor` 配方。`just doctor --fix` 是生成项目**唯一的初始化入口**（不提供独立的 setup 配方）：doctor 检查已选层实际用到的工具（`just` / `pwsh` / `cargo` / `bque` / `pengj-templates-cli` / `node`）与 gate 钩子激活状态；`--fix` 优先调用 `.githooks/install.sh`（零依赖）激活钩子，并在 `agent` 层项目里调用 `pengj-templates-cli repo init` 创建 `.trae` / `.claude/skills` / `.workbuddy/skills` 工作区链接（`.trae` 整目录链到 `.agents`，另两条只链 `skills`，因为 `.claude` / `.workbuddy` 里还有客户端自己的状态；链接路径由 agent 层的 `.gitignore` 忽略，Trae 写进 `.agents/` 的 `specs|plans|documents` 由随附的 `.agents/.gitignore` 挡住）。`gate` / `agent` 层项目里缺 `pengj-templates-cli` 记为**错误**（门禁与链接都依赖它），安装是唯一会联网的动作，只能由显式 `--install-cli` 触发，`--fix` 永不联网。`repo init` 还会补齐仓库级 Git 配置（存在 `.gitmodules` 时开启 `submodule.recurse`，跟踪符号链接时开启 `core.symlinks`）并只初始化尚未初始化的子模块。用户把自定义检查与修复脚本放进 `.pengj-templates/scripts/doctor.d/` 即可扩展（退出码非 0 记为失败），契约见该目录的 `README.md`。
- `options["just_recipes"]`：`"rust"` / `"workspace"` 显式指定语言配方；`"none"` 关闭语言配方导入（已有自定义编排的项目只接入排队基础设施）；未设置时按所选层自动推断。
- 勾 `bque` 层时会自动把 `bque` 技能并入 `options["skills"]`，避免「有配方没文档」的半接入；勾 `gate` 层时会自动并入 `ci` 层，保证「未装 CLI 的机器由服务端 CI 兜底」始终成立；`audit` 对半接入、以及缺少 justfile 托管块的项目给出配置提示。
- 存量项目加层走 `add-layer --dir <项目> --layers just,bque`：只合并层清单并重跑 `update`，不做全量重纳管。
- bque 是 Windows 全局锁；`[unix]` 分支的同一批配方直接透传，CI 与 Unix 裸跑属预期行为。

## 语言轴 / Language axes

- `skill_lang`（`zh` / `en`）决定**所有层**面向人的文案：注释、报错提示、脚本日志、`.gitignore` 拼接段标题以及 agent 层文档。模板统一用 `{% if options["skill_lang"] == "en" %}…{% else %}…{% endif %}` **行内分支**（不引入额外空行），中文分支渲染结果必须与改动前逐字节一致，避免存量中文项目漂移。`common` 层的 `.pengj-templates/scripts/doctor.sh` / `.pengj-templates/scripts/doctor.ps1` 同样按 `skill_lang` 双语输出（保留 `[doctor] ok/warn/error` 结构化前缀，只翻译文案）；注意 `tools/install.*` 一键安装脚本随 Release 独立分发、不经模板渲染，其语言走**系统语言**（`zh-*` 显示中文，其余英文），与 `skill_lang` 无关。
- `commit_zh` 独立控制门禁的提交主题语言规则（`zh` / `en`）与提交规范技能文档。
- **代码类产物按语言拆文件，而不是在脚本里写双语分支**：同一份逻辑各写一版完整文件，
  命名 `<stem>.<lang>.<ext>`（跟随 `skill_lang`）或 `<stem>.commit.<lang>.<ext>`（跟随 `commit_zh`，
  用于提交校验类产物）。生成时只写命中那份，产物路径去掉语言标记，另一版不会落到项目里，
  因此英文项目的脚本里不会残留中文校验逻辑，反之亦然。
- 全英文项目用 `--skill-lang en --no-commit-zh`；生成后所有文本文件零汉字（`crates/core` 有端到端单测保障）。`.agents/skills/*/scripts/` 恒为英文，不受语言轴影响。
- **文档语言双向门禁**：`crates/core/src/i18n_audit.rs` 对全部生成文本文件双向判定——英文项目不得出现汉字，中文项目里面向人的文案必须含汉字。只查 en 不查 zh 会漏掉「纯英文、没写双语分支」的模板（`doctor.d/README.md` 就是这么把英文文档带进中文项目的）。机器产物走窄白名单：`.json`、`.agents/skills/*/scripts/*`、`Cargo.toml`、`rust-toolchain.toml`、`src/main.rs`。已生成项目跑 `pengj-templates-cli check-lang --dir <项目>`（可 `--lang zh|en` 覆盖、`--json`）或 `just gate-lang`，只审计 manifest 里的模板托管文件；模板改动由 `skill_lang_drives_every_generated_file_language` 端到端测试在 CI 拦截。
- **代码语言守卫**：`gate` 层产出 `.pengj-templates/gate.toml`（配置）与 `.githooks/pre-commit`（钩子），pre-commit 扫描暂存的 `.rs`。判定方向来自 `.pengj-templates/gate.toml` 的 `mode`（`english` 查非拉丁文字、`chinese` 查非中文），按文件算占比：字符串违规 ≤ `lang.max_ratio`（默认 1%）、标识符违规 ≤ `lang.max_ident_ratio`（默认 0）。中文模式只判定面向人的文案，机器字面量（参数、路径、协议字段、内嵌源码）不判违规但仍计入分母。真实例外用行尾 `lang-ok: 理由` 或文件头 `lang-allow: 理由` 认证。门禁是纯 Rust 实现、零 Node 依赖，已收编进 `pengj-templates-cli`（唯一入口 `pengj-templates-cli gate ...`），不另有独立二进制。
- **提交门禁装配**：`gate` 层依赖 `common`，产出 `.pengj-templates/gate.toml`（首次生成后归用户所有，列入 `update_ignore`）与 `.githooks/pre-commit` / `.githooks/commit-msg`（模板托管，随更新同步）。钩子脚本在写盘后补可执行位——git 在 Unix 上靠它决定跑不跑钩子，缺位会静默跳过。生成项目后（或新克隆后）跑 `just doctor --fix`（无 just 时用 `sh .pengj-templates/scripts/doctor.sh --fix` / `pwsh -NoProfile -File .pengj-templates/scripts/doctor.ps1 -Fix`）：doctor 检查工具链并优先调用 `.githooks/install.sh` 把 `core.hooksPath` 指向 `.githooks`；CLI 安装器不再写全局 `core.hooksPath`，doctor 也不再检测或告警旧的全局 dispatcher（legacy global gate）——钩子激活一律按仓库本地 `core.hooksPath` 判定。`gate` 层自动带上 `ci` 层，服务端跑的是同一个二进制。钩子只负责找到 CLI 并把活交给它，**模板不带任何兜底脚本**：没装 CLI 时 `commit-msg` 与 `pre-commit` 都拦下提交并打印安装命令；也没有 `$PENGJ_GATE` / 独立 `gate` / 仓库 URL 之类的多级回退分支。
- **历史层名迁移**：老的 `lefthook` 层已被 `gate` 层取代。存量项目的 manifest 里仍写着 `lefthook`，引擎按 `LEGACY_LAYER_ALIASES` 映射到 `gate`，不会被当成「已移除的层」静默丢弃。

## 没装 CLI 时

门禁只有在 `pengj-templates-cli` 里的一份实现，模板不自带第二套脚本。所以没装 CLI 时：

- **本地**：`.githooks/commit-msg` 与 `.githooks/pre-commit` 都拦下提交，打印安装命令后退出——不放行，也不用一套更宽松的 shell 规则顶替。
- **服务端**：`ci` 层的 `gate` 任务跑同一个二进制，`.github/rulesets/gate.json` 把它设成必需状态检查，PR 一样过不去。CI 失败时把可操作的修复提示写进 `GITHUB_STEP_SUMMARY`。

钩子激活本身不需要 CLI：`just doctor --fix`（或 `sh .pengj-templates/scripts/doctor.sh --fix`）调用 `.githooks/install.sh`，后者只用 git 与一个 POSIX shell 就把 `core.hooksPath` 指向 `.githooks` 并补上可执行位；`just gate-hooks-local` 是同一个脚本的裸入口。

`ci` 层的 workflow 里 job 名固定为 `gate`：必需状态检查引用的就是这个名字，不能因为 `skill_lang` 切换而失效。

## `.githooks/` 的语言写法

`.githooks/` 下的脚本（`commit-msg`、`pre-commit`、`install.sh`）用**行内双语分支**而不是 `<stem>.<lang>.<ext>` 拆文件：它们是一套钩子的三个入口，拆成两份完整副本会让修复只落到一半。渲染结果仍然满足「英文项目零汉字」——每个中文字符串都在 `{% if %}` 分支里。
