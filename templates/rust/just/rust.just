# just/rust.just — Rust 单 crate 常用任务（rust 层托管，模板更新整页覆盖）
# 由 justfile 主入口按层条件 import；项目若有自己的编排，请在 manifest 里设
# options["just_recipes"] = "none"，把这些配方留给项目专属区。
{% if "bque" in layers %}
# 构建（Windows 走全局排队 + Idle 优先级）
[windows]
build *ARGS:
    @bque run -- cargo build {% raw %}{{ARGS}}{% endraw %}
[unix]
build *ARGS:
    cargo build {% raw %}{{ARGS}}{% endraw %}

# 检查（clippy 全开，warnings 视为错误）
[windows]
check *ARGS:
    @bque run -- cargo clippy --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}
[unix]
check *ARGS:
    cargo clippy --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# 测试
[windows]
test *ARGS:
    @bque run -- cargo test --all-features {% raw %}{{ARGS}}{% endraw %}
[unix]
test *ARGS:
    cargo test --all-features {% raw %}{{ARGS}}{% endraw %}

# 专项基准测试（自动挂起后台编译，提供纯净 CPU 环境）
[windows]
bench *ARGS:
    @bque bench -- cargo bench {% raw %}{{ARGS}}{% endraw %}
[unix]
bench *ARGS:
    cargo bench {% raw %}{{ARGS}}{% endraw %}
{% else %}
# 构建
build *ARGS:
    cargo build {% raw %}{{ARGS}}{% endraw %}

# 检查（clippy 全开）
check *ARGS:
    cargo clippy --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# 测试
test *ARGS:
    cargo test --all-features {% raw %}{{ARGS}}{% endraw %}

# 专项基准测试
bench *ARGS:
    cargo bench {% raw %}{{ARGS}}{% endraw %}
{% endif %}

# 格式化检查（轻量，不排队）
fmt:
    cargo fmt --check

# 一键全检查（CI 用；各子命令各自取锁，bque 原生防重入穿透）
ci: fmt check test
