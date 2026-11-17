#!/bin/sh
# gate install: {% if options["skill_lang"] | default("zh") == "en" %}activate the commit hooks on this clone{% else %}在当前克隆上激活提交钩子{% endif %}.
#
# {% if options["skill_lang"] | default("zh") == "en" %}`core.hooksPath` lives in `.git/config`, which never comes with a clone, so every clone needs this once. The script needs git and a POSIX shell and nothing else — it does not need the CLI, because activating the hooks and running them are separate jobs.{% else %}沿用 `core.hooksPath` 存在 `.git/config` 里、不会随克隆传播，所以每个克隆跑一次即可。脚本只需要 git 与一个 POSIX shell——它不需要 CLI，因为「激活钩子」与「跑钩子」是两件事。{% endif %}
#
# {% if options["skill_lang"] | default("zh") == "en" %}The gate itself lives in `pengj-templates-cli`; this script ships no rule set of its own. When the CLI is missing the hooks block commits and print the install command, so the check below only warns.{% else %}门禁本体在 `pengj-templates-cli` 里，本脚本不自带规则。没有 CLI 时钩子会拦下提交并打印安装命令，所以下面的检测只做告警。{% endif %}

set -eu

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}this is not a git work tree{% else %}当前目录不是 git 工作区{% endif %}" >&2
    exit 1
}
cd "$root"

if [ ! -f .githooks/commit-msg ]; then
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}this repository does not carry the gate hooks{% else %}这个仓库没有携带门禁钩子{% endif %}" >&2
    exit 1
fi

# {% if options["skill_lang"] | default("zh") == "en" %}git reads the hooks from `.git/hooks` by default, and `core.hooksPath` lives in `.git/config`, which is never cloned. Pointing it at the tracked `.githooks/` is what makes the gate run at all.{% else %}git 默认只读 `.git/hooks`，而 `core.hooksPath` 存在 `.git/config` 里、不会随克隆传播。把它指向仓库里托管的 `.githooks/`，门禁才真的跑得起来。{% endif %}
git config core.hooksPath .githooks

# {% if options["skill_lang"] | default("zh") == "en" %}On Unix git silently skips a hook that is not executable. The tracked mode bit is the source of truth, and `update-index` is what records it.{% else %}Unix 上没有可执行位的钩子会被 git 静默跳过。模式位以索引记录为准，`update-index` 负责把它写进去。{% endif %}
for hook in commit-msg pre-commit; do
    [ -f ".githooks/$hook" ] || continue
    if git ls-files --error-unmatch ".githooks/$hook" >/dev/null 2>&1; then
        git update-index --chmod=+x ".githooks/$hook"
    else
        chmod +x ".githooks/$hook" 2>/dev/null || true
        echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}.githooks/$hook is not tracked yet; the executable bit was set on disk only{% else %}.githooks/$hook 还没纳入版本管理，可执行位只写到了磁盘上{% endif %}" >&2
    fi
done

if command -v pengj-templates-cli >/dev/null 2>&1; then
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli is installed: the hooks will run the full rule set{% else %}检测到 pengj-templates-cli：钩子会跑完整规则集{% endif %}"
else
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}the hooks are active, but pengj-templates-cli is not installed: commits will be blocked until it is{% else %}钩子已激活，但没检测到 pengj-templates-cli：装好之前提交会被拦下{% endif %}" >&2
    echo "[gate]   {% if options["skill_lang"] | default("zh") == "en" %}the gate lives in the CLI; install it once:{% else %}门禁就在 CLI 里，装一次即可：{% endif %}" >&2
    echo "[gate]   irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex" >&2
    echo "[gate]   curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh" >&2
fi

configured=$(git config --local --get core.hooksPath 2>/dev/null || true)
if [ "$configured" = ".githooks" ]; then
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}the local commit gate is enabled: core.hooksPath=.githooks{% else %}本地提交门禁已启用：core.hooksPath=.githooks{% endif %}" 2>/dev/null || true
else
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}could not write core.hooksPath; run: git config core.hooksPath .githooks{% else %}未能写入 core.hooksPath；请手动执行 git config core.hooksPath .githooks{% endif %}" >&2
    exit 1
fi
