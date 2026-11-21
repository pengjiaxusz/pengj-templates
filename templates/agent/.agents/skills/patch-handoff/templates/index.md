{% if options["skill_lang"] == "en" %}{% raw %}# {{REPO}} · Issue Export

- Scope: {{COUNT}} issues ({{RANGE}})
- Generated: {{GENERATED_AT}}
- Each issue is one Markdown file with front matter metadata, full body, and comments
- Layout is template-driven: replace this file and pass `-Templates <dir>` to restyle

## Index

{{TABLE}}

{{TOPOLOGY}}

## Notes

{{NOTES}}
{% endraw %}{% else %}{% raw %}# {{REPO}} · Issue 导出

- 范围：{{COUNT}} 个 issue（{{RANGE}}）
- 生成时间：{{GENERATED_AT}}
- 每个 issue 一份 Markdown：front matter 元数据 + 完整正文 + 评论
- 版式由模板驱动：替换本文件并用 `-Templates <dir>` 指定即可换版式

## 清单

{{TABLE}}

{{TOPOLOGY}}

## 说明

{{NOTES}}
{% endraw %}{% endif %}
