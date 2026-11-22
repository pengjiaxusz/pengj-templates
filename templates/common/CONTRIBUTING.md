<!-- PENGJ_TEMPLATE_START -->
{%- set clone_url = repo_url if repo_url else "<repo-url>" -%}
{%- if options["skill_lang"] | default("zh") == "en" %}
# Contributing to {{ project_name }}

## 1. Clone

```sh
{% if has_submodules -%}
git clone --recursive {{ clone_url }}
{% else -%}
git clone {{ clone_url }}
{% endif -%}
```

## 2. Environment Check

Run the bootstrap script to inspect your local environment (read-only by default; never downloads or modifies tools automatically):

```sh
# Linux / macOS
sh .pengj-templates/scripts/doctor.sh
# Windows
.pengj-templates\scripts\doctor.cmd
```
{% else %}
# {{ project_name }} 贡献指南

## 1. 克隆仓库

```sh
{% if has_submodules -%}
git clone --recursive {{ clone_url }}
{% else -%}
git clone {{ clone_url }}
{% endif -%}
```

## 2. 环境自检

运行引导脚本执行本地环境体检（默认只读，永不自动下载或修改工具）：

```sh
# Linux / macOS
sh .pengj-templates/scripts/doctor.sh
# Windows
.pengj-templates\scripts\doctor.cmd
```
{% endif -%}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] | default("zh") == "en" -%}
<!-- Project-specific contribution notes go below (outside the block; 100% preserved on template updates). -->
{% else -%}
<!-- 项目专属贡献说明写在下方（块外，模板更新时 100% 保留）。 -->
{% endif %}
