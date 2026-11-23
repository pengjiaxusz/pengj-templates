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

## 2. Environment Check & Setup

- Personal workstation (read-only check; never downloads or modifies tools):
  ```sh
  # Linux / macOS
  sh .pengj-templates/scripts/doctor.sh
  # Windows
  .pengj-templates\scripts\doctor.cmd
  ```
- Fresh VM / CI / automated environment (one-shot setup; add `--dry-run` to preview):
  ```sh
  # Linux / macOS
  sh .pengj-templates/scripts/setup.sh
  # Windows
  .pengj-templates\scripts\setup.cmd
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

## 2. 环境准备与自检

- 个人开发机（默认只读体检，不自动下载或修改工具）：
  ```sh
  # Linux / macOS
  sh .pengj-templates/scripts/doctor.sh
  # Windows
  .pengj-templates\scripts\doctor.cmd
  ```
- 新虚拟机 / CI / 自动化环境（一键装齐依赖并初始化，可先加 `--dry-run` 预览）：
  ```sh
  # Linux / macOS
  sh .pengj-templates/scripts/setup.sh
  # Windows
  .pengj-templates\scripts\setup.cmd
  ```
{% endif -%}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] | default("zh") == "en" -%}
<!-- Project-specific contribution notes go below (outside the block; 100% preserved on template updates). -->
{% else -%}
<!-- 项目专属贡献说明写在下方（块外，模板更新时 100% 保留）。 -->
{% endif %}
