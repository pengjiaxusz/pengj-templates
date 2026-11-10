# pengj-templates

分层模板生成与同步更新工具：把仓库模板按「层（layer）」组织，勾选所需层即可生成新仓库；模板更新后，一键同步到所有基于它生成的项目。

## 核心思路：分层

模板内容按适用面拆成若干层，生成时自由组合，后选的层覆盖先选层的同名文件：

| 层 | 内容 | 适用 |
| --- | --- | --- |
| `common` | 所有仓库共用：`.gitignore` | 所有仓库 |
| `just` | 唯一的根 `justfile` 主入口：托管块框架区 + 块外项目专属区，按层 import 语言/排队配方 | 需要统一任务入口的仓库 |
| `bque` | Windows 全局编译排队：`just/bque.just`（`queue-*` / `queue-doctor` / `q-run` / `q-bench`） | 需要跨工作树排队与纯净跑分 |
| `lefthook` | Git hooks（lefthook）+ commitlint 约定式提交校验 | 需要 Git 钩子的仓库 |
| `agent` | AI 编码助手规范（`AGENTS.md`），按所选层/选项生成 rust、中文编程等约定 | 希望 agent 遵循规范 |
| `vscode` | VS Code 工作区配置：推荐扩展、中文编程下 rust 检查走 clippy | 使用 VS Code 开发 |
| `rust` | Rust 仓库专属：`Cargo.toml`、clippy/rustfmt、sccache、`just/rust.just` | Rust 项目 |
| `rust-workspace` | Rust 多 crate 工作区：工作区 `Cargo.toml`、sccache/lld、`just/rust-workspace.just` | Rust workspace |
| ... | 按需扩展（新增目录 + `layer.toml` 即可） | ... |

模板文件支持 minijinja 占位符（`{{ project_name }}`、`{{ project_slug }}`、`{{ year }}`）。

## 技术栈

- **引擎**：Rust（`crates/core`）— 分层合并、渲染、diff 与更新逻辑，模板编译期嵌入二进制
- **MCP 安装**：Rust（`crates/mcp`）— 配方目录 + 客户端适配 + 真实握手巡检，机器级（跨项目）配置
- **GUI**：Tauri 2 + React + Vite + TypeScript + Tailwind v4 + shadcn/ui（`crates/app`）
- **CLI**：clap（`crates/cli`，命令 `pengj-templates-cli`），与 GUI 共用同一套引擎

## 使用

### CLI

```bash
cargo run -p pengj-templates-cli -- list-layers                          # 列出可用层
cargo run -p pengj-templates-cli -- create my-app --layers rust          # 生成 common + rust
cargo run -p pengj-templates-cli -- create my-app --layers rust --edition 2024 --channel nightly --no-lld
cargo run -p pengj-templates-cli -- create my-app --layers agent --skills commit,caveman   # 只生成选中的技能
cargo run -p pengj-templates-cli -- create my-app --layers rust --skill-lang en --no-commit-zh  # 全英文项目：所有文案与提交信息均为英文
cargo run -p pengj-templates-cli -- update --dir ./my-app                # 同步模板更新
cargo run -p pengj-templates-cli -- add-layer --dir ./my-app --layers just,bque  # 存量项目增量加层
cargo run -p pengj-templates-cli -- audit --dir ./my-app                 # 巡检半接入与托管块漂移
```

**just / bque 排队装配**：根 `justfile` 由 `just` 层统一产出，框架区在 `# PENGJ_TEMPLATE_START/END` 托管块内（`set shell`、`default`、按层条件 `import`），块外的项目专属区归用户所有、`update` 只替换块内。`bque` / `rust` / `rust-workspace` 层分别产出 `just/bque.just`、`just/rust.just`、`just/rust-workspace.just`，三层都依赖 `just`；`options["just_recipes"] = "none"` 可关闭语言配方导入，让已有自定义编排的老项目只接入排队基础设施。勾 `bque` 层会自动补齐 `bque` 技能，避免半接入；`audit` 会提示半接入与缺少 justfile 托管块的项目。bque 是 Windows 全局锁，Unix / CI 下同一批配方直跑透传。详见 [`templates/README.md`](templates/README.md)。

