# pengj-templates

> English | [简体中文](README.zh-CN.md)

Layered template generation and sync tool: repository templates are organized into "layers", you pick the layers you need and get a new repository out of it; once a template updates, one command syncs it to every project built on top of it.

## Core idea: layers

Template content is split into layers by applicability. Layers compose freely at generation time, and a later layer overrides same-named files from an earlier one:

| Layer | Content | Applies to |
| --- | --- | --- |
| `common` | Shared by every repository: `.gitignore`, `scripts/doctor.*` environment check | All repositories |
| `just` | The single root `justfile` entry point: managed framework block + project-owned area outside the block, importing language/queue recipes per layer; provides the `doctor` entry point | Repositories that want one unified task entry point |
| `bque` | Windows global build queueing: `just/bque.just` (`queue-*` / `queue-doctor` / `q-run` / `q-bench`) | Repositories that need cross-worktree queueing and clean benchmarks |
| `gate` | Commit gate: `.gate.toml` config + `.githooks/*` hooks (conventional-commit validation, code language guard, rustfmt); pure Rust, zero third-party dependencies; implies the `ci` layer for server-side fallback | Repositories that want Git hooks |
| `ci` | Server-side gate: GitHub Actions validates commit subjects and the code language on pull requests, merge queue and main pushes, plus a required-status-check ruleset and a pull request checklist | Repositories hosted on GitHub |
| `agent` | AI coding assistant standards (`AGENTS.md`), generating Rust / Chinese-programming conventions per the selected layers and options | Repositories where an agent should follow the standards |
| `vscode` | VS Code workspace configuration: recommended extensions, clippy for the Rust checks under Chinese programming | Repositories developed in VS Code |
| `rust` | Rust-specific: `Cargo.toml`, clippy/rustfmt, optional sccache, `just/rust.just` | Rust projects |
| `rust-workspace` | Rust multi-crate workspace: workspace `Cargo.toml`, optional sccache/lld, `just/rust-workspace.just` | Rust workspaces |
| ... | Extend as needed (just add a directory + `layer.toml`) | ... |

Template files support minijinja placeholders (`{{ project_name }}`, `{{ project_slug }}`, `{{ year }}`).

## Tech stack

- **Engine**: Rust (`crates/core`) — layer merge, rendering, diff and update logic; templates are loaded from a runtime directory (`PENGJ_TEMPLATES` > `templates/` in the current directory > `templates/` next to the executable > `templates/` two levels above the executable > platform data directory `pengj-templates/templates/`)
- **MCP installer**: Rust (`crates/mcp`) — recipe catalog, client adapters and real handshake checks; machine-level (cross-project) configuration
- **Commit gate**: Rust (`crates/gate`) — conventional-commit validation, code language guard, staged-file rustfmt; zero third-party dependencies, now an internal library reached only through `pengj-templates-cli gate`
- **CLI**: clap (`crates/cli`, command `pengj-templates-cli`), the single entry point of the whole repository; the built-in `gate` subcommand forwards gate arguments verbatim

## Usage

### One-liner install (recommended)

One command on a fresh machine installs the **template CLI + commit gate** (the same binary) — there is no separate `gate` to install:

```powershell
irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex   # Windows
```

```bash
curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh   # Linux / macOS
```

The installer pulls the bundle matching your "os-arch" from the latest Release (the `pengj-templates-cli.exe` executable plus the accompanying `templates/` tree), drops it into the platform data directory (Windows `%LOCALAPPDATA%\pengj-templates`, Unix `${XDG_DATA_HOME:-~/.local/share}/pengj-templates`) and puts `bin/` on `PATH`. The installer is a standalone script rather than a rendered template, so its messages follow the **system language**: Chinese on a Chinese UI / locale (`zh-*`), English otherwise. (The generated project is a separate matter — its copy follows `--skill-lang`.) Once installed:

```bash
pengj-templates-cli --version
pengj-templates-cli gate --help          # the gate ships with the CLI
pengj-templates-cli create my-app --layers rust
```

> An older install is replaced in place; the template tree and the binary are upgraded together, so you never end up with a new CLI against stale templates. To point at a different template root temporarily, set the `PENGJ_TEMPLATES` environment variable.

### CLI

