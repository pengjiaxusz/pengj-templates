# PENGJ_TEMPLATE_START
[unix]
set shell := ["sh", "-uc"]
[windows]
set shell := ["pwsh", "-NoProfile", "-Command"]
# 允许项目专属区用同名 recipe 覆盖框架配方（例如 `test` 前需要先产出某个产物），避免去改受管的 import 文件。
set allow-duplicate-recipes := true


import '.pengj-templates/just/bque.just'






# 列出所有可用任务
default:
    @just --list

# 环境自检：工具链、提交钩子与工作区链接（可用 .pengj-templates/scripts/doctor.d/ 扩展）
[unix]
doctor *ARGS:
    @sh .pengj-templates/scripts/doctor.sh {{ARGS}}
# 环境自检：工具链、提交钩子与工作区链接（可用 .pengj-templates/scripts/doctor.d/ 扩展）
[windows]
doctor *ARGS:
    @pwsh -NoProfile -File .pengj-templates/scripts/doctor.ps1 {{ARGS}}
# PENGJ_TEMPLATE_END

# ---------------------------------------------------------------------------
# 项目专属区：模板更新只替换上方托管块，本区域归项目所有、完整保留。
# 重型任务统一走 `just q-run <cmd>`，性能跑分走 `just q-bench <cmd>`。
# ---------------------------------------------------------------------------

# justfile for pengj-templates
# 设置默认 shell 为 pwsh 7（本机默认）

# ---------- 提交门禁（crates/gate） ----------

# 运行提交门禁：gate <子命令>
# 子命令：audit（代码语言守卫）/ commit-msg <文件> / fmt / hooks install / selftest
gate *ARGS:
    cargo run -q -p pengj-templates-cli -- gate {{ARGS}}

# 安装 git 钩子（core.hooksPath=.githooks）
gate-hooks:
    cargo run -q -p pengj-templates-cli -- gate hooks install

# ---------- Rust 构建与检查 ----------

# 格式化 Rust 代码
fmt:
    cargo fmt

# 格式化检查（CI 校验用）
fmt-check:
    cargo fmt --check

# 编译检查（全 workspace，受 bque 全局排队与优先级管控）
check:
    @bque run -- cargo check --workspace

# clippy 检查（全 workspace，warnings 视为错误）
clippy:
    @bque run -- cargo clippy --workspace --all-targets -- -D warnings

# 运行测试
test:
    @bque run -- cargo test --workspace

# 专项基准测试（bque bench 自动挂起后台编译以提供纯净 CPU 环境）
bench *ARGS:
    @bque bench -- cargo bench --workspace {{ARGS}}

# 构建所有 Rust crate（支持透传参数，如 `just build -r`）
build FLAGS="":
    @bque run -- cargo build {{FLAGS}}

# ---------- CLI ----------

# 列出可用层
layers:
    cargo run -q -p pengj-templates-cli -- list-layers

# 生成新项目（例：just create my-app  或  just create my-app common,gate,agent  或  just create my-app rust-workspace "D:\out"）
create NAME LAYERS="rust" OUTPUT=".":
    cargo run -q -p pengj-templates-cli -- create {{NAME}} --layers {{LAYERS}} --output {{OUTPUT}}

# 同步模板更新到已生成的项目（例：just update ./my-app）
update DIR=".":
    cargo run -q -p pengj-templates-cli -- update --dir {{DIR}}

# MCP 安装与巡检（例：just mcp list   /   just mcp add renderdoc   /   just mcp doctor）
mcp *ARGS:
    cargo run -q -p pengj-templates-cli -- mcp {{ARGS}}

# ---------- 综合 ----------

# CI 全量校验：格式化检查 + 编译 + clippy + 测试
ci: fmt-check check clippy test
