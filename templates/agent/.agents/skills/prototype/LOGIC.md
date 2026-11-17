{% if options["skill_lang"] == "en" -%}
# Logic Prototype

A single, self-contained HTML file (a **shareable demo**) that lets anyone drive a state model by clicking buttons. Use this when the question is about **business logic, state transitions, or data shape**: the kind of thing that looks reasonable on paper but only feels wrong once you push it through real cases.

Because it's one file with nothing to install, you can hand it to a non-developer (a designer, a PM, a domain expert) and let them feel the model for themselves. So it speaks their language, not the code's.

## When this is the right shape

- "I'm not sure if this state machine handles the edge case where X then Y."
- "Does this data model actually let me represent the case where..."
- "I want to feel out what the API should look like before writing it."
- Anything where someone wants to **press buttons and watch state change**.

If the question is "what should this look like," this is the wrong branch. Use [UI.md](UI.md).

## Process

### 1. State the question

Before writing code, write down what state model and what question you're prototyping. One paragraph, at the top of the demo (in a visible intro, not just a comment). A logic prototype that answers the wrong question is pure waste, so make the question explicit so it can be checked later, whether the user is watching now or returning to it AFK.

### 2. Isolate the logic in a portable module

Put the actual logic (the bit that's answering the question) in a single `<script>` block written as a small, pure module that could be lifted out and dropped into the real codebase later. The page around it is throwaway; this module isn't.

The right shape depends on the question:

- **A pure reducer**: `(state, action) => state`. Good when actions are discrete events and state is a single value.
- **A state machine**: explicit states and transitions. Good when "which actions are even legal right now" is part of the question.
- **A small set of pure functions** over a plain data type. Good when there's no implicit current state, just transformations.
- **A class or module with a clear method surface** when the logic genuinely owns ongoing internal state.

Pick whichever shape best fits the question being asked, *not* whichever is easiest to wire to a page. Keep it pure: no DOM, no `document`, no button handlers reaching inside it. The page calls into it; nothing flows the other direction. This is what makes the prototype useful past its own lifetime: once the question's answered, the validated reducer / machine / function set lifts into the real module on its own.

### 3. Build the shareable HTML file

One file, plain HTML/CSS/JS: no framework, no bundler, no server, everything inline so it opens by double-click and survives being emailed around. Anyone should be able to run it by opening it.

Write it for a non-developer. Every label is in **domain language**, not code: buttons and state read like the business, not the reducer. Explain in plain words what's happening.

Lay it out with a clean hierarchy, top to bottom:

1. **Title and one-line explanation** of what this demo lets you explore (the question from step 1).
2. **Current state**: the full relevant state, rendered as a readable panel (labelled fields, not a raw JSON dump), re-rendered after every click so the change is visible. Where it helps a non-developer follow, call out what just changed.
3. **Free-play buttons**: one button per action, always available, so anyone can poke at the model in any order. Each click dispatches its action and re-renders the state.
4. **Guided walkthroughs**: a set of **scenarios**, one per tab. Each tab holds a short plain-language description of the scenario (the situation it sets up and what to watch for) and underneath it, the ordered **buttons to press** for that scenario. Each step is a real button: clicking it performs that action and moves to the next step. Starting a walkthrough resets to a known initial state so the scenario runs the same way every time.

Choose scenarios that demonstrate the awkward cases, the ones hard to reason about on paper: the happy path, a tricky edge case, an attempt at something that should be illegal.

Keep it beautiful but restrained: clean typography, generous spacing, one accent colour. No animations, no gimmicks: nothing that competes with the state and the buttons.

### 4. Hand it over

Send them the file, or open it for them. They'll click through the walkthroughs and free-play whenever they get to it; the interesting moments are when they say "wait, that shouldn't be possible" or "huh, I assumed X would be different"; those are the bugs in the _idea_, which is the whole point. If they want new actions or a new scenario, add them. Prototypes evolve.

### 5. Capture the answer and the prototype

Once the prototype has answered its question, capture the answer, then capture the prototype the way the [SKILL](SKILL.md) describes. The logic-specific mapping: the validated reducer / machine / function set lifts into the real module (the decision, absorbed); the HTML shell rides along to the throwaway branch that keeps the prototype as a primary source, and being one self-contained file, it stays trivially re-runnable there.

## Anti-patterns

- **Don't add tests.** A prototype that needs tests is no longer a prototype.
- **Don't wire it to the real database.** Use in-memory state unless the question is specifically about persistence.
- **Don't generalise.** No "what if we wanted to support X later." The prototype answers one question.
- **Don't blur the logic and the page together.** If the pure module references the DOM, `document`, or button handlers, it's no longer liftable. Keep the page as a thin shell over a pure module.
- **Don't reach for a framework, bundler, or server.** One file the recipient double-clicks; a React app or a dev server defeats "shareable".
- **Don't ship the HTML shell into production.** The page is optimised for being clicked through by hand. The logic module behind it is the bit worth keeping.
{%- else -%}
# 逻辑原型（Logic Prototype）

一个独立的自包含 HTML 文件（即**可分享的演示页**），让任何人都能通过点按按钮来驱动状态模型。当问题围绕**业务逻辑、状态流转或数据结构**展开时使用——也就是那些在纸面上看着合理、但一跑真实用例就会觉得别扭的东西。

因为它只有一个文件且无需安装任何依赖，你可以直接丢给非开发人员（设计师、产品经理、领域专家），让他们亲手感受这个模型。因此界面必须说他们的业务语言，而不是代码语言。

## 适用场景

- “我不确定这个状态机能不能处理先 X 后 Y 的边界情况。”
- “这个数据模型真的能表达……这种场景吗？”
- “我想在动手写 API 之前先感受一下它用起来顺不顺手。”
- 任何需要**点按钮看状态变化**的场景。

如果问题问的是“界面应该长什么样”，那走错了分支，请改用 [UI.md](UI.md)。

## 流程

### 1. 写明要验证的问题

写代码前，先写清楚你正在原型化什么状态模型、要回答什么问题。一段话，放在演示页顶部（写在页面可见的引言里，不要只写在注释里）。一个答非所问的逻辑原型纯属浪费，所以把问题写明白，方便用户无论是当场观看还是离线回来查看时都能核对。

### 2. 把逻辑隔离在可移植的模块中

把真正的核心逻辑（回答问题的那部分）放进单个 `<script>` 块，写成一个小巧的纯模块，以便日后能直接拎出来放进正式代码库。外面的页面是一次性的，这个模块不是。

具体形态取决于问题本身：

- **纯 reducer**：`(state, action) => state`。适合动作是离散事件、状态是单一值的情况。
- **状态机**：显式的状态与转移。适合“当前究竟哪些动作合法”本身就是问题一部分的情况。
- **围绕普通数据类型的一小组纯函数**。适合没有隐式当前状态、只有数据变换的情况。
- **接口清晰的 class 或模块**：当逻辑确实拥有持续的内部状态时。

选择最贴合问题本质的形态，*而不是*最方便绑到页面上的形态。保持纯净：不能碰 DOM、`document`，也不能让按钮回调伸手进模块内部。页面调用模块；反过来绝不依赖页面。这正是原型在自身生命周期结束后依然有价值的原因：一旦问题得到验证，已验证的 reducer / 状态机 / 函数集就能独立抽进正式模块。

### 3. 构建可分享的 HTML 文件

单文件，纯 HTML/CSS/JS：不用框架、不用打包器、不用起服务器，全部内联，双击就能打开，发邮件传阅也不会坏。任何人都应该打开即用。

面向非开发人员编写。每个标签都用**领域语言**而非代码术语：按钮和状态读起来像业务语言，而不是 reducer。用通俗的话解释正在发生什么。

自上而下采用清晰的层次布局：

1. **标题与一句话说明**：点明这个演示允许你探索什么（第 1 步的问题）。
2. **当前状态**：完整呈现相关状态，渲染成易读的面板（带标签的字段，不要直接扔原始 JSON），每次点击后重新渲染以体现变化。在有助于非技术人员理解的地方，标出刚刚发生了什么变化。
3. **自由试玩按钮**：每个动作一个按钮，始终可点，方便任何人按任意顺序试探模型。每次点击派发对应动作并重绘状态。
4. **引导式演练（Guided walkthroughs）**：一组**场景（scenarios）**，每个场景一个标签页。每个标签页包含对该场景的简短白话描述（设定了什么情境、注意观察什么），下方是该场景按顺序排列的**可点击按钮**。每一步都是真实按钮：点下去就执行该动作并推进到下一步。启动演练时重置到已知初始状态，保证每次运行结果一致。

挑选能暴露棘手情况的场景——那些在纸面上很难推演的场景：正常主路径、刁钻边界情况、试图执行理论上应该非法的动作。

保持克制的美观：干净的排版、充足的留白、一种强调色。不要动画、不要花哨噱头——不要有任何东西跟状态与按钮抢注意力。

### 4. 交付试用

把文件发给对方，或直接为他们打开。他们有空时会把引导演练和自由试玩点一遍；真正有价值的时刻是他们说出“等等，这不应该允许啊”或“咦，我以为 X 会是另一个结果”——那些就是*想法本身*的 bug，也正是原型的全部意义。如果他们想加新动作或新场景，补上即可，原型本来就是演进的。

### 5. 沉淀答案与原型

原型回答完问题后，先沉淀结论，再按 [SKILL.md](SKILL.md) 的方式保存原型。具体到逻辑原型的映射方式：已验证的 reducer / 状态机 / 函数集抽进正式模块（决策被吸收）；HTML 外壳随一次性分支一起保留作为一手来源——而且因为它是单文件自包含的，留在分支里随时都能重新打开运行。

## 反模式

- **不要加测试。** 需要写测试的原型就不再是原型。
- **不要连真实数据库。** 除非问题专门关于持久化，否则一律用内存状态。
- **不要过度泛化。** 不要想“万一以后我们要支持 X 呢”。原型只回答一个问题。
- **不要把逻辑和页面糊在一起。** 如果纯模块引用了 DOM、`document` 或按钮事件，它就拎不出来了。把页面保持成纯模块外面的一层薄壳。
- **不要上框架、打包器或服务器。** 接收者双击就能开的单个文件才是目的；一个 React 应用或 dev server 直接毁掉了“可分享性”。
- **不要把 HTML 外壳发进生产环境。** 这个页面是为手工点按优化的，真正值得保留的是它背后的逻辑模块。
{%- endif %}