The examples below use `cargo run` (development mode). If you installed the CLI, just write `pengj-templates-cli`.

> The CLI's own messages are written in Chinese by design (they follow the `skill_lang=zh` axis of this repository). The generated project is language-neutral: `--skill-lang en --no-commit-zh` makes every generated file, log line and commit message English.

```bash
cargo run -p pengj-templates-cli -- list-layers                          # list the available layers
cargo run -p pengj-templates-cli -- create my-app --layers rust          # generate common + rust
cargo run -p pengj-templates-cli -- create my-app --layers rust --edition 2024 --channel nightly --no-lld
cargo run -p pengj-templates-cli -- create my-app --layers agent --skills commit,caveman   # only the selected skills
cargo run -p pengj-templates-cli -- create my-app --layers rust --skill-lang en --no-commit-zh  # all-English project: every string and commit message in English
cargo run -p pengj-templates-cli -- update --dir ./my-app                # sync template updates
cargo run -p pengj-templates-cli -- add-layer --dir ./my-app --layers just,bque  # add layers to an existing project
cargo run -p pengj-templates-cli -- audit --dir ./my-app                 # audit half-wired setups and managed-block drift
cargo run -p pengj-templates-cli -- check-lang --dir ./my-app           # audit docs follow the language axis (en: no Han; zh: Chinese copy)
```

**just / bque assembly**: the root `justfile` is produced by the `just` layer; its framework area lives in the `# PENGJ_TEMPLATE_START/END` managed block (`set shell`, `default`, conditional `import` per layer), while the project-owned area outside the block belongs to the user and `update` only replaces the block. The `bque` / `gate` / `rust` / `rust-workspace` layers produce `just/bque.just`, `just/gate.just`, `just/rust.just` and `just/rust-workspace.just` respectively, and all four depend on `just`; `options["just_recipes"] = "none"` turns off the language recipe imports so that older projects with a custom orchestration only pick up the queueing infrastructure. Selecting the `bque` layer also pulls in the `bque` skill to avoid a half-wired setup; `audit` flags projects that are half-wired or missing the justfile managed block. bque is a Windows global lock; on Unix and in CI the same recipes run straight through. See [`templates/README.md`](templates/README.md).

`create` options for Rust (only effective with the `rust` layer): `--edition 2015|2018|2021|2024` (default 2024), `--channel stable|beta|nightly|<version>` (default stable), `--sccache` to enable the compile cache (off by default), `--no-lld` to disable the lld linker, `--chinese` to enable Chinese programming (allows Chinese identifiers, disables the related naming lints). The choices are frozen into `.pengj-templates.json` at generation time, and `update` re-renders with the options each project recorded back then.

Options for the `agent` layer (only effective when the agent layer is selected): `--skills git-workflow,caveman,grill-me,write-docs,template-sync,subrepo-sync,write-a-skill` selects which skills to generate (comma separated, all of them by default), `--skill-lang zh|en` picks the documentation language of the skills (default zh), `--no-commit-zh` switches commit messages to English (Chinese by default), and `--commit-and-push` asks the agent to commit and push to the remote whenever a phase of work finishes (off by default). `mcp-bootstrap` is a baseline skill: it is always included whenever the agent layer is present (unless an explicitly empty skill list is passed), so a project keeps being able to install and validate MCP after it leaves the template repository. Skills are generated to `.agents/skills/<name>/SKILL.md`: `git-workflow` (conventional commits and branch syncing), `template-sync` (template sync / adoption), `subrepo-sync` (subrepository sync and upgrade), `bque` (build queueing), `caveman` (ultra-compressed communication), `grill-me` (design grilling), `write-docs` (general documentation writing and updates, rules decided by the project itself), `write-a-skill` (adding a skill), `mcp-bootstrap` (in-project MCP install and handshake validation), and more — adding a skill only requires a new directory under `templates/agent/.agents/skills/`. When the template drops a skill, the leftover skill names in an existing project's manifest are cleaned up during `update` / `add-layer` (a `[清理]` line is printed), so the manifest never drifts from what is actually on disk. The three language options are independent: **Chinese programming** (code identifiers, `chinese_programming`), **project string language** (`skill_lang`, which decides the comments, error messages, logs and skill docs of every layer) and **commit message language** (`commit_zh`, which decides the gate's commit subject language rule and the commit guideline docs); skill scripts are always English and have no option. A fully English project must pass both `--skill-lang en` and `--no-commit-zh`, after which every generated text file contains zero Chinese characters (covered by an end-to-end test).

