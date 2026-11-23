# 跨平台一键装机（setup）与临时受限环境自动优化

> 本 spec 由 `/grill-with-docs` + `/to-spec` 产出。配套文档：[`GLOSSARY.md`](https://github.com/pengjiaxusz/pengj-templates/blob/main/GLOSSARY.md)、[`docs/adr/0006-setup-command-and-ephemeral-env.md`](https://github.com/pengjiaxusz/pengj-templates/blob/main/docs/adr/0006-setup-command-and-ephemeral-env.md)

## 问题陈述（Problem Statement）

当前下游项目在两类高频干净环境——**一次性云端 Linux 虚拟机（AI 编程沙盒）**与 **GitHub Actions CI**——中启动开发或构建时，存在明显的重复摩擦：

- **重复人工/Agent 摸排**：虽然 `doctor` 能检测缺失工具，但新环境仍然需要人工或 Agent 逐一阅读文档并拼装安装命令来补齐 `pengj-templates-cli`、`just`、`lld`、`bque`、Rust 工具链与子模块。
- **CI 工作流与沙盒各自为政**：每个下游仓库的 CI 都要重复维护一套安装步骤，与新电脑、云端沙盒的环境准备割裂。
- **小盘虚拟机爆盘风险**：一次性云端虚拟机往往既没有 `CI=true` 也没有容器标记，但磁盘空间紧张（例如不足 20 GB）；Rust 默认开启增量编译会额外占用大量磁盘并拖慢冷编译，而直接改项目配置文件又会影响主力开发机。

用户希望模板本身提供一套**显式、幂等、无交互阻塞的一键「装机（Setup）」能力**，让云端虚拟机、GitHub CI 与新电脑都能通过同一条命令把项目依赖全部装齐，同时继续守住「体检（Doctor）」默认只读、永不偷偷联网的边界。

## 解决方案（Solution）

在 `pengj-templates` 中引入独立的**装机（Setup）**能力，并配套零前置薄引导脚本与**临时受限环境**自动优化：

- **新增内置子命令 `setup`**：以项目 `.pengj-templates/manifest.json` 为唯一事实源，按已启用层自动计算并幂等安装缺失工具（`just`、`bque`、Linux 下的 `lld`、Windows 下的 `pwsh`、Rust 工具链与 `rust-toolchain.toml` 声明的组件/目标），自动拉齐子模块，最后调用离线 `doctor --fix` 激活提交钩子、工作区链接与共享编译目录。
- **零前置薄入口脚本**：在模板中提供极薄的 `setup.sh`（Unix）与 `setup.cmd`（Windows），仅负责在 `pengj-templates-cli` 缺失时安装 CLI，然后立刻转交 `pengj-templates-cli setup --dir .`；既有的 `doctor.sh` / `doctor.cmd` 继续保持 100% 只读、零下载逻辑。
- **项目级扩展槽**：预留类似 `doctor.d/` 的项目自定义扩展目录（`setup.d/`），用于承载极少数项目专属系统依赖，模板层不内置 CMake/C++ 等当前无需求的工具链。
- **临时受限环境自动优化**：当检测到 `CI=true`、显式开启临时环境参数，或工作目录所在分区磁盘可用空间低于阈值（默认 20 GB）时，自动关闭 Rust 增量编译（CI 写入 `$GITHUB_ENV`，普通 Linux/macOS 虚拟机写入用户级 Cargo 配置，绝不修改项目仓库文件），并在 GitHub Actions 中自动把安装目录写入 `$GITHUB_PATH`。
- **命令即授权，全程零 `y/n` 阻塞**：支持 `--dry-run` 预览完整安装计划（工具、目标路径、大致体积、将写入的环境项）；显式执行 `setup` 即视为授权，不弹交互确认；需要 `sudo` 的系统包先探测免密权限，无权限时优雅报错并给出单行命令而不卡在密码输入。

## 用户故事（User Stories）

1. 作为云端 Linux 沙盒中的开发者或 Agent，我希望克隆任意受管仓库后只运行一条 `setup` 命令，就能自动装齐 CLI、`just`、`lld`、Rust 工具链、子模块与本地 Git 钩子，以便零摸排直接进入开发。
2. 作为 GitHub Actions 工作流维护者，我希望在 CI 里直接复用同一条 `setup` 命令并自动把工具路径注入 `$GITHUB_PATH`、环境变量注入 `$GITHUB_ENV`，以便不再在每个仓库的 workflow 里重复手写安装步骤。
3. 作为主力开发机上的开发者，我希望日常运行 `doctor` 依然是 100% 只读且永不联网，只有在显式运行 `setup` 时才会下载工具，以便我的本机环境绝不被静默改动。
4. 作为在安装前想确认影响的开发者，我希望运行 `setup --dry-run` 时看到清晰的安装计划（将安装哪些工具、安装路径、大致体积、是否触发临时受限环境优化），以便评估后再决定是否真正执行。
5. 作为在已配好环境的机器上误跑或重跑 `setup` 的开发者，我希望所有已存在的工具与配置都被幂等跳过（耗时接近零且不覆盖 CI 预装运行时），以便可以放心重复执行。
6. 作为在磁盘受限（可用空间 `< 20 GB`）但无 `CI=true` 标记的云端虚拟机上编译 Rust 项目的开发者，我希望 `setup` 自动识别为临时受限环境并通过用户级 Cargo 配置关闭增量编译，以便防止爆盘且不污染 `git status`。
7. 作为在磁盘充裕的主力开发机上运行 `setup` 的开发者，我希望系统不会关闭我的 Rust 增量编译，以便保留本机日常迭代的增量编译速度。
8. 作为使用无免密 `sudo` 权限 Linux 机器的开发者，我希望 `setup` 在需要安装 `lld` 等系统包时先检测免密权限，无权限时直接报错并给出明确安装命令而不是卡死在密码提示上。
9. 作为维护含 `rust-toolchain.toml` 项目的开发者，我希望 `setup` 自动确保 `rustup` 就位并同步项目声明的 channel、components（如 `rustfmt`、`clippy`）与 targets，以便克隆后立刻能跑 `cargo fmt`、`clippy` 和交叉检查。
10. 作为拥有极少数项目专属依赖的维护者，我希望能在项目自定义扩展目录（`setup.d/`）中放置脚本由 `setup` 按字典序调用，以便扩展项目特有准备步骤而不污染通用模板。
11. 作为英文项目（`skill_lang = "en"`）的维护者，我希望 `setup` 的安装计划、进度与错误提示遵循项目语言轴（英文零汉字），且 `--json` 输出键名恒为英文，以便国际化协作者和自动化脚本无障碍使用。
12. 作为尚未安装 `pengj-templates-cli` 的新机器用户，我希望运行项目内极薄的 `setup.sh`（Linux/macOS）或 `setup.cmd`（Windows）即可一条命令先装 CLI 再无缝完成全部装机。

## 实现决策（Implementation Decisions）

### 领域模型与命令契约

- 在 CLI 新增顶层子命令 `setup`，参数包括：
  - `--dir <路径>`（默认 `.`）
  - `--dry-run`（仅计算并打印装机计划，不执行任何下载、安装或写入）
  - `--ephemeral` / `--ci`（显式强制按临时受限环境执行）
  - `--no-ephemeral`（显式禁止临时受限环境优化，覆盖自动探测）
  - `--lang <zh|en>`（覆盖输出语言）
  - `--json`（输出结构化计划/执行报告，键名恒为英文）
- `setup` 与 `doctor` 共享同一套基于 manifest 的层与技能解析逻辑，避免两套“该项目需要什么工具”的判定清单产生漂移。
- 全程严格无交互（不读取 stdin `y/n`），退出码语义与 `doctor` 对齐（`0` 表示全部就绪或仅含非阻断告警，`1` 表示存在必需步骤失败）。

### 内置装机项与幂等策略

按 manifest 已启用的层与仓库元数据按序执行（已满足即跳过）：

1. **子模块拉齐**：若仓库存在非空 `.gitmodules` 或启用了 `subrepo` 层且存在未初始化子模块，执行递归子模块初始化。
2. **通用工具安装**（优先用户级免 `sudo` 安装目录，并确保追加到进程 `PATH` 与 `$GITHUB_PATH`）：
   - `just` 层：`just`（以及 Windows 下的 `pwsh`，走 `winget` 或明确指引）。
   - `bque` 层：`bque`（Linux 走官方 `install.sh` / musl 静态包，Windows 走官方 `install.ps1`）。
   - `rust` / `rust-workspace` 层：确保 `rustup` 就位并按项目 `rust-toolchain.toml` 同步所需 channel 与组件；Linux 且启用 `use_lld` 时检测 `ld.lld`，缺失时通过 `sudo -n true` 探测免密权限并按发行版包管理器（`apt-get`/`dnf`/`pacman`）自动安装，无免密权限则记录清晰错误与手动安装命令。
3. **临时受限环境优化**：
   - 判定规则：`--no-ephemeral` 优先关闭；否则满足 `--ephemeral`/`--ci`、存在 `CI=true`/`GITHUB_ACTIONS=true`、或项目所在分区可用磁盘空间 `< 20 GB`（或总容量 `< 40 GB`）即启用。
   - 生效方式：若存在 `$GITHUB_ENV`，追加 `CARGO_INCREMENTAL=0`；否则在启用了 Rust 层时，向用户级 Cargo 全局配置文件合并写入 `[build] incremental = false`（仅修改用户目录，绝不改动仓库内文件）。
4. **本地离线初始化**：进程内调用 `doctor --fix` 完成提交钩子激活、agent 工作区链接与共享编译目录准备。
5. **项目自定义扩展**：若存在 `.pengj-templates/scripts/setup.d/`，按字典序执行匹配平台的扩展脚本（注入 `SETUP_ROOT`、`SETUP_DRY_RUN`、`SETUP_EPHEMERAL` 环境变量）。

### 模板薄入口脚本

- `common` 层新增极薄的 `setup.sh`（POSIX `sh`，可执行位 `100755`）与 `setup.cmd`（`chcp 65001`，CRLF），支持 `skill_lang` 双语。
- 薄脚本只做两件事：若 `pengj-templates-cli` 未安装则调用官方安装脚本装到默认用户目录并加入当前进程 `PATH`，随后透传参数执行 `pengj-templates-cli setup --dir .`。
- `doctor.sh` / `doctor.cmd` 保持完全只读、不引入任何下载代码。

## 测试决策（Testing Decisions）

- **好测试的标准**：只验证外部可观测行为（装机计划内容、临时环境判定结果、幂等跳过、用户级配置合并结果、退出码、双语零混杂与 JSON 契约），不耦合内部私有函数或真实外网下载。
- **唯一核心测试接缝（Seam）**：抽象一个高层的环境探针与执行器接缝（涵盖命令存在性探测、磁盘空间查询、环境变量读取、免密 `sudo` 探测与外部命令执行），单元测试通过内存替身注入各种平台、磁盘余量、已装/未装工具与无 `sudo` 权限场景，在不触发真实外网请求的前提下完整覆盖整条 `setup` 流水线。
- **重点覆盖场景**：
  - 按不同层组合生成准确的 `--dry-run` 计划，已安装工具自动标记跳过。
  - 临时受限环境判定矩阵：`CI=true`、磁盘 `< 20 GB`、磁盘充裕主力机、`--ephemeral` 与 `--no-ephemeral` 显式覆盖。
  - 用户级 Cargo 配置合并幂等性（保留用户已有配置、避免重复写入 `incremental = false`、不触碰项目目录）。
  - 无免密 `sudo` 时 `lld` 安装步骤优雅失败并输出对应平台的手动安装命令。
  - 中英文双语审计（英文模式输出零汉字、JSON 键名恒为英文）与模板生成的 `setup.sh` / `setup.cmd` 语法及双语检查。
- **现有测试先例**：直接复用 `doctor` 模块现有的进程内测试风格、双语审计测试以及模板引擎的生成/更新回归测试框架。

## 范围之外（Out of Scope）

- 不内置 CMake / C++ / Python / Java 等当前仓库未使用工具链的自动安装（如有个别项目需要，通过 `setup.d/` 自行扩展）。
- 不做交互式 `y/n` 询问菜单或 TUI 进度界面。
- 不修改受版本控制的项目级 `.cargo/config.toml` 或 `Cargo.toml` 来关闭增量编译或改动优化级别。
- 不在 `setup` 中内置任何云厂商或特定沙盒平台的私有环境变量硬编码。

## 补充说明（Further Notes）

- 本特性是对 [ADR 0005](./0005-doctor-in-cli-binary.md) 的增量补充而非推翻：`doctor` 仍保持离线只读与本地修复的纯净定位，`setup` 负责联网装机并在末尾复用 `doctor --fix`。
- 下游仓库的 GitHub Actions 工作流后续可逐步简化为直接调用 `sh .pengj-templates/scripts/setup.sh`，减少各仓库 CI 配置重复。
