<!-- PENGJ_TEMPLATE_START -->
{%- if options["skill_lang"] | default("zh") == "en" %}
# Contributing to {{ project_name }}

## 1. Clone & Prerequisites

{% if 'subrepo' in layers -%}
- Clone with submodules initialized:
  ```sh
  git clone --recursive <repo-url>
  # If already cloned:
  git submodule update --init --recursive
  ```
- Sub-repositories are tracked in `.pengj-templates/subrepo.toml` (single source of truth).
{% else -%}
- Clone the repository (add `--recursive` or run `git submodule update --init --recursive` if submodules are used).
{% endif -%}
- Ensure `pengj-templates-cli` is installed and on your `PATH` (default user bin: `${XDG_DATA_HOME:-$HOME/.local/share}/pengj-templates/bin`).

## 2. Environment Check & Hook Setup

`doctor` is the single initialization entry point for this repository (checks required toolchains, activates local Git hooks, creates agent workspace links, initializes submodules, and prepares shared build directories according to enabled layers):

{% if 'just' in layers -%}
```sh
just doctor --fix   # inspect & auto-fix local repo state (offline)
just doctor         # inspect only
```
- Without `just` installed yet, run the CLI directly:
  ```sh
  pengj-templates-cli doctor --dir . --fix
  ```
{% if 'gate' in layers -%}
  - To activate only `.githooks`: `pengj-templates-cli gate hooks install` (sets `core.hooksPath=.githooks`).
{% endif -%}
{% else -%}
```sh
pengj-templates-cli doctor --dir . --fix   # inspect & auto-fix local repo state (offline)
pengj-templates-cli doctor --dir .         # inspect only
```
{% if 'gate' in layers -%}
- To activate only `.githooks`: `pengj-templates-cli gate hooks install` (sets `core.hooksPath=.githooks`).
{% endif -%}
{% endif -%}
- Project-specific checks can be extended under `.pengj-templates/scripts/doctor.d/`.

## 3. Common Checks & Tests

