{% if options["skill_lang"] == "en" -%}
# Skill mechanics

The skill-specific branch of [`writing-for-agents`](SKILL.md): what changes when the document is a skill (frontmatter, the invocation choice, and router skills). Everything else about writing it is the universal reference in `SKILL.md`.

## Invocation

Two choices, trading the two loads:

- A **model-invoked** skill keeps a `description`, so the agent can fire it autonomously, and other skills can reach it. You can still type its name: model-invocation always _includes_ user reach; a description only ever adds agent discovery, never removes the human's. The description is the skill's top-level context pointer, forced to stay loaded at all times: permanent context load in exchange for discoverability. A model-invoked skill whose content is all reference is also one home for shared reference: another skill can invoke it, so reference needed by several skills lives in one place. Mechanics: omit `disable-model-invocation`, and write a model-facing description carrying the trigger branches (the pointer-writing rules in `SKILL.md` apply in full).
- A **user-invoked** skill strips the description from the agent's reach: only the human typing its name can invoke it, and no other skill can. Zero context load, but it spends cognitive load: you are the index that must remember it exists. Mechanics: set `disable-model-invocation: true`; the `description` becomes human-facing: a one-line summary, trigger lists stripped.

Pick model-invocation only when the agent must reach the skill on its own, or another skill must. If it only ever fires by hand, make it user-invoked and pay no context load.

Shared reference that two user-invoked skills both need can live in neither: with no descriptions, neither can fire the other. Push it to a plain file outside the skill system: external reference any skill can point at.

## Splitting by invocation

The invocation cut of splitting (the sequence cut lives in `SKILL.md`): split off a model-invoked skill when you have a distinct leading word that should trigger it on its own (a trigger word you actually use in your prompts), or another skill must reach it. You pay context load for the new always-loaded description, so that independent reach has to be worth it.

## Router skills

When user-invoked skills multiply past what you can remember, that piled-up cognitive load is cured by a **router skill**: one user-invoked skill that names the others and when to reach for each, so the human has one skill to remember instead of many. It can only hint, never fire them: user-invoked skills have no description, so nothing but the human can reach them.
{%- else -%}
# 技能机制（Skill mechanics）

[`writing-for-agents`](SKILL.md) 中专属于技能（skill）的分支：当一份文档是技能时有哪些特殊之处（frontmatter、调用方式选择与路由技能）。关于编写本身的其余通用原则，全部见 `SKILL.md`。

## 调用方式（Invocation）

两种选择，对应两种负载之间的权衡：

- **模型触发（model-invoked）**的技能保留 `description`，使 Agent 可以自主触发它，其他技能也能调用它。你依然可以手动输入名称触发：模型触发始终*包含*人工触发，`description` 只会增加 Agent 自动发现的能力，绝不会剥夺人工触发入口。`description` 是技能的顶层上下文指针，会强制常驻加载：用常驻上下文负载换取可发现性。如果一个模型触发技能的内容全是参考资料，它同时也是共享参考资料的集中存放点：其他技能可以调用它，多个技能共用的参考信息得以收拢一处。写法：省略 `disable-model-invocation`，编写面向模型的 `description` 并列出触发分支（完全适用 `SKILL.md` 中的指针编写规则）。
- **用户触发（user-invoked）**的技能会将 `description` 从 Agent 的感知范围中移除：只有用户手动输入名称才能调用它，其他技能也无法调用。零上下文负载，但会消耗人的认知负载：你必须自己充当索引去记住它的存在。写法：设置 `disable-model-invocation: true`；此时 `description` 改为面向人类的一句话摘要，去掉触发词列表。

仅当 Agent 必须能自主触达该技能、或需要被其他技能调用时，才选择模型触发。如果它只会由人手动触发，就设为人工触发，省下上下文负载。

如果两个用户触发技能都需要同一份共享参考资料，它不能放在两者任意一个里面：因为两者都没有对模型的 `description`，彼此无法触发对方。请将其抽成技能系统之外的普通文件，作为任何技能都能指向的外部参考。

## 按调用方式拆分

按调用方式的拆分维度（按流程阶段拆分见 `SKILL.md`）：当你有一个应该独立自动触发的明确引导词（你在提示词里确实会用到这个触发词），或者需要被另一个技能调用时，才把它拆成独立的模型触发技能。每新增一个常驻加载的 `description` 都要付出上下文负载，因此这种独立触达必须物有所值。

## 路由技能（Router skills）

当用户触发技能多到你记不住时，积累的认知负载可以通过**路由技能**来化解：创建一个用户触发技能，在里面列出其他技能以及各自的使用时机，这样人只需要记住一个入口技能即可。它只能提示、不能替你自动触发它们：用户触发技能没有对外 `description`，除了人之外没有任何东西能直接调起它们。
{%- endif %}
