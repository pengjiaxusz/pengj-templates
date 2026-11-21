{% if options["skill_lang"] == "en" -%}
# Git Mechanics for Landing a Patch Series

## 1. Hook trigger matrix

This is measured behaviour, not documentation folklore. It decides which gate can still catch a bad commit.

| How the commit is made | `pre-commit` | `commit-msg` | `pre-applypatch` / `post-applypatch` | `pre-push` |
| :--- | :--- | :--- | :--- | :--- |
| `git commit` | yes | yes | no | later |
| `git commit --amend` | yes | yes | no | later |
| `git am` | **no** | **no** | yes | later |
| `git cherry-pick` | **no** | **no** | no | later |
| `git rebase` | **no** | **no** | no | later |

Consequences:

- A series landed with `git am` never passed the project's commit-time checks (formatting, language guards, message rules). The only remaining backstop is `pre-push`.
- Do not try to repair in `pre-applypatch`: writing files and `git add`-ing them inside that hook does not reach the commit — `git am` runs `read-tree` afterwards, which wipes the index and leaves only a dirty worktree. That hook can reject, never amend.
- Hooks live in `.git/config` (`core.hooksPath`). They do not travel with a clone. In a throwaway clone, run the repo's hook installer first, or `pre-push` simply does not exist.
- Proof the backstop is live: the push output shows the hook's own log lines. No log lines means no hook.

## 2. `git am` failure triage

Work down this list; do not start editing patches.

1. **Wrong base.** `git log --oneline -3` on the target ref versus the doc's claimed SHA. Retry on the real base.
2. **Patch already applied.** `git am` reports "patch does not apply" while `git apply --check --reverse <patch>` succeeds. Use `git am --skip` only after confirming the content is present.
3. **Context drift, content fine.** `git am -3` (three-way) usually lands it; review the diff afterwards.
4. **Real conflict.** `git am --show-current-patch` to see what it wanted; resolve, `git add`, `git am --continue`; `git am --abort` to back out entirely.
5. **Binary or mode-only patch.** The base blob is missing; ask for the base commit instead of hand-crafting it.
6. **Mailbox damage.** A missing `From ` separator or CRLF mangling in the `.patch` file. Re-export rather than repair.

Leave the worktree in place when a replay stops mid-series — the half-applied state is the evidence — and say so in the report.

## 3. Fold a repair back into the commit that introduced it

Precondition: the file was touched by exactly one commit in the series.

```bash
git log --oneline <base>..HEAD -- <path> | wc -l    # must be 1
```

Then: overwrite the file with the repaired version, `git add`, `git commit --amend --no-edit` while that commit is `HEAD` — which is what `scripts/replay-patch-series.sh` automates. If the count is greater than 1, do not fold: fix in a final commit and state in the report why the 1:1 mapping broke.

`git rebase -i` with `--exec` is the alternative when repairs are already committed; it is fine, but never let it open an editor (`GIT_SEQUENCE_EDITOR=true`).

## 4. Author normalization

```bash
FILTER_BRANCH_SQUELCH_WARNING=1 git filter-branch -f --env-filter \
  'export GIT_AUTHOR_NAME="<name>"; export GIT_AUTHOR_EMAIL="<email>"' \
  <base>..HEAD
git log --format="%an <%ae>" <base>..HEAD | sort -u     # must be one line
```

- `--env-filter` changes authorship and keeps author dates. `--reset-author` also resets the date, which destroys the timeline the handoff described.
- Keep committer identity alone unless the project says otherwise; only the author line is the attribution.
- `filter-branch` rewrites every commit in the range, so the range must stay small and pre-push: run it before pushing, never after.

## 5. After the force push

```bash
git fetch --prune --force <remote>        # without --force the remote ref stays stale
git rev-list --count <remote>/<branch>..HEAD   # must be 0, else stop and ask
git status --porcelain -uall                   # must be empty
git reset --hard <remote>/<branch>
```

`git pull` after a force push merges the old history back in; `reset --hard` is the correct verb. If the branch is a submodule pointer in a parent repo, expect the parent to show the submodule as modified — that is a pointer difference, not damage. Do not commit or revert the parent on the user's behalf.

Also note: shared build caches (`CARGO_TARGET_DIR`, `node_modules` in a shared location) mean a throwaway clone can materialize gigabytes of build output. Clean it up in S8.
{% else %}
# 落补丁集时的 Git 机制

## 一、钩子触发面矩阵

