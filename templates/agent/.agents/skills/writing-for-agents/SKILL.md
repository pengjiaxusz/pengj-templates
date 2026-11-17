---
name: writing-for-agents
description: >-
{%- if options["skill_lang"] == "en" %}
  Writing documents for agents. Use when creating or editing skills, or modifying AGENTS.md or CLAUDE.md.
{%- else %}
  为 Agent 编写文档的指南。在创建或编辑技能，或修改 AGENTS.md、CLAUDE.md 时使用。
{%- endif %}
---

{% if options["skill_lang"] == "en" -%}
Reference for writing any document an agent consumes: a skill, an `AGENTS.md` / `CLAUDE.md`, a doc reached by a pointer. The packaging differs; the writing does not: the same levers make each one predictable, since the agent takes the same _process_ every run rather than producing the same output.

When the document you're writing is a skill, read [`SKILL-MECHANICS.md`](SKILL-MECHANICS.md) for frontmatter, invocation choice, and router skills.

## Context pointers

A **context pointer** is a reference held in the agent's context that names some out-of-context material and encodes the condition for reaching it. A skill's description is one; a line in `AGENTS.md` naming a doc is the same object. The pointer's _wording_, not its target, decides when the agent reaches the material, and how reliably. A must-have target behind a weakly worded pointer is a variance bug: sharpen the wording first, and inline the material only if sharpening fails.

A pointer does two jobs: state what the material is, and list the **branches** that should trigger reaching it (a branch is a distinct case the document handles, so different runs take different paths through it). Every word of an always-loaded pointer costs on every turn, so it earns even harder pruning than the body:

- **Front-load the leading word**: the pointer is where it does its triggering work.
- **One trigger per branch.** Synonyms that rename a single branch are one branch written twice; collapse them and keep only genuinely distinct branches.
- **Cut identity the body already carries.**

## The two loads

Every document and pointer you add spends one of two budgets:

- **Context load** is the cost of always-loaded material on the agent's window: an `AGENTS.md` line, a skill description, anything sitting in context every turn, spending tokens and attention whether or not it fires.
- **Cognitive load** is the cost on the human: which documents exist and when to reach for each. The human is the index. Not a cost to minimise: it is the price of human agency; spend it where human judgement matters, remove it where it does not.

Material reached only through a pointer escapes context load at the price of the pointer's own line; material with no pointer at all rides entirely on cognitive load.

## Information hierarchy

