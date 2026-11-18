# {% if options["skill_lang"] | default("zh") == "en" %}.pengj-templates/just/worktree.just — parallel worktrees sharing one build directory (managed by the worktree layer; template updates overwrite this file){% else %}.pengj-templates/just/worktree.just — 多工作树共享编译目录（worktree 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}The shared directory is the **main worktree's target/**: the CLI resolves the main worktree with git (git-dir vs common-dir, falling back to `git worktree list`), so every worktree of the same repository gets the same value; the build recipes in rust.just / rust-workspace.just read that value via `worktree target-dir`, so there is exactly one derivation.{% else %}共享目录就是**主工作树的 `target/`**：CLI 用 git 判定主工作树（比对 git-dir 与 common-dir，链接工作树才退回 `git worktree list`），同一仓库的所有工作树都得到同一个值；rust.just / rust-workspace.just 的构建配方也通过 `worktree target-dir` 读同一个值，推导逻辑只有一份。{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}Cost: `cargo clean` (or `git clean -xdf`) in the main worktree wipes the intermediate artifacts of every worktree — a full rebuild for all of them.{% else %}代价：在主工作树里 `cargo clean`（或 `git clean -xdf`）会清掉所有工作树的中间产物，需全量重建。{% endif %}
# {% if options["skill_lang"] | default("zh") == "en" %}Only intermediate artifacts are shared. Final binaries keep a fixed path inside the shared directory, so parallel worktrees overwrite each other — isolate run artifacts per worktree in the project.{% else %}共享的只有中间产物。最终二进制在共享目录里路径固定，多个工作树会互相覆盖——运行产物的隔离由项目自理。{% endif %}

# {% if options["skill_lang"] | default("zh") == "en" %}Print the shared build directory (the single source: the CLI derives it, recipes and the health check both read it){% else %}打印共享编译目录（唯一来源：CLI 推导，配方与自检读同一个值）{% endif %}
wt-target:
    @pengj-templates-cli worktree target-dir --dir .

# {% if options["skill_lang"] | default("zh") == "en" %}Check the shared build directory: exists, writable, not overridden elsewhere{% else %}校验共享编译目录：存在、可写、没有被指向别处的覆盖值{% endif %}
wt-check:
    @pengj-templates-cli worktree check --dir .

# {% if options["skill_lang"] | default("zh") == "en" %}Fix mode: create the shared directory when it is missing (only fills gaps, never overwrites){% else %}修复模式：共享目录缺失时创建它（只补缺、不覆盖）{% endif %}
wt-fix:
    @pengj-templates-cli worktree check --dir . --fix

# {% if options["skill_lang"] | default("zh") == "en" %}Usage: just wt-create feature-a — creates the branch and the sibling directory, wires the shared build directory and fills in submodules{% else %}用法：just wt-create feature-a —— 建分支与同级目录、接线共享编译目录、补齐子模块{% endif %}
wt-create name:
    @pengj-templates-cli worktree create {% raw %}{{name}}{% endraw %} --dir .
