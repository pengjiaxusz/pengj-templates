# {% if options["skill_lang"] | default("zh") == "en" %}.pengj-templates/just/subrepo.just — sub-repository recipes (managed by the subrepo layer; template updates overwrite this file){% else %}.pengj-templates/just/subrepo.just — 子仓库配方（subrepo 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}The registry `.pengj-templates/subrepo.toml` is the single source of truth; both recipes below hand over to the CLI so that the skill, the recipes and the health check never disagree about which sub-repositories exist.{% else %}登记表 `.pengj-templates/subrepo.toml` 是唯一事实源；下面两条配方都把活交给 CLI，技能、配方与体检命令因此不会对「有哪些子仓库」产生分歧。{% endif %}

# {% if options["skill_lang"] | default("zh") == "en" %}Render the registry as a human-readable list (name, relative path, dependency mode, upstream branch){% else %}把登记表渲染成人类可读的清单（名称、相对路径、依赖模式、上游跟踪分支）{% endif %}
subrepo-list:
    @pengj-templates-cli subrepo list --dir .

# {% if options["skill_lang"] | default("zh") == "en" %}Health check: nested-workspace red line, submodule readiness, dependency-mode consistency{% else %}体检：嵌套工作区红线、子模块就绪状态、依赖模式自洽{% endif %}
subrepo-check:
    @pengj-templates-cli subrepo check --dir .