`create` 的 Rust 选项（仅 rust 层生效）：`--edition 2015|2018|2021|2024`（默认 2024）、`--channel stable|beta|nightly|<版本>`（默认 stable）、`--sccache` 开启编译缓存（默认关）、`--no-lld` 关闭 lld 链接、`--chinese` 开启中文编程（允许中文标识符、关闭相关命名 lint）。选择在生成时固化进 `.pengj-templates.json`，`update` 同步时按各项目当年的选项重新渲染。

`agent` 层的技能与规范选项（选 agent 层时生效）：`--skills commit,caveman,grill-me,arch-align,template-sync,subrepo-sync,branch-sync` 决定生成哪些技能（逗号分隔，默认全部；GUI 里用勾选框选择）、`--skill-lang zh|en` 决定技能文档书写语言（默认 zh）、`--no-commit-zh` 让提交信息用英文（默认中文）、`--commit-and-push` 要求 Agent 阶段性完工与收尾时自动提交并推送到远端仓库（默认关）。`mcp-bootstrap` 是基线技能：只要有 agent 层就始终附带（显式传空技能列表除外），保证项目离开模板仓库后仍能自行安装与校验 MCP。技能生成到 `.agents/skills/<name>/SKILL.md`，目前有 `commit`（约定式提交）、`template-sync`（模板同步/纳管）、`subrepo-sync`（子仓库同步与升级）、`branch-sync`（Worktree感知分支极速同步）、`caveman`（超压缩通信）、`grill-me`（设计质询）、`arch-align`（架构对齐）、`mcp-bootstrap`（项目内 MCP 安装与握手校验）等，新增技能只需在 `templates/agent/.agents/skills/` 下加目录即可。三条语言选项互相独立：**中文编程**（代码标识符，`chinese_programming`）、**项目文案语言**（`skill_lang`，决定所有层的注释、报错、日志与技能文档）、**提交信息语言**（`commit_zh`，决定提交校验脚本选版与提交规范文档）；技能脚本恒为英文，无选项。全英文项目须同时传 `--skill-lang en --no-commit-zh`，生成后所有文本文件零汉字（有端到端单测保障）。

**技能扩展**：每个技能 = 托管框架 + 项目专属区。`SKILL.md` 的 `PENGJ_TEMPLATE_START/END` 块内是模板框架（`update` 时原位替换），块外归项目所有、永不覆盖——提交前检查的定义、领域判定表、红线全部写在块外；可执行门禁用什么形式（任意语言脚本 / task runner / 纯清单）由项目自定并在块外声明，框架不预设实现。存量全自定义技能纳管时自动接管：模板整页（含 description）直接覆盖，原正文下移到「纳管过渡区」，双流程由用户合并去重。

### MCP 安装与巡检（`mcp` 子命令）

模板管的是「仓库里的文件」，MCP 管的是「机器上各客户端的配置」——同一个 MCP 服务器通常要在多个编辑器和 CLI 里各注册一次，而很多客户端只支持全局配置。`mcp` 子命令就是这一层的入口：一份配方，写到所有客户端。

```bash
cargo run -p pengj-templates-cli -- mcp list                      # 配方 + 本机安装状态
cargo run -p pengj-templates-cli -- mcp add renderdoc --dry-run    # 预演：只打印计划
cargo run -p pengj-templates-cli -- mcp add renderdoc              # 装主体并注册到全部客户端
cargo run -p pengj-templates-cli -- mcp add renderdoc --clients workbuddy,opencode
cargo run -p pengj-templates-cli -- mcp add renderdoc --with-side --enable-app   # 扩展自动浅克隆到托管缓存
cargo run -p pengj-templates-cli -- mcp add renderdoc --with-side --source-root <已有仓库副本>
cargo run -p pengj-templates-cli -- mcp doctor                     # 静态查死链 + 真实 stdio 握手
cargo run -p pengj-templates-cli -- mcp doctor --fix               # 就地清理死链
cargo run -p pengj-templates-cli -- mcp print renderdoc --client trae   # 出片段，手工粘贴兜底
cargo run -p pengj-templates-cli -- mcp remove renderdoc [--purge]      # 摘注册 / 连主体与源码缓存一起卸
```

