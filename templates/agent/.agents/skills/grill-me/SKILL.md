---
name: grill-me
description: >-
{% if options["skill_lang"] == "en" %}
  Continuously interrogate the user about a plan or design until consensus is reached, resolving every branch of the decision tree one by one. Use when the user wants to stress-test a proposal, accept design scrutiny, or mentions "grill me".
{% else %}
  就某个计划或设计对用户进行持续追问，直至达成共识，逐一解决决策树上的每个分支。当用户想要压力测试一个方案、接受设计质询，或提到"grill me"时使用。
{% endif %}
---

{% if options["skill_lang"] == "en" %}
# Grill Me

Keep questioning every aspect of the plan until we reach consensus. Go deep along each branch of the design tree, resolving the dependencies between decisions one by one. Give your recommended answer for every question.

Ask one question at a time.

If a question can be answered by exploring the codebase, explore the codebase instead.
{% else %}
# Grill Me 模式

就这个计划的每个方面对我进行持续追问，直至我们达成共识。沿着设计树的每个分支深入，逐一解决决策之间的依赖关系。对每个问题，给出你的推荐答案。

每次只问一个问题。

如果某个问题可以通过探索代码库来回答，则改为探索代码库。
{% endif %}
