{% if options["skill_lang"] == "en" %}{% raw %}{{FRONT_MATTER}}
# {{TITLE}}

{{META_TABLE}}

---

{{BODY}}

{{COMMENTS}}
{% endraw %}{% else %}{% raw %}{{FRONT_MATTER}}
# {{TITLE}}

{{META_TABLE}}

<!-- 版式由 issue-sync 的 templates/issue.md 驱动，可用 -Templates 覆盖 -->

---

{{BODY}}

{{COMMENTS}}
{% endraw %}{% endif %}
