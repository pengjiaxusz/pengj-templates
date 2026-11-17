---
name: grill-with-docs
description: >-
{%- if options["skill_lang"] == "en" %}
  A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary) as we go.
{%- else %}
  通过持续追问打磨计划或设计，并在过程中同步创建或更新文档（ADR 与术语表）。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
Call the Skill tool twice, for "grilling" and "domain-modeling".
{%- else -%}
先后调用两次 Skill 工具：一次调用 "grilling"，另一次调用 "domain-modeling"。
{%- endif %}
