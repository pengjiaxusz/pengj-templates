---
name: arch-align
description: >-

  编写、更新并对齐超紧凑、Agent 原生（面向 AI Agent 而非人类读者）的架构文档（docs/architecture/）。当需要为 AI Agent 创建架构文档、在编码或重构前核对系统边界、生成架构索引，或用户提及 arch-align、架构对齐、查阅架构、架构文档、架构先行、Agent 友好架构规范时使用。 Triggers: arch-align, 架构对齐, 查阅架构, 架构文档, 架构先行, architecture-alignment, 先看架构.

---


# 架构对齐（Arch Align）

编写与使用高密度、低 token 的架构文档，服务对象是 AI Agent 而非人类读者。用电报式约束、符号锚点、取舍锁定与可执行守卫测试，取代冗长散文。

## 适用场景

- 开始跨模块功能、重构或复杂根因排查
- 创建或更新 `docs/architecture/README.md`，或 `docs/architecture/<module>.md` 领域规范
- 把冗长的人类设计文档压缩为低 token 的 Agent 架构规范
- 让实施方案对齐架构红线、代价取舍与守卫测试

## Agent 原生文档核心原则

- **只记增量（Delta-Only）**：Agent 一次工具调用就能从文件名或类型签名推断的内容，一律不写。只记录跨文件拓扑、非显然的取舍与硬不变量。
- **电报式语法（Telegraphic）**：禁止叙述段落、背景故事与宽 Markdown 表格。使用 `KEY: Value`、`A -> B [async] -> C` 链路与严格标签（`[IN]`、`[OUT]`、`[MUST]`、`[NEVER]`、`[WHY]`、`[GUARD]`）。
- **严格 token 预算**：`docs/architecture/README.md` 不超过 60 行，每篇 `docs/architecture/<module>.md` 不超过 80 行。
- **删除测试（Deletion Test）**：写任何一行前先问：删掉这行，Agent 是否会做出错误的架构决策或浪费超过 2 次检索调用？若不会，删掉。

## 步骤

### 步骤 1：定位主路由索引

1. 检查 `docs/architecture/README.md` 是否存在（唯一权威入口）。
2. **若缺失**：询问用户是否扫描代码库并生成 `docs/architecture/README.md`。用户同意后，读取 `references/GENERATE-INDEX.md`，在 60 行内生成索引。
3. **若已存在**：阅读 `docs/architecture/README.md`，将当前任务映射到 1~2 篇目标领域架构文档。

### 步骤 2：加载或生成目标领域规范

1. **若目标领域文档已存在**：只阅读 `docs/architecture/` 中直接匹配的 1~2 篇，不预加载无关模块文档。
2. **若目标领域文档缺失**：检查是否存在草稿笔记（如 `docs/design/`），询问用户是否为该模块生成 Agent 原生规范。用户同意后，读取 `references/GENERATE-DOC.md`，在 80 行内生成 `docs/architecture/<module>.md`，并登记到 `docs/architecture/README.md`。

### 步骤 3：编码前输出紧凑对齐简报

修改代码前，输出 5~10 行对齐块确认约束：

```text
[ARCH-ALIGN]
- Doc: docs/architecture/<module>.md (Status: LANDED | PARTIAL | BLUEPRINT)
- Anchors: <入口路径与核心类型>
- Trade-off Lock: <禁止以重构之名破坏的既有取舍>
- Hard Invariants: <1-3 条 [NEVER]/[MUST] 规则>
- Guardrail Cmd: <改动后必须执行的确切测试命令>
```

### 步骤 4：验证守卫并同步增量

1. 完成代码改动后，执行架构文档中指定的 `Guardrail Cmd`。
2. 只有在锚点、边界、取舍或守卫测试发生变化时，才更新 `docs/architecture/<module>.md`。

## 详细参考

- [架构主索引生成指南](references/GENERATE-INDEX.md)
- [领域架构文档生成指南](references/GENERATE-DOC.md)

## 常见坑

- **代码复制漂移**：不要把实现片段或完整结构体定义贴进架构文档。改用 `path/to/file.ext::SymbolName` 引用，文档才能保持极小且不过期。
- **误删刻意取舍**：Agent 天然倾向“清理”不合惯例的代码。先看 `[TRADEOFFS]` 节，避免破坏刻意的性能或编译期绕行方案。
- **没有守卫的模糊规则**：不要写“保持低耦合”这类无法验证的散文规则。每条 `[NEVER]`/`[MUST]` 都必须点名具体模块/类型，并绑定 `[GUARD]` 测试或 lint 命令。
