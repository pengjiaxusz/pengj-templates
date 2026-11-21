# pengj-templates

> [English](README.md) | **简体中文**

分层模板生成与同步更新工具：把仓库模板按「层（layer）」组织，勾选所需层即可生成新仓库；模板更新后，一键同步到所有基于它生成的项目。

## 核心思路：分层

模板内容按适用面拆成若干层，生成时自由组合，后选的层覆盖先选层的同名文件：

| 层 | 内容 | 适用 |
| --- | --- | --- |
| `common` | 所有仓库共用：`.gitignore`、`.pengj-templates/scripts/doctor.*` 环境自检脚本 | 所有仓库 |
| `just` | 唯一的根 `justfile` 主入口：托管块框架区 + 块外项目专属区，按层 import 语言/排队配方；提供 `doctor` 入口 | 需要统一任务入口的仓库 |
| `bque` | Windows 全局编译排队，只做声明：并入 `bque` 技能、让 `doctor` 检测本机安装；不产出配方——`just` 包装带不动 bque 自己的 `--cwd` / `--observer` / `--priority`，重型命令直接 `bque run --` | 需要跨工作树排队与纯净跑分 |
| `gate` | 提交门禁：`.pengj-templates/gate.toml` 配置 + `.githooks/*` 钩子（约定式提交校验、代码语言守卫、rustfmt），纯 Rust 实现、零第三方依赖；自动带上 `ci` 层在服务端执行同一套规则 | 需要 Git 钩子的仓库 |
| `ci` | 服务端门禁：GitHub Actions 在 PR/合并队列/主干推送时校验提交标题与代码语言，附必需状态检查 ruleset 与 PR 自查模板 | 托管在 GitHub 上的仓库 |
| `agent` | AI 编码助手规范（`AGENTS.md`），按所选层/选项生成 rust、中文编程等约定 | 希望 agent 遵循规范 |
| `vscode` | VS Code 工作区配置：推荐扩展、中文编程下 rust 检查走 clippy | 使用 VS Code 开发 |
| `rust` | Rust 仓库专属：`Cargo.toml`、clippy/rustfmt、可选 sccache、`.pengj-templates/just/rust.just` | Rust 项目 |
| `rust-workspace` | Rust 多 crate 工作区：工作区 `Cargo.toml`、可选 sccache/lld、`.pengj-templates/just/rust-workspace.just` | Rust workspace |
| `subrepo` | 子仓库登记与校验：机器可读的 `.pengj-templates/subrepo.toml`（名称、相对路径、依赖模式、上游分支、是否自带工作区定义）与 `.pengj-templates/just/subrepo.just`，带走 `subrepo-sync` 技能 | 依赖外部仓库（子模块 / FetchContent / 本地同级目录）的项目 |
| `worktree` | 多工作树**与**共享编译目录：`.pengj-templates/just/worktree.just`（`wt-target` / `wt-check` / `wt-fix` / `wt-create`），共享目录就是主工作树的 `target/`，由构建配方注入 `CARGO_TARGET_DIR` 到链接工作树，新建工作树时补齐子模块，带走 `worktree` 技能 | 需要并行开多个 git 工作树的项目 |
| ... | 按需扩展（新增目录 + `layer.toml` 即可） | ... |

模板文件支持 minijinja 占位符（`{{ project_name }}`、`{{ project_slug }}`、`{{ year }}`）。

## 技术栈

- **引擎**：Rust（`crates/core`）— 分层合并、渲染、diff 与更新逻辑；模板从运行时目录加载（`PENGJ_TEMPLATES` > 当前目录 `templates/` > 可执行文件旁 `templates/` > 可执行文件上两级 `templates/` > 平台数据目录 `pengj-templates/templates/`）
- **MCP 安装**：Rust（`crates/mcp`）— 配方目录 + 客户端适配 + 真实握手巡检，机器级（跨项目）配置
- **仓库初始化**：Rust（`crates/core/src/repo.rs`，CLI 入口 `repo init/status`）— 工作区客户端链接、仓库级 Git 配置与子模块补缺，供 `just doctor --fix` 调用，不依赖模板层
- **提交门禁**：Rust（`crates/gate`）— 约定式提交校验、代码语言守卫、rustfmt 暂存格式化；零第三方依赖，已收编为内部库，唯一入口是 `pengj-templates-cli gate`
- **CLI**：clap（`crates/cli`，命令 `pengj-templates-cli`），全仓库唯一入口；内置 `repo` 与 `gate` 子命令

