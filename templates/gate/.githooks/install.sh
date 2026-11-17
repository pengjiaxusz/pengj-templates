#!/bin/sh
# gate install: {% if options["skill_lang"] | default("zh") == "en" %}enable the local commit gate without the CLI{% else %}不依赖 CLI 启用本地提交门禁{% endif %}.
#
# {% if options["skill_lang"] | default("zh") == "en" %}This script is the answer to a machine that has no `pengj-templates-cli`: it needs git and a POSIX shell, nothing else. Run it once per clone.{% else %}这个脚本是给「机器上没有 pengj-templates-cli」准备的答案：只需要 git 与一个 POSIX shell，别的不用。每个克隆跑一次即可。{% endif %}
#
# {% if options["skill_lang"] | default("zh") == "en" %}The gate then runs in its fallback mode: the commit message is still checked, but by the reduced rule set in `gate-fallback.sh` instead of the CLI. The server-side CI keeps the full rule set in force either way.{% else %}此后门禁以降级模式运行：提交信息照样被检查，只是走 `gate-fallback.sh` 里那套缩减规则，而不是 CLI。完整规则集无论如何都由服务端 CI 兜住。{% endif %}

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
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli is installed: the full rule set applies{% else %}检测到 pengj-templates-cli：完整规则集已就位{% endif %}"
else
    echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}pengj-templates-cli is not installed: the hooks fall back to the reduced rule set, and the server-side CI still enforces the full one{% else %}没检测到 pengj-templates-cli：钩子会走缩减规则集，服务端 CI 仍按完整规则集把关{% endif %}" >&2
    echo "[gate]   {% if options["skill_lang"] | default("zh") == "en" %}install it once to get the full rule set locally:{% else %}装一次就能在本地拿到完整规则集：{% endif %}" >&2
    echo "[gate]   irm https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.ps1 | iex" >&2
    echo "[gate]   curl -fsSL https://github.com/pengjiaxusz/pengj-templates/releases/latest/download/install.sh | sh" >&2
fi

echo "[gate] {% if options["skill_lang"] | default("zh") == "en" %}the local commit gate is enabled: core.hooksPath=.githooks{% else %}本地提交门禁已启用：core.hooksPath=.githooks{% endif %}"
