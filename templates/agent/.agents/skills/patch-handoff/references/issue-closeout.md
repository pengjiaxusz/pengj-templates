{% if options["skill_lang"] == "en" -%}
# Filing and Closing Out Issues for a Handoff

Two moments matter: while verifying (problems get filed), and after landing (every issue in scope gets a terminal state). A handoff that ends with an unnumbered "open items" paragraph has produced homework, not tracking.

## 1. What deserves an issue

| What you found | File it? | Why |
| :--- | :--- | :--- |
| Does not compile as delivered | Yes, in the same pass | It is a fact, not an opinion, and the upstream agent can act on an issue |
| Doc claims a gate that does not exist, or a pass that does not reproduce | Yes | Keeps the next handoff honest |
| Ticket cited but no patch behind it | Yes | A mapping gap belongs on the tracker |
| Behavioural gap tests cannot see | Yes, unless the user accepts it in this session | Otherwise it evaporates with the report |
| Repair you already folded | No — the commit is the record | Close the source issue if there was one |
| Style nit fixed in passing | No | The commit message is enough |

Rule of thumb: if someone owes a follow-up, it is an issue. If the commit already says it, it is not.

## 2. Dedupe before opening anything

```bash
gh issue list --state all --search '"<package name>" in:title'
gh issue list --state open --search '<symptom words>'
```

Same cause → comment on the existing issue ("also hit by handoff `<pkg>`, patch 0007") and cite that number in your report instead of opening a twin. Two issues for one defect double the closing work and halve the trust in the tracker.

## 3. Body template

```bash
gh issue create \
  --title '[handoff] <pkg>: <one-line symptom>' \
  --label bug \
  --body "$(cat <<'EOF'
Package: <file name>, patches 0001-0012
Where: patch 0007 touches src/foo.rs

Claimed: `cargo build` clean.
Actual: fails with E0609 `no field 'size'` at src/foo.rs:88 — the repo has `font_size`.

Reproduce:
    git am patches/0007-*.patch && cargo build

Fixed means: the series applies and the gate chain passes on the real base <sha>.
EOF
)"
```

`<<'EOF'` keeps backticks and `$` literal. The command prints the issue URL, which is where the number for your report comes from — copy it, do not guess it.

## 4. State machine

| Moment | Action | Command |
| :--- | :--- | :--- |
| Landed, folded into a commit | Comment with the SHA, then close | `gh issue comment <n> --body "..."` then `gh issue close <n>` |
| Decided not to fix | Keep open, label it, name the owner | `gh issue edit <n> --add-label needs-decision --add-assignee @me` |
| Duplicate of an existing issue | Comment on the survivor, close the twin | `gh issue close <n> --reason "not planned"` |
| Belongs to the upstream agent | Keep open, say who acts | comment, then `--add-assignee` or @-mention |

Labels come from `gh label list`. If the project has none, use one neutral label and say so in the report rather than inventing a taxonomy for a single handoff.

## 5. `gh` failure modes that cost you the record

- `gh issue close <n> --comment "..."` on an already-closed issue discards the comment and still exits 0. Always comment first, then close, and read the output.
- Flip labels in the right order: `--add-label` first, `--remove-label` second, or the issue is briefly label-less and drops off the board.
- In a fork, `gh` targets the upstream repo unless `-R <owner>/<repo>` is passed. Check `gh repo view --json nameWithOwner` first — filing into someone else's repo is worse than not filing.
- `gh` missing or unauthenticated: the report must say `issue not filed: gh not authenticated` **plus the exact command**. Do not paraphrase the issue into prose and call it tracked.
- On Windows Git Bash, a heredoc can mangle quotes; write the body to a temp file and use `--body-file`, then delete that file in S8 like every other temp artifact.

## 6. Report footer

```
closed: #101, #102, #103 (folded 3f2a1c9)
filed:  #141 (E0609 in 0007), #142 (missing test for menu labels)
open:   #142 — needs the user to decide whether blank rows are acceptable
```

That footer is the difference between "landed" and "landed, and nobody has to re-derive what happened".
{% else %}
# 交接时建单与收口

