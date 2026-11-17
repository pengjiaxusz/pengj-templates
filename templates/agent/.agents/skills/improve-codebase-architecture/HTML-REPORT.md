{% if options["skill_lang"] == "en" -%}
# HTML Report Format

The architectural review is rendered as a single self-contained HTML file in the OS temp directory. Tailwind and Mermaid both come from CDNs. Mermaid handles graph-shaped diagrams reliably; hand-built divs and inline SVG handle the more editorial visuals (mass diagrams, cross-sections). Mix the two: don't lean on Mermaid for everything, it'll start to look generic.

## Scaffold

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Architecture review for {% raw %}{{repo name}}{% endraw %}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script type="module">
      import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";
      mermaid.initialize({ startOnLoad: true, theme: "neutral", securityLevel: "loose" });
    </script>
    <style>
      /* small custom layer for things Tailwind doesn't cover cleanly:
         dashed seam lines, hand-drawn-feeling arrow heads, etc. */
      .seam { stroke-dasharray: 4 4; }
      .leak { stroke: #dc2626; }
      .deep { background: linear-gradient(135deg, #0f172a, #1e293b); }
    </style>
  </head>
  <body class="bg-stone-50 text-slate-900 font-sans">
    <main class="max-w-5xl mx-auto px-6 py-12 space-y-12">
      <header>...</header>
      <section id="candidates" class="space-y-10">...</section>
      <section id="top-recommendation">...</section>
    </main>
  </body>
</html>
```

## Header

Repo name, date, and a compact legend: solid box = module, dashed line = seam, red arrow = leakage, thick dark box = deep module. No introduction paragraph. Straight into the candidates.

## Candidate card

The diagrams carry the weight. Prose is sparse, plain, and uses the glossary terms (from the `/codebase-design` skill) without ceremony.

Each candidate is one `<article>`:

- **Title**: short, names the deepening (e.g. "Collapse the Order intake pipeline").
- **Badge row**: recommendation strength (`Strong` = emerald, `Worth exploring` = amber, `Speculative` = slate), plus a tag for the dependency category (`in-process`, `local-substitutable`, `ports & adapters`, `mock`).
- **Files**: monospaced list, `font-mono text-sm`.
- **Before / After diagram**: the centrepiece. Two columns, side by side. See patterns below.
- **Problem**: one sentence. What hurts.
- **Solution**: one sentence. What changes.
- **Wins**: bullets, ≤6 words each. e.g. "Tests hit one interface", "Pricing logic stops leaking", "Delete 4 shallow wrappers".
- **ADR callout** (if applicable): one line in an amber-tinted box.

No paragraphs of explanation. If the diagram needs a paragraph to be understood, redraw the diagram.

## Diagram patterns

Pick the pattern that fits the candidate. Mix them. Don't make every diagram look the same. Variety is part of the point.

### Mermaid graph (the workhorse for dependencies / call flow)

Use a Mermaid `flowchart` or `graph` when the point is "X calls Y calls Z, and look at the mess." Wrap it in a Tailwind-styled card so it doesn't feel parachuted in. Style with classDef to colour leakage edges red and the deep module dark. Sequence diagrams work well for "before: 6 round-trips; after: 1."

```html
<div class="rounded-lg border border-slate-200 bg-white p-4">
  <pre class="mermaid">
    flowchart LR
      A[OrderHandler] --> B[OrderValidator]
      B --> C[OrderRepo]
      C -.leak.-> D[PricingClient]
      classDef leak stroke:#dc2626,stroke-width:2px;
      class C,D leak
  </pre>
</div>
```

### Hand-built boxes-and-arrows (when Mermaid's layout fights you)

Modules as `<div>`s with borders and labels. Arrows as inline SVG `<line>` or `<path>` elements positioned absolutely over a relative container. Reach for this when you want the "after" diagram to feel like one thick-bordered deep module with greyed-out internals, since Mermaid won't render that with the right weight.

### Cross-section (good for layered shallowness)

Stack horizontal bands (`h-12 border-l-4`) to show layers a call passes through. Before: 6 thin layers each doing nothing. After: 1 thick band labelled with the consolidated responsibility.

### Mass diagram (good for "interface as wide as implementation")

Two rectangles per module: one for interface surface area, one for implementation. Before: interface rectangle is nearly as tall as the implementation rectangle (shallow). After: interface rectangle is short, implementation rectangle is tall (deep).

### Call-graph collapse

Before: a tree of function calls rendered as nested boxes. After: the same tree collapsed into one box, with the now-internal calls shown faded inside it.

## Style guidance

- Lean editorial, not corporate-dashboard. Generous whitespace. Serif optional for headings (`font-serif` works well with stone/slate).
- Colour sparingly: one accent (emerald or indigo) plus red for leakage and amber for warnings.
- Keep diagrams ~320px tall so before/after sits comfortably side by side without scrolling.
- Use `text-xs uppercase tracking-wider` for module labels inside diagrams, so they read as schematic, not as UI.
- The only scripts are the Tailwind CDN and the Mermaid ESM import. The report is otherwise static: no app code, no interactivity beyond Mermaid's own rendering.

## Top recommendation section

One larger card. Candidate name, one sentence on why, anchor link to its card. That's it.

## Tone

Plain English, concise, but the architectural nouns and verbs come straight from the `/codebase-design` skill. Concision is not an excuse to drift.

**Use exactly:** module, interface, implementation, depth, deep, shallow, seam, adapter, leverage, locality.

**Never substitute:** component, service, unit (for module) · API, signature (for interface) · boundary (for seam) · layer, wrapper (for module, when you mean module).

**Phrasings that fit the style:**

- "Order intake module is shallow: interface nearly matches the implementation."
- "Pricing leaks across the seam."
- "Deepen: one interface, one place to test."
- "Two adapters justify the seam: HTTP in prod, in-memory in tests."

**Wins bullets** name the gain in glossary terms: *"locality: bugs concentrate in one module"*, *"leverage: one interface, N call sites"*, *"interface shrinks; implementation absorbs the wrappers"*. Don't write *"easier to maintain"* or *"cleaner code"*, because those terms aren't in the glossary and don't earn their place.

No hedging, no throat-clearing, no "it's worth noting that…". If a sentence could be a bullet, make it a bullet. If a bullet could be cut, cut it. If a term isn't in the `/codebase-design` glossary, reach for one that is before inventing a new one.
{%- else -%}
# HTML 报告格式规范

架构评审报告渲染为操作系统临时目录下的单个自包含 HTML 文件。Tailwind 与 Mermaid 均通过 CDN 引入。Mermaid 擅长可靠绘制图结构（流程图/依赖图/时序图）；手写 `<div>` 与内联 SVG 则适合更有编排感的可视化（体量对比图、横截面图）。两者混用：不要所有图都指望 Mermaid，否则整份报告会显得千篇一律。

## 基础骨架

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <title>架构评审：{% raw %}{{repo name}}{% endraw %}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script type="module">
      import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";
      mermaid.initialize({ startOnLoad: true, theme: "neutral", securityLevel: "loose" });
    </script>
    <style>
      /* 补充 Tailwind 不便覆盖的小样式：虚线接缝、手绘感箭头等 */
      .seam { stroke-dasharray: 4 4; }
      .leak { stroke: #dc2626; }
      .deep { background: linear-gradient(135deg, #0f172a, #1e293b); }
    </style>
  </head>
  <body class="bg-stone-50 text-slate-900 font-sans">
    <main class="max-w-5xl mx-auto px-6 py-12 space-y-12">
      <header>...</header>
      <section id="candidates" class="space-y-10">...</section>
      <section id="top-recommendation">...</section>
    </main>
  </body>
</html>
```

## 页眉（Header）

仓库名称、日期，以及一个紧凑图例：实线框 = 模块（module），虚线 = 接缝（seam），红色箭头 = 泄漏（leakage），深色粗框 = 深度模块（deep module）。不要写引言段落，直接进入候选卡片。

## 候选卡片

图表承担主要信息量。文字要精简、平实，不加客套地直接采用 `/codebase-design` 技能中的术语。

每个候选对象对应一个 `<article>`：

- **标题**：简短，点明深化动作（例如“收拢 Order 接入流水线”）。
- **徽章行**：推荐强度（`Strong` = 翡翠绿，`Worth exploring` = 琥珀黄，`Speculative` = 石板灰），外加依赖类别标签（`in-process`、`local-substitutable`、`ports & adapters`、`mock`）。
- **涉及文件**：等宽字体列表，`font-mono text-sm`。
- **改造前 / 改造后对比图**：视觉核心。两栏并排，模式见下文。
- **问题**：一句话，点出痛点在哪。
- **方案**：一句话，点出要改什么。
- **收益（Wins）**：精简要点，每条不超过十余字（如“测试统一走单一接口”、“定价逻辑不再跨接缝泄漏”、“删掉 4 个浅包装层”）。
- **ADR 提示**（如适用）：放在琥珀色提示框中的一行说明。

不要写整段解释。如果一张图需要一整段文字才能看懂，重画那张图。

## 图表模式

为每个候选挑选最契合的模式并混搭使用，不要每张图长一个样——多样性本身就是可读性的一部分。

### Mermaid 图（依赖 / 调用流的主力）

当重点在于“X 调 Y 调 Z，一团乱麻”时，使用 Mermaid `flowchart` 或 `graph`。包在 Tailwind 卡片里以免显得突兀，并用 `classDef` 把泄漏边标红、深度模块标深色。时序图很适合表现“改造前 6 次往返；改造后 1 次”。

```html
<div class="rounded-lg border border-slate-200 bg-white p-4">
  <pre class="mermaid">
    flowchart LR
      A[OrderHandler] --> B[OrderValidator]
      B --> C[OrderRepo]
      C -.leak.-> D[PricingClient]
      classDef leak stroke:#dc2626,stroke-width:2px;
      class C,D leak
  </pre>
</div>
```

### 手写方框与箭头（当 Mermaid 排版跟你作对时）

用带边框和标签的 `<div>` 表示模块，用绝对定位在相对容器上的内联 SVG `<line>` 或 `<path>` 画箭头。当你想让“改造后”的图呈现出“一个粗边框深度模块里包着淡化内部细节”的视觉份量时，选这种方式——Mermaid 画不出这种份量感。

### 横截面图（适合层层浅薄透传）

堆叠横向条带（`h-12 border-l-4`）展示一次调用穿过多少层。改造前：6 个什么都不干的薄层；改造后：1 个标明整合职责的厚层。

### 体量对比图（适合“接口几乎和实现一样宽”）

每个模块画两个矩形：一个代表接口表面积，一个代表实现体量。改造前：接口矩形几乎和实现矩形一样高（浅）；改造后：接口矩形矮小，实现矩形高大（深）。

### 调用树收拢图

改造前：嵌套方框呈现的函数调用树；改造后：同一棵树收拢进一个方框，原本的调用退化为框内淡显的内部细节。

## 视觉风格指南

- 走克制的编辑部排版风格，而不是企业仪表盘。留白要足，标题可选衬线体（`font-serif` 与 stone/slate 灰调很搭）。
- 克制用色：一种主强调色（翡翠绿或靛蓝），外加红色表示泄漏、琥珀色表示警告。
- 图表高度控制在 ~320px 左右，使改造前/后能舒适地并排同屏对比、无需滚动。
- 图表内模块标签使用 `text-xs uppercase tracking-wider`，让它看起来像架构示意图而不是 UI 组件。
- 页面仅引入 Tailwind CDN 与 Mermaid ESM；其余完全静态，无应用逻辑代码，除了 Mermaid 自身渲染外无交互。

## 首推建议区（Top recommendation）

一张较大的卡片：候选名称、一句话理由、跳往对应卡片的锚点链接。仅此而已。

## 语气与用词

平实、精炼，但架构名词和动词必须直接取自 `/codebase-design` 技能。精炼不是用词漂移的借口。

**严格使用：** module（模块）、interface（接口）、implementation（实现）、depth（深度）、deep（深）、shallow（浅）、seam（接缝）、adapter（适配器）、leverage（杠杆）、locality（局部性）。

**绝不替换：** 不要用 component、service、unit 代替 module；不要用 API、signature 代替 interface；不要用 boundary 代替 seam；指代模块时不要用 layer、wrapper。

**符合风格的表达示例：**

- “Order 接入模块过浅：接口几乎与实现等宽。”
- “定价逻辑跨接缝泄漏。”
- “深化：单一接口，单一测试面。”
- “两个适配器证明接缝真实存在：生产用 HTTP，测试用内存。”

**收益要点**用词汇表术语点出所得：*“局部性：bug 集中于单一模块”*、*“杠杆：一个接口覆盖 N 处调用”*、*“接口收窄，实现吸收薄包装层”*。不要写“更易维护”或“代码更干净”，因为这些词不在词汇表里。

不要含糊铺垫、不要客套开场、不要写“值得注意的是……”。能写成要点的句子就写成要点，能砍掉的要点就砍掉。如果某个词不在 `/codebase-design` 词汇表里，先去词汇表找现成词，不要自己另造。
{%- endif %}