## 使用

### 一键安装（推荐）

新机器上一条命令装好 **模板 CLI + 提交门禁**（同一个二进制），不需要单独装 `gate`：

```powershell
irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex   # Windows
```

```bash
curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh   # Linux / macOS
```

安装脚本从最新 Release 拉取对应「系统-架构」的 bundle（`pengj-templates-cli.exe` + 随附 `templates/` 模板目录），落到平台数据目录（Windows `%LOCALAPPDATA%\pengj-templates`，Unix `${XDG_DATA_HOME:-~/.local/share}/pengj-templates`），并把 `bin/` 加进 `PATH`。安装脚本是独立脚本而非渲染模板，所以它的提示按**系统语言**显示：中文系统 / 区域设置（`zh-*`）显示中文，其余显示英文。（生成项目的文案是另一回事，走 `--skill-lang`。）装完即可：

```bash
pengj-templates-cli --version
pengj-templates-cli gate --help          # 门禁随 CLI 一起来
pengj-templates-cli create my-app --layers rust
```

> 已有旧版会被脚本热替换；模板目录与二进制一起升级，不会出现「CLI 新、模板旧」的错配。想临时用别的模板根目录，设 `PENGJ_TEMPLATES` 环境变量即可覆盖。

### CLI

以下示例用 `cargo run`（开发态）。已安装的用户直接写 `pengj-templates-cli` 即可。

```bash
cargo run -p pengj-templates-cli -- list-layers                          # 列出可用层
cargo run -p pengj-templates-cli -- create my-app --layers rust          # 生成 common + rust
cargo run -p pengj-templates-cli -- create my-app --layers rust --edition 2024 --channel nightly --no-lld
cargo run -p pengj-templates-cli -- create my-app --layers agent --skills commit,caveman   # 只生成选中的技能
cargo run -p pengj-templates-cli -- create my-app --layers rust --skill-lang en --no-commit-zh  # 全英文项目：所有文案与提交信息均为英文
cargo run -p pengj-templates-cli -- update --dir ./my-app                # 同步模板更新
cargo run -p pengj-templates-cli -- add-layer --dir ./my-app --layers just,bque  # 存量项目增量加层
cargo run -p pengj-templates-cli -- audit --dir ./my-app                 # 巡检半接入与托管块漂移
cargo run -p pengj-templates-cli -- check-lang --dir ./my-app           # 校验文案语言轴（en 零汉字；zh 文案含中文）
cargo run -p pengj-templates-cli -- repo status --dir ./my-app          # 检查工作区链接、Git 配置与子模块
cargo run -p pengj-templates-cli -- repo init --dir ./my-app            # 创建缺失的工作区链接（幂等，不覆盖）
cargo run -p pengj-templates-cli -- subrepo list --dir ./my-app         # 把机器可读的子仓库登记表渲染成清单
cargo run -p pengj-templates-cli -- subrepo check --dir ./my-app        # 子仓库体检（退出码 0 通过 / 1 有失败 / 2 仅警告）
cargo run -p pengj-templates-cli -- worktree target-dir --dir ./my-app  # 打印本机推导出的共享编译目录
cargo run -p pengj-templates-cli -- worktree check --dir ./my-app --fix # 校验（并创建）共享编译目录
cargo run -p pengj-templates-cli -- worktree create feature-a --dir ./my-app  # 新建工作树：分支 + 同级目录 + 共享编译目录 + 子模块补齐
cargo run -p pengj-templates-cli -- check-update                       # 查 CLI 自身有没有新版本
```

