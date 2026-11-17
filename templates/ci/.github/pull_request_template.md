<!-- {% if options["skill_lang"] | default("zh") == "en" %}Pull request checklist. Delete the sections that do not apply.{% else %}拉取请求自查清单。用不上的小节可以删掉。{% endif %} -->

## {% if options["skill_lang"] | default("zh") == "en" %}What changed{% else %}改了什么{% endif %}

## {% if options["skill_lang"] | default("zh") == "en" %}Checklist{% else %}自查清单{% endif %}

- [ ] {% if options["skill_lang"] | default("zh") == "en" %}Every commit subject follows `type(scope): subject`.{% else %}每个提交标题都写成 `type(scope): 摘要`。{% endif %}
{%- if options["commit_zh"] | default(false) %}
- [ ] {% if options["skill_lang"] | default("zh") == "en" %}Every commit subject is written in Chinese (at least 2 Han characters); the type and the scope stay English.{% else %}每个提交标题都用中文（至少 2 个汉字），type 与 scope 保留英文。{% endif %}
{%- else %}
- [ ] {% if options["skill_lang"] | default("zh") == "en" %}Every commit subject is written in English; no Han characters in it.{% else %}每个提交标题都用英文，不含汉字。{% endif %}
{%- endif %}
- [ ] {% if options["skill_lang"] | default("zh") == "en" %}The `gate` check is green.{% else %}`gate` 这项检查是绿的。{% endif %}

{% if options["skill_lang"] | default("zh") == "en" %}
> The check runs on the server, so it needs nothing from your machine. To catch the same problems before pushing, enable the local gate once: `sh .githooks/install.sh`.
{% else %}
> 这项检查跑在服务端，不依赖你的机器。想在推送前就发现同样的问题，本地门禁启用一次即可：`sh .githooks/install.sh`。
{% endif %}