这是实测行为，不是文档传说。它决定哪道门禁还拦得住一条坏提交。

| 造成提交的方式 | `pre-commit` | `commit-msg` | `pre-applypatch` / `post-applypatch` | `pre-push` |
| :--- | :--- | :--- | :--- | :--- |
| `git commit` | 触发 | 触发 | 不触发 | 稍后 |
| `git commit --amend` | 触发 | 触发 | 不触发 | 稍后 |
| `git am` | **不触发** | **不触发** | 触发 | 稍后 |
| `git cherry-pick` | **不触发** | **不触发** | 不触发 | 稍后 |
| `git rebase` | **不触发** | **不触发** | 不触发 | 稍后 |

推论：

- 用 `git am` 落的序列，天然没经过项目的提交期检查（格式、语言守卫、消息规范）。唯一还拦得住的兜底是 `pre-push`。
- 别在 `pre-applypatch` 里修补：在钩子里写文件 + `git add` 进不了提交——`git am` 之后会 `read-tree` 冲掉索引，只留一棵脏工作树。这个钩子只能拦，不能改。
- 钩子存在 `.git/config`（`core.hooksPath`），不随 clone 传播。临时克隆里必须先跑仓库的钩子安装脚本，否则 `pre-push` 等于不存在。
- 兜底真的生效的证据：推送输出里有钩子自己的日志行。没有日志行就是没钩子。

## 二、`git am` 失败诊断

按这个顺序往下走，不要一上来就改补丁。

1. **基线不对。** 对比目标 ref 的 `git log --oneline -3` 与文档声称的 SHA，在真实基线上重试。
2. **补丁已经应用过。** 报「patch does not apply」但 `git apply --check --reverse <patch>` 通过，确认内容已在树上再用 `git am --skip`。
3. **上下文漂移、内容没问题。** 用 `git am -3` 三方合并，落地后复查 diff。
4. **真冲突。** `git am --show-current-patch` 看它想要什么；解决后 `git add` + `git am --continue`；整体退出用 `git am --abort`。
5. **二进制或仅改模式的补丁。** 缺基线 blob，去要基线提交，不要手搓。
6. **邮箱格式损坏。** `.patch` 里缺 `From ` 分隔行或被 CRLF 弄坏，重新导出，不要修。

中途停下时把工作树留在原地——半应用状态就是证据——并在报告里写明。

## 三、把修复折回引入缺陷的提交

前提：该文件在整个序列中只被一条提交碰过。

```bash
git log --oneline <base>..HEAD -- <path> | wc -l    # 必须为 1
```

然后：用修好的版本覆盖文件、`git add`、在该提交仍是 `HEAD` 时 `git commit --amend --no-edit`——`scripts/replay-patch-series.sh` 做的就是这件事。若计数大于 1 就不要折叠：在末条提交单独修，并在报告里说明映射为何被破坏。

修复已经提交了的话，用 `git rebase -i --exec` 也可以，但绝不能让它弹编辑器（`GIT_SEQUENCE_EDITOR=true`）。

## 四、作者归一

```bash
FILTER_BRANCH_SQUELCH_WARNING=1 git filter-branch -f --env-filter \
  'export GIT_AUTHOR_NAME="<name>"; export GIT_AUTHOR_EMAIL="<email>"' \
  <base>..HEAD
git log --format="%an <%ae>" <base>..HEAD | sort -u     # 必须只剩一行
```

- `--env-filter` 改作者、保留 author date；`--reset-author` 连时间一起重置，会毁掉交接文档描述的时间线。
- committer 身份一般不动，除非项目另有约定；署名看的是 author 行。
- `filter-branch` 会重写区间内每一条提交，所以区间要小，且在推送之前做，绝不要推完再改。

## 五、强推之后

```bash
git fetch --prune --force <remote>        # 不加 --force，远端 ref 还是旧值
git rev-list --count <remote>/<branch>..HEAD   # 必须为 0，否则停手问用户
git status --porcelain -uall                   # 必须为空
git reset --hard <remote>/<branch>
```

强推后不要 `git pull`——它会把旧历史合回来；正确的动词是 `reset --hard`。若这个分支在父仓里是子模块指针，父仓会显示子模块已修改——那是指针差异，不是损坏，不要替用户提交或回滚父仓。

另外：共享构建缓存（`CARGO_TARGET_DIR`、放在公共位置的 `node_modules`）会让临时克隆凭空长出几十 GB 构建产物，S8 必须清掉。
{% endif %}
