---
name: to-questionnaire
description: >-
{%- if options["skill_lang"] == "en" %}
  Turn a decision you can't fully answer into a questionnaire for someone else to fill in.
{%- else %}
  将你无法独自完全回答的决策转化为一份发给他人填写的问卷。
{%- endif %}
disable-model-invocation: true
---

{% if options["skill_lang"] == "en" -%}
Turn something the user can't answer alone into a **questionnaire**: a Markdown document they hand to one person to fill in async, or fill out together over a meeting. The recipient holds knowledge the user lacks; the questionnaire pulls it out of them.

**Grill the send, not the subject.** Interview the user only about the _send_, which they can always answer: who it goes to, and what they need back. The questions in the document then target the **gap** between what the recipient knows and what the user needs.


1. **Who is it going to?** Ask, in one exchange, the recipient's role, expertise, and relationship to the user. This fixes the questionnaire's tone and how much context it must carry. Done when you know who the recipient is and what they know that the user doesn't.

2. **What do you need back?** Ask, in one exchange, the specific decisions or facts the user can't resolve alone and needs from this person. Done when you have a concrete list of what the user must walk away able to do or decide.

3. **Write the questionnaire.** Draft questions aimed at the gap from steps 1–2, following the Document structure below. Write it to `to-questionnaire-<slug>.md` in the current directory (slug from the topic) and report the path. Done when the file exists and every item the user named in step 2 is covered by a question.

## Document structure

Frame the document as a **discovery questionnaire**: the user lacks context, the recipient holds it. Order questions most-important-first, since async means you may only get one pass, and group them under `##` headings by theme once there are more than a handful. Write it using the template below.

<questionnaire-template>

# <Questionnaire title>

**Purpose:** why this questionnaire exists and the decision riding on it.

**From:** <the user>, **To:** <the recipient>, **How your answers will be used:** <where they go>

## Context

One paragraph orienting a recipient who wasn't in the user's head. Enough to answer well, not a page.

## How to answer

Deadline and rough effort. Partial answers and "I don't know" are useful: flag anything you're unsure of rather than skipping it.

## <Theme heading>

One `##` section per theme. Under each, its questions, most-important-first. Every question is one idea, never compound, with an answer stub directly beneath, and a one-line _why this matters_ only where the question could be misread or invite a throwaway answer.

<question-example>
### What load is the system expected to handle at launch?

_Why this matters: it decides whether we provision for burst traffic now or defer it._

>
</question-example>

## Anything else?

A closing catch-all: anything we didn't ask that we should know?

</questionnaire-template>
{%- else -%}
将用户无法独自回答的问题转化为一份**问卷（questionnaire）**：一份可以异步发给某个人填写、或在会议中一起填完的 Markdown 文档。接收人掌握着用户缺乏的知识，问卷的作用就是把这些信息提取出来。

**追问发送本身，而不是追问主题内容。** 只围绕用户一定能回答的*发送上下文*访谈用户：发给谁、需要对方反馈什么。文档中的问题则精准瞄准接收人已知与用户所需之间的**信息差（gap）**。


1. **发给谁？** 用一轮对话问清接收人的角色、专业背景以及与用户的关系。这决定了问卷的语气以及需要铺垫多少背景信息。完成标准：明确接收人是谁，以及对方掌握哪些用户不知道的信息。

2. **需要对方反馈什么？** 用一轮对话问清用户无法独自拍板、需要向此人获取的具体决策或事实。完成标准：拿到一份具体清单，明确用户读完回复后必须能够做出什么决定或采取什么行动。

3. **撰写问卷。** 针对第 1–2 步梳理出的信息差起草问题，遵循下方的文档结构。将问卷写入当前目录的 `to-questionnaire-<slug>.md`（slug 取自主题）并告知文件路径。完成标准：文件已创建，且用户在第 2 步提到的每一项都有对应问题覆盖。

## 文档结构

将文档定位为**探索性问卷（discovery questionnaire）**：用户缺少上下文，接收人掌握上下文。由于异步沟通可能只有一次往返机会，按重要性从高到低排列问题；若问题超过五六个，按主题用 `##` 二级标题分组。使用下方模板撰写。

<questionnaire-template>

# <问卷标题>

**目的：**说明这份问卷存在的意义，以及依赖这些答案做出的决策。

**发起人：**<用户>，**接收人：**<接收人>，**答案用途：**<回复将如何被使用>

## 背景说明

用一段话为不在用户上下文中的接收人做好铺垫。信息量够对方准确作答即可，不要写成一整页。

## 作答方式

说明截止时间与大致耗时。部分回答和“我不知道”同样有价值：不确定的地方请直接标出，不要跳过。

## <主题标题>

每个主题一个 `##` 小节。小节内问题按重要性从高到低排列。每个问题只包含一个要点，绝不复合提问；问题下方直接留出作答区，仅在问题容易被误解或被敷衍回答时附上一行*_为什么这很重要_*。

<question-example>
### 上线初期系统预计需要承载多大的负载？

_为什么这很重要：这决定了我们是现在就为突发流量预留容量，还是延后处理。_

>
</question-example>

## 还有其他补充吗？

结尾兜底问题：还有什么我们没问到、但应该了解的情况吗？

</questionnaire-template>
{%- endif %}
