{% if options["skill_lang"] == "en" -%}
# ADR Format

ADRs live in `docs/adr/` and use sequential numbering: `0001-slug.md`, `0002-slug.md`, etc.

Create the `docs/adr/` directory lazily: only when the first ADR is needed.

## Template

```md
# {Short title of the decision}

{1-3 sentences: what's the context, what did we decide, and why.}
```

That's it. An ADR can be a single paragraph. The value is in recording *that* a decision was made and *why*, not in filling out sections.

## Optional sections

Only include these when they add genuine value. Most ADRs won't need them.

- **Status** frontmatter (`proposed | accepted | deprecated | superseded by ADR-NNNN`): useful when decisions are revisited
- **Considered Options**: only when the rejected alternatives are worth remembering
- **Consequences**: only when non-obvious downstream effects need to be called out

## Numbering

Scan `docs/adr/` for the highest existing number and increment by one.

## When to offer an ADR

All three of these must be true:

1. **Hard to reverse**: the cost of changing your mind later is meaningful
2. **Surprising without context**: a future reader will look at the code and wonder "why on earth did they do it this way?"
3. **The result of a real trade-off**: there were genuine alternatives and you picked one for specific reasons

If a decision is easy to reverse, skip it: you'll just reverse it. If it's not surprising, nobody will wonder why. If there was no real alternative, there's nothing to record beyond "we did the obvious thing."

### What qualifies

- **Architectural shape.** "We're using a monorepo." "The write model is event-sourced, the read model is projected into Postgres."
- **Integration patterns between contexts.** "Ordering and Billing communicate via domain events, not synchronous HTTP."
- **Technology choices that carry lock-in.** Database, message bus, auth provider, deployment target. Not every library: just the ones that would take a quarter to swap out.
- **Boundary and scope decisions.** "Customer data is owned by the Customer context; other contexts reference it by ID only." The explicit no-s are as valuable as the yes-s.
- **Deliberate deviations from the obvious path.** "We're using manual SQL instead of an ORM because X." Anything where a reasonable reader would assume the opposite. These stop the next engineer from "fixing" something that was deliberate.
- **Constraints not visible in the code.** "We can't use AWS because of compliance requirements." "Response times must be under 200ms because of the partner API contract."
- **Rejected alternatives when the rejection is non-obvious.** If you considered GraphQL and picked REST for subtle reasons, record it; otherwise someone will suggest GraphQL again in six months.
{%- else -%}
# ADR 格式规范

ADR 存放在 `docs/adr/` 下，采用顺序编号：`0001-slug.md`、`0002-slug.md` 等。

采用懒创建：仅在确实需要第一份 ADR 时才创建 `docs/adr/` 目录。

## 模板

```md
# {决策的简短标题}

{1–3 句话：背景是什么、我们决定了什么、以及为什么。}
```

就这么简单。一份 ADR 完全可以只有一段话。价值在于记录*做出了某个决定*以及*为什么*，而不是填满各种固定栏目。

## 可选小节

仅在确实增加价值时包含，大多数 ADR 不需要：

- **Status** 前置状态（`proposed | accepted | deprecated | superseded by ADR-NNNN`）：当决策后续可能被重新审视时有用
- **Considered Options**：仅当被否决的备选方案值得被记住时才写
- **Consequences**：仅当需要点出不直观的下游影响时才写

## 编号

扫描 `docs/adr/` 下的最大现有编号并加一。

## 何时提议写 ADR

以下三条必须全部满足：

1. **难以逆转**：日后改变主意的成本相当可观
2. **缺少上下文会让人困惑**：未来的读者看到代码时会纳闷“他们当初到底为什么这么写？”
3. **真实权衡的结果**：确实存在像样的备选方案，而你基于特定理由选中了其中之一

如果某个决策很容易反悔，跳过：到时候直接改就是了。如果并不令人意外，没人会问为什么。如果没有真正的替代方案，那也只是“做了显而易见的事”，没什么好记的。

### 哪些情况符合条件

- **架构形态。** “我们采用 monorepo。”“写模型采用事件溯源，读模型投影到 Postgres。”
- **上下文之间的集成模式。** “Ordering 与 Billing 通过领域事件通信，而非同步 HTTP。”
- **带有锁定效应的技术选型。** 数据库、消息总线、认证服务商、部署目标。不是每一个库都要记，只记那些换掉需要一个季度的选型。
- **边界与作用域决策。** “客户数据归 Customer 上下文所有；其他上下文仅通过 ID 引用。”明确说“不做什么”和“做什么”同样有价值。
- **有意偏离常规显而易见路径的做法。** “我们手写 SQL 而不用 ORM，因为 X。”任何理性读者第一眼会默认反过来写的设计。这能防止下一位工程师顺手把你刻意为之的设计“修掉”。
- **代码中看不见的约束。** “出于合规要求不能用 AWS。”“受合作方 API 契约限制，响应时间必须低于 200ms。”
- **理由不直观的被否决方案。** 如果你评估过 GraphQL 并因微妙原因选了 REST，记下来；否则六个月后还会有人再提一次 GraphQL。
{%- endif %}