**just / bque 排队装配**：根 `justfile` 由 `just` 层统一产出，框架区在 `# PENGJ_TEMPLATE_START/END` 托管块内（`set shell`、`default`、按层条件 `import`），块外的项目专属区归用户所有、`update` 只替换块内。`gate` / `rust` / `rust-workspace` / `subrepo` / `worktree` 层分别产出 `.pengj-templates/just/gate.just`、`rust.just`、`rust-workspace.just`、`subrepo.just`、`worktree.just`，全部依赖 `just`；`bque` 层**有意不产出配方**，只做排队声明（技能 + doctor 检测）——`just` 包装能转发命令却带不动 bque 自己的 `--cwd` / `--observer` / `--priority`，所以重型命令在项目配方里直接 `bque run -- <cmd>`。`options["just_recipes"] = "none"` 可关闭语言配方导入，让已有自定义编排的老项目只接入排队基础设施。勾某个层会自动带上它在 `layer.toml` 里声明的配套技能（`skills = [...]`），层不会孤零零落地而没有文档：`just` → `write-just`、`bque` → `bque`、`subrepo` → `subrepo-sync`、`worktree` → `worktree`；`audit` 会提示半接入与缺少 justfile 托管块的项目。bque 是 Windows 全局锁，Unix / CI 下同一批配方直跑透传。详见 [`templates/README.md`](templates/README.md)。

**子仓库与工作树**：`subrepo` 层只产出一份机器可读的登记表（`.pengj-templates/subrepo.toml`，托管说明块 + 归项目的 `[[subrepo]]` 条目），技能、配方与体检命令读的都是它，因此三方对「有哪些外部子仓库」永远不会分歧。`worktree` 层把共享编译目录定为**主工作树的 `target/`**，由构建配方注入 `CARGO_TARGET_DIR` 到链接工作树——推导只有一份，配方与 `just wt-check` 读同一个值。共享的只有中间产物：最终二进制在该目录里路径固定，并行工作树会互相覆盖，运行产物的隔离仍归项目自理；同时因为目录在主工作树里，`cargo clean` 会一并清掉所有工作树的中间产物（ADR 0004）。两层都**不**往 `.cargo/config.toml` 里写 `target-dir`：那个文件走结构化合并，重复的表头会让整文件不予写盘。

`create` 的 Rust 选项（仅 rust 层生效）：`--edition 2015|2018|2021|2024`（默认 2024）、`--channel stable|beta|nightly|<版本>`（默认 stable）、`--sccache` 开启编译缓存（默认关）、`--no-lld` 关闭 lld 链接、`--chinese` 开启中文编程（允许中文标识符、关闭相关命名 lint）。选择在生成时固化进 `.pengj-templates/manifest.json`，`update` 同步时按各项目当年的选项重新渲染。

