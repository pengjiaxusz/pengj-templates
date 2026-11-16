# {% if options["skill_lang"] | default("zh") == "en" %}just/gate.just — commit gate recipes (managed by the gate layer; template updates overwrite this file){% else %}just/gate.just — 提交门禁配方（gate 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}The gate binary is resolved the same way the git hooks resolve it: `$PENGJ_GATE` (explicit path), a standalone `gate` on PATH, the `pengj-templates-cli gate` subcommand, or a vendored copy under `crates/gate`.{% else %}门禁二进制的定位方式与 git 钩子一致：`$PENGJ_GATE`（显式路径）、PATH 上的独立 `gate`、`pengj-templates-cli gate` 子命令，或 `crates/gate` 下内置的一份。{% endif %}

# {% if options["skill_lang"] | default("zh") == "en" %}Run the code language guard over the whole working tree{% else %}对整个工作树跑代码语言守卫{% endif %}
[unix]
gate-check:
    @if [ -n "${PENGJ_GATE:-}" ]; then "${PENGJ_GATE}" check; elif command -v gate >/dev/null 2>&1; then gate check; elif command -v pengj-templates-cli >/dev/null 2>&1; then pengj-templates-cli gate check; elif [ -f "crates/gate/Cargo.toml" ]; then cargo run --quiet --manifest-path "crates/gate/Cargo.toml" -- check; else echo "[gate] the gate binary was not found"; exit 2; fi
[windows]
gate-check:
    @if ($env:PENGJ_GATE) { & $env:PENGJ_GATE check } elseif (Get-Command gate -ErrorAction SilentlyContinue) { gate check } elseif (Get-Command pengj-templates-cli -ErrorAction SilentlyContinue) { pengj-templates-cli gate check } elseif (Test-Path "crates/gate/Cargo.toml") { cargo run --quiet --manifest-path "crates/gate/Cargo.toml" -- check } else { Write-Host "[gate] the gate binary was not found"; exit 2 }

# {% if options["skill_lang"] | default("zh") == "en" %}Install the commit gate: point core.hooksPath at .githooks and record the executable bit. Run it once after cloning, because core.hooksPath lives in .git/config and is never cloned.{% else %}安装提交门禁：把 core.hooksPath 指向 .githooks，并记录可执行位。克隆后执行一次即可——core.hooksPath 存在 .git/config 里，不会随克隆传播。{% endif %}
[unix]
gate-hooks:
    @if [ -n "${PENGJ_GATE:-}" ]; then "${PENGJ_GATE}" hooks install; elif command -v gate >/dev/null 2>&1; then gate hooks install; elif command -v pengj-templates-cli >/dev/null 2>&1; then pengj-templates-cli gate hooks install; elif [ -f "crates/gate/Cargo.toml" ]; then cargo run --quiet --manifest-path "crates/gate/Cargo.toml" -- hooks install; else echo "[gate] the gate binary was not found"; exit 2; fi
[windows]
gate-hooks:
    @if ($env:PENGJ_GATE) { & $env:PENGJ_GATE hooks install } elseif (Get-Command gate -ErrorAction SilentlyContinue) { gate hooks install } elseif (Get-Command pengj-templates-cli -ErrorAction SilentlyContinue) { pengj-templates-cli gate hooks install } elseif (Test-Path "crates/gate/Cargo.toml") { cargo run --quiet --manifest-path "crates/gate/Cargo.toml" -- hooks install } else { Write-Host "[gate] the gate binary was not found"; exit 2 }

# {% if options["skill_lang"] | default("zh") == "en" %}Run the built-in rule self-test{% else %}运行内置规则自检{% endif %}
[unix]
gate-selftest:
    @if [ -n "${PENGJ_GATE:-}" ]; then "${PENGJ_GATE}" selftest; elif command -v gate >/dev/null 2>&1; then gate selftest; elif command -v pengj-templates-cli >/dev/null 2>&1; then pengj-templates-cli gate selftest; elif [ -f "crates/gate/Cargo.toml" ]; then cargo run --quiet --manifest-path "crates/gate/Cargo.toml" -- selftest; else echo "[gate] the gate binary was not found"; exit 2; fi
[windows]
gate-selftest:
    @if ($env:PENGJ_GATE) { & $env:PENGJ_GATE selftest } elseif (Get-Command gate -ErrorAction SilentlyContinue) { gate selftest } elseif (Get-Command pengj-templates-cli -ErrorAction SilentlyContinue) { pengj-templates-cli gate selftest } elseif (Test-Path "crates/gate/Cargo.toml") { cargo run --quiet --manifest-path "crates/gate/Cargo.toml" -- selftest } else { Write-Host "[gate] the gate binary was not found"; exit 2 }
