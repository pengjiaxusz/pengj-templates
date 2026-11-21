{% if options["skill_lang"] == "en" %}{% raw %}## Dependency Topology

Tree follows parent-child nesting only; cross-branch `blocked_by` edges are listed on each issue.

{{TREE}}

> {{NOTE}}
{% endraw %}{% else %}{% raw %}## 依赖拓扑

树只表达父子归属；跨分支的 `blocked_by` 边在各 issue 的元数据里完整列出。

{{TREE}}

> {{NOTE}}
{% endraw %}{% endif %}
