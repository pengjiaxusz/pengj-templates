{% if options["skill_lang"] == "en" -%}
# UI Prototype

Generate **several radically different UI variations** on a single route, switchable from a floating bottom bar. The user flips between variants in the browser, picks one (or steals bits from each), then throws the rest away.

If the question is about logic/state rather than what something looks like, this is the wrong branch. Use [LOGIC.md](LOGIC.md).

## When this is the right shape

- "What should this page look like?"
- "I want to see a few options for this dashboard before committing."
- "Try a different layout for the settings screen."
- Any time the user would otherwise spend a day picking between three vague mockups in their head.

## Two sub-shapes: strongly prefer sub-shape A

A UI prototype is much easier to judge when it's **butting up against the rest of the app**: real header, real sidebar, real data, real density. A throwaway route on its own is a vacuum: every variant looks fine in isolation. Default to sub-shape A whenever there's a plausible existing page to host the variants. Only reach for sub-shape B if the prototype genuinely has no nearby home.

### Sub-shape A: adjustment to an existing page (preferred)

The route already exists. Variants are rendered **on the same route**, gated by a `?variant=` URL search param. The existing data fetching, params, and auth all stay. Only the rendering swaps. This is the default; pick it unless there's a specific reason not to.

If the prototype is for something that doesn't yet have a page but *would naturally live inside one* (a new section of the dashboard, a new card on the settings screen, a new step in an existing flow), it's still sub-shape A. Mount the variants inside the host page.

### Sub-shape B: a new page (last resort)