两个时刻决定成败：验收时（问题当场建单），落地后（范围内每张单都有终态）。一份以「遗留问题」纯文字段落收尾的交接，产出的是作业，不是跟踪。

## 一、什么值得建单

| 你发现的东西 | 建单？ | 为什么 |
| :--- | :--- | :--- |
| 交付的补丁集编译不过 | 是，发现的那一轮就建 | 这是事实不是观点，上游 agent 拿到单就能动手 |
| 文档点了不存在的门禁，或声称通过却复现不出来 | 是 | 让下一份交接包不敢再随手写「全绿」 |
| 文档点名的工单背后没有补丁 | 是 | 映射缺口就该落在 tracker 上 |
| 测试看不见的行为缺口 | 是，除非用户当场决定接受 | 否则它随报告一起蒸发 |
| 你已经折回的修复 | 否——提交就是记录 | 若它来自某张单，顺手把那张单关掉 |
| 顺手改掉的风格小问题 | 否 | 提交信息已经写清楚了 |

判断标准：**有人欠一个后续动作 → 建单；提交里已经写明白 → 不建。**

## 二、开单前先查重

```bash
gh issue list --state all --search '"<包名>" in:title'
gh issue list --state open --search '<症状关键词>'
```

同因的：在已有单上补评论（「交接包 `<包名>` 的 0007 同样命中」），报告里引用那个编号，别再开一张。一个缺陷两张单，收尾工作量翻倍，看板的可信度对半砍。

## 三、正文模板

```bash
gh issue create \
  --title '[handoff] <包名>: <一句话症状>' \
  --label bug \
  --body "$(cat <<'EOF'
交接包：<文件名>，补丁 0001-0012
位置：0007 改到 src/foo.rs

声称：`cargo build` 干净。
实测：E0609 `no field 'size'`，src/foo.rs:88 —— 仓库里叫 `font_size`。

复现：
    git am patches/0007-*.patch && cargo build

什么算修好：序列能在真实基线 <sha> 上应用，且整条门禁链通过。
EOF
)"
```

`<<'EOF'` 保证反引号和 `$` 不被展开。命令会打印 issue 的 URL，报告里要写的编号就从那里来——复制，不要猜。

## 四、状态机

| 时刻 | 动作 | 命令 |
| :--- | :--- | :--- |
| 已落地、折回了提交 | 先评论贴 SHA，再关闭 | `gh issue comment <n> --body "..."` → `gh issue close <n>` |
| 决定不修 | 留开、打标签、点名责任人 | `gh issue edit <n> --add-label needs-decision --add-assignee @me` |
| 与已有单重复 | 在存活的那张上评论，关掉重复的 | `gh issue close <n> --reason "not planned"` |
| 属于上游 agent 的活 | 留开，写清该谁动 | 评论后 `--add-assignee` 或 @ 提及 |

标签一律取自 `gh label list`。项目一个标签都没有时，用一个中性标签并在报告里说明，别为一次交接自造一套体系。

## 五、会让你丢掉记录的 `gh` 失败模式

- `gh issue close <n> --comment "..."` 作用在已关闭的 issue 上会丢掉评论并仍然返回 0。永远先 comment、再 close，并读输出。
- 翻标签顺序反了会留出无标签空档：先 `--add-label`，再 `--remove-label`。
- fork 里 `gh` 默认打上游仓库，除非传 `-R <owner>/<repo>`。先 `gh repo view --json nameWithOwner` 确认——发到别人的仓库，比不发更糟。
- `gh` 没装或未登录：报告必须写 `issue 未建：gh 未认证` **外加原样命令**。不许把 issue 改写成一段文字然后声称已跟踪。
- Windows Git Bash 下 heredoc 容易把引号弄坏：正文写进临时文件用 `--body-file`，然后像其他临时产物一样在 S8 删掉。

## 六、报告结尾

```
已关闭：#101、#102、#103（折回 3f2a1c9）
新建：#141（0007 的 E0609）、#142（菜单文案缺测试）
仍开：#142 —— 待用户决定空行是否可接受
```

这段结尾，就是「落地了」与「落地了，且没人需要重新推导发生过什么」的差别。
{% endif %}