{% if 'just' in layers -%}
- Run `just` to list all available recipes.
{% if ('rust' in layers or 'rust-workspace' in layers) and options["just_recipes"] | default("") != "none" -%}
- Format, lint, and test: `just fmt-check`, `just check`, `just test`, or all-in-one `just ci`.
{% elif 'rust-workspace' in layers -%}
- Rust workspace checks: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (or the project's `just` recipes).
{% elif 'rust' in layers -%}
- Rust checks: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (or the project's `just` recipes).
{% endif -%}
{% if 'bque' in layers -%}
- Heavy builds/tests on Windows are queued via `bque run -- <cmd>` (benchmarks via `bque bench -- <cmd>`).
{% endif -%}
{% if 'gate' in layers -%}
- Gate & language checks: `just gate-check` (`pengj-templates-cli gate check`) and `just gate-lang` (`pengj-templates-cli check-lang --dir .`).
{% endif -%}
{% if 'subrepo' in layers -%}
- Sub-repository checks: `just subrepo-list` and `just subrepo-check` (`pengj-templates-cli subrepo check --dir .`).
{% endif -%}
{% if 'worktree' in layers -%}
- Shared target dir check: `just wt-check` (`pengj-templates-cli worktree check --dir .`).
{% endif -%}
{% else -%}
{% if 'rust-workspace' in layers -%}
- Rust workspace checks: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
{% elif 'rust' in layers -%}
- Rust checks: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
{% endif -%}
{% if 'bque' in layers -%}
- Heavy builds/tests on Windows should run through `bque run -- <cmd>` (benchmarks via `bque bench -- <cmd>`).
{% endif -%}
{% if 'gate' in layers -%}
- Gate & language checks: `pengj-templates-cli gate check` and `pengj-templates-cli check-lang --dir .`.
{% endif -%}
{% if 'subrepo' in layers -%}
- Sub-repository checks: `pengj-templates-cli subrepo list --dir .` and `pengj-templates-cli subrepo check --dir .`.
{% endif -%}
{% if 'worktree' in layers -%}
- Shared target dir check: `pengj-templates-cli worktree check --dir .`.
{% endif -%}
{% if 'rust' not in layers and 'rust-workspace' not in layers and 'gate' not in layers and 'subrepo' not in layers and 'worktree' not in layers -%}
- Run the project's test and lint suite before opening a PR (see project-specific notes below).
{% endif -%}
{% endif %}
## 4. Commit Conventions & Local Gates

- Use Conventional Commits: `type(scope): subject`{% if options["commit_zh"] | default(false) %} (subject in Chinese per `.pengj-templates/gate.toml`){% endif %}.
{% if 'gate' in layers -%}
- Local Git hooks in `.githooks/` (`commit-msg`, `pre-commit`, `pre-push`) are activated by `doctor --fix` and enforce rules configured in `.pengj-templates/gate.toml` (matching CI).
- Before pushing or opening a PR, verify commit history and gates:
  ```sh
  pengj-templates-cli gate check
  pengj-templates-cli check-lang --dir .
  pengj-templates-cli gate check-commits --base origin/main
  ```
{% else -%}
- Keep commits focused and run local verification checks before opening a PR.
{% endif -%}
{% else %}
# {{ project_name }} 贡献指南

## 1. 克隆与前置准备

{% if 'subrepo' in layers -%}
- 首次克隆请连同子模块一起拉取：
  ```sh
  git clone --recursive <repo-url>
  # 若已克隆但未拉取子模块：
  git submodule update --init --recursive
  ```
- 子仓库清单由 `.pengj-templates/subrepo.toml` 统一维护（唯一事实源）。
{% else -%}
- 克隆仓库（若仓库含子模块，请加 `--recursive` 或执行 `git submodule update --init --recursive`）。
{% endif -%}
- 确认本机已安装 `pengj-templates-cli` 并加入 `PATH`（默认目录：`${XDG_DATA_HOME:-$HOME/.local/share}/pengj-templates/bin`）。

## 2. 环境自检与钩子初始化

`doctor` 是本仓库的唯一初始化入口（按已选模板层检查工具链、激活本地 Git 钩子、创建 Agent 工作区链接、初始化子模块并准备共享编译目录）：

{% if 'just' in layers -%}
```sh
just doctor --fix   # 自检并自动修复本地仓库状态（纯离线，不联网）
just doctor         # 仅自检，不修改状态
```
- 本机尚未安装 `just` 时，可直接调用 CLI 回退命令：
  ```sh
  pengj-templates-cli doctor --dir . --fix
  ```
{% if 'gate' in layers -%}
  - 仅激活 `.githooks` 时也可执行：`pengj-templates-cli gate hooks install`（写入 `core.hooksPath=.githooks`）。
{% endif -%}
{% else -%}
```sh
pengj-templates-cli doctor --dir . --fix   # 自检并自动修复本地仓库状态（纯离线，不联网）
pengj-templates-cli doctor --dir .         # 仅自检，不修改状态
```
{% if 'gate' in layers -%}
- 仅激活 `.githooks` 时也可执行：`pengj-templates-cli gate hooks install`（写入 `core.hooksPath=.githooks`）。
{% endif -%}
{% endif -%}
- 项目自定义检查与修复脚本可放在 `.pengj-templates/scripts/doctor.d/` 扩展。

## 3. 常用检查与测试

{% if 'just' in layers -%}
- 运行 `just` 查看所有可用任务配方。
{% if ('rust' in layers or 'rust-workspace' in layers) and options["just_recipes"] | default("") != "none" -%}
- 格式化、静态检查与测试：`just fmt-check`、`just check`、`just test`，或一键 `just ci`。
{% elif 'rust-workspace' in layers -%}
- Rust 工作区校验：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`（或使用项目专属 `just` 配方）。
{% elif 'rust' in layers -%}
- Rust 校验：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（或使用项目专属 `just` 配方）。
{% endif -%}
{% if 'bque' in layers -%}
- Windows 下重型编译/测试通过 `bque run -- <cmd>` 全局排队执行（基准测试用 `bque bench -- <cmd>`）。
{% endif -%}
{% if 'gate' in layers -%}
- 门禁与双语审计：`just gate-check`（`pengj-templates-cli gate check`）、`just gate-lang`（`pengj-templates-cli check-lang --dir .`）。
{% endif -%}
{% if 'subrepo' in layers -%}
- 子仓库检查：`just subrepo-list`、`just subrepo-check`（`pengj-templates-cli subrepo check --dir .`）。
{% endif -%}
{% if 'worktree' in layers -%}
- 共享编译目录检查：`just wt-check`（`pengj-templates-cli worktree check --dir .`）。
{% endif -%}
{% else -%}
{% if 'rust-workspace' in layers -%}
- Rust 工作区校验：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`。
{% elif 'rust' in layers -%}
- Rust 校验：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
{% endif -%}
{% if 'bque' in layers -%}
- Windows 下重型编译/测试请通过 `bque run -- <cmd>` 排队执行（基准测试用 `bque bench -- <cmd>`）。
{% endif -%}
{% if 'gate' in layers -%}
- 门禁与双语审计：`pengj-templates-cli gate check`、`pengj-templates-cli check-lang --dir .`。
{% endif -%}
{% if 'subrepo' in layers -%}
- 子仓库检查：`pengj-templates-cli subrepo list --dir .`、`pengj-templates-cli subrepo check --dir .`。
{% endif -%}
{% if 'worktree' in layers -%}
- 共享编译目录检查：`pengj-templates-cli worktree check --dir .`。
{% endif -%}
{% if 'rust' not in layers and 'rust-workspace' not in layers and 'gate' not in layers and 'subrepo' not in layers and 'worktree' not in layers -%}
- 提交 PR 前请先运行项目自有的检查与测试套件（见下方项目专属说明）。
{% endif -%}
{% endif %}
## 4. 提交规范与本地门禁

- 遵循约定式提交：`type(scope): {% if options["commit_zh"] | default(true) %}中文摘要{% else %}English subject{% endif %}`。
{% if 'gate' in layers -%}
- 本地 Git 钩子位于 `.githooks/`（`commit-msg`、`pre-commit`、`pre-push`），由 `doctor --fix` 激活，规则见 `.pengj-templates/gate.toml`（与 CI 一致）。
- 推送或提 PR 前建议执行全量门禁校验：
  ```sh
  pengj-templates-cli gate check
  pengj-templates-cli check-lang --dir .
  pengj-templates-cli gate check-commits --base origin/main
  ```
{% else -%}
- 保持提交原子清晰，推送或提 PR 前请完成本地检查与测试。
{% endif -%}
{% endif -%}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] | default("zh") == "en" -%}
<!-- Project-specific contribution notes go below (outside the block; 100% preserved on template updates). -->
{% else -%}
<!-- 项目专属贡献说明写在下方（块外，模板更新时 100% 保留）。 -->
{% endif %}
