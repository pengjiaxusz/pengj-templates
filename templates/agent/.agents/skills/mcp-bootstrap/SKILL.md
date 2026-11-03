---
name: mcp-bootstrap
description: >-
{%- if options["skill_lang"] == "en" %}
  Project-local MCP bootstrap: install server binaries, verify them with a real stdio handshake, and keep every client config derived from one .mcp.json. Node built-ins plus uv/git only — no external CLI required.
  Triggers: mcp, mcp doctor, mcp install, mcp bootstrap, mcp check, mcp sync, install mcp, verify mcp, mcp not working, renderdoc-mcp unavailable.
{%- else %}
  项目内 MCP 自举：以 `.mcp.json` 为单一事实源，安装服务器主体、用真实 stdio 握手校验、把各客户端配置由源统一派生。只依赖 Node 与 uv/git，不依赖任何外部 CLI。
  Triggers: mcp, mcp doctor, mcp install, mcp 校验, mcp 安装, 检查 mcp, 安装 mcp, mcp 配置同步, mcp 不生效, renderdoc-mcp 不可用.
{%- endif %}
---

<!-- PENGJ_TEMPLATE_START -->
{% if options["skill_lang"] == "en" %}
# MCP bootstrap — install & verify inside the project

MCP servers are usually configured per client, and many clients only read a global config. This skill keeps the project able to install and verify its own MCP dependencies, without depending on any external tool or repository.

Script: `.agents/skills/mcp-bootstrap/scripts/ensure-mcp.mjs` (Node ≥ 20). Run it from the project root; it also walks up to find `.mcp.json`.

## Three commands

| Command | What it does |
|---|---|
| `node .agents/skills/mcp-bootstrap/scripts/ensure-mcp.mjs` | `doctor` — resolve every command, then a real `initialize` + `tools/list` handshake; exit 1 when something is broken |
| `... install [name]` | Install what is missing (idempotent). `--global` also writes client-wide configs |
| `... sync [--write]` | Regenerate the per-client configs from `.mcp.json` |

`doctor --static` skips spawning processes and only checks paths.

A project without `.mcp.json` is treated as "no project-level MCP": all three commands exit 0 with a notice. Create the file only when you actually add a server.

## Single source of truth

- `.mcp.json` — the declaration; commands are bare names so the file stays portable.
- Derived (never hand-edit): `.agents/mcp.json`, `.trae/mcp.json` (`url` form) and `.agents/mcp_config.json`, `.trae/mcp_config.json` (`serverUrl` form — Antigravity-style clients reject `url`).
- Change `.mcp.json`, then run `sync --write`.

## Adding a server

1. Add an entry to `.mcp.json` (bare command name, no absolute paths).
2. Needs a real install or a side artifact? Add a recipe — either the built-in table in the script, or `scripts/mcp/recipes.local.json` (same shape, project-owned, survives template updates). Pure `npx` / `uvx` servers need no recipe.
3. `sync --write` → `install` → `doctor`.

Recipe fields: `类型` (`uv-git` installs via `uv tool install`; `none` means already on PATH), `源` (`git+https://…`), `入口` (executable name), `扩展` (`从` / `到`, optionally `仅当存在` and its own `启用`), `启用` (`文件` / `键` / `值`), `要求` (host processes that must be closed). Paths accept `~`, `%VAR%`, `$VAR`.

`扩展` also accepts an array, for servers that must land in **several host builds**: stock `qrenderdoc` and a renamed custom build (e.g. `qchahurdd`) each read their own `%APPDATA%\<build name>` directory, and installing into the wrong one fails silently. An item with `仅当存在: <path>` is processed only when that build has actually been run; an item without `启用` inherits the top-level one.

Changing `源` is destructive in a useful way: a source cache pointing at a different URL is re-cloned, and a uv tool whose receipt says `git=…` no longer matches the recipe is reinstalled with `--force`. Switching from upstream to your own fork needs no manual cache cleanup.

Do not judge a cold start by seconds: right after install the first handshake has to write bytecode caches and get past antivirus scanning — measured 8s+ the first time and 1.2s the next, so the timeout defaults to 20s.

## Red lines

- Never hand-edit derived configs; they are generated and reviewed as such.
- Do not write project-specific MCP into global configs. `--global` is opt-in on purpose.
- A green handshake means the server answers — not that it has data. RenderDoc MCP still needs a `.rdc` loaded in RenderDoc.
{% else %}
# MCP 自举 — 项目内安装与校验