Only use this when the thing being prototyped genuinely has no existing page to live inside (e.g. an entirely new top-level surface, or a flow that can't be embedded anywhere sensible).

Create a **throwaway route** following whatever routing convention the project already uses. Don't invent a new top-level structure. Name it so it's obviously a prototype (e.g. include the word `prototype` in the path or filename). Same `?variant=` pattern.

Before committing to sub-shape B, sanity-check: is there really no existing page this could be embedded in? An empty route hides design problems that a populated one would expose.

In both sub-shapes the floating bottom bar is identical.

## Process

### 1. State the question and pick N

Default to **3 variants**. More than 5 stops being radically different and starts being noise, so cap there.

Write down the plan in one line, in the prototype's location or a top-of-file comment:

> "Three variants of the settings page, switchable via `?variant=`, on the existing `/settings` route."

This works whether the user is here to push back or not.

### 2. Generate radically different variants

Draft each variant. Hold each one to:

- The page's purpose and the data it has access to.
- The project's component library / styling system (TailwindCSS, shadcn, MUI, plain CSS, whatever).
- A clear exported component name, e.g. `VariantA`, `VariantB`, `VariantC`.

Variants must be **structurally different**: different layout, different information hierarchy, different primary affordance, not just different colours. Three slightly-tweaked card grids isn't a UI prototype, it's wallpaper. If two drafts come out too similar, redo one with explicit "do not use a card grid" guidance.

### 3. Wire them together

Create a single switcher component on the route:

```tsx
// pseudo-code, adapt to the project's framework
const variant = searchParams.get('variant') ?? 'A';
return (
  <>
    {variant === 'A' && <VariantA {...data} />}
    {variant === 'B' && <VariantB {...data} />}
    {variant === 'C' && <VariantC {...data} />}
    <PrototypeSwitcher variants={['A','B','C']} current={variant} />
  </>
);
```

For sub-shape A (existing page): keep all the existing data fetching above the switcher; only the rendered subtree changes per variant.

For sub-shape B (new page): the throwaway route under `/prototype/<name>` mounts the same switcher.

### 4. Build the floating switcher

A small fixed-position bar at the bottom-centre of the screen with three pieces:

- **Left arrow**: cycles to the previous variant (wraps around).
- **Variant label**: shows the current variant key and, if the variant exports a name, that name too. e.g. `B (Sidebar layout)`.
- **Right arrow**: cycles forward (wraps around).

Behaviour:

- Clicking an arrow updates the URL search param (use the framework's router, e.g. `router.replace` on Next, `navigate` on React Router, etc) so the variant is shareable and reload-stable.
- Keyboard: `←` and `→` arrow keys also cycle. Don't intercept arrow keys when an `<input>`, `<textarea>`, or `[contenteditable]` is focused.
- Visually distinct from the page (e.g. high-contrast pill, subtle shadow) so it's obviously not part of the design being evaluated.
- Hidden in production builds: gate on `process.env.NODE_ENV !== 'production'` or an equivalent check, so a stray prototype merge can't ship the bar to users.

Put the switcher in a single shared component so both sub-shapes can reuse it. Locate it wherever shared UI lives in the project.

### 5. Hand it over

Surface the URL (and the `?variant=` keys). The user will flip through whenever they get to it. The interesting feedback is usually **"I want the header from B with the sidebar from C"**, which is the actual design they want.

### 6. Capture the answer and clean up

Once a variant has won, capture the answer (which variant and why), then capture the prototype the way the [SKILL](SKILL.md) describes. Fold the winner into the real code and move the rest onto the throwaway branch, not into main:

- **Sub-shape A**: fold the winner into the existing page; drop the losing variants and the switcher from main.
- **Sub-shape B**: promote the winning variant to a real route; drop the throwaway route and the switcher from main.

The full set of variants is the primary source, so it lands on the throwaway branch, not the bin, since variant components and the switcher left in the main branch rot fast and confuse the next reader.

## Anti-patterns

- **Variants that differ only in colour or copy.** That's a tweak, not a prototype. Real variants disagree about structure.
- **Sharing too much code between variants.** A shared `<Header>` is fine; a shared `<Layout>` defeats the point. Each variant should be free to throw out the layout.
- **Wiring variants to real mutations.** Read-only prototypes are fine. If a variant needs to mutate, point it at a stub: the question is "what should this look like", not "does the backend work".
- **Promoting the prototype directly to production.** The variant code was written under prototype constraints (no tests, minimal error handling). Rewrite it properly when you fold it in.
{%- else -%}
# UI 原型（UI Prototype）

在同一个路由上生成**几套截然不同的 UI 变体**，通过底部悬浮切换栏切换。用户在浏览器里来回翻看各套方案、选中一套（或各取所长），然后把其余的丢掉。

如果问题围绕逻辑/状态而不是界面长什么样，那走错了分支，请改用 [LOGIC.md](LOGIC.md)。

## 适用场景

- “这个页面应该长什么样？”
- “定下来之前，我想先看几个这个仪表盘的布局选项。”
- “给设置页换几种不同的布局试试。”
- 任何用户原本可能要花一天时间在脑子里纠结三套模糊线框稿的场景。

## 两种子形态：强烈优先选子形态 A

当 UI 原型**紧挨着应用其余部分一起运行**时，判断起来容易得多：真实的顶部栏、真实的侧边栏、真实的数据、真实的信息密度。孤立的新建路由是一块真空，每个方案单独看都挺好。只要存在合理的现有页面能承载这些变体，默认选子形态 A；只有当原型实在找不到寄宿页面时，才用子形态 B。

### 子形态 A：改造现有页面（首选）

路由已经存在。多个变体渲染在**同一个路由**上，通过 URL 查询参数 `?variant=` 控制切换。原有的数据请求、参数与鉴权全部保留，只替换渲染部分。这是默认方案，除非有明确理由否则一律选它。

如果原型要做的是尚未单独成页、但*天然应该嵌入某个现有页面*的东西（仪表盘上的新区块、设置页的新卡片、现有流程里的新步骤），同样属于子形态 A，把变体挂载进宿主页面即可。

### 子形态 B：新建独立页面（最后手段）

仅当原型对象确实没有任何现有页面可寄宿时使用（例如全新的顶层界面，或无法嵌入任何现有处的流程）。

遵循项目现有的路由约定创建一个**一次性路由**，不要发明新的顶层结构。命名要一眼看出是原型（例如在路径或文件名中包含 `prototype`），同样采用 `?variant=` 模式。

选子形态 B 之前先自查一遍：真的没有任何现有页面可以嵌入吗？空白路由会掩盖在真实页面里一眼就能暴露的排版问题。

两种子形态共用完全相同的底部悬浮切换栏。

## 流程

### 1. 明确问题并选定变体数量 N

默认生成 **3 个变体**。超过 5 个就不再是“截然不同”而变成噪音了，因此以此为上限。

在原型所在位置或文件顶部注释写下一句话计划：

> “设置页的三个变体，挂载在现有 `/settings` 路由上，通过 `?variant=` 切换。”

无论用户此时是否在场提出异议，这样做都清晰可靠。

### 2. 生成截然不同的变体

起草每个变体，确保每一套都遵循：

- 页面的核心目标与它能访问到的数据。
- 项目现有的组件库 / 样式体系（TailwindCSS、shadcn、MUI、纯 CSS 等）。
- 清晰的导出组件名，例如 `VariantA`、`VariantB`、`VariantC`。

变体之间必须是**结构级差异**：不同的布局、不同的信息层级、不同的主交互载体，而不只是换个颜色。三套微调过的卡片网格不是 UI 原型，而是壁纸。如果两套草稿长得太像，带着“严禁使用卡片网格”的明确约束重做其中一套。

### 3. 串联到路由上

在路由上放一个统一切换器：

```tsx
// 伪代码，按项目所用框架适配
const variant = searchParams.get('variant') ?? 'A';
return (
  <>
    {variant === 'A' && <VariantA {...data} />}
    {variant === 'B' && <VariantB {...data} />}
    {variant === 'C' && <VariantC {...data} />}
    <PrototypeSwitcher variants={['A','B','C']} current={variant} />
  </>
);
```

对于子形态 A（现有页面）：把原有数据获取全部留在切换器上方，每个变体只替换下方渲染子树。

对于子形态 B（新页面）：在 `/prototype/<name>` 下的一次性路由挂载同样的切换器。

### 4. 构建悬浮切换栏

在屏幕底部居中的固定定位小条，包含三个部分：

- **左箭头**：切换到上一个变体（循环）。
- **变体标签**：显示当前变体键名，如果导出了名称也一并显示，例如 `B (Sidebar layout)`。
- **右箭头**：切换到下一个变体（循环）。

交互行为：

- 点击箭头更新 URL 查询参数（使用框架路由，如 Next 的 `router.replace`、React Router 的 `navigate` 等），保证链接可分享且刷新不丢失。
- 键盘支持：`←` 和 `→` 方向键也能切换；当 `<input>`、`<textarea>` 或 `[contenteditable]` 处于聚焦状态时不要拦截方向键。
- 视觉上与页面本身明显区分（例如高对比度胶囊条、轻微阴影），让人一眼看出它不属于被评估的设计。
- 生产构建隐藏：用 `process.env.NODE_ENV !== 'production'` 或等效条件门控，防止原型误合入主干后把切换栏带给线上用户。

把切换器做成单个共享组件，使两种子形态都能复用，放在项目共享 UI 组件目录下。

### 5. 交付试用

把 URL（以及各 `?variant=` 参数）交给用户。用户有空时会逐个翻看。最有价值的反馈通常是**“我想要 B 的顶部栏配 C 的侧边栏”**——那往往才是他们真正想要的设计。

### 6. 沉淀结论并清理

一旦某个变体胜出，先记录结论（选了哪个变体、为什么），再按 [SKILL.md](SKILL.md) 的方式保存原型。把胜出方案吸收进正式代码，其余全部移到一次性分支，不要留在 main：

- **子形态 A**：把胜出变体合入现有页面；从 main 中移除落选变体与切换器。
- **子形态 B**：把胜出变体升级为正式路由；从 main 中移除一次性路由与切换器。

完整的一组变体是一手来源，所以它进一次性分支而不是回收站——因为留在 main 分支里的变体组件和切换器会迅速腐烂并误导后来的读者。

## 反模式

- **只在颜色或文案上做差异的变体。** 那叫微调，不叫原型。真正的变体必须在结构上意见相左。
- **在变体之间共享过多代码。** 共享 `<Header>` 可以，共享 `<Layout>` 就失去了意义。每个变体都应该能彻底推翻布局。
- **把变体接到真实的写操作变更上。** 只读原型很好；如果变体需要触发变更，接到桩函数（stub）上即可——问题问的是“界面该长什么样”，不是“后端通不通”。
- **把原型代码直接转正上线。** 变体是在原型约束下写的（没有测试、极简错误处理）。合入正式代码时要按生产标准重写。
{%- endif %}
