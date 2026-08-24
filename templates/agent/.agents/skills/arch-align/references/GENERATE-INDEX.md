{% if options["skill_lang"] == "en" %}
# Architecture Index Generation Guide (`docs/architecture/README.md`)

> **Purpose**: When `docs/architecture/README.md` does not exist in the project, follow this guide with user approval to scan existing architecture documents and module structures, generating a standard, universal architecture index file.

---

## 1. Generation Principles

1. **Sole Authoritative Entry Point**: All architecture indexes recognize only `docs/architecture/README.md` without introducing redundant aliases or scattered entry points.
2. **Progressive Disclosure**: Keep the index file itself within 100-200 lines, providing fast semantic routing and module mapping without dumping raw implementation details.
3. **Semantic Hierarchy & Authority**:
   - Mark [Authoritative Specification] (mandatory lifecycles, protocols, core data flows, configuration schemas, etc.).
   - Mark [Design & Implementation] (detailed architecture and implementations for specific business modules/subsystems).
   - Attach [Associated Drafts / Research] (non-authoritative references and exploratory investigations).

---

## 2. Scanning & Extraction Steps

1. **Scan Architecture Directory**:
   - Check all `.md` files and subdirectories under `docs/architecture/`.
   - Check for auxiliary design directories like `docs/design/`.
2. **Extract Metadata**:
   For each architecture document, extract:
   - **Document Path**: Markdown link relative to `docs/architecture/` (e.g. `[system-architecture.md](./system-architecture.md)`).
   - **Core Responsibilities/Content**: Concise 1-2 sentence summary of what problem this architecture solves.
   - **Involved Modules / Code Paths**: Corresponding directories or packages in codebase (e.g. `src/core/`, `packages/server/`).
   - **Authority Level**: [Authoritative Specification] or [Design & Implementation].
3. **Output Standard Template**:
   Generate `docs/architecture/README.md` following this structure:

```markdown
# Project Architecture & Core Specification Index

> **AI / Developer Guide**: This file is the **sole authoritative architecture index** for this project.
> Before starting new features, cross-module changes, or complex debugging, consult the relevant documents for your domain.
> Items marked [Authoritative Specification] must be strictly followed; [Design & Implementation] guides concrete code.

---

## 1. Global System Architecture & Core Skeleton

| Authoritative Architecture Doc | Core Content / Scope | Related Modules / Code Paths | Authority Level |
| :--- | :--- | :--- | :--- |
| [`system-architecture.md`](./system-architecture.md) | Overall layered architecture, core process topology & lifecycle | `src/core/`, `src/bootstrap/` | [Authoritative Specification] |
...

## 2. Core Subsystems & Domain Modules

| Authoritative Architecture Doc | Core Content / Scope | Related Modules / Code Paths | Authority Level |
| :--- | :--- | :--- | :--- |
| [`communication-channel.md`](./communication-channel.md) | Cross-service/process communication protocol & RPC channels | `src/ipc/`, `src/transport/` | [Authoritative Specification] |
...

## 3. Associated Design Drafts & Explorations (Non-authoritative, reference only)

- `docs/design/xxx/` - Experimental feature design draft...
```

4. **Complete Generation**:
   Write to `docs/architecture/README.md` and summarize the indexed sections to the user.
{% else %}
# 架构主索引生成指南 (`docs/architecture/README.md`)

> **用途**：当项目中不存在 `docs/architecture/README.md` 时，通过本指南在用户授权后扫描现有架构文档与模块结构，生成标准、通用的架构主索引文件。

---

## 一、 生成原则

1. **唯一权威入口**：所有架构文档索引只认 `docs/architecture/README.md`，不引入多余的别名或分散的入口。
2. **渐进式披露**：主索引文件自身控制在 100~200 行以内，提供快速语义路由与模块映射，不堆砌大段实现细节。
3. **语义分层与权威度**：
   - 标注【权威规范】（必须严格遵守的生命周期、通信协议、核心数据流、配置体系等）。
   - 标注【设计实现】（针对具体业务模块、子系统的详细架构与实现方案）。
   - 附带【关联草稿 / 历史调研】（项目中供参考但非权威的临时方案与调研）。

---

## 二、 扫描与提炼步骤

1. **扫描架构目录**：
   - 检查 `docs/architecture/` 目录下的所有 `.md` 文件与子目录。
   - 检查是否存在 `docs/design/`、`docs/设计/` 等辅助设计目录。
2. **提取元数据**：
   针对每个架构文档，提取以下信息：
   - **文档路径**：相对 `docs/architecture/` 的 Markdown 链接（如 `[系统架构.md](./系统架构.md)`）。
   - **核心职责/内容**：用 1~2 句话精炼概括该架构解决的问题。
   - **涉及模块 / 代码目录**：代码库中的实际对应目录或包（如 `src/core/`, `packages/server/`）。
   - **规范级别**：【权威规范】或【设计实现】。
3. **输出标准模板**：
   按以下通用格式生成 `docs/architecture/README.md`：

```markdown
# 项目架构与核心规范索引

> **AI / 开发者查阅指南**：本文件是项目的**唯一权威架构主索引**。
> 处理新需求、跨模块改动或定位复杂问题前，请根据当前任务涉及的领域查阅对应文档。
> 标注【权威规范】的必须严格遵守，【设计实现】用于指导具体编码。

---

## 1. 全局系统架构与核心骨架

| 权威架构文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`系统架构.md`](./系统架构.md) | 系统整体分层架构、核心进程/服务拓扑与生命周期 | `src/core/`, `src/bootstrap/` | 【权威规范】 |
...

## 2. 核心子系统与领域模块

| 权威架构文档 | 核心内容 / 职责范围 | 关联模块 / 源码路径 | 规范级别 |
| :--- | :--- | :--- | :--- |
| [`通信通道.md`](./通信通道.md) | 跨服务/进程通信协议与 RPC 交互通道 | `src/ipc/`, `src/transport/` | 【权威规范】 |
...

## 3. 关联设计草稿与探索（非权威架构，仅作参考）

- `docs/design/xxx/` - 某实验性功能设计草稿...
```

4. **完成生成**：
   写入 `docs/architecture/README.md`，并在会话中向用户汇总已收录的架构章节。
{% endif %}
