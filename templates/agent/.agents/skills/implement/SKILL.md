---
name: implement
description: >-
{%- if options["skill_lang"] == "en" %}
  Implement a piece of work based on a spec or set of tickets.
{%- else %}
  依据 spec 或一组工单实现具体开发工作。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Once done, call the Skill tool with "code-review" to review the work.

Commit your work to the current branch.
{%- else -%}
根据用户在 spec 或工单中的描述完成开发实现。

如果用户传入了工单引用，先从 issue tracker 拉取并在动工前说明其标题。若引用存在歧义，先向用户确认。

在预先对齐好的接缝处，尽可能调用 Skill 工具执行 "tdd"。

定期运行类型检查，定期运行单个测试文件，并在收尾时运行一次完整测试套件。

完成后，调用 Skill 工具执行 "code-review" 进行评审。

将你的工作提交到当前分支。
{%- endif %}
