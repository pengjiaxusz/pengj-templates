{% if options["skill_lang"] == "en" -%}
# GLOSSARY.md Format

`GLOSSARY.md` is the canonical language for this teaching workspace. All explainers, exercises, and learning records should adhere to its terminology. Building it is itself part of learning: compressing a concept into a tight definition is evidence the user understands it.

## Structure

```md
# {Topic} Glossary

{One or two sentence description of the topic this glossary covers.}

## Terms

**Hypertrophy**:
Muscle growth driven by mechanical tension and metabolic stress over repeated training sessions.
_Avoid_: Bulking, getting big

**Progressive overload**:
Systematically increasing the demand on a muscle over time, via load, volume, or intensity.
_Avoid_: Pushing harder, levelling up

**RPE (Rate of Perceived Exertion)**:
A 1–10 self-rating of how hard a set felt, where 10 is failure and 8 means two reps left in the tank.
_Avoid_: Effort score, intensity rating
```

## Rules

- **Add a term only when the user understands it.** The glossary is a record of compressed knowledge, not a dictionary the user reads to learn. If the user has just been introduced to a concept, wait until they can use it correctly before promoting it here.
- **Be opinionated.** When several words exist for the same concept, pick the best one and list the rest as aliases to avoid. This is how language compresses.
- **Keep definitions tight.** One or two sentences. Define what the term IS, not what it does or how to do it.
- **Use the glossary's own terms inside definitions.** Once a term is in the glossary, prefer it everywhere, including inside other definitions. This is what makes complex terms easier to grasp later.
- **Group under subheadings** when natural clusters emerge (e.g. `## Anatomy`, `## Programming`). A flat list is fine when terms cohere.
- **Flag ambiguities explicitly.** If a term is used loosely in the wider field, note the resolution: "In this workspace, 'set' always means a working set; warm-ups are tracked separately."
- **Revise as understanding deepens.** A definition the user wrote in week one may be wrong by week six. Update in place; do not leave stale entries.
{%- else -%}
# GLOSSARY.md 格式规范

`GLOSSARY.md` 是当前教学工作区的规范术语表。所有讲解、练习与学习记录都应遵循其中的术语。构建术语表本身就是学习的一部分：把概念压缩成紧凑的定义，正是用户真正理解它的证明。

## 结构

```md
# {主题} 术语表

{用一两句话说明本术语表涵盖的主题范围。}

## 术语

**肌肥大（Hypertrophy）**：
在反复训练刺激下，由机械张力与代谢压力驱动的肌肉生长。
_避免使用_：增肌变大、练壮

**渐进超负荷（Progressive overload）**：
随时间推移，通过负荷、训练量或强度系统性地提高对肌肉的要求。
_避免使用_：更拼命练、升级

**RPE（自觉运动强度）**：
1–10 分的主观组间费力程度自评，10 分表示力竭，8 分表示还留有两次余力。
_避免使用_：费力分、强度打分
```

## 规则

- **只在用户真正理解后再加入术语。** 术语表是压缩后的知识记录，而不是供用户从头啃的词典。如果用户刚接触某个概念，等他能正确使用后再收录进来。
- **保持明确立场。** 同一个概念有多种叫法时，选出最准确的一个，其余列为应避免的别名。这是压缩语言的方式。
- **定义要紧凑。** 一到两句话即可。定义这个术语*是什么*，而不是它能做什么或怎么做。
- **在定义内部复用术语表已有术语。** 一旦某个术语进入术语表，尽量在所有地方优先使用它，包括其他术语的定义里。这能让复杂概念在后续更容易掌握。
- **按自然聚类分组。** 当术语自然聚集时使用二级标题（如 `## 解剖学`、`## 训练编排`）；数量不多、联系紧密时平铺列表即可。
- **显式消歧。** 如果某个术语在外部领域用法宽松，在工作区里明确界定：例如“在本工作区中，‘组（set）’始终指正式工作组；热身组单独记录。”
- **随理解加深而修订。** 用户第一周写下的定义到第六周可能不再准确。就地更新，不要留下过时条目。
{%- endif %}