已内置的客户端适配（`--clients` 的取值）：

| id | 配置位置 | 结构差异 |
|----|----------|----------|
| `workbuddy` | `~/.workbuddy/mcp.json` | `mcpServers`，写完需在 GUI 点一次「信任」 |
| `opencode` | `~/.config/opencode/opencode.json` | `mcp.servers`，`command` 为**数组** |
| `claude-code` | `~/.claude.json` | 顶层 `mcpServers`（文件含大量无关状态，原样保留） |
| `agy` | `~/.gemini/config/mcp_config.json` | `mcpServers`，远程用 `serverUrl`（不支持 `url`） |
| `trae` | `%APPDATA%\Trae CN\User\mcp.json` | `mcpServers`，`command` **不能含空格** |

要点：

- **写前备份、保留未知键、幂等**：只改目标条目，其余字段与键序不动；首次改写前留 `.pengj-mcp.bak`；重复执行不产生改动。
- **命令写绝对路径**：GUI 客户端 spawn 子进程时 PATH 往往不全，裸命令名会挂；`add` 会先解析成绝对路径再写入。
- **注册 ≠ 生效**：多数客户端需要重启或点确认，`add` 会把「待人工动作」逐条列出，`doctor` 事后复查。
- **配方可外置**：内置配方编在二进制里，`~/.pengj-mcp/catalog/*.toml`（或 `PENGJ_MCP_CATALOG`）按同名覆盖，改配方不必重编译。配方字段见 `crates/mcp/catalog/renderdoc.toml`。
- **磁盘落点只有三处**：主体在 `uv tool` 的安装目录（`uv tool uninstall` 可卸）、源码缓存在 `~/.pengj-mcp/src/<name>`（仅「wheel 不含附属产物」的项目需要，`remove --purge` 一并删除）、状态在 `~/.pengj-mcp/manifest.json`；不写工作目录，也不建临时 venv。
- **已装状态**：`~/.pengj-mcp/manifest.json` 记录来源与写进了哪些客户端，`remove` / `doctor` 据此判断。

## CI / Release
- **CI**（`.github/workflows/ci.yml`）：
  - 按变更自动分流：改动 Rust/模板/构建配置 → 跑 `cargo fmt --check`、`clippy -D warnings`、build、test；改动前端（`crates/app/src` 等）→ 跑类型检查 + 前端构建（tsc + vite）。
  - **纯文档改动自动跳过**（根目录 `*.md`、`docs/**`、`.agents/**`），不占 CI 额度；同一分支/PR 连续推送会自动取消上一次未完成的运行。
  - 缓存：Rust 编译缓存（rust-cache，含失败缓存）+ pnpm 依赖缓存。
- **自动版本号 + Release**（`.github/workflows/release.yml`）：
  - 纯文档改动（与 CI 相同的忽略列表）不触发本流程，只有改动代码/模板/配置时才跑。
  - `release-please` 读 conventional commits 自动 bump semver，多人改版本号到 root `Cargo.toml`、`crates/app` Cargo/package.json、`tauri.conf.json`，并创建 release PR；合并后打 tag、建 GitHub Release。发布后由 `sync-lockfile` job 自动把新版本号写回 `Cargo.lock` 并提交，无需手动同步。
  - 有 `release` 时自动构建发布。**产物命名统一带「系统-架构」**：`pengj-templates_<版本>_<os>-<arch>.<后缀>`。
    - 覆盖 Windows x64/arm64、Linux x86_64/arm64、macOS arm64/x86_64（arm 用 GitHub 原生 arm64 runner 构建）。
    - 安装包：Windows `_windows-<arch>.msi` / `-setup.exe`、Linux `_linux-<arch>.deb` / `.AppImage`、macOS `_macos-<arch>.dmg`。
    - 每组合一个**便携版 zip** `pengj-templates_<版本>_<os>-<arch>_portable.zip`，同时含 GUI + CLI（CLI 命名 `pengj-templates-cli`）。
  - **全自动发布**：`auto-merge` job 在 release-please 创建 Release PR 后用 PAT 自动合并（无需手动），合并即触发上文构建。
  - **beta/preview 分阶段**：release-please 原生支持用 commit footer 指定预发布号并自动递增：
    - 切到某预发布号：正常提交时在其描述加 `Release-As: 0.5.0-beta.1`、`0.5.0-rc.1`、`0.5.0-preview.1` 等；
    - 之后它会在同阶段自动 +1（`beta.1`→`beta.2`），GitHub Release 会自动标为 **Prerelease**；
    - 要回到正式版，再提交 `Release-As: 0.5.0` 即可。
  - 版本号方案（release-please 规则）：`feat`→minor、`fix`→patch、`BREAKING CHANGE`→major（`0.x` 阶段因 `bump-minor-pre-major=true`，BREAKING 只升 minor 而不跳 major；`feat` 在 `0.x` 也仍升 minor）。

