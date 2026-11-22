<!-- PENGJ_TEMPLATE_START -->
# pengj-templates 贡献指南

## 1. 克隆与前置准备

- 克隆仓库：
  ```sh
  git clone https://github.com/pengjiaxusz/pengj-templates.git
  ```
- 确认本机已安装 `pengj-templates-cli` 并加入 `PATH`（默认目录：Linux/macOS 为 `${XDG_DATA_HOME:-$HOME/.local/share}/pengj-templates/bin`，Windows 为 `%LOCALAPPDATA%\pengj-templates\bin`）。

## 2. 环境自检与钩子初始化

运行引导脚本执行本地环境体检（默认只读，永不自动下载或修改工具）：

```sh
# Linux / macOS
sh .pengj-templates/scripts/doctor.sh
# Windows (PowerShell)
pwsh -File .pengj-templates/scripts/doctor.ps1
```

- 本机已安装 `just` 时，也可直接运行 `just doctor`（默认只读体检）。
- 需要激活本地 Git 钩子、创建 Agent 工作区链接或修复本地仓库状态（纯离线）时，再显式追加 `--fix`：
  ```sh
  sh .pengj-templates/scripts/doctor.sh --fix   # 或：just doctor --fix
  ```
- 项目自定义检查与修复脚本可放在 `.pengj-templates/scripts/doctor.d/` 扩展。

## 3. 常用检查与测试

- 运行 `just` 查看所有可用任务配方。
- Rust 工作区校验：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`（或使用项目专属 `just` 配方）。
- Windows 下重型编译/测试通过 `bque run -- <cmd>` 全局排队执行（基准测试用 `bque bench -- <cmd>`）。
- 共享编译目录检查：`just wt-check`（`pengj-templates-cli worktree check --dir .`）。

## 4. 提交规范与本地门禁

- 遵循约定式提交：`type(scope): 中文摘要`。
- 保持提交原子清晰，推送或提 PR 前请完成本地检查与测试。
<!-- PENGJ_TEMPLATE_END -->

<!-- 项目专属贡献说明写在下方（块外，模板更新时 100% 保留）。 -->
