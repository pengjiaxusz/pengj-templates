# 新增独立装机命令（setup）与临时受限环境自动优化

将「联网安装项目所需工具链与环境准备」抽象为独立显式命令 `pengj-templates-cli setup` 及配套的极薄零前置引导脚本（`setup.sh` / `setup.cmd`），与只读/离线语义的「体检（Doctor）」彻底分离；同时基于 CI 环境变量或磁盘可用容量自动识别「临时受限环境」，以用户级或 CI 级配置关闭 Rust 增量编译，避免污染项目仓库或误伤主力开发机。

## Status

accepted

## Why

在云端 Linux 虚拟机（AI 编程沙盒）与 GitHub Actions CI 等干净环境中，每次会话都需要从零准备 `pengj-templates-cli`、`just`、`bque`、`lld`、Rust 工具链、子模块与本地钩子。此前每个仓库的 CI 工作流和每次新开沙盒都不得不各自重复找齐并手动安装这些依赖，但直接把联网安装塞进 `doctor --fix` 又会破坏 [ADR 0005](./0005-doctor-in-cli-binary.md) 确立的「体检默认只读、`--fix` 永不联网」契约。

此外，临时虚拟机往往不携带 `CI=true` 或容器标记，却普遍面临磁盘容量吃紧（例如 20 GB 以下）的问题；Rust 默认开启增量编译会在一次性环境里迅速耗尽磁盘并拖慢冷编译，而主力开发机则需要保留增量编译。

## Considered Options

- **将联网安装并入 `doctor --fix`**：减少一个子命令，但破坏了 `doctor --fix` 离线、可预期、不静默改写本机环境的契约，也让只想激活提交钩子的用户意外触发外网下载。
- **在 `doctor.sh` / `doctor.cmd` 上增加 `--setup` 参数**：不新增脚本文件，但让只读引导脚本混入外网下载逻辑，职责不清且容易被误触发。
- **仅靠 `CI=true` 或容器标记判定临时环境**：无法覆盖不设 `CI` 环境变量且无容器标记的云端 Linux 虚拟机，仍会在小盘沙盒里爆盘。
- **独立 `setup` 命令 + 极薄 `setup.{sh,cmd}` + 磁盘容量兜底识别（采纳）**：`doctor` 永远保持只读与纯离线修复；`setup` 显式表达「允许联网装机」的授权，全程无 `y/n` 阻塞；按 `CI=true`、显式参数或磁盘可用空间阈值自动启用临时受限环境优化，并通过 `$GITHUB_ENV` 或用户级 Cargo 配置持久生效。

## Consequences

- **命令即授权，零交互阻塞**：人类开发者先跑只读 `doctor` 看缺失清单，确认后显式跑 `setup`；CI 工作流与云端 Agent 直接跑 `setup` 一条命令即可幂等装齐全部依赖，不再弹 `y/n` 卡住自动化流程。
- **内置共性 + 扩展槽应对个性**：模板内建处理 `pengj-templates-cli`、`just`、`bque`、`lld`（Linux 探测免密 `sudo -n`，无权限则报错给命令不卡密码）、`pwsh`（Windows）以及 Rust 工具链（按 `rust-toolchain.toml` 对齐 channel/components/targets）和子模块拉齐；少数项目专属依赖留给项目自定义扩展目录（类似 `doctor.d/`）。
- **零仓库污染的临时环境优化**：临时受限环境下仅写入 `$GITHUB_ENV` / `$GITHUB_PATH` 或用户级 Cargo 配置（关闭 `incremental`），绝不修改受版本控制的项目文件，也不影响磁盘充裕的主力开发机。
