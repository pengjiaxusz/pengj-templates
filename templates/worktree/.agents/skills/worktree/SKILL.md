---
name: worktree
description: >-
{% if options["skill_lang"] | default("zh") == "en" %}
  Parallel development with git worktrees sharing the main worktree's `target/`. Covers creating a worktree in one command (branch + sibling directory + submodules), how the shared directory is derived and injected as `CARGO_TARGET_DIR`, how to verify it actually took effect, the artifact-overwrite red line, and the `cargo clean` blast radius. Use when running parallel worktrees, when builds feel slower than expected in a worktree, when disk usage explodes with duplicate target directories, or when a shared target directory silently degraded into an independent one.
  Triggers: worktree, git worktree, parallel development, shared target, CARGO_TARGET_DIR, shared build directory, duplicate target, disk usage, wt-create, wt-check, wt-target.
{% else %}
  多工作树并行开发与共享编译目录。涵盖一条命令新建工作树（分支 + 同级目录 + 子模块补齐）、共享目录如何推导并注入为 `CARGO_TARGET_DIR`、如何验证真的生效、产物互相覆盖的红线，以及 `cargo clean` 的波及范围。当需要并行开工作树、发现工作树里编译变慢、磁盘被重复 target 目录撑爆，或怀疑共享目录已静默退化成独立目录时使用。
  Triggers: worktree, git worktree, 工作树, 共享编译目录, 共享 target, CARGO_TARGET_DIR, 重复编译, 磁盘占用, 产物覆盖, wt-create, wt-check, wt-target.
{% endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] | default("zh") == "en" %}
# worktree — parallel worktrees sharing one build directory

This project uses git worktrees for parallel development, and every worktree builds into **the main worktree's `target/`**. Dependencies compile once instead of once per worktree, and artifacts stay inside the project's own directory tree — nothing piles up in a system-wide shared folder.

## Creating a worktree

```sh
just wt-create feature-a
```

That one command creates the branch, the sibling directory (`<main-repo>-feature-a`, same parent as the main repo), wires the shared build directory, and fills in submodules through the repository init entry point. It refuses to run when the directory or the branch already exists — it never reuses or overwrites an existing worktree.

## How the shared directory works

The path is **the main worktree's `target/`**, derived from `git worktree list --porcelain` (its first entry is the main worktree). Every worktree of the same repository therefore derives the same value — a feature branch computes the main worktree's path, not its own. The `just` build recipes inject it as `CARGO_TARGET_DIR`, and the health check reads the same value — there is exactly one derivation.

Because the main worktree's `target/` *is* the shared directory, a plain `cargo build` there needs no injection at all; only linked worktrees are redirected.

```sh
just wt-target    # print the shared directory
just wt-check     # verify: exists, writable, not overridden elsewhere
just wt-fix       # create it when missing (only fills gaps, never overwrites)
```

Do **not** set `target-dir` in `.cargo/config.toml`: that file is merged structurally by the template, and a duplicate table header with your own settings makes the whole file skip writing. Do not hand-build NTFS junctions or symlinks either — an unverifiable link is exactly what silently degrades.

## Red line: final artifacts are overwritten

Intermediate artifacts are safe to share (Cargo's fingerprinting keeps them consistent). **Final binaries keep a fixed path inside the shared directory, so two worktrees building at the same time overwrite each other's output — and so does the main worktree's own build.**

Isolation is the project's decision, not the template's: copy the artifact out of the shared directory, or configure your own output directory. Never assume `target/release/<binary>` belongs to the worktree you are sitting in.

## Cost: `cargo clean` hits every worktree

The shared directory lives inside the main worktree, so `cargo clean` (or `git clean -xdf`) run there deletes the intermediate artifacts of **all** worktrees — every one of them pays a full rebuild. Treat cleaning as a repository-wide action. If you only need to rebuild your own crate, `touch` a source file instead.

## When a build feels slow

A shared directory that silently degraded to an independent one looks exactly like "the build got a little slower". Run `just wt-check` first — it reports missing, unwritable and overridden states instead of leaving you guessing. `just doctor --fix` also creates the directory when it is missing.
{% else %}
# worktree —— 多工作树共享编译目录

本项目用 git 工作树做并行开发，所有工作树都编译到**主工作树的 `target/`**：依赖只编译一次，不必每个工作树各编一遍；产物仍留在项目自己的目录树里，不会堆到系统级共享目录。

## 新建工作树

```sh
just wt-create feature-a
```

一条命令建分支、建同级目录（`<主仓名>-feature-a`，与主仓同级）、接线共享编译目录，并通过仓库初始化入口把子模块补齐。目录或分支已存在时它直接报错——不复用、不覆盖已有工作树。

## 共享目录怎么来的

路径**就是主工作树的 `target/`**，由 `git worktree list --porcelain` 推导（第一条即主工作树）。因此同一仓库的所有工作树推出同一个值——你站在特性分支的工作树里，算出来的是主工作树的路径，不是它自己的。`just` 构建配方把它注入为 `CARGO_TARGET_DIR`，自检读的是同一个值——推导逻辑只有一份。

正因为主工作树的 `target/` 本身就是共享目录，在那里裸跑 `cargo build` 根本不需要注入；被重定向的只有链接工作树。

```sh
just wt-target    # 打印共享目录
just wt-check     # 校验：存在、可写、没有被指向别处的覆盖值
just wt-fix       # 缺失时创建（只补缺、不覆盖）
```

**不要**在 `.cargo/config.toml` 里写 `target-dir`：那个文件由模板做结构化合并，与你自己写的同名表头撞车会让整文件不予写盘。也不要手工建 NTFS Junction 或符号链接——无法被校验的链接正是会静默退化的那一种。

## 红线：最终产物会被覆盖

中间产物可以安全共享（Cargo 的指纹机制保证一致）。**最终二进制在共享目录里路径固定，两个工作树同时构建会互相覆盖产物——主工作树自己的构建同样会覆盖。**

隔离是项目的决定，不是模板的：把产物复制出共享目录，或自己配置输出目录。不要假设 `target/release/<二进制>` 属于你当前所在的工作树。

## 代价：`cargo clean` 会波及所有工作树

共享目录就在主工作树里，所以在那里跑 `cargo clean`（或 `git clean -xdf`）会删掉**所有**工作树的中间产物，每个都得全量重建。请把清理当成仓库级操作；只想重建自己的 crate，就 `touch` 一个源文件。

## 编译变慢时

共享目录悄悄退化成独立目录，表现就是「编译好像慢了一点」。先跑 `just wt-check`：它会报告缺失、不可写与被覆盖三种状态，而不是让你猜。`just doctor --fix` 也会在目录缺失时创建它。
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] | default("zh") == "en" %}
<!-- Project-specific notes go below (outside the block; preserved on update). -->
## Project-specific worktree notes

> Record here: which worktrees exist and what each is for, how your project isolates run artifacts, and any per-worktree configuration that is not derived automatically.
{% else %}
<!-- 项目专属区域：模板更新只替换上方托管块，本区域归项目所有、完整保留。 -->
## 项目专属工作树说明

> 在此记录：本项目有哪些工作树、各自负责什么，运行产物如何隔离，以及任何无法自动推导的按工作树配置。
{% endif %}
