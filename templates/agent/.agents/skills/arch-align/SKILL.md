---
name: arch-align
description: >-
{% if options["skill_lang"] == "en" %}
  Universal architecture search and constraint alignment skill before feature development and complex debugging. Discovers the authoritative index at docs/architecture/README.md via progressive disclosure; asks the user and generates missing indexes/docs following guidelines; aligns architectural invariants and boundaries before coding. Use when inspecting architecture, planning features, modifying cross-module code, or when the user mentions "check architecture", "architecture docs", or "arch-align". Triggers: arch-align, architecture-alignment, check architecture, architecture docs.
{% else %}
  需求开发与复杂问题定位前的通用架构检索与约束对齐技能。通过渐进式披露检索 docs/architecture/README.md 权威架构主索引；若缺失主索引或领域架构文档，询问用户并按指南生成；对齐架构红线与系统边界后再展开实施。当需要查阅架构、新需求规划、跨模块修改、架构对齐、或用户要求"先看架构"/"架构文档"时使用。 Triggers: arch-align, 架构对齐, 查阅架构, 架构文档, 架构先行, architecture-alignment, 先看架构.
{% endif %}
---

{% if options["skill_lang"] == "en" %}
# Architecture Alignment (arch-align)

## Quick Start

Before starting feature design, refactoring, or diagnosing complex cross-module issues, follow this workflow to align on architecture:

```
[Check docs/architecture/README.md] ──► [Inspect domain architecture docs] ──► [Extract invariants & boundaries] ──► [Proceed to implementation]
```

## Workflow

### Step 1: Locate authoritative architecture index

1. Check if `docs/architecture/README.md` exists (**the sole authoritative architecture index; do not look for alternative file names**).
2. **If the index file does NOT exist**:
   - Ask the user: "Detected that `docs/architecture/README.md` does not exist. Would you like me to scan existing architecture docs and codebase structure to generate the index file?"
   - If user agrees, read reference guide `references/GENERATE-INDEX.md` to generate the index, then proceed.
   - If user declines, fall back to current context and general conventions.
3. **If the index file exists**:
   - Read `docs/architecture/README.md` to understand subsystem and domain architecture distribution across the project.

### Step 2: Identify domain docs & progressive inspection

1. Search the main index for the most relevant authoritative architecture doc based on modules/directories/keywords involved in current task.
2. **If relevant architecture doc is found**:
   - Accurately read 1-2 corresponding docs.
3. **If no authoritative architecture doc is indexed for this domain**:
   - Check if the project has relevant design drafts or research notes (e.g. `docs/design/` or `docs/设计/`).
   - Ask the user: "Authoritative architecture documentation is missing for this domain. Would you like me to generate a standard architecture doc from existing code implementation and drafts?"
   - If user agrees, read reference guide `references/GENERATE-DOC.md` to generate the new document, and **register it into the `docs/architecture/README.md` index table**.

### Step 3: Extract invariants & enter development

After reading the relevant architecture docs, briefly summarize to the user:
1. **Reference Docs**: Paths of the aligned architecture documents.
2. **Core Invariants & Boundaries**: Mandatory architectural constraints for this task (e.g., data flow direction, module boundaries, concurrency and lifecycle rules, forbidden anti-patterns).
3. **Implementation Plan**: Proposed coding or planning steps aligned with above constraints.

## Detailed Guides (Progressive Disclosure)

- [Architecture Index Generation Guide](references/GENERATE-INDEX.md)
- [Domain Architecture Doc Generation Guide](references/GENERATE-DOC.md)
{% else %}
# 架构对齐 (Architecture Alignment)

## 快速开始

在开始新功能设计、系统重构或排查复杂跨模块问题前，按以下流程快速完成架构对齐：

```
[检查 docs/architecture/README.md] ──► [精准查阅领域架构文档] ──► [提炼架构红线与系统边界] ──► [进入需求实施]
```

## 工作流

### 步骤 1：检索权威架构主索引

1. 检查 `docs/architecture/README.md` 是否存在（**唯一权威架构主索引，不认其他文件名**）。
2. **若主索引文件不存在**：
   - 询问用户：“检测到 `docs/architecture/README.md` 架构主索引不存在，是否需要我扫描现有架构文档与代码结构并生成该索引文件？”
   - 用户同意后，读取参考指南 `references/GENERATE-INDEX.md` 生成主索引，再继续下一步。
   - 用户拒绝则退回基于当前上下文或通用规范开发。
3. **若主索引文件已存在**：
   - 阅读 `docs/architecture/README.md`，获取项目各子系统与领域模块的架构分布。

### 步骤 2：判断领域文档与渐进式查阅

1. 根据当前任务涉及的模块/目录/关键词，从主索引中查找最相关的权威架构文档。
2. **若找到了相关架构文档**：
   - 精准读取 1~2 篇对应文档。
3. **若主索引中未收录对应领域的权威架构文档**：
   - 检查项目中是否有相关的设计草稿或调研记录（如 `docs/design/` 或 `docs/设计/`）。
   - 询问用户：“当前领域缺失权威架构文档，是否需要结合现有代码实现与已有草稿，生成该领域的标准架构文档？”
   - 用户同意后，读取参考指南 `references/GENERATE-DOC.md` 生成新文档，并**将其登记回 `docs/architecture/README.md` 索引表格**。

### 步骤 3：提取架构红线并进入开发

阅读相关架构文档后，向用户简要陈述：
1. **参考文档**：已对齐的架构文档路径。
2. **核心红线与边界**：本次任务必须严格遵守的架构原则（如：数据流向、模块边界、并发与生命周期约束、禁止的反模式等）。
3. **实施方案**：基于上述约束展开的后续编码或规划。

## 详细指南（渐进式披露）

- [架构主索引生成指南](references/GENERATE-INDEX.md)
- [领域架构文档生成指南](references/GENERATE-DOC.md)
{% endif %}