`rust` 层的 `src/main.rs` 在 `update_ignore` 黑名单中：仅首次生成时写入，之后归用户所有，模板更新时跳过（不覆盖、不冲突、不删除上报）。若模板里还有这类「种子文件、后续归用户」的文件，在各 `layer.toml` 的 `update_ignore` 里列出即可。

`package.json` 走**结构化并集合并**：生成时各层并集（后层覆盖同名项）；更新/纳管时以用户现有文件为底——**依赖类字段（dependencies/devDependencies 等）的同名包版本一律以模板为准**（依赖集合由模板统一维护；模板用 `latest` 时实际解析版本由项目 lockfile 锁定，升级走 `pnpm update --latest`），`scripts` 与其余字段仍用户优先、模板只补缺失，键顺序保持原样不产生噪声 diff。`layer.toml` 里用 `update_ignore` 声明黑名单文件。

`.cargo/config.toml` 走 **TOML 结构化受管合并**：受管块内的表与用户配置做表级并集、键级去重——用户已有的等价键自动去重（避免重复表定义炸掉 cargo），模板新增表进入受管块；同键不同值或需要往用户已有表里补新键时报冲突并跳过写盘，由用户手工对齐后重跑。文本类文件（AGENTS.md、技能 SKILL.md）走**受管块注入**：磁盘已有托管块则原位替换，无块的 legacy 文件则把托管块追加到末尾并计入 `needs_review` 提示人工复核。纳管/更新时若既有 `commitlint.config.js` 未继承 `commitlint.base.js`，会自动改写为继承形态（base 规则铺底、项目规则同名覆盖，计入 `needs_review`）；无法安全改写的形态（CJS 等）才在报告中给出手动接线指引。

`.vscode/settings.json` 的模板配置在**项目根已有 `*.code-workspace` 文件时并入工作区文件的 `settings` 节点**（不新建独立 settings 文件）；没有工作区文件时才创建/维护 `.vscode/settings.json`。

lefthook 层用法：commitlint CLI 走本地依赖（`pnpm exec commitlint`），`pnpm-lock.yaml` 由 git 托管以保持可复现；生成项目时先 `git init` 再 `pnpm install`，`prepare` 自动装 lefthook git 钩子；升级用 `pnpm update --latest`。

### GUI

```bash
just frontend-install
just dev
```

生成项目时写入 `.pengj-templates.json` manifest（记录层与模板托管文件哈希），`update` 依据它只更新模板托管文件：本地未改动的文件直接覆盖、被改过的报冲突跳过、含受管块的文件做块级合并（TOML 结构化合并 / 文本托管块注入）、模板新增的自动创建。报告中 `needs_review` 列出建议人工复核的合并结果，adopt 的 `manual_steps` 给出需要手动完成的接线步骤。

## 目录结构

```
crates/
├── core/         # 引擎：层发现、依赖排序、合并渲染、生成/更新、manifest
├── cli/          # CLI（pengj-templates-cli）
├── mcp/          # MCP 安装与巡检：配方目录、客户端适配、握手校验
└── app/          # Tauri GUI（src-tauri + React 前端）
templates/        # 层模板（common/ rust/ frontend/ ...），编译期嵌入
```

## 开发

```bash
just check            # Rust 全部编译检查
just frontend-build   # 前端构建
```
