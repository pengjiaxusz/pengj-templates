# pengj-templates

分层模板生成与同步更新工具：把仓库模板按「层（layer）」组织，勾选所需层即可生成新仓库；模板更新后，一键同步到所有基于它生成的项目。

## 核心思路：分层

模板内容按适用面拆成若干层，生成时自由组合，后选的层覆盖先选层的同名文件：

| 层 | 内容 | 适用 |
| --- | --- | --- |
| `common` | 所有仓库共用：`.gitignore` | 所有仓库 |
| `lefthook` | Git hooks（lefthook）+ commitlint 约定式提交校验 | 需要 Git 钩子的仓库 |
| `agent` | AI 编码助手规范（`AGENTS.md`），按所选层/选项生成 rust、中文编程等约定 | 希望 agent 遵循规范 |
| `vscode` | VS Code 工作区配置：推荐扩展、中文编程下 rust 检查走 clippy | 使用 VS Code 开发 |
| `rust` | Rust 仓库专属：`Cargo.toml`、clippy/rustfmt、sccache、justfile | Rust 项目 |
| ... | 按需扩展（新增目录 + `layer.toml` 即可） | ... |

模板文件支持 minijinja 占位符（`{{ project_name }}`、`{{ project_slug }}`、`{{ year }}`）。

## 技术栈

- **引擎**：Rust（`crates/core`）— 分层合并、渲染、diff 与更新逻辑，模板编译期嵌入二进制
- **GUI**：Tauri 2 + React + Vite + TypeScript + Tailwind v4 + shadcn/ui（`crates/app`）
- **CLI**：clap（`crates/cli`，命令 `pengj`），与 GUI 共用同一套引擎

## 使用

### CLI

```bash
cargo run -p pengj-cli -- list-layers                          # 列出可用层
cargo run -p pengj-cli -- create my-app --layers rust          # 生成 common + rust
cargo run -p pengj-cli -- create my-app --layers rust --edition 2024 --channel nightly --no-sccache --no-lld
cargo run -p pengj-cli -- update --dir ./my-app                # 同步模板更新
```

`create` 的 Rust 选项（仅 rust 层生效）：`--edition 2015|2018|2021|2024`（默认 2021）、`--channel stable|beta|nightly|<版本>`（默认 stable）、`--no-sccache` 关闭编译缓存、`--no-lld` 关闭 lld 链接、`--chinese` 开启中文编程（允许中文标识符、关闭相关命名 lint）。选择在生成时固化进 `.pengj.json`，`update` 同步时按各项目当年的选项重新渲染。

`agent` 层的技能选项（选 agent 层时生效）：`--skill-lang zh|en` 决定技能文档书写语言（默认 zh）、`--no-commit-zh` 让提交信息用英文（默认中文）。技能生成到 `.agents/skills/<name>/SKILL.md`，目前有 `commit` 技能。三个中文概念互相独立：**中文编程**（代码标识符）、**技能用中文写**（文档语言，`skill_lang`）、**提交信息是中文**（提交信息语言，`commit_zh`）。

## CI / Release

- **CI**（`.github/workflows/ci.yml`）：推 `main` 或 PR 时跑 `cargo fmt --check`、`clippy -D warnings`、build、test。
- **自动版本号 + Release**（`.github/workflows/release.yml`）：
  - `release-please` 读 conventional commits 自动 bump semver，多人改版本号到 root `Cargo.toml`、`crates/app` Cargo/package.json、`tauri.conf.json`，并创建 release PR；合并后打 tag、建 GitHub Release。
  - 有 `release` 时自动构建发布：每平台一个**便携版 zip** `pengj-templates_<版本>_<平台>-x64_portable.zip`（含 CLI，命名 `pengj-templates-cli`；Windows 便携版同时含 GUI 可执行文件 + CLI），另附 Windows 安装包 `setup.exe` / `.msi`。Linux/macOS GUI 安装包需额外系统依赖，后续按需加。
  - 版本号方案：`feat`→minor、`fix`→patch、`BREAKING CHANGE`→major；`bump-minor-pre-major=false` 让 `0.x` 阶段 `feat` 也只 +patch。

`rust` 层的 `src/main.rs` 在 `update_ignore` 黑名单中：仅首次生成时写入，之后归用户所有，模板更新时跳过（不覆盖、不冲突、不删除上报）。若模板里还有这类「种子文件、后续归用户」的文件，在各 `layer.toml` 的 `update_ignore` 里列出即可。

`package.json` 走**结构化并集合并**：生成时各层并集；更新时以用户现有文件为底，模板新增的依赖/脚本并入（同名依赖模板优先），用户自己加的库与 name/version 等字段保留不动。模板中依赖用 `latest`，用户 `pnpm install` 时自动拉最新。`layer.toml` 里用 `update_ignore` 声明黑名单文件。

lefthook 层用法：commitlint CLI 走本地依赖（`pnpm exec commitlint`），`pnpm-lock.yaml` 由 git 托管以保持可复现；生成项目时先 `git init` 再 `pnpm install`，`prepare` 自动装 lefthook git 钩子；升级用 `pnpm update --latest`。

### GUI

```bash
pnpm --dir crates/app install
pnpm --dir crates/app tauri dev
```

生成项目时写入 `.pengj.json` manifest（记录层与模板托管文件哈希），`update` 依据它只更新模板托管文件：本地未改动的文件直接覆盖、被改过的报冲突跳过、模板新增的自动创建。

## 目录结构

```
crates/
├── core/         # 引擎：层发现、依赖排序、合并渲染、生成/更新、manifest
├── cli/          # CLI（pengj）
└── app/          # Tauri GUI（src-tauri + React 前端）
templates/        # 层模板（common/ rust/ frontend/ ...），编译期嵌入
```

## 开发

```bash
cargo check --workspace       # Rust 全部编译检查
pnpm --dir crates/app build   # 前端构建
```
