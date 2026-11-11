# {% if options["skill_lang"] == "en" %}just/rust.just — common tasks for a single-crate Rust project (managed by the rust layer){% else %}just/rust.just — Rust 单 crate 常用任务（rust 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] == "en" %}Imported by the root justfile based on the selected layers. If the project has its own{% else %}由 justfile 主入口按层条件 import；项目若有自己的编排，请在 manifest 里设{% endif %}
# {% if options["skill_lang"] == "en" %}orchestration, set options["just_recipes"] = "none" and keep these recipes in the project-owned area.{% else %}options["just_recipes"] = "none"，把这些配方留给项目专属区。{% endif %}
{% if "bque" in layers %}
# {% if options["skill_lang"] == "en" %}Build (Windows: global queue + Idle priority){% else %}构建（Windows 走全局排队 + Idle 优先级）{% endif %}
[windows]
build *ARGS:
    @bque run -- cargo build {% raw %}{{ARGS}}{% endraw %}
[unix]
build *ARGS:
    cargo build {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Check (clippy with all features, warnings are errors){% else %}检查（clippy 全开，warnings 视为错误）{% endif %}
[windows]
check *ARGS:
    @bque run -- cargo clippy --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}
[unix]
check *ARGS:
    cargo clippy --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Test{% else %}测试{% endif %}
[windows]
test *ARGS:
    @bque run -- cargo test --all-features {% raw %}{{ARGS}}{% endraw %}
[unix]
test *ARGS:
    cargo test --all-features {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Benchmark (suspends background builds for a clean CPU environment){% else %}专项基准测试（自动挂起后台编译，提供纯净 CPU 环境）{% endif %}
[windows]
bench *ARGS:
    @bque bench -- cargo bench {% raw %}{{ARGS}}{% endraw %}
[unix]
bench *ARGS:
    cargo bench {% raw %}{{ARGS}}{% endraw %}
{% else %}
# {% if options["skill_lang"] == "en" %}Build{% else %}构建{% endif %}
build *ARGS:
    cargo build {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Check (clippy with all features){% else %}检查（clippy 全开）{% endif %}
check *ARGS:
    cargo clippy --all-targets --all-features -- -D warnings {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Test{% else %}测试{% endif %}
test *ARGS:
    cargo test --all-features {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Benchmark{% else %}专项基准测试{% endif %}
bench *ARGS:
    cargo bench {% raw %}{{ARGS}}{% endraw %}
{% endif %}

# {% if options["skill_lang"] == "en" %}Format code (lightweight, no queue){% else %}代码就地格式化（轻量，不排队）{% endif %}
fmt:
    cargo fmt

# {% if options["skill_lang"] == "en" %}Format check (CI only){% else %}格式化检查（CI 校验用，不修改文件）{% endif %}
fmt-check:
    cargo fmt --check

# {% if options["skill_lang"] == "en" %}Full CI check (each sub-command takes the lock separately; bque handles re-entrancy){% else %}一键全检查（CI 用；各子命令各自取锁，bque 原生防重入穿透）{% endif %}
ci: fmt-check check test