A document is built from two content types: **steps** (the ordered actions the agent performs) and **reference** (definitions, rules, facts consulted on demand). The two mix freely: all steps (a recipe), all reference (a review's rules, this skill), or both. The core decision is where each piece sits on the **information hierarchy**, a ladder ranked by how immediately the agent needs the material:

1. **In-file step** is the primary tier: what the agent does, in order.
2. **In-file reference** is consulted on demand. Often a legitimately flat peer-set (every rule of a review on one rung), which is a fine arrangement, not a smell.
3. **Disclosed reference** is pushed out into a separate file, reached by a context pointer, loaded only when the pointer fires. Spans a sibling file in the same folder through fully external reference that lives anywhere and any document can point at.

Push too little down and the top bloats; push too much and you hide material the agent actually needs. That tension is the whole decision.

**Progressive disclosure** is the move down the ladder (out of the main file and behind a pointer) so the top stays legible. Not primarily a token optimisation: it is how the hierarchy is protected. Branching is the cleanest disclosure test: inline what every branch needs, and push behind a pointer what only some branches reach. When a document has steps, in-file reference that should be disclosed buries them and turns attending to them into a coin-flip: a variance lever, not just a legibility one.

**Co-location** is the within-file companion: where the ladder decides _how far down_ a piece sits, co-location decides _what sits beside it_ once there. Keep a concept's definition, rules, and caveats under one heading rather than scattered, so reading one part brings its neighbours with it. The test: the document should read like documentation written for the agent. Grouped material reads that way; scattered material does not. (Distinct from duplication: that repeats one meaning in two places; scattering fragments one meaning across many.)

**Sprawl** is the failure mode here: a document simply too long, even when every line is live and unique. Attention thins across the excess, and every extra line is one more to keep relevant. The cure is the ladder: disclose reference behind pointers, and split by branch or sequence so each path carries only what it needs.

## Steps and completion criteria

Every step ends on a **completion criterion**, the condition that tells the agent the work is done. Two properties make it a lever:

- **Clarity**: can the agent tell done from not-done? A vague bound ("understanding reached") invites **premature completion**: ending the step before it is genuinely done, attention slipping to _being done_. The visible steps still ahead (the **post-completion steps**) supply the pull; the criterion's clarity is the resistance. Defend in order: **sharpen the bound first** (local and cheap); only if it is irreducibly fuzzy _and_ you observe the rush, hide the later steps by splitting the sequence. Hiding only works across a real context boundary (a hand-off or a subagent dispatch; an inline call leaves the later steps in context and clears nothing).
- **Demand**: how much it requires. "Every modified model accounted for" forces thorough work where "produce a change list" does not. Demand drives **legwork** (the digging the agent does within the work, latent in the wording rather than written as its own step), and it is not step-bound: "every rule applied" binds a body of flat reference just as "every step done" binds a sequence, which is how an all-reference document still carries an exhaustiveness bar.

The strongest criteria are both checkable and exhaustive.

## When to split

Splitting one document into two spends one of the two loads, so split only when the cut earns it:

- **By sequence**: split a run of steps where the post-completion steps tempt the agent to rush the one in front of it. Keeping them out of view drives more legwork on the current task. Beware the reverse: merging sequences exposes each step's later steps to what follows, inviting premature completion.
- **By invocation**, skill-specific: see [`SKILL-MECHANICS.md`](SKILL-MECHANICS.md).

## Leading words

A **leading word** is a compact concept already living in the model's pretraining that the agent thinks with while running the document (_lesson_, _fog of war_, _tracer bullets_). Repeated as a token, never as a sentence, it accumulates a distributed definition and anchors a whole region of behaviour in the fewest tokens, by recruiting priors the model already holds. Coining your own works if you define it clearly, but a made-up word recruits no priors: you pay in definition tokens what a pretrained word gives free; reach for an existing word first.

It anchors twice. In the body, _execution_: the agent reaches for the same behaviour every time the word appears, and inside flat reference it focuses attention on a class of thing to look for. In a pointer, _invocation_: when the same word lives in your prompts, your docs, and your codebase, the agent links that shared language to the material and reaches it more reliably.

Hunt for opportunities to refactor with leading words. A triad spelled out at three sites, a pointer spending a sentence to gesture at one idea. Each is a passage begging to collapse into a single token:

- "fast, deterministic, low-overhead" → _tight_ (a _tight_ loop).
- "a loop you believe in" → _red_, turning a fuzzy gate into a binary observable state (the loop goes _red_ on the bug, or it doesn't).

You win twice: fewer tokens, and a sharper hook for the agent to hang its thinking on. Assume every document is carrying restatements that leading words retire. Go find them.

**Negation** is the failure mode beside this lever: steering by prohibition drags the forbidden behaviour into context and makes it _more_ available, not less. _Don't think of an elephant_, and the elephant is all there is; the negation is a weak modifier the strongly-activated concept overruns, so the ban half-reads as an instruction to do the thing. Prompt the **positive**: state the target behaviour ("write one-line comments") so the banned one is never spoken. A prohibition earns its place only as a hard guardrail you cannot phrase positively; even then, pair it with the positive target so attention lands on what to do.

## Pruning

- Keep each meaning in a **single source of truth**: one authoritative place, so changing the behaviour is a one-place edit. **Duplication** (the same meaning in more than one place) costs maintenance and tokens, and inflates a meaning's prominence on the ladder past its real rank. (The accidental inverse of a leading word, which repeats a token on purpose, never the meaning.)
- The **environment** is a source of truth too (`package.json` scripts, config files, the directory layout, `--help` output), and a document that restates it is a **cache**: a copy of a lookup, earning its load only when the lookup is expensive. Cache what the agent cannot find by looking: the unwritten convention, the reason behind a choice, the gotcha no config confesses. Leave the one-file, one-command lookups to the environment, where they cannot go stale.
- Check every line for **relevance**: does it still bear on what the document does? A line loses relevance by never bearing on the task (mere exposition, or a branch that should be disclosed) or by going stale as the behaviour or world it describes changes. Shorter documents are easier to keep relevant. Without a pruning discipline the default fate is **sediment**: stale layers that settle because adding feels safe and removing feels risky, until you must core down through them to find what is still live.
- Hunt **no-ops** sentence by sentence: an instruction the model already obeys by default pays load to say nothing. The test (does it change behaviour versus the default?) is model-relative, not reader-relative: two people disagreeing about a no-op disagree about the default, and settle it by running the document, not by debate. When a sentence fails, delete the whole sentence rather than trim words from it. The test also grades leading words: a word too weak to beat the default (_be thorough_ when the agent is already thorough-ish) is a no-op, and the fix is a stronger word (_relentless_), not a different technique.
{%- else -%}
编写任何供 Agent 消费的文档的参考指南：技能、`AGENTS.md` / `CLAUDE.md`、通过指针引用的文档。包装形式各异，写作原理相通：同样的控制杠杆能让每份文档都变得可预测，因为 Agent 在每次运行时走的是相同的*流程*，而不是机械产出相同的输出。

当你编写的文档是技能（skill）时，请阅读 [`SKILL-MECHANICS.md`](SKILL-MECHANICS.md) 了解 frontmatter、调用方式选择与路由技能。

## 上下文指针（Context pointers）

**上下文指针**是保留在 Agent 上下文中的一条引用，它指明某份上下文之外的资料，并编码了何时去读取它的条件。技能的 `description` 就是一个指针；`AGENTS.md` 中指向某份文档的一行说明也是同一种东西。决定 Agent 何时去读这份资料以及触发有多稳定的，是指针的*措辞*，而不是目标文档本身。如果一个必读目标因为指针措辞太弱而时读时不读，这属于稳定性缺陷：先打磨指针措辞，只有打磨无效时才考虑把内容内联进来。

指针承担两项任务：说明这份资料是什么，以及列出应当触发读取它的**分支（branches）**（每个分支对应文档处理的一种独立场景，不同运行会走不同路径）。常驻加载的指针每多一个词都会在每一轮对话消耗成本，因此它比正文更需要严格修剪：

- **把引导词前置**：指针开头最先发挥触发作用。
- **每个分支只留一个触发词**：把同一个分支换个同义词再写一遍属于重复；合并同义词，只保留真正不同的分支。
- **删掉正文已经承载的身份信息**。

## 两种负载

你每增加一份文档或一条指针，都在消耗以下两种预算之一：

- **上下文负载（Context load）**：常驻加载内容对 Agent 上下文窗口的占用——`AGENTS.md` 里的一行、技能的 `description`，以及任何每轮都占着上下文、不管用不用都在消耗 token 和注意力的内容。
- **认知负载（Cognitive load）**：加在人身上的负担——记住有哪些文档、什么时候该用哪一份。人自己充当了索引。这不是要一味最小化的成本，而是人类掌控力的代价：把认知负载花在需要人类判断的地方，在不需要的地方去掉它。

仅通过指针才去读取的资料可以避开常驻上下文负载，代价只是一行指针；完全没有指针的资料则全部依赖人的认知负载。

## 信息层级（Information hierarchy）

文档由两类内容构成：**步骤（steps）**（Agent 按顺序执行的动作）与**参考（reference）**（按需查阅的定义、规则、事实）。两者可以自由组合：纯步骤（菜谱）、纯参考（代码评审规则、本技能），或两者兼有。核心决策在于把每块内容放在**信息层级**的哪一级——按 Agent 多快需要它来排序：

1. **文件内步骤**：第一层级，Agent 按顺序做什么。
2. **文件内参考**：按需查阅。常见形态是一组合理扁平的并列规则（比如一次评审的所有规则都在同一层级），这是很好的组织方式，不是坏味道。
3. **渐进披露的参考（Disclosed reference）**：下沉到独立文件中，通过上下文指针引出，仅在指针触发时加载。既可以是同目录下的兄弟文件，也可以是放在任何位置、供任何文档指向的完全外部参考。

下沉太少会让顶层臃肿；下沉太多则会把 Agent 真正需要的内容藏起来。这种张力就是全部权衡所在。

**渐进式披露（Progressive disclosure）**就是沿着层级向下移动（移出主文件，放到指针后面），让顶层保持清晰可读。它首要目的不是省 token，而是保护信息层级。按分支判断是最干净的检验标准：所有分支都需要的内联在主文件；只有部分分支才用到的放到指针后面。当文档包含步骤时，把本该渐进披露的参考硬塞在文件里会把步骤淹没，让 Agent 到底看不看这些参考变成抛硬币——这是稳定性控制杆，不只是可读性问题。

**共置（Co-location）**是文件内部的配套原则：信息层级决定内容放多*深*，共置决定到了那一深度的同一位置旁边该放*什么*。把一个概念的定义、规则和注意事项放在同一个标题下，而不是打散在各处，这样读到一部分时顺带就把相邻内容一起读进来了。检验标准：文档读起来应当像专门写给 Agent 的技术文档。聚拢的内容读起来如此，打散的内容则不是。（注意区分于“重复”：重复是在两处写同一个意思；打散是把一个意思撕碎在好几处。）

**蔓延（Sprawl）**是这里的失效模式：即使每一行都还在生效、没有重复，整份文档也太长了。注意力会在过长的篇幅上被稀释，每多一行都要额外维持相关性。解药就是信息层级：把参考内容放到指针后渐进披露，并按分支或流程阶段拆分，让每条执行路径只携带它真正需要的内容。

## 步骤与完成标准

每个步骤都以一个**完成标准（completion criterion）**收尾——告诉 Agent 这件事何时算做完的条件。它有两个控制属性：

- **清晰度（Clarity）**：Agent 能不能分清“做完了”和“没做完”？模糊的边界（比如“达成理解”）会诱发**过早完成（premature completion）**：还没真正做完就草草收工，注意力滑向“快点结束”。前面还看得见的后续步骤（**完成后的后续步骤**）会产生向前拉扯的拉力，而完成标准的清晰度就是抵抗这种拉力的阻力。按顺序防御：**先把边界写死写清**（局部且低成本）；只有当边界天然模糊*且*你确实观察到 Agent 抢跑时，才通过拆分流程来隐藏后续步骤。注意：隐藏只在跨越真实的上下文边界时才有效（例如交接或派发 sub-agent；内联调用会把后续步骤留在同一个上下文里，起不到隔离作用）。
- **要求强度（Demand）**：标准要求做到多深。“把每一个被修改的模型都核对到位”会倒逼扎实的查证工作，而“列一份改动清单”则不会。要求强度驱动**跑腿活（legwork）**（Agent 在任务中主动去挖细节的行为，它潜藏在措辞里，不需要单独写成一步），而且它不只绑在步骤上：“每条规则都逐一应用”对扁平参考文档同样有效，就像“每一步都完成”对序列有效一样——纯参考文档也同样能靠它设立穷尽性门槛。

最强的完成标准应当既可核验，又具备穷尽性。

## 何时拆分

把一份文档拆成两份会消耗两种负载之一，因此只在切分确实划算时才拆：

- **按流程阶段拆分**：当一串步骤里的后续步骤会诱惑 Agent 草率跳过眼前这一步时，按阶段拆开。让后续步骤暂时不可见，能促使 Agent 在当前任务上做更充分的跑腿查证。反之亦然：把两个阶段合并，会让前一步看到后面还要做什么，从而诱发过早完成。
- **按调用方式拆分**（技能专属）：见 [`SKILL-MECHANICS.md`](SKILL-MECHANICS.md)。

## 引导词（Leading words）

**引导词**是指模型在预训练阶段就已经熟悉、Agent 在执行文档时会拿来辅助思考的紧凑概念（如 _lesson_、_fog of war_、_tracer bullets_）。作为固定词符在文档中反复出现（而不是每次用一整句话解释），它能累积出分布式语义，并用极少的 token 锚定一整片行为空间——因为它调动了模型原有的先验。自己造词只要定义清楚也能用，但生造词没有先验可借：你得花 token 去定义，而现成词汇免费自带语义；优先使用现成词汇。

它有双重锚定作用。在正文中锚定*执行*：每次出现这个词，Agent 都会触发同一套行为；在扁平参考里，它把注意力聚焦到某类特定事物上。在指针中锚定*触发*：当同一个词同时出现在你的提示词、文档和代码库里时，Agent 会把这套共享语言和对应资料绑定起来，触发更加稳定。

主动寻找用引导词重构的机会。在三处重复展开同一套三元组、用一整句话在指针里比划一个概念——这些地方都适合收束成一个词：

- “快速、确定、低开销” → _紧凑（tight）_（一个 _tight_ 循环）。
- “一个你信得过的验证回路” → _红灯（red）_，把模糊门槛变成可观察的二元状态（这个回路在 bug 上要么亮 _red_，要么不亮）。

一举两得：既省 token，又给 Agent 一个更锋利的思维抓手。假定每份文档里都藏着可以用引导词替换掉的啰嗦复述，去把它们找出来。

**否定句（Negation）**是与这个杠杆相邻的失效模式：靠“禁止做某事”来引导，反而会把被禁止的行为拽进上下文里，让它变得*更容易*被触发而不是更少。“不要去想大象”，结果满脑子都是大象；否定词是个弱修饰符，强激活的概念会直接盖过它，于是禁止指令有一半像是在教模型去干那件事。改用**正向提示（positive）**：直接陈述目标行为（“写单行注释”），让被禁止的行为根本不出现在上下文中。只有在你实在无法用正向语言表述某条硬性护栏时，才保留一条禁止规则；即便如此，也要紧挨着写上正向目标，把注意力拉回“该做什么”上。

## 修剪（Pruning）

- 让每个语义保持**单一事实来源（single source of truth）**：只在一个权威位置定义，修改行为时只需改一处。**重复**（在多处表达同一个意思）既增加维护成本和 token，又会虚增该语义在信息层级中的权重。（这恰好与有意识复用同一个“引导词”相反——引导词重复的是词符，绝不重复整段语义。）
- **环境本身也是事实来源**（`package.json` 脚本、配置文件、目录结构、`--help` 输出），文档如果复述这些内容就是在做**缓存**：一份查询副本，只有当查询本身很昂贵时才值得承担它的负载。只缓存 Agent 仅靠看代码/目录查不到的东西：未成文的约定、某个选择背后的原因、配置文件不会主动坦白的坑。单文件、一条命令就能查到的东西留给环境自己，永远不会过时。
- 逐行检查**相关性**：这一行是否仍然直接作用于文档要做的事？一行文字失去相关性，要么是因为它压根不直接作用于任务（纯背景铺陈，或本该渐进披露的分支），要么是因为它描述的行为或世界变了而过时。短文档更容易保持相关性。缺乏修剪纪律时，文档的默认宿命就是**沉积（sediment）**：觉得“加上安全、删掉有风险”，于是过时的层次越积越厚，直到最后不得不钻透层层积灰才能找到真正还在生效的规则。
- 逐句猎杀**空操作（no-ops）**：一条模型默认就会遵守的指令，是在白白花负载说废话。检验方法（它相对于默认行为真的改变了什么吗？）是相对于*模型*而言的，而不是相对于*读者*而言的：两个人争论某句话是不是空操作，本质是在争论模型的默认行为是什么——直接跑一遍文档来验证，不要空口辩论。一旦某句话验证失败，整句删掉，不要只是抠字眼精简。同样的测试也用来检验引导词：太弱以至于压不过默认行为的词（在 Agent 本来就还算仔细时写“请彻底检查”）就是空操作，修法是换更强的词（“毫不留情地追问 / relentless”），而不是换套技巧。
{%- endif %}
