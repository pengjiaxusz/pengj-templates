{% if options["skill_lang"] == "en" -%}
# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual label strings used in this repo's issue tracker.

| Label in mattpocock/skills | Label in our tracker | Meaning                                  |
| -------------------------- | -------------------- | ---------------------------------------- |
| `needs-triage`             | `needs-triage`       | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info`         | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent`    | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human`    | Requires human implementation            |
| `wontfix`                  | `wontfix`            | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the corresponding label string from this table.

Edit the "Label in our tracker" column to match whatever vocabulary you actually use.
{%- else -%}
# 分诊标签映射（Triage Labels）

本套技能统一使用五个规范分诊角色名。本文件将这五个角色映射到你仓库 issue tracker 实际使用的标签字符串。

| 技能中的规范名 | 本仓库 tracker 中的标签 | 含义 |
| :--- | :--- | :--- |
| `needs-triage` | `needs-triage` | 需要维护者评估此 issue |
| `needs-info` | `needs-info` | 等待提交者补充更多信息 |
| `ready-for-agent` | `ready-for-agent` | 规格已明确，可交由 AFK Agent 执行 |
| `ready-for-human` | `ready-for-human` | 需要人类亲自实现 |
| `wontfix` | `wontfix` | 不予处理 / 关闭 |

当某个技能提到某个角色时（例如“贴上可交由 AFK Agent 处理的分诊标签”），使用本表中对应的标签字符串。

如需匹配你仓库既有的标签命名体系，直接修改“本仓库 tracker 中的标签”这一列即可。
{%- endif %}