**Skill extension**: every skill = managed framework + project-owned area. Inside the `PENGJ_TEMPLATE_START/END` block of `SKILL.md` sits the template framework (replaced in place on `update`); everything outside belongs to the project and is never overwritten — the definitions of pre-commit checks, domain decision tables and red lines all live outside the block, and the form of the executable gate (a script in any language / a task runner / a plain checklist) is decided by the project and declared outside the block; the framework prescribes no implementation. Fully custom legacy skills are taken over automatically when adopted: the template page (including the description) replaces them outright and the original body moves down into a "takeover transition zone" for the user to merge.

### MCP install and doctor (`mcp` subcommand)

Templates manage "the files inside a repository"; MCP manages "each client's configuration on the machine" — the same MCP server usually has to be registered once per editor and CLI, and many clients only support global configuration. The `mcp` subcommand is the entry point for that layer: one recipe, written to every client.

```bash
cargo run -p pengj-templates-cli -- mcp list                      # recipes + local install state
cargo run -p pengj-templates-cli -- mcp add renderdoc --dry-run    # rehearsal: print the plan only
cargo run -p pengj-templates-cli -- mcp add renderdoc              # install the payload and register with every client
cargo run -p pengj-templates-cli -- mcp add renderdoc --clients workbuddy,opencode
cargo run -p pengj-templates-cli -- mcp add renderdoc --with-side --enable-app   # extensions are shallow-cloned into the managed cache
cargo run -p pengj-templates-cli -- mcp add renderdoc --with-side --source-root <existing checkout>
cargo run -p pengj-templates-cli -- mcp doctor                     # static dead-link scan + real stdio handshake
cargo run -p pengj-templates-cli -- mcp doctor --fix               # clean up dead links in place
cargo run -p pengj-templates-cli -- mcp print renderdoc --client trae   # emit a snippet for manual pasting
cargo run -p pengj-templates-cli -- mcp remove renderdoc [--purge]      # unregister / also drop payload and source cache
```

Built-in client adapters (the values accepted by `--clients`):

| id | Config location | Structural difference |
|----|-----------------|-----------------------|
| `workbuddy` | `~/.workbuddy/mcp.json` | `mcpServers`; you have to click "trust" once in the GUI afterwards |
| `opencode` | `~/.config/opencode/opencode.json` | `mcp.servers`; `command` is an **array** |
| `claude-code` | `~/.claude.json` | top-level `mcpServers` (the file holds a lot of unrelated state, kept as is) |
| `agy` | `~/.gemini/config/mcp_config.json` | `mcpServers`; remote uses `serverUrl` (`url` is not supported) |
| `trae` | `%APPDATA%\Trae CN\User\mcp.json` | `mcpServers`; `command` **must not contain spaces** |

Key points:

- **Back up before writing, keep unknown keys, stay idempotent**: only the target entry changes, other fields and key order stay untouched; a `.pengj-mcp.bak` is left behind before the first rewrite; running it again produces no change.
- **Commands are written as absolute paths**: GUI clients often spawn subprocesses with an incomplete PATH and a bare command name then fails; `add` resolves the path to an absolute one before writing.
- **Registered ≠ effective**: most clients need a restart or a confirmation click, so `add` lists every "manual action pending" and `doctor` rechecks afterwards.
- **Recipes can be externalized**: the built-in recipes are compiled into the binary, and `~/.pengj-mcp/catalog/*.toml` (or `PENGJ_MCP_CATALOG`) overrides them by name, so changing a recipe needs no recompilation. See `crates/mcp/catalog/renderdoc.toml` for the recipe fields.
- **Only three on-disk locations**: the payload lives in the `uv tool` install directory (`uv tool uninstall` removes it), source caches live in `~/.pengj-mcp/src/<name>` (only needed for projects whose "wheels do not ship side artifacts"; `remove --purge` deletes them too), and state lives in `~/.pengj-mcp/manifest.json`; nothing is written into the working directory and no temporary venv is created.
- **Installed state**: `~/.pengj-mcp/manifest.json` records where the payload came from and which clients were written, and `remove` / `doctor` use it to decide.

