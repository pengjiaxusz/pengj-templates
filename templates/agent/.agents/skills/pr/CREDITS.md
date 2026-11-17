{% if options["skill_lang"] == "en" -%}
# Credits

The **Summary** section's menu of visuals (pseudocode, call trees, component trees, file trees, Mermaid, diffs) and its placement guidance come from [Dex Horthy](https://github.com/dexhorthy)'s [`show-me`](https://github.com/humanlayer/humanlayer) skill, reproduced almost word for word and aimed at a diff instead of a live conversation. `pr` does not depend on `show-me` as a skill (it isn't part of this repo, and a hard dependency would break standalone installs), so the content is copied in rather than pointed at; this file is the attribution a dependency would otherwise have carried.
{%- else -%}
# 致谢（Credits）

**Summary** 小节的可视化表达菜单（伪代码、调用树、组件树、文件树、Mermaid、diff）及其放置指南源自 [Dex Horthy](https://github.com/dexhorthy) 的 [`show-me`](https://github.com/humanlayer/humanlayer) 技能，几乎逐字复用并针对 diff 场景（而非实时对话）做了调整。`pr` 并不把 `show-me` 声明为技能依赖（它不在本仓库中，硬依赖会破坏独立安装），而是直接将内容纳入其中；本文件用于保留原本应由依赖关系承载的署名。
{%- endif %}