MCP 服务器通常按客户端分别配置，而不少客户端只读全局配置。本技能让项目自己就能装好、校验好自己的 MCP 依赖，不依赖任何外部工具或仓库。

脚本：`.agents/skills/mcp-bootstrap/scripts/ensure-mcp.mjs`（Node ≥ 20）。在项目根执行即可，脚本会自动向上查找 `.mcp.json`。

## 三条命令

| 命令 | 作用 |
|---|---|
| `node .agents/skills/mcp-bootstrap/scripts/ensure-mcp.mjs` | `doctor` — 逐条解析命令，再做真实 `initialize` + `tools/list` 握手；有问题时退出码为 1 |
| `... install [name]` | 缺什么补什么（幂等）。加 `--global` 才写客户端全局配置 |
| `... sync [--write]` | 由 `.mcp.json` 重新生成各客户端配置 |

`doctor --static` 不起进程，只查配置与路径。

项目没有 `.mcp.json` 时视为「不需要项目级 MCP」：三条命令都正常退出并给出提示；真要加服务器时再创建该文件。

## 单一事实源

- `.mcp.json` — 声明文件，命令写裸名，保持跨机器可移植。
- 派生物（**严禁手改**）：`.agents/mcp.json`、`.trae/mcp.json`（`url` 形式）与 `.agents/mcp_config.json`、`.trae/mcp_config.json`（`serverUrl` 形式，Antigravity 系客户端不认 `url`）。
- 改完源后执行 `sync --write`。

## 新增一个服务器

1. 在 `.mcp.json` 加条目（写裸命令名，不要绝对路径）。
2. 需要落盘安装或附属产物时补配方 —— 脚本内置表，或项目自己的 `scripts/mcp/recipes.local.json`（结构相同、归项目所有、模板更新不覆盖）；纯 `npx` / `uvx` 型不需要配方。
3. `sync --write` → `install` → `doctor`。

配方字段：`类型`（`uv-git` 走 `uv tool install`；`none` 表示已在 PATH）、`源`（`git+https://…`）、`入口`（可执行名）、`扩展`（`从` / `到`，可带 `仅当存在` 与自带 `启用`）、`启用`（`文件` / `键` / `值`）、`要求`（必须关闭的宿主进程）。路径支持 `~`、`%VAR%`、`$VAR`。

`扩展` 可以写成数组，用于同一个服务器要装进**多个宿主构建**的场景：官方 `qrenderdoc` 与自编译改名版（如 `qchahurdd`）各读自己的 `%APPDATA%\<构建名>` 目录，装错一处就静默不加载。数组项带 `仅当存在: <路径>` 时只有该构建确实装过才处理，避免给没装的机器凭空建目录；不带 `启用` 时继承顶层。

改 `源` 是**破坏性**的：源码缓存若指向别的 URL 会被重新克隆，uv tool 的安装记录 `git=…` 与配方不符时会 `--force` 重装——从上游换成自有 fork 时不必手工清缓存。

冷启动别按秒级判死：刚装完的 uv tool 首次握手要建字节码缓存、还要过杀软扫描，实测首次 8s+、第二次 1.2s，所以默认超时是 20s。

## 红线

- 不要手改派生配置，它们由脚本生成、也应按生成物评审。
- 不要把项目特化的 MCP 写进全局配置；`--global` 是需要时才加的显式开关。
- 握手通过只代表「服务能应答」，不代表有数据：RenderDoc MCP 仍需在 RenderDoc 里打开一个 `.rdc`。
{% endif %}
<!-- PENGJ_TEMPLATE_END -->

{% if options["skill_lang"] == "en" %}
## Project-specific notes (preserved across template updates)

- **MCP servers enabled here**: list the `command` names and what they are for.
- **Extra recipes**: put them in `scripts/mcp/recipes.local.json`; do not edit the managed script above.
- **Environment notes**: host program paths, services that must be started first, etc.
{% else %}
## 项目专属区（模板更新不覆盖）

- **本项目启用的 MCP**：（列出 `command` 名与用途）
- **额外配方**：写到 `scripts/mcp/recipes.local.json`，不要改上面的托管脚本
- **已知环境差异**：（宿主程序路径、需要先启动的服务等）
{% endif %}
