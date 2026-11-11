# {% if options["skill_lang"] == "en" %}just/rust-workspace.just — common tasks for a multi-crate Rust workspace (managed by the rust-workspace layer){% else %}just/rust-workspace.just — Rust 多 crate 工作区常用任务（rust-workspace 层托管）{% endif %}
# {% if options["skill_lang"] == "en" %}Imported by the root justfile based on the selected layers. If the project has its own{% else %}由 justfile 主入口按层条件 import；项目若有自己的编排，请在 manifest 里设{% endif %}
# {% if options["skill_lang"] == "en" %}orchestration, set options["just_recipes"] = "none" and keep these recipes in the project-owned area.{% else %}options["just_recipes"] = "none"，把这些配方留给项目专属区。{% endif %}
{% if "bque" in layers %}
# {% if options["skill_lang"] == "en" %}Build the whole workspace (Windows: global queue + Idle priority){% else %}全工作区构建（Windows 走全局排队 + Idle 优先级）{% endif %}
[windows]
build *ARGS:
    @bque run -- cargo build --workspace {% raw %}{{ARGS}}{% endraw %}
[unix]
build *ARGS:
    cargo build --workspace {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Check the whole workspace (clippy warnings are errors){% else %}全工作区检查（clippy 警告视为错误）{% endif %}
[windows]
check *ARGS:
    @bque run -- cargo clippy --workspace --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}
[unix]
check *ARGS:
    cargo clippy --workspace --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Test the whole workspace{% else %}全工作区测试{% endif %}
[windows]
test *ARGS:
    @bque run -- cargo test --workspace --all-features {% raw %}{{ARGS}}{% endraw %}
[unix]
test *ARGS:
    cargo test --workspace --all-features {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Benchmark (suspends background builds for a clean CPU environment){% else %}专项基准测试（自动挂起后台编译，提供纯净 CPU 环境）{% endif %}
[windows]
bench *ARGS:
    @bque bench -- cargo bench --workspace {% raw %}{{ARGS}}{% endraw %}
[unix]
bench *ARGS:
    cargo bench --workspace {% raw %}{{ARGS}}{% endraw %}
{% else %}
# {% if options["skill_lang"] == "en" %}Build the whole workspace{% else %}全工作区构建{% endif %}
build *ARGS:
    cargo build --workspace {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Check the whole workspace{% else %}全工作区检查{% endif %}
check *ARGS:
    cargo clippy --workspace --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Test the whole workspace{% else %}全工作区测试{% endif %}
test *ARGS:
    cargo test --workspace --all-features {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Benchmark{% else %}专项基准测试{% endif %}
bench *ARGS:
    cargo bench --workspace {% raw %}{{ARGS}}{% endraw %}
{% endif %}

# {% if options["skill_lang"] == "en" %}Format code (lightweight, no queue){% else %}代码就地格式化（轻量，不排队）{% endif %}
fmt:
    cargo fmt

# {% if options["skill_lang"] == "en" %}Format check (CI only){% else %}格式化检查（CI 校验用，不修改文件）{% endif %}
fmt-check:
    cargo fmt --check

# {% if options["skill_lang"] == "en" %}Full CI check (each sub-command takes the lock separately; bque handles re-entrancy){% else %}一键全检查（CI 用；各子命令各自取锁，bque 原生防重入穿透）{% endif %}
ci: fmt-check check test
