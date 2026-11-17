# {% if options["skill_lang"] | default("zh") == "en" %}just/gate.just — commit gate recipes (managed by the gate layer; template updates overwrite this file){% else %}just/gate.just — 提交门禁配方（gate 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}The commit gate ships inside `pengj-templates-cli`, so every recipe below calls `pengj-templates-cli gate ...`. Install the CLI once and the gate is available.{% else %}提交门禁内置在 `pengj-templates-cli` 里，下面每条配方都直接调 `pengj-templates-cli gate ...`。装一次 CLI，门禁即随之可用。{% endif %}

# {% if options["skill_lang"] | default("zh") == "en" %}Run the code language guard over the whole working tree{% else %}对整个工作树跑代码语言守卫{% endif %}
gate-check:
    @pengj-templates-cli gate check

# {% if options["skill_lang"] | default("zh") == "en" %}Install the commit gate: point core.hooksPath at .githooks and record the executable bit. Machines with the CLI installed already have the global dispatcher enabled (`gate hooks install --global`): git reads the global core.hooksPath on every invocation, so a fresh clone is guarded with no extra step. This recipe is the fallback for when the global dispatcher is not enabled.{% else %}安装提交门禁：把 core.hooksPath 指向 .githooks，并记录可执行位。装了 CLI 的机器默认已开启全局分发（`gate hooks install --global`）：git 每次执行都读全局 core.hooksPath，所以新克隆的仓库无需任何操作即被门禁接管。此配方是未开启全局分发时的兜底。{% endif %}
gate-hooks:
    @pengj-templates-cli gate hooks install

# {% if options["skill_lang"] | default("zh") == "en" %}Run the built-in rule self-test{% else %}运行内置规则自检{% endif %}
gate-selftest:
    @pengj-templates-cli gate selftest

# {% if options["skill_lang"] | default("zh") == "en" %}Environment check: locate the CLI and print the one-line installer when it is missing{% else %}环境自检：定位 CLI，缺失时给出一键安装命令{% endif %}
[unix]
gate-doctor:
    @if command -v pengj-templates-cli >/dev/null 2>&1; then echo "pengj-templates-cli: $(command -v pengj-templates-cli)"; pengj-templates-cli --version; else echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli not found. Install:{% else %}找不到 pengj-templates-cli。安装命令：{% endif %} curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh"; exit 2; fi
[windows]
gate-doctor:
    @if (Get-Command pengj-templates-cli -ErrorAction SilentlyContinue) { Write-Host "pengj-templates-cli: $((Get-Command pengj-templates-cli).Source)"; pengj-templates-cli --version } else { Write-Host "[gate] {% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli not found. Install:{% else %}找不到 pengj-templates-cli。安装命令：{% endif %} irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex"; exit 2 }
