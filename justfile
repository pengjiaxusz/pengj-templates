# PENGJ_TEMPLATE_START
set minimum-version := "1.56.0"
[unix]
set shell := ["sh", "-uc"]
[windows]
set shell := ["pwsh", "-NoProfile", "-Command"]
# 允许项目专属区用同名 recipe 覆盖框架配方（例如 `test` 前需要先产出某个产物），避免去改受管的 import 文件。
set allow-duplicate-recipes := true



import '.pengj-templates/just/worktree.just'




# 列出所有可用任务
default:
    @just --list

# 环境自检：工具链、提交钩子与工作区链接（可用 .pengj-templates/scripts/doctor.d/ 扩展）
doctor *ARGS:
    @pengj-templates-cli doctor --dir . {{ARGS}}
# PENGJ_TEMPLATE_END

# ---------------------------------------------------------------------------
# 项目专属区：模板更新只替换上方托管块，本区域归项目所有、完整保留。
# 重型任务直接走 `bque run -- <cmd>`，性能跑分走 `bque bench -- <cmd>`（bque 层不再产出包装配方）。
# ---------------------------------------------------------------------------

# justfile for pengj-templates

# ---------- 共享编译目录（worktree 层） ----------

# 多工作树复用同一份编译产物：路径由 CLI 按机器推导（主工作树的 target/），
# 配方与 `just wt-check` 读同一个值——推导逻辑只有 CLI 一份，本仓库不另写一套。
# 目标机器未安装 pengj-templates-cli 时，优雅回退到当前 CARGO_TARGET_DIR 或默认 target。
export CARGO_TARGET_DIR := if os_family() == "windows" {
    `$d = if (Get-Command pengj-templates-cli -ErrorAction SilentlyContinue) { pengj-templates-cli worktree target-dir --dir . 2>$null }; if ($LASTEXITCODE -eq 0 -and $d) { $d } elseif ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }`
} else {
    `command -v pengj-templates-cli >/dev/null 2>&1 && pengj-templates-cli worktree target-dir --dir . 2>/dev/null || printf '%s' "${CARGO_TARGET_DIR:-target}"`
}

# ---------- 提交门禁（crates/gate） ----------

# 运行提交门禁：gate <子命令>
# 子命令：audit（代码语言守卫）/ commit-msg <文件> / fmt / hooks install / selftest
[windows]
gate *ARGS:
    @bque run -- cargo run -q -p pengj-templates-cli -- gate {{ARGS}}
[unix]
gate *ARGS:
    cargo run -q -p pengj-templates-cli -- gate {{ARGS}}

# 安装 git 钩子（core.hooksPath=.githooks）
[windows]
gate-hooks:
    @bque run -- cargo run -q -p pengj-templates-cli -- gate hooks install
[unix]
gate-hooks:
    cargo run -q -p pengj-templates-cli -- gate hooks install

# ---------- Rust 构建与检查 ----------

# 格式化 Rust 代码
fmt:
    cargo fmt

# 格式化检查（CI 校验用）
fmt-check:
    cargo fmt --check

# 编译检查（全 workspace；Windows 走 bque 全局排队，Unix 直跑）
[windows]
check:
    @bque run -- cargo check --workspace
[unix]
check:
    cargo check --workspace

# clippy 检查（全 workspace，warnings 视为错误）
[windows]
clippy:
    @bque run -- cargo clippy --workspace --all-targets -- -D warnings
[unix]
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# 运行测试
[windows]
test:
    @bque run -- cargo test --workspace
[unix]
test:
    cargo test --workspace

# 专项基准测试（Windows 下 bque bench 自动挂起后台编译以提供纯净 CPU 环境）
[windows]
bench *ARGS:
    @bque bench -- cargo bench --workspace {{ARGS}}
[unix]
bench *ARGS:
    cargo bench --workspace {{ARGS}}

# 构建所有 Rust crate（支持透传参数，如 `just build -r`）
[windows]
build FLAGS="":
    @bque run -- cargo build {{FLAGS}}
[unix]
build FLAGS="":
    cargo build {{FLAGS}}

# ---------- CLI ----------

# 列出可用层
[windows]
layers:
    @bque run -- cargo run -q -p pengj-templates-cli -- list-layers
[unix]
layers:
    cargo run -q -p pengj-templates-cli -- list-layers

# 生成新项目（例：just create my-app  或  just create my-app common,gate,agent  或  just create my-app rust-workspace "D:\out"）
[windows]
create NAME LAYERS="rust" OUTPUT=".":
    @bque run -- cargo run -q -p pengj-templates-cli -- create {{NAME}} --layers {{LAYERS}} --output {{OUTPUT}}
[unix]
create NAME LAYERS="rust" OUTPUT=".":
    cargo run -q -p pengj-templates-cli -- create {{NAME}} --layers {{LAYERS}} --output {{OUTPUT}}

# 同步模板更新到已生成的项目（例：just update ./my-app）
[windows]
update DIR=".":
    @bque run -- cargo run -q -p pengj-templates-cli -- update --dir {{DIR}}
[unix]
update DIR=".":
    cargo run -q -p pengj-templates-cli -- update --dir {{DIR}}

# 用当前代码升级本机的 CLI 与模板（日常开发替代 install.ps1：拿得到未发布的改动）
[windows]
upgrade:
    @bque run -- cargo run -q -p pengj-templates-cli -- self update --from-source
[unix]
upgrade:
    cargo run -q -p pengj-templates-cli -- self update --from-source

# MCP 安装与巡检（例：just mcp list   /   just mcp add renderdoc   /   just mcp doctor）
[windows]
mcp *ARGS:
    @bque run -- cargo run -q -p pengj-templates-cli -- mcp {{ARGS}}
[unix]
mcp *ARGS:
    cargo run -q -p pengj-templates-cli -- mcp {{ARGS}}

# ---------- 综合 ----------

# CI 全量校验：格式化检查 + 编译 + clippy + 测试
ci: fmt-check check clippy test