## CI / Release
- **CI** (`.github/workflows/ci.yml`):
  - Changes to Rust / templates / build configuration → `cargo fmt --check`, `clippy -D warnings`, build, `cargo test --workspace`; a documentation-only change skips the whole job.
  - **Documentation-only changes are skipped automatically** (root `*.md`, `docs/**`, `.agents/**`), so they cost no CI quota; consecutive pushes on the same branch/PR cancel the previous unfinished run.
  - Caching: the Rust build cache (rust-cache, failures included).
- **Automatic versioning + Release** (`.github/workflows/release.yml`):
  - Documentation-only changes (the same ignore list as CI) do not trigger this workflow; only code / template / configuration changes do.
  - `release-please` reads conventional commits, bumps semver, writes the version into the root `Cargo.toml` and creates a release PR; merging it tags the release and creates the GitHub Release. After publishing, the `sync-lockfile` job writes the new version back into `Cargo.lock` and commits it, so no manual sync is needed.
  - When there is a `release`, the CLI is built and published. **Artifacts are consistently named "os-arch"**: `pengj-templates_<version>_<os>-<arch>_portable.zip`.
    - Covers Windows x64/arm64, Linux x86_64/arm64, macOS arm64/x86_64 (arm builds use the native GitHub arm64 runners).
    - One portable bundle per combination, containing the `pengj-templates-cli` executable plus the accompanying `templates/` tree.
    - **Fixed-name** assets `pengj-templates-cli-<os>-<arch>.zip` / `.tar.gz` are produced as well; the one-liner installers always point at those, so the URL keeps resolving across versions.
    - `install.ps1` / `install.sh` are uploaded to the root of the Release by a dedicated job, so `releases/latest/download/install.ps1` is always reachable.
  - **Fully automated publishing**: the `auto-merge` job merges the Release PR created by release-please with a PAT (no manual step), and the merge triggers the build above.
  - **beta / preview stages**: release-please natively supports pinning a pre-release number through a commit footer and incrementing it automatically:
    - to switch to a pre-release number, add `Release-As: 0.5.0-beta.1`, `Release-As: 0.5.0-rc.1`, `Release-As: 0.5.0-preview.1`, … to the description of a normal commit;
    - afterwards it increments within the same stage (`beta.1` → `beta.2`), and the GitHub Release is marked as a **Prerelease**;
    - to go back to a stable release, commit `Release-As: 0.5.0`.
  - **This project's own version follows stable semver**: released versions carry no `-beta.N` suffix. The manifest is pinned to the last released tag so release-please can always locate the previous release; a stable release is cut by putting `Release-As: <version>` in the description of a code / template / config commit. The same footer switches to a pre-release explicitly whenever you want one.
  - Versioning rules (release-please): `feat` → minor, `fix` → patch, `BREAKING CHANGE` → major (during `0.x`, `bump-minor-pre-major=true` means a BREAKING change only bumps the minor instead of jumping to major; `feat` also only bumps the minor during `0.x`).

`src/main.rs` of the `rust` layer is on the `update_ignore` blacklist: it is written on the first generation only and belongs to the user afterwards, so template updates skip it (no overwrite, no conflict, no deletion report). If the template holds other "seed files that later belong to the user", list them in the `update_ignore` of the corresponding `layer.toml`.

`.cargo/config.toml` goes through **structured managed merging in TOML**: tables inside the managed block are unioned with the user's configuration at table level and de-duplicated at key level — an equivalent key the user already has is de-duplicated automatically (avoiding duplicate table definitions that break cargo), and tables the template adds enter the managed block; a different value for the same key, or the need to add a new key into a table the user already has, is reported as a conflict and the write is skipped so you can align it by hand and rerun. Text files (AGENTS.md, skill SKILL.md) go through **managed-block injection**: if the file on disk already has a managed block it is replaced in place; a legacy file without one gets the block appended at the end and is counted in `needs_review` to prompt a human review. The blacklist of files is declared with `update_ignore` in `layer.toml` — `.gate.toml` of the `gate` layer is on that blacklist: after the first generation it belongs to the project and template updates skip it, so your scope whitelist is never overwritten.

