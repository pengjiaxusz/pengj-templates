# PENGJ_TEMPLATE_START
[unix]
set shell := ["sh", "-uc"]
[windows]
set shell := ["pwsh", "-NoProfile", "-Command"]


import 'just/bque.just'





# 列出所有可用任务
default:
    @just --list
# PENGJ_TEMPLATE_END

# ---------------------------------------------------------------------------
# 项目专属区：模板更新只替换上方托管块，本区域归项目所有、完整保留。
# 重型任务统一走 `just q-run <cmd>`，性能跑分走 `just q-bench <cmd>`。
# ---------------------------------------------------------------------------

# justfile for pengj-templates
# 设置默认 shell 为 pwsh 7（本机默认）

# 克隆后一键初始化：安装 pnpm 依赖（自动装 lefthook git 钩子）并验证编译
setup:
    pnpm install
    cargo check --workspace

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

# ---------- 前端 ----------

# 安装前端依赖
frontend-install:
    pnpm --dir crates/app install

# 前端构建（tsc + vite，受 bque 排队管控）
frontend-build:
    @bque run -- pnpm --dir crates/app build

# 前端 dev server（浏览器调试用，纯前端无 Tauri 环境）
frontend-dev:
    pnpm --dir crates/app dev

# ---------- Tauri GUI ----------

# 启动 GUI 开发模式（热更新）
dev:
    pnpm --dir crates/app tauri dev

# 打包 GUI 安装包
tauri-build:
    @bque run -- pnpm --dir crates/app tauri build

# ---------- CLI ----------

# 列出可用层
layers:
    cargo run -q -p pengj-templates-cli -- list-layers

# 生成新项目（例：just create my-app  或  just create my-app frontend,rust  或  just create my-app rust "D:\out"）
create NAME LAYERS="rust" OUTPUT=".":
    cargo run -q -p pengj-templates-cli -- create {{NAME}} --layers {{LAYERS}} --output {{OUTPUT}}

# 同步模板更新到已生成的项目（例：just update ./my-app）
update DIR=".":
    cargo run -q -p pengj-templates-cli -- update --dir {{DIR}}

# MCP 安装与巡检（例：just mcp list   /   just mcp add renderdoc   /   just mcp doctor）
mcp *ARGS:
    cargo run -q -p pengj-templates-cli -- mcp {{ARGS}}

# ---------- 综合 ----------

# CI 全量校验：格式化检查 + 编译 + clippy + 测试 + 前端构建
ci: fmt-check check clippy test frontend-build
