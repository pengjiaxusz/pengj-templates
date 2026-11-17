---
name: research
description: >-
{%- if options["skill_lang"] == "en" %}
  Investigate a question against high-trust primary sources and capture the findings as a Markdown file in the repo. Use when the user wants a topic researched, docs or API facts gathered, or reading legwork delegated to a background agent.
{%- else %}
  对照高可信度的一手来源调研问题，并将结论沉淀为仓库中的 Markdown 文件。当用户要求调研某个主题、收集文档或 API 事实，或将阅读跑腿活委托给后台 Agent 时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
Spin up a **background agent** to do the research, so you keep working while it reads. Start it with a fresh context window.

Its job:

1. Investigate the question against **primary sources** (official docs, source code, specs, first-party APIs), not a secondary write-up of them. Follow every claim back to the source that owns it.
2. Write the findings to a single Markdown file, citing each claim's source.
3. Save it where the repo already keeps such notes; match the existing convention, and if there is none, put it somewhere sensible and say where.
{%- else -%}
拉起一个**后台 Agent** 来执行调研，这样它在阅读查证时你可以继续当前工作。务必以全新上下文窗口启动它。

它的职责：

1. 对照**一手来源**（官方文档、源码、规范标准、一方 API）调研问题，而不是看二手转述文章。把每一条论断都追溯到拥有它的原始来源。
2. 将调研结论写入单个 Markdown 文件，为每条论断附上来源引用。
3. 保存到仓库原本存放此类笔记的位置；遵循现有约定，若暂无约定则放到合适位置并明确告知路径。
{%- endif %}