`agent` 层的技能与规范选项（选 agent 层时生效）：`--skills git-workflow,caveman,grill-me,write-docs,template-sync,subrepo-sync,write-a-skill,issue-sync` 决定生成哪些技能（逗号分隔，默认全部）、`--skill-lang zh|en` 决定技能文档书写语言（默认 zh）、`--no-commit-zh` 让提交信息用英文（默认中文）、`--commit-and-push` 要求 Agent 阶段性完工与收尾时自动提交并推送到远端仓库（默认关）。`mcp-bootstrap` 与 `patch-handoff` 是基线技能：只要有 agent 层就始终附带（显式传空技能列表除外），保证项目离开模板仓库后仍能自行安装与校验 MCP、以及验收并合入外部 agent 交付的补丁包。技能生成到 `.agents/skills/<name>/SKILL.md`，目前有 `git-workflow`（约定式提交与分支同步）、`template-sync`（模板同步/纳管）、`subrepo-sync`（子仓库同步与升级）、`bque`（编译排队）、`write-just`（justfile 配方编写，随 `just` 层自动附带）、`caveman`（超压缩通信）、`grill-me` / `grilling`（设计质询）、`write-docs`（通用文档编写与更新，规则由项目自己决定）、`write-a-skill`（新增技能）、`write-just`（编写 justfile 与 `.just` 层配方，保证重型命令接入 bque）、`issue-sync`（把 GitHub Issues 拉成可搬运的 Markdown 包，并把本地 issue 文件回推成真实 issue、自动挂 sub-issue 与 blocked_by）、`patch-handoff`（验收并合入离线补丁交接包：按序 `git am` 落库、修编译、把修复折回引入缺陷的提交、作者归一、强推，把包里带的 issue 条目上传成真实 GitHub issue 并留住补丁↔工单映射，落地后按映射逐条收口，再删掉解包出来的临时内容）、`mcp-bootstrap`（项目内 MCP 安装与握手校验），以及合入的 `engineering`（20 个：`tdd`、`code-review`、`codebase-design`、`diagnosing-bugs`、`domain-modeling`、`grill-with-docs`、`implement`、`implement-spec`、`improve-codebase-architecture`、`pr`、`prototype`、`research`、`retro`、`setup-matt-pocock-skills`、`to-spec`、`to-tickets`、`triage`、`wayfinder`、`wizard`、`ask-matt`）与 `productivity`（`grilling`、`handoff`、`teach`、`to-questionnaire`、`wait-what`、`writing-for-agents`）技能集，新增技能只需在 `templates/agent/.agents/skills/` 下加目录即可（层专属技能也可以放在该层自己目录下，例如 `templates/worktree/.agents/skills/worktree/`，这样只有勾选该层才会下发）；模板删掉某个技能后，存量项目 manifest 里遗留的技能名会在 `update` / `add-layer` 时自动清理（输出 `[清理]` 行），不会长期与磁盘实际技能不一致。三条语言选项互相独立：**中文编程**（代码标识符，`chinese_programming`）、**项目文案语言**（`skill_lang`，决定所有层的注释、报错、日志与技能文档）、**提交信息语言**（`commit_zh`，决定门禁的提交主题语言规则与提交规范文档）；技能脚本恒为英文，无选项。全英文项目须同时传 `--skill-lang en --no-commit-zh`，生成后所有文本文件零汉字（有端到端单测保障）。

**技能扩展**：每个技能 = 托管框架 + 项目专属区。`SKILL.md` 的 `PENGJ_TEMPLATE_START/END` 块内是模板框架（`update` 时原位替换），块外归项目所有、永不覆盖——提交前检查的定义、领域判定表、红线全部写在块外；可执行门禁用什么形式（任意语言脚本 / task runner / 纯清单）由项目自定并在块外声明，框架不预设实现。存量全自定义技能纳管时自动接管：模板整页（含 description）直接覆盖，原正文下移到「纳管过渡区」，双流程由用户合并去重。

### 更新检查（CLI 自身版本）

CLI 自带更新检查：每天最多问一次 GitHub「有没有新版本」，有更新就在命令跑完后提示一行升级命令（Release 里每版都带 `install.ps1` / `install.sh`，重跑安装脚本即完成升级——二进制与模板一起换，不会出现「CLI 新、模板旧」）。

```bash
pengj-templates-cli check-update          # 立即检查（同步联网，退出码不受结果影响）
pengj-templates-cli check-update --json   # 给脚本用：current / latest / updateAvailable
```

