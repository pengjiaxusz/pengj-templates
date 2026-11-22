<!-- PENGJ_TEMPLATE_START -->
# pengj-templates 贡献指南

## 1. 克隆与前置准备

- 克隆仓库（若仓库含子模块，请加 `--recursive` 或执行 `git submodule update --init --recursive`）。
- 确认本机已安装 `pengj-templates-cli` 并加入 `PATH`（默认目录：`${XDG_DATA_HOME:-$HOME/.local/share}/pengj-templates/bin`）。

## 2. 环境自检与钩子初始化

`doctor` 是本仓库的唯一初始化入口（按已选模板层检查工具链、激活本地 Git 钩子、创建 Agent 工作区链接、初始化子模块并准备共享编译目录）：

```sh
just doctor --fix   # 自检并自动修复本地仓库状态（纯离线，不联网）
just doctor         # 仅自检，不修改状态
```
- 本机尚未安装 `just` 时，可直接调用 CLI 回退命令：
  ```sh
  pengj-templates-cli doctor --dir . --fix
  ```
- 项目自定义检查与修复脚本可放在 `.pengj-templates/scripts/doctor.d/` 扩展。

## 3. 常用检查与测试

- 运行 `just` 查看所有可用任务配方。
- 格式化、静态检查与测试：`just fmt-check`、`just check`、`just test`，或一键 `just ci`。
- Windows 下重型编译/测试通过 `bque run -- <cmd>` 全局排队执行（基准测试用 `bque bench -- <cmd>`）。
- 共享编译目录检查：`just wt-check`（`pengj-templates-cli worktree check --dir .`）。

## 4. 提交规范与本地门禁

- 遵循约定式提交：`type(scope): 中文摘要`。
- 保持提交原子清晰，推送或提 PR 前请完成本地检查与测试。
<!-- PENGJ_TEMPLATE_END -->

<!-- 项目专属贡献说明写在下方（块外，模板更新时 100% 保留）。 -->
