---
name: grill-me
description: >-
{%- if options["skill_lang"] == "en" %}
  A relentless interview to sharpen a plan or design.
{%- else %}
  就某个计划或设计对用户进行持续追问，直至打磨清晰。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
Call the Skill tool with "grilling".
{%- else -%}
使用 "grilling" 调用 Skill 工具。
{%- endif %}
