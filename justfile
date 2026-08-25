# justfile for pengj-templates
# 设置默认 shell 为 pwsh 7（本机默认）
set shell := ["pwsh", "-c"]

# 克隆后一键初始化：安装 pnpm 依赖（自动装 lefthook git 钩子）并验证编译
setup:
    pnpm install
    cargo check --workspace

# 默认执行任务：格式化 + 全量编译检查
default: fmt check

# 格式化 Rust 代码
fmt:
    cargo fmt

# 编译检查（全 workspace）
check:
    cargo check --workspace

# clippy 检查（全 workspace，warnings 视为错误）
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# 运行测试
test:
    cargo test --workspace

# 构建所有 Rust crate（支持透传参数，如 `just build -r`）
build FLAGS="":
    cargo build {{FLAGS}}

alias b := build

# ---------- 前端 ----------

# 安装前端依赖
frontend-install:
    pnpm --dir crates/app install

# 前端构建（tsc + vite）
frontend-build:
    pnpm --dir crates/app build

# 前端 dev server（浏览器调试用，纯前端无 Tauri 环境）
frontend-dev:
    pnpm --dir crates/app dev

# ---------- Tauri GUI ----------

# 启动 GUI 开发模式（热更新）
dev:
    pnpm --dir crates/app tauri dev

# 打包 GUI 安装包
tauri-build:
    pnpm --dir crates/app tauri build

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

# ---------- 综合 ----------

# CI 全量校验：格式化检查 + 编译 + clippy + 测试 + 前端构建
ci: fmt check clippy test frontend-build
