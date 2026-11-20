# AI 时代代码评审 资源清单

主题：读懂 Anthropic《How the Claude Code team does code review》及其背后的工程逻辑。

## 知识（Knowledge）

- [文章原文：How the Claude Code team does code review（Addy Osmani、Morgan Westlee Lunt、Qing Wang，2026-10-15）](https://claude.dev/blog/solving-code-review-at-anthropic/)
  本工作区的第一手核心资料。涵盖评审目的重构、七层关卡、风险分级审批、评审收敛、事故转 eval、人类保留的位置。**适用场景**：一切论断的核对源头；每节课都应回到原文引用。
- [Best practices for Claude Code（Anthropic Engineering）](https://www.anthropic.com/engineering/claude-code-best-practices)
  官方最佳实践，提出「给 Claude 一个可验证的检查」与「证据优于断言」，并给出 subagent 复核等做法。原文第 1 层做法的来源。**适用场景**：理解「验证先于评审」为什么被团队称为最关键的习惯。
- [Redesigning source control at Anthropic（Anthropic Research，2026-10）](https://www.anthropic.com/research/redesigning-source-control)
  同期的工程文章，给出 monorepo 每天合并数从 5 月约 2,500 到 9 月超过 30,000 的上下文与自建 Scion 的动因。**适用场景**：解释「为什么人工逐行评审在这种规模下不可行」的背景数据。
- [Code Review is now generally available（Claude 官方产品公告）](https://claude.com/resources/articles/code-review-is-now-generally-available)
  对照文章提到的对外产品：评审速度约从 20 分钟降到 6–10 分钟、Quick/Standard/Deep 三档、以及「产品只提交发现，不提交批准」的边界。**适用场景**：区分内部流程与对外产品能力，避免误以为官方产品会替人批准 PR。
- [Building verification loops in Claude Code with skills（Claude 官方资源文章）](https://claude.com/fr/resources/articles/building-verification-loops-in-claude-code-with-skills?utm_term=HAHS)
  讲如何把人工检查写成可复用 skill、形成自动验证回路。**适用场景**：理解「把重复出现的评审意见固化进 skill / lint / CLAUDE.md」这一反馈闭环的思路。
- [Agentic coding is straining CI. Here's how we scaled test impact analysis at Anthropic（Anthropic Engineering，2026-09）](https://claude.com/blog/agentic-coding-is-straining-ci-heres-how-we-scaled-test-impact-analysis-at-anthropic)
  CI 扩容的同源背景（CI job 数量六个月增长约 25 倍、测试影响分析）。**适用场景**：理解第 4 层「阻塞式自动检查」背后的基础设施压力。若此链接跳转，从 [Redesigning source control](https://www.anthropic.com/research/redesigning-source-control) 进入同源背景。

## 智慧（社区 / Wisdom）

- [Hacker News 上关于该文的讨论](https://hn.algolia.com/?q=solving-code-review-at-anthropic)
  用于看外部工程师如何质疑「自报数据」「零人工合并」的边界。**适用场景**：在第二节课后检验自己的判断，区分公司自述与可推广结论。
- [r/ExperiencedDevs](https://www.reddit.com/r/ExperiencedDevs/)
  有审查门槛的工程实践讨论社区，常有对 AI 生成代码评审流程的实证反思。**适用场景**：想听一线工程师的反例时使用；当前使命以理解为主，不强制参与。

## 缺口（Gaps）

- 文章中的效果数字（合并 PR 翻倍、回滚比例下降 40%+、实质性评论占比 16%→54%）均为团队自报，缺少独立第三方复现。
- 「事故造成的成本」与「多一轮 AI 评审的成本」没有公开对照数据，讲课时需标为未决问题。
- 中文一手资料稀缺；现有中文文章多为二手转述，不作为课堂引用源。