- **不拖慢命令**：启动时只读一次本地缓存。真到该联网时，把活派给一个独立的隐藏子进程（`__update-check`）——父进程立刻继续干活；子进程不占终端，父进程退出后它照样跑完，把结果写进 `update-check.json`（Windows `%LOCALAPPDATA%\pengj-templates\`，Unix `${XDG_DATA_HOME:-~/.local/share}/pengj-templates/`），供下一次运行提示。
- **不反复打扰**：联网默认 24 小时一次（`PENGJ_UPDATE_CHECK_INTERVAL=3600` / `12h` / `1d` 可改，`off` 关闭），提示本身也是每天最多一次；发现比上次提示更新的版本时立刻提示。
- **安静**：提示只写 stderr，且只在 stderr 是终端时出现——管道、脚本、AI agent 调用看不到噪音；`gate` 跑在 git 钩子里，一律不检查；`CI` 环境自动跳过。
- **关掉**：`--no-update-check`（全局参数）或 `PENGJ_NO_UPDATE_CHECK=1`。
- **换源**：`PENGJ_UPDATE_API` 把查询地址指到镜像或自建接口（返回 GitHub 同构的 `releases/latest` JSON）。

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
  - 改动 Rust/模板/构建配置 → 跑 `cargo fmt --check`、`clippy -D warnings`、build、`cargo test --workspace`；只改纯文档则整个 job 跳过。
  - **纯文档改动自动跳过**（根目录 `*.md`、`docs/**`、`.agents/**`），不占 CI 额度；同一分支/PR 连续推送会自动取消上一次未完成的运行。
  - 缓存：Rust 编译缓存（rust-cache，含失败缓存）。
- **自动版本号 + Release**（`.github/workflows/release.yml`）：
  - 纯文档改动（与 CI 相同的忽略列表）不触发本流程，只有改动代码/模板/配置时才跑。
  - `release-please` 读 conventional commits 自动 bump semver，多人改版本号到 root `Cargo.toml`，并创建 release PR；合并后打 tag、建 GitHub Release。发布后由 `sync-lockfile` job 自动把新版本号写回 `Cargo.lock` 并提交，无需手动同步。
  - 有 `release` 时自动构建发布 CLI。**产物命名统一带「系统-架构」**：`pengj-templates_<版本>_<os>-<arch>_portable.zip`。
    - 覆盖 Windows x64/arm64、Linux x86_64/arm64、macOS arm64/x86_64（arm 用 GitHub 原生 arm64 runner 构建）。
    - 每个组合一个便携版 zip，内含 `pengj-templates-cli` 可执行文件与随附的 `templates/` 模板目录。
    - 同时产出**固定名**资产 `pengj-templates-cli-<os>-<arch>.zip` / `.tar.gz`，一键安装脚本固定指向它，跨版本命中不会失效。
    - `install.ps1` / `install.sh` 由独立 job 上传到 Release 根目录，`releases/latest/download/install.ps1` 恒定可达。
  - **全自动发布**：`auto-merge` job 在 release-please 创建 Release PR 后用 PAT 自动合并（无需手动），合并即触发上文构建。
  - **beta/preview 分阶段**：release-please 原生支持用 commit footer 指定预发布号并自动递增：
    - 切到某预发布号：正常提交时在其描述加 `Release-As: 0.5.0-beta.1`、`0.5.0-rc.1`、`0.5.0-preview.1` 等；
    - 之后它会在同阶段自动 +1（`beta.1`→`beta.2`），GitHub Release 会自动标为 **Prerelease**；
    - 要回到正式版，再提交 `Release-As: 0.5.0` 即可。
  - **本项目自身的版本号走稳定 semver**：发布的版本号不带 `-beta.N` 后缀。manifest 固定指向上一个已发布 tag，保证 release-please 永远能定位到上次发布；发布稳定版只需在代码/模板/配置类提交的描述里加 `Release-As: <版本>`；需要预发布时用同一个 footer 显式切换即可。
  - 版本号方案（release-please 规则）：`feat`→minor、`fix`→patch、`BREAKING CHANGE`→major（`0.x` 阶段因 `bump-minor-pre-major=true`，BREAKING 只升 minor 而不跳 major；`feat` 在 `0.x` 也仍升 minor）。

`rust` 层的 `src/main.rs` 在 `update_ignore` 黑名单中：仅首次生成时写入，之后归用户所有，模板更新时跳过（不覆盖、不冲突、不删除上报）。若模板里还有这类「种子文件、后续归用户」的文件，在各 `layer.toml` 的 `update_ignore` 里列出即可。注意黑名单冻结的是**值**、不是说明：文件里的 `PENGJ_TEMPLATE_START/END` 受管说明块仍随模板刷新（已有块原位替换；旧版无块文件置顶插入并列入「待复核」），因为文档一旦冻结在用户手里，模板改了语义它也不会跟着改，只会继续按旧说法误导。没有受管块的黑名单文件整份冻结。

`.cargo/config.toml` 走 **TOML 结构化受管合并**：受管块内的表与用户配置做表级并集、键级去重——用户已有的等价键自动去重（避免重复表定义炸掉 cargo），模板新增表进入受管块；同键不同值或需要往用户已有表里补新键时报冲突并跳过写盘，由用户手工对齐后重跑。文本类文件（AGENTS.md、技能 SKILL.md）走**受管块注入**：磁盘已有托管块则原位替换，无块的 legacy 文件则把托管块追加到末尾并计入 `needs_review` 提示人工复核。`layer.toml` 里用 `update_ignore` 声明黑名单文件——`gate` 层的 `.pengj-templates/gate.toml` 就在黑名单里：首次生成后归项目所有，模板更新时跳过，用户的 scope 白名单不会被覆盖（只刷新文件头部的说明块，0.67 起的 scope 三态语义写在那块里）。跨过破坏性变更时，`update` 会把 `crates/core/src/migrations.rs` 里版本区间内的条目打印成「迁移提示」，不需要你去翻 CHANGELOG。

`.vscode/settings.json` 的模板配置在**项目根已有 `*.code-workspace` 文件时并入工作区文件的 `settings` 节点**（不新建独立 settings 文件）；没有工作区文件时才创建/维护 `.vscode/settings.json`。

gate 层用法：门禁是纯 Rust 实现，没有任何 Node 依赖，且已收编进 `pengj-templates-cli`——装一个二进制就同时拿到模板引擎、MCP 安装器和提交门禁。生成项目后先 `git init`，再执行一次 `just doctor --fix`（无 just 时用 `sh .pengj-templates/scripts/doctor.sh --fix` / `pwsh -NoProfile -File .pengj-templates/scripts/doctor.ps1 -Fix`）——它检查工具链、把 `core.hooksPath` 指向 `.githooks`，并确认 CLI 在 PATH 上；CLI 安装器不再写全局钩子。门禁只在 CLI 里实现：没装 CLI 时钩子会拦下提交并打印安装命令，本地没有兜底规则。`gate` 层会自动带上 `ci` 层，服务端跑的是同一个二进制。`core.hooksPath` 存在 `.git/config` 里，不会随克隆传播，所以新克隆后同样跑一次 doctor 即可。

scope 策略：现在省略 `commit.scopes` 或留空列表都会**拒绝一切 `(scope)`**（fail-closed），旧的「空列表表示接受任意 scope」行为已移除。设 `commit.scopes = ["*"]` 放行任意 scope，或填入 crate 名收紧为白名单。本次改动之前生成的项目里写的是 `commit.scopes = []`，升级 CLI 后会开始拒绝带 scope 的标题——在 `.pengj-templates/gate.toml` 里加上 `["*"]` 即可恢复旧行为。

钩子只做一件事：找到 CLI 并把活交给它（`pengj-templates-cli gate hook pre-commit` / `gate hook commit-msg <msgfile>` / `gate hook pre-push`），找不到 CLI 就拦下提交或推送并打印安装命令。`pre-push` 补的是 `pre-commit` 覆盖不到的那条路径：`git am`、`git cherry-pick`、`git rebase` 既不触发 `pre-commit` 也不触发 `commit-msg`，只有 `git commit` 会——批量落补丁因此会一路不带格式与语言检查地走到 CI。推送前的这一层跑 `cargo fmt --check`、整个工作树的语言守卫，以及 `prepush.commands` 里声明的全部命令。模板不带任何兜底脚本，本地与 CI 永远是同一套规则；原先的多级回退探测（`$PENGJ_GATE` 环境变量、独立 `gate` 二进制、仓库 URL 分支）与 shell 兜底脚本均已移除。

### 没装 CLI 时

门禁只有一套实现，规则就在 `pengj-templates-cli` 里，模板不自带第二套脚本。所以没装 CLI 时：

- **本地**：`commit-msg`、`pre-commit` 与 `pre-push` 拦下提交或推送，打印安装命令后退出——不放行，也不用一套更宽松的 shell 规则顶替。
- **服务端**：`ci` 层的 `gate` 任务跑的是同一个二进制，`.github/rulesets/gate.json` 把它设成必需状态检查，PR 一样过不去。CI 失败时把可操作的修复提示写进 `GITHUB_STEP_SUMMARY`，PR 页面直接能看到该怎么改。

钩子激活本身不需要 CLI：`just doctor --fix`（或 `sh .pengj-templates/scripts/doctor.sh --fix`）调用 `.githooks/install.sh`，它只用 git 与 POSIX shell 就把 `core.hooksPath` 指向 `.githooks` 并补上可执行位；`just gate-hooks-local` 是同一个脚本的裸入口。装 CLI：Windows 用 `irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex`，Unix 用 `curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh`。

环境自检本身在 `.pengj-templates/scripts/doctor.sh` / `.pengj-templates/scripts/doctor.ps1`，只检查已选层用到的工具；把 `*.sh`（Unix）/ `*.ps1`（Windows）放进 `.pengj-templates/scripts/doctor.d/` 即可追加项目自己的检查，退出码非 0 记为失败，`--fix` 会设置 `DOCTOR_FIX=1` 让自定义检查自行修复。

ruleset 需要维护者执行一次才会生效：

```bash
gh api -X POST "repos/$(gh repo view --json nameWithOwner -q .nameWithOwner)/rulesets" \
  --input .github/rulesets/gate.json
```

它把 `gate` 设成默认分支的必需状态检查，从此红灯的 PR 合不进去——CI 从「参考」变成「门禁」。

私有仓库 + 免费套餐拿不到这个能力：GitHub 要求 GitHub Pro 才允许启用 ruleset 与分支保护。此时 CI 照样是红的，只是拦不住合并——`.github/rulesets/gate.json` 仍留在仓库里，升级套餐或转公开后一条命令就能生效。

`.pengj-templates/gate.toml` 是门禁的唯一配置面，扁平 TOML 子集：`mode`（`english`/`chinese`，代码语言方向）、`commit.subject_lang`（`zh`/`en`，提交主题语言）、`commit.types`（类型白名单）、`commit.scopes`（scope 白名单，空列表接受任意）、`commit.subject_min_han`（中文主题的最少汉字数）、`commit.max_header`（标题行长度上限）、`lang.max_ratio` / `lang.max_ident_ratio`（代码语言守卫的字符串/标识符违规容忍比例）、`project.disabled`（项目自行关闭的检查）。

代码语言守卫在 pre-commit 阶段扫描暂存的 `.rs`：按 `mode` 判定方向（英文编程拒绝非英文，中文编程拒绝非中文），并按文件计算占比——字符串违规不超过 `lang.max_ratio`（默认 1%）、标识符违规不超过 `lang.max_ident_ratio`（默认 0），额度向下取整。中文模式只判定面向人的文案：命令行参数、路径、协议字段名、内嵌源码/着色器代码等机器字面量不算违规（仍计入分母，避免额度被稀释）。真实例外在源码里认证：行尾 `lang-ok: 理由` 只免该行，文件头 `lang-allow: 理由` 免整个文件（i18n 词表、CJK 测试样本）。全量审计跑 `just gate-check`（或 `pengj-templates-cli gate check`），从仓库根递归扫描 `.rs`，跳过 `target`、`node_modules`、`third-party`、`generated` 与 `submodules`，因此 `backend/`、`launcher/` 这类自定义布局也能覆盖；加 `--list` 列出全部违规、`--json` 输出机器可读报告；`--max-ratio` / `--max-ident-ratio` 可临时放宽阈值，`PENGJ_CODE_LANG=english|chinese` 可覆盖判定方向。

生成项目时写入 `.pengj-templates/manifest.json` manifest（记录层与模板托管文件哈希），`update` 依据它只更新模板托管文件：本地未改动的文件直接覆盖、被改过的报冲突跳过、含受管块的文件做块级合并（TOML 结构化合并 / 文本托管块注入）、模板新增的自动创建。报告中 `needs_review` 列出建议人工复核的合并结果。

## 生成项目的根目录布局

模板产物尽量收在一个目录里，根下只留必须待在根上的东西：

| 根下条目 | 归属 |
|---|---|
| `justfile` | `just` 层托管（just 只认项目根的 `justfile`，搬不走） |
| `.pengj-templates/manifest.json` | 引擎写入的项目清单：层列表、选项与模板托管文件哈希 |
| `.pengj-templates/gate.toml` | `gate` 层产出的门禁配置（列入 `update_ignore`：配置值归项目所有） |
| `.pengj-templates/scripts/doctor.sh` / `.ps1` + `doctor.d/` | `common` 层的环境自检本体与自定义检查扩展点 |
| `.pengj-templates/just/*.just` | `gate` / `rust` / `rust-workspace` / `subrepo` / `worktree` 层产出的配方模块，由根 `justfile` 条件 import |
| `.pengj-templates/subrepo.toml` | `subrepo` 层的机器可读登记表（托管说明块 + 归项目的条目） |
| `.githooks/` | `gate` 层的 git 钩子，`core.hooksPath` 指向它 |

`.pengj-templates/` 下的文件都是模板托管资产，手改等于改模板（`update` 会照模板覆盖，`gate.toml` 的配置值除外）。因此根的 `scripts/`、`just/` 现在完全空出来归项目自己用。

## 目录结构

```
crates/
├── core/         # 引擎：层发现、依赖排序、合并渲染、生成/更新、manifest
├── cli/          # CLI（pengj-templates-cli，内置 gate 子命令）
├── mcp/          # MCP 安装与巡检：配方目录、客户端适配、握手校验
└── gate/         # 提交门禁内部库（pengj-gate，仅由 CLI 调用）
templates/        # 层模板（common/ gate/ rust/ ...），运行时目录加载
tools/            # 一键安装脚本（install.ps1 / install.sh），随 Release 分发
```

## 开发

```bash
just gate-hooks       # 激活提交钩子（每个克隆一次）
just check            # Rust 全部编译检查
just ci               # 格式化检查 + 编译 + clippy + 测试
```

> Linux 上仓库的 `.cargo/config.toml` 把 `linux-gnu` 的链接走 `cc -fuse-ld=lld`，所以需要先装 `lld`（`apt install lld`），否则编译会以 `linker 'lld' not found` 中止。刻意走编译器驱动是必要的：裸 `lld` 不带默认库搜索路径，链接时会找不到 `-lgcc_s` / `-lc`。

### 本仓库的环境构成

本仓库是**纯 Rust 工作区**，没有任何 Node 生态依赖：没有 `package.json`、没有 `pnpm-lock.yaml`、没有前端构建链。

| 目录 | 用途 |
|---|---|
| `crates/core` | 模板引擎：层发现、依赖排序、合并渲染、生成/更新、manifest |
| `crates/cli` | 命令行入口 `pengj-templates-cli`（含 `gate` 子命令） |
| `crates/mcp` | MCP 安装与巡检 |
| `crates/gate` | 提交门禁内部库 `pengj-gate`（零第三方依赖，仅由 CLI 调用） |
| `templates/` | 层模板目录，运行时加载 |

提交门禁由 `crates/gate` 承担（Rust 实现，无 Node 依赖），以内部库形式收编进 CLI，唯一入口 `pengj-templates-cli gate ...`：生成项目用 `just doctor --fix` 检查环境并激活 git 钩子，规则见 `.pengj-templates/gate.toml`；`just gate` 透传任意门禁子命令（`check` / `commit-msg` / `fmt` / `hooks install` / `selftest`）。
