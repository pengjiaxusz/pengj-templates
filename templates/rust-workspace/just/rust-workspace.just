# just/rust-workspace.just — Rust 多 crate 工作区常用任务（rust-workspace 层托管）
# 由 justfile 主入口按层条件 import；项目若有自己的编排，请在 manifest 里设
# options["just_recipes"] = "none"，把这些配方留给项目专属区。
{% if "bque" in layers %}
# 全工作区构建（Windows 走全局排队 + Idle 优先级）
[windows]
build *ARGS:
    @bque run -- cargo build --workspace {% raw %}{{ARGS}}{% endraw %}
[unix]
build *ARGS:
    cargo build --workspace {% raw %}{{ARGS}}{% endraw %}

# 全工作区检查（clippy 警告视为错误）
[windows]
check *ARGS:
    @bque run -- cargo clippy --workspace --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}
[unix]
check *ARGS:
    cargo clippy --workspace --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# 全工作区测试
[windows]
test *ARGS:
    @bque run -- cargo test --workspace --all-features {% raw %}{{ARGS}}{% endraw %}
[unix]
test *ARGS:
    cargo test --workspace --all-features {% raw %}{{ARGS}}{% endraw %}

# 专项基准测试（自动挂起后台编译，提供纯净 CPU 环境）
[windows]
bench *ARGS:
    @bque bench -- cargo bench --workspace {% raw %}{{ARGS}}{% endraw %}
[unix]
bench *ARGS:
    cargo bench --workspace {% raw %}{{ARGS}}{% endraw %}
{% else %}
# 全工作区构建
build *ARGS:
    cargo build --workspace {% raw %}{{ARGS}}{% endraw %}

# 全工作区检查
check *ARGS:
    cargo clippy --workspace --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# 全工作区测试
test *ARGS:
    cargo test --workspace --all-features {% raw %}{{ARGS}}{% endraw %}

# 专项基准测试
bench *ARGS:
    cargo bench --workspace {% raw %}{{ARGS}}{% endraw %}
{% endif %}

# 格式化检查（轻量，不排队）
fmt:
    cargo fmt --check

# 一键全检查（CI 用；各子命令各自取锁，bque 原生防重入穿透）
ci: fmt check test