The template configuration for `.vscode/settings.json` is merged into the `settings` node of the workspace file when the project root already has a `*.code-workspace` file (no separate settings file is created); only when there is no workspace file are `.vscode/settings.json` created / maintained.

Using the gate layer: the gate is a pure Rust implementation with no Node dependency, and it now lives inside `pengj-templates-cli` — installing one binary gives you the template engine, the MCP installer and the commit gate at the same time. After generating a project, run `git init` first, then run `just doctor --fix` once (or `sh scripts/doctor.sh --fix` / `pwsh -NoProfile -File scripts/doctor.ps1 -Fix` without just) — it checks the toolchain and points `core.hooksPath` at `.githooks`, no CLI install required. The installers no longer write a global hook path. The gate layer implies the `ci` layer, so machines without the CLI are covered by the server-side workflow. `core.hooksPath` lives in `.git/config` and never travels with a clone, so run the doctor once after a fresh clone too.

Scope policy: an absent `commit.scopes` key or an empty list now **rejects every `(scope)`** (fail-closed); the old "empty means accept any" behaviour is gone. Set `commit.scopes = ["*"]` to accept any scope, or list crate names to pin a whitelist. Existing projects generated before this change carry `commit.scopes = []` and will start rejecting scoped headers once the CLI is upgraded — add `["*"]` to `.gate.toml` to restore the old behaviour.

The hooks call `pengj-templates-cli gate hook pre-commit` / `pengj-templates-cli gate hook commit-msg <msgfile>` directly; when the CLI is missing, `commit-msg` falls back to `.githooks/gate-fallback.sh` (see below). The old multi-level probing (the `$PENGJ_GATE` environment variable, the standalone `gate` binary and the repository-URL branch) is gone.

### Machines without the CLI

A machine without `pengj-templates-cli` cannot run the full rule set locally — the rules live inside the CLI. The template lays out four lines of defence for that case, two local and two server-side:

1. **Enable it (zero dependencies)**: run `just doctor --fix` (or `sh scripts/doctor.sh --fix`). The doctor checks the toolchain and calls `.githooks/install.sh`, which needs git and a POSIX shell, nothing else; that script points `core.hooksPath` at `.githooks` and records the executable bit. `just gate-hooks-local` is the raw recipe for the same script.
2. **Commit subjects (a reduced local check)**: when `commit-msg` finds no CLI it runs `.githooks/gate-fallback.sh` instead — a minimal rule set written in plain shell (header grammar, type whitelist, non-empty subject, trailing full stop, subject language), with the thresholds read from `.gate.toml`. **The invariant: the fallback is always more permissive than the CLI.** It catches the worst offenders and never rejects a message the CLI would accept.
3. **Code language and formatting (filled in by the server)**: rustfmt and the language guard cannot be reproduced by a shell script, so without the CLI the `pre-commit` hook lets the commit through with a loud warning instead of blocking on a check that cannot run. Both run on the server instead, in the `gate` job of the `ci` layer (Rust projects also get `cargo fmt --check` there).
4. **Enforcement (server-side)**: the `ci` layer ships `.github/workflows/gate.yml`, `.github/rulesets/gate.json` (the `gate` job as a required status check, configuration as code) and `.github/pull_request_template.md`. When a run fails, the workflow writes an actionable summary into `GITHUB_STEP_SUMMARY`, so the pull request page says what to change.

The environment check itself lives in `scripts/doctor.sh` / `scripts/doctor.ps1` and only checks the tools the selected layers use. Add project-specific checks by dropping `*.sh` (Unix) or `*.ps1` (Windows) into `scripts/doctor.d/`; a non-zero exit is reported as a failure, and `--fix` sets `DOCTOR_FIX=1` so a custom check can repair its own tool.

The ruleset takes effect after a maintainer applies it once:

```bash
gh api -X POST "repos/$(gh repo view --json nameWithOwner -q .nameWithOwner)/rulesets" \
  --input .github/rulesets/gate.json
```

It makes `gate` a required status check on the default branch, so a red pull request cannot be merged — the CI turns from a reference into a gate.

A private repository on the free plan cannot get that: GitHub asks for GitHub Pro before it enables rulesets or branch protection. The CI still goes red, it just cannot block a merge — `.github/rulesets/gate.json` stays in the repository, ready to be applied with one command once the plan is upgraded or the repository goes public.

