---
name: wait-what
description: >-
{%- if options["skill_lang"] == "en" %}
  Stop. That last message did not land: re-pitch it.
{%- else %}
  停一下。上一条消息没讲清楚：重新用更易懂的方式讲一遍。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
Wait, I don't understand where you've got to here. Re-pitch that: give me a little bit of context, talk in ASD-STE100 Simplified Technical English, and use the ubiquitous language from `GLOSSARY.md` (follow `GLOSSARY-MAP.md` to the right one if the repo has more than one).
{%- else -%}
等等，我没跟上你这里讲到哪了。重新讲一遍：先给我一点上下文，用简明清晰的受控技术语言（相当于 ASD-STE100 简化技术英语的风格）来表达，并统一使用 `GLOSSARY.md` 里的通用语言（如果仓库中有多个术语表，按 `GLOSSARY-MAP.md` 定位对应那份）。
{%- endif %}
