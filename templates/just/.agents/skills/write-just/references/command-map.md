---
name: write-just-reference
description: >-
{% if options["skill_lang"] | default("zh") == "en" %}
  Detailed command-to-recipe mapping for justfile authoring: heavy command wrappers, Windows/Unix attribute pairs, variadic argument forwarding, and the legitimate no-bque downgrade. Loaded on demand from the write-just skill.
{% else %}
  justfile 编写的详细命令到配方映射：重型命令包装、Windows/Unix 属性成对、变参转发，以及无 bque 时的合法降级。由 write-just 技能按需加载。
{% endif %}
---

{% if options["skill_lang"] | default("zh") == "en" %}
# just Recipe Command Map

## 1. Native command → wrapped recipe

Applies when `bque` is in the project's `layers`. On Windows the recipe takes the global lock and `Idle` priority; the `[unix]` branch passes through.

| Native command | Recipe | Windows body | Unix body |
| :--- | :--- | :--- | :--- |
| `cargo run` | `just run` | `bque run -- cargo run …` | `cargo run …` |
| `cargo build --release` | `just build` | `bque run -- cargo build …` | `cargo build …` |
| `cargo clippy --all-targets …` | `just check` | `bque run -- cargo clippy …` | `cargo clippy …` |
| `cargo test` | `just test` | `bque run -- cargo test …` | `cargo test …` |
| `cargo bench` | `just bench` | `bque bench -- cargo bench …` | `cargo bench …` |
| `cmake --build <dir>` | `just build` | `bque run -- cmake --build …` | `cmake --build …` |
| `ninja -C <dir> <target>` | `just test-build <target>` | `bque run -- ninja -C …` | `ninja -C …` |
| `ctest --output-on-failure` | `just test` | `bque run -- ctest …` | `ctest …` |
| `pnpm build` / `tsc` | `just build` | `bque run -- pnpm build` | `pnpm build` |
| `cargo fmt` | `just fmt` | direct (lightweight) | direct |
| ad-hoc heavy command | — (no wrapper) | `bque run -- <cmd>` | `<cmd>` |
| ad-hoc benchmark | — (no wrapper) | `bque bench -- <cmd>` | `<cmd>` |

Ad-hoc commands have no wrapper recipe: a `just` wrapper can forward the command but not bque's own
flags (`--cwd`, `--observer`, `--priority`), so the shell is the right place. Promote a recurring one
to a recipe of its own.

## 2. Attribute pairs

- `[windows]` / `[unix]` must appear together for any recipe whose body differs. A lone `[windows]` recipe makes `just` on Linux/macOS report "no recipe found".
- `[unix]` is a real attribute in just (covers Linux and macOS). Do not use `[linux]` and assume macOS is included.
- bque is a Windows-only global kernel lock. The `[unix]` branch running the raw command is the intended behavior — never call `bque` there.
- If a recipe is identical on all platforms (e.g. `fmt`), write it once with no attribute.

## 3. Arguments and shell details

- Variadic recipes: `build *ARGS:` then forward with `{% raw %}{{ARGS}}{% endraw %}`. In a template file, wrap that variable in a minijinja raw block so it survives rendering.
- Prefix the command with `@` to suppress just's command echo when the recipe itself prints useful output.
- When a layer uses worktrees, inject the shared target dir through `pengj-templates-cli worktree target-dir` — see `templates/rust/.pengj-templates/just/rust.just` for the `shared_win()` / `shared_sh()` macro pattern. Do not hardcode `CARGO_TARGET_DIR`.
- Multi-line recipes: just runs each line in its own shell by default. Use `#!/usr/bin/env bash` shebang recipes or explicit line continuation when state must be shared.

## 4. Managed-block discipline

- Root `justfile`: the framework inside `# PENGJ_TEMPLATE_START/END` belongs to the `just` layer; anything outside is the project's and survives `update`. Never move project recipes into the block.
- `.pengj-templates/just/*.just`: full-overwrite managed files. To change one, edit its source under `templates/<layer>/.pengj-templates/just/` and re-render; do not patch it in a generated project.
- Adding a layer module: add a matching conditional `import` in the root `justfile`'s managed block and add the layer as a dependency of `just` in the module's layer metadata.

## 5. No-bque downgrade (legitimate)

When `bque` is not among `layers`, write the direct command with a short comment explaining that the project has no global queue:

```just
# Direct build: this project does not select the bque layer.
build *ARGS:
    cargo build {% raw %}{{ARGS}}{% endraw %}
```

