# {% if options["skill_lang"] | default("zh") == "en" %}.pengj-templates/just/gate.just — commit gate recipes (managed by the gate layer; template updates overwrite this file){% else %}.pengj-templates/just/gate.just — 提交门禁配方（gate 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}The commit gate ships inside `pengj-templates-cli`, so the check recipes below call `pengj-templates-cli gate ...`. Activate the hooks with `just doctor --fix` (or `pengj-templates-cli gate hooks install`) — the built-in doctor is the unified activation entry point.{% else %}提交门禁内置在 `pengj-templates-cli` 里，下面的检查配方都直接调 `pengj-templates-cli gate ...`。钩子激活走 `just doctor --fix`（或 `pengj-templates-cli gate hooks install`）——内置 doctor 是统一的激活入口。{% endif %}

# {% if options["skill_lang"] | default("zh") == "en" %}Run the code language guard over the whole working tree{% else %}对整个工作树跑代码语言守卫{% endif %}
gate-check:
    @pengj-templates-cli gate check

# {% if options["skill_lang"] | default("zh") == "en" %}Check generated docs follow the language axis: en projects contain no Han, zh projects' human-facing copy contains Chinese. Acts only on template-managed files from the manifest.{% else %}校验生成项目文案是否跟随语言轴：en 项目零汉字，zh 项目面向人的文案必须含中文。只检查 manifest 记录的模板托管文件。{% endif %}
gate-lang:
    @pengj-templates-cli check-lang --dir .

# {% if options["skill_lang"] | default("zh") == "en" %}Run the built-in rule self-test{% else %}运行内置规则自检{% endif %}
gate-selftest:
    @pengj-templates-cli gate selftest
