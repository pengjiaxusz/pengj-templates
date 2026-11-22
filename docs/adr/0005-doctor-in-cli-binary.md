# 环境自检与钩子激活收编进 CLI 二进制

将项目环境自检（`doctor`）与提交门禁钩子激活从模板产出的 Shell/PowerShell 脚本收编进 `pengj-templates-cli` 二进制（新增 `pengj-templates-cli doctor` 子命令），不再产出 `.pengj-templates/scripts/doctor.sh`、`.pengj-templates/scripts/doctor.ps1` 与 `.githooks/install.sh`，并退役 `--install-cli` 引导下载参数。

## Status

accepted

## Why

在引入 `gate`、`agent`、`subrepo`、`worktree` 等层之后，任何使用本模板的下游项目在日常开发中都已硬依赖 `pengj-templates-cli`（提交钩子、工作区链接、子仓库体检、共享编译目录推导全部由它执行）。继续在模板里维护两份双语自检脚本暴露出四个问题：

- **双倍实现与持续漂移**：`doctor.sh` 与 `doctor.ps1` 合计超过 660 行，夹杂大量 minijinja 双语条件分支；每新增一个层或检查项，都要在 POSIX shell 与 PowerShell 7 里各写一遍并手动对齐行为与文案。
- **多余的进程往返与信息丢失**：`repo`、`subrepo check`、`worktree check` 本就实现在 CLI 内部，脚本层却不得不反复拉起子进程、把标准输出丢进 `/dev/null`，再靠退出码翻译出一句笼统提示，用户排错时只能再手工重跑子命令。
- **Windows 上无谓的 `sh` 依赖**：`crates/gate` 早已内置完整的 Rust 钩子安装逻辑（含 foreign hook 防覆盖、可选 `pre-push`、索引可执行位与竞品残留告警），但 `doctor.ps1 --fix` 为了复用 `.githooks/install.sh`，反而在 Windows 上多引入了对 `sh` 的依赖。
- **零依赖引导已名存实亡**：此前删掉 `gate-fallback.sh` 后，没装 CLI 时钩子本就会直接拦下提交并打印安装命令；仅为了在 CLI 缺失时提供一条 `--install-cli` 参数，而长期背负两份 300+ 行的双平台模板脚本，得不偿失。

## Considered Options

- **保留极薄引导壳脚本（`doctor.sh` / `doctor.ps1` 仅负责检测 CLI 与 `--install-cli`，其余转发给 CLI）**：保留了无 CLI 时的引导安装入口，但模板仍需维护双平台双语脚本与 `justfile` 的 `[unix]`/`[windows]` 分支，且与 `gate` 层「不产出兜底脚本、缺 CLI 直接引导安装」的既有策略不一致。
- **保留 `.githooks/install.sh` 作为零依赖裸激活入口**：钩子激活与运行拆成两套实现，且激活完没有 CLI 依然无法通过提交门禁，徒增双语脚本维护面。
- **彻底收编进 `pengj-templates-cli doctor`（采纳）**：`common` 层不再产出 `doctor.sh` / `doctor.ps1`，`gate` 层不再产出 `.githooks/install.sh` 与 `just gate-hooks-local`；`justfile` 里的 `doctor` 配方跨平台合并为一条直调命令；所有内置检查在进程内复用核心模块，仅保留 `.pengj-templates/scripts/doctor.d/` 供项目扩展自定义脚本。

## Consequences

- **前置安装成为显式前提**：新机器在首次运行 `just doctor --fix` 之前，必须先执行官方安装命令安装 `pengj-templates-cli`；`--install-cli` 参数随之退役。
- **事实源从「渲染期硬编码」转为「运行时读 manifest」**：`pengj-templates-cli doctor` 以 `.pengj-templates/manifest.json` 为唯一事实源（不依赖 `--templates`），按 `layers` 与 `options.skills` 裁剪检查项；未纳管项目需先 `adopt`。
- **自定义扩展向下兼容**：`.pengj-templates/scripts/doctor.d/` 的脚本目录、排序执行规则与 `DOCTOR_ROOT` / `DOCTOR_FIX=1` 环境变量契约保持不变，存量项目的自定义检查无需改写。
- **破坏性模板变更需走迁移表**：删除 `doctor.sh` / `doctor.ps1` / `.githooks/install.sh` 与 `gate-hooks-local` 会影响直接调用旧脚本路径的下游脚本或 CI，必须按 `feat!` 在迁移表（`MIGRATIONS`）中记录一次清晰的迁移提示；未修改过的旧脚本会在存量项目执行 `update` 时自动清理。