This is the current layer's legitimate downgrade, not the recommended end state. If heavy parallel builds later cause CPU contention or lock collisions, the fix is to add the `bque` layer and re-wrap the recipes — not to keep the raw command.

{% else %}
# just 配方命令映射

## 1. 原生命令 → 包装配方

适用于项目 `layers` 含 `bque` 的情况。Windows 下配方取全局锁并降到 `Idle` 优先级；`[unix]` 分支透传。

| 原生命令 | 配方 | Windows 命令体 | Unix 命令体 |
| :--- | :--- | :--- | :--- |
| `cargo run` | `just run` | `bque run -- cargo run …` | `cargo run …` |
| `cargo build --release` | `just build` | `bque run -- cargo build …` | `cargo build …` |
| `cargo clippy --all-targets …` | `just check` | `bque run -- cargo clippy …` | `cargo clippy …` |
| `cargo test` | `just test` | `bque run -- cargo test …` | `cargo test …` |
| `cargo bench` | `just bench` | `bque bench -- cargo bench …` | `cargo bench …` |
| `cmake --build <dir>` | `just build` | `bque run -- cmake --build …` | `cmake --build …` |
| `ninja -C <dir> <target>` | `just test-build <target>` | `bque run -- ninja -C …` | `ninja -C …` |
| `ctest --output-on-failure` | `just test` | `bque run -- ctest …` | `ctest …` |
| `pnpm build` / `tsc` | `just build` | `bque run -- pnpm build` | `pnpm build` |
| `cargo fmt` | `just fmt` | 直跑（轻量） | 直跑 |
| 临时重命令 | —（无包装配方） | `bque run -- <命令>` | `<命令>` |
| 临时跑分 | —（无包装配方） | `bque bench -- <命令>` | `<命令>` |

临时命令不设包装配方：`just` 包装只能转发命令，传不进 bque 自己的 `--cwd` / `--observer` /
`--priority`，这类调用本来就该在命令行里写。反复要用的，就单独写成一条配方。

## 2. 属性成对

- 只要 Windows 与 Unix 命令体不同，就必须同时写 `[windows]` 与 `[unix]`。只写 `[windows]` 会让 Linux / macOS 上的 `just` 报「找不到配方」。
- `[unix]` 是 just 的真实属性（含 Linux 与 macOS）。不要用 `[linux]` 然后以为 macOS 也覆盖了。
- bque 是 Windows 全局内核锁，`[unix]` 分支直跑是预期行为——绝不在那里调 bque。
- 所有平台命令体一样时（如 `fmt`）只写一遍，不加属性。

## 3. 变参与 shell 细节

- 变参配方写成 `build *ARGS:`，再用 `{% raw %}{{ARGS}}{% endraw %}` 转发。模板文件里要把这个变量包在 minijinja raw 块内才能扛住渲染。
- 配方本身有有用输出时，命令前加 `@` 关掉 just 的回显。
- 层用了工作树时，通过 `pengj-templates-cli worktree target-dir` 取共享目录——参照 `templates/rust/.pengj-templates/just/rust.just` 的 `shared_win()` / `shared_sh()` 宏写法，不要硬编码 `CARGO_TARGET_DIR`。
- 多行配方默认每行独立 shell。需要共享状态时用 `#!/usr/bin/env bash` shebang 配方或显式续行。

## 4. 托管块纪律

- 根 `justfile`：`# PENGJ_TEMPLATE_START/END` 内是 `just` 层的框架，块外归项目所有、`update` 时保留。不要把项目配方挪进块里。
- `.pengj-templates/just/*.just`：整页覆盖的托管文件。要改就改 `templates/<层>/.pengj-templates/just/` 下的源再重渲染，别在生成项目里打补丁。
- 新增层模块：在根 `justfile` 托管块里加对应的条件 `import`，并在该层的 layer 元数据里把 `just` 列为依赖。

## 5. 无 bque 的合法降级

当 `layers` 不含 `bque` 时，直跑并写一句注释说明本项目没有全局队列：

```just
# Direct build: this project does not select the bque layer.
build *ARGS:
    cargo build {% raw %}{{ARGS}}{% endraw %}
```

这是当前层的合法降级，不是推荐终点。若之后并行重构建引发 CPU 争用或编译锁冲突，正确修法是加 `bque` 层并重新包装配方，而不是继续留着裸命令。

{% endif %}