`.gate.toml` is the gate's single configuration surface, a flat TOML subset: `mode` (`english`/`chinese`, the code language direction), `commit.subject_lang` (`zh`/`en`, the commit subject language), `commit.types` (type whitelist), `commit.scopes` (scope whitelist, an empty list accepts anything), `commit.subject_min_han` (minimum number of Han characters in a Chinese subject), `commit.max_header` (maximum header length), `lang.max_ratio` / `lang.max_ident_ratio` (tolerated violation ratios for strings / identifiers in the code language guard), and `project.disabled` (the checks the project opts out of).

The code language guard scans the staged `.rs` files at the pre-commit stage: the direction comes from `mode` (English programming rejects non-English, Chinese programming rejects non-Chinese) and the ratio is computed per file — string violations stay within `lang.max_ratio` (1% by default) and identifier violations within `lang.max_ident_ratio` (0 by default), with the allowance rounded down. Chinese mode only judges human-facing copy: command-line flags, paths, protocol field names, embedded source or shader code and other machine literals are not violations (they still count toward the denominator so the budget is not diluted). Genuine exceptions are certified in the source: `lang-ok: <reason>` at the end of a line exempts that line, `lang-allow: <reason>` near the top of the file exempts the whole file (i18n word lists, CJK test fixtures). The full audit runs `just gate-check` (or `pengj-templates-cli gate check`), scanning `.rs` recursively from the repository root and skipping `target`, `node_modules`, `third-party`, `generated` and `submodules`, so custom layouts such as `backend/` or `launcher/` are covered too; `--list` prints every violation, `--json` emits a machine-readable report; `--max-ratio` / `--max-ident-ratio` relax the thresholds temporarily, and `PENGJ_CODE_LANG=english|chinese` overrides the direction.

Generating a project writes the `.pengj-templates.json` manifest (recording the layers and the hashes of the template-managed files), and `update` uses it to touch only the template-managed files: files that were not changed locally are overwritten, changed ones are reported as conflicts and skipped, files with managed blocks get a block-level merge (structured TOML merge / text managed-block injection), and newly added template files are created. The report lists the merged results worth a human review under `needs_review`.

## Directory layout

```
crates/
├── core/         # Engine: layer discovery, dependency ordering, merged rendering, generate/update, manifest
├── cli/          # CLI (pengj-templates-cli, with the built-in gate subcommand)
├── mcp/          # MCP install and doctor: recipe catalog, client adapters, handshake validation
└── gate/         # Commit gate internal library (pengj-gate, called only by the CLI)
templates/        # Layer templates (common/ gate/ rust/ ...), loaded at runtime
tools/            # One-liner installers (install.ps1 / install.sh), shipped with the Release
```

## Development

```bash
just gate-hooks       # activate the commit hooks (once per clone)
just check            # full Rust compile check
just ci               # format check + compile + clippy + tests
```

> On Linux the repository's `.cargo/config.toml` routes the `linux-gnu` link through `cc -fuse-ld=lld`, so `lld` has to be installed (`apt install lld`); otherwise the build stops with `linker 'lld' not found`. Going through the compiler driver is deliberate: bare `lld` carries no default library search paths and fails to find `-lgcc_s` / `-lc`.

### What this repository is made of

This repository is a **pure Rust workspace** with no Node ecosystem dependency at all: no `package.json`, no `pnpm-lock.yaml`, no front-end build chain.

| Directory | Purpose |
|---|---|
| `crates/core` | Template engine: layer discovery, dependency ordering, merged rendering, generate/update, manifest |
| `crates/cli` | Command-line entry point `pengj-templates-cli` (with the `gate` subcommand) |
| `crates/mcp` | MCP install and doctor |
| `crates/gate` | Commit gate internal library `pengj-gate` (zero third-party dependencies, called only by the CLI) |
| `templates/` | Layer template directory, loaded at runtime |

The commit gate is implemented in `crates/gate` (Rust, no Node dependency) and is now an internal library inside the CLI, reachable only through `pengj-templates-cli gate ...`: generated projects activate the hooks with `just doctor --fix`, the rules live in `.gate.toml`, and `just gate` forwards any gate subcommand (`check` / `commit-msg` / `fmt` / `hooks install` / `selftest`).
