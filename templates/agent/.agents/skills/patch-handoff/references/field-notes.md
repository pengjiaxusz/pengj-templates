{% if options["skill_lang"] == "en" -%}
# Field Notes — One Series, Landed

The concrete details below come from a single landed handoff in a Rust workspace: 12 patches, 12 tickets. The code names are project-specific; the **patterns** repeat in almost every package of this kind.

## 1. The dominant failure: the package targets an imaginary API

| What the package wrote | What the repo actually has | Compiler message |
| :--- | :--- | :--- |
| escaped char literal written as one backslash | needs the double-backslash form | `character literal may only contain one codepoint` + `unterminated character literal` |
| `fn f(text: &str, ...) -> &str` | needs a lifetime tied to `text` | `missing lifetime specifier` |
| `style.size` | `style.font_size` | `no field 'size'` |
| `State::Normal` | `State::None` (variants: `None` / `Error` / `Warning`) | `no variant named 'Normal'` |
| `rect.contains(p)` | `contains_point(p)` / `contains_rect(&r)` | `no method named 'contains'` |
| `place_popup(anchor, size, placement, viewport, gap, offset)` | `(anchor, popup_size, viewport, placement)` | `takes 4 arguments but 6 were supplied` |
| outer type derives `Eq` while containing a type with only `PartialEq` | drop `Eq` from the outer type | `... : Eq is not satisfied` |

**Rule of thumb.** When the errors cluster as "method / field / variant does not exist", fix the call site to match the repo — never add methods to a shared public type to accommodate a patch. When the error is a trait bound, remove the extra derive from the *new* type rather than loosening the shared one.

## 2. Borrow conflicts have one fixed shape

A helper returning `Vec<&str>` borrows `self`; mutating `self` while that borrow is alive fails with `cannot assign to self.x because it is borrowed`.

```rust
// wrong: borrow alive across the mutation
let lines = self.split_lines();
self.cursor_line -= 1;
self.cursor_col = Self::line_char_len(lines[self.cursor_line]);

// right: take the scalars inside a block, mutate after the borrow ends
let len = {
    let lines = self.split_lines();
    Self::line_char_len(lines[target])
};
self.cursor_line = target;
self.cursor_col = self.cursor_col.min(len);
```

When one movement needs two derived values (line length and line count), return a tuple from the block.

## 3. Lint details that block a push

- `let mut x = Vec::with_capacity(n)` that is only moved into a struct and never pushed → drop the `mut`.
- Unused `Result` / `.unwrap()` in non-test code → handle explicitly; the gate is not lenient.

## 4. Fold precondition, concretely

`cp final → git add → commit --amend` is valid only when the file was touched by one commit in the series:

```bash
git log --oneline <base>..HEAD -- src/<file>.rs | wc -l   # must be 1
```

Above 1 means the repair belongs in a separate final commit, with the mapping break stated in the report.

## 5. The gap tests never see

A popup reserved capacity for its menu labels and never pushed any of them. Every shape test passed, because the assertion compared **element counts** across 24 states — a blank row still counts as a row. The menu looked empty to a human and perfect to the suite.

So: after the gates are green, open the new output path and look at it. Text, icons, separators, side effects. Then put the finding in the report and let the user decide whether it blocks landing.

## 6. Numbers worth recording

Record the passing-test total of the run you landed. Next time, a handoff claiming "all tests pass" can be compared against a real number instead of a vague adjective.
{% else %}
# 实战记录 —— 一次真实落地的补丁集

以下细节来自一个真实落地的交接包（Rust 工作区，12 补丁 / 12 工单）。代码名是项目专属的，但**模式**几乎在每一份同类包里重复。

## 一、主要失败形态：包对着一个想象中的 API 写

| 包里的写法 | 仓库真实形态 | 编译器报错 |
| :--- | :--- | :--- |
| 转义字符字面量只写了一个反斜杠 | 需要双反斜杠 | `character literal may only contain one codepoint` + `unterminated character literal` |
| `fn f(text: &str, ...) -> &str` | 需要绑定到 `text` 的生命周期 | `missing lifetime specifier` |
| `style.size` | `style.font_size` | `no field 'size'` |
| `State::Normal` | `State::None`（变体为 `None` / `Error` / `Warning`） | `no variant named 'Normal'` |
| `rect.contains(p)` | `contains_point(p)` / `contains_rect(&r)` | `no method named 'contains'` |
| `place_popup(anchor, size, placement, viewport, gap, offset)` | 形参为 `(anchor, popup_size, viewport, placement)` | `takes 4 arguments but 6 were supplied` |
| 外层类型派生 `Eq`，但含只有 `PartialEq` 的类型 | 外层去掉 `Eq` | `... : Eq is not satisfied` |

**判定原则。** 报错集中在「方法 / 字段 / 变体不存在」时，改调用点去对齐仓库，**绝不**为了迁就补丁给共享公共类型加方法；报 `trait bound` 时，优先去掉**新**类型上多余的派生，而不是放松共享类型。

## 二、借用冲突只有一种固定改法

返回 `Vec<&str>` 的辅助方法借用了 `self`，在同一借用存活期间改 `self` 就会 `cannot assign to self.x because it is borrowed`。

```rust
// 反例：借用跨过了赋值
let lines = self.split_lines();
self.cursor_line -= 1;
self.cursor_col = Self::line_char_len(lines[self.cursor_line]);

// 正解：块内只取标量，借用结束再赋值
let len = {
    let lines = self.split_lines();
    Self::line_char_len(lines[target])
};
self.cursor_line = target;
self.cursor_col = self.cursor_col.min(len);
```

一次移动需要两个派生值（行长与行数）时，在块内返回元组。

## 三、会拦住推送的 lint 细节

- `let mut x = Vec::with_capacity(n)`，之后只被 move 进结构体、从不 push → 去掉 `mut`。
- 非测试代码里未处理的 `Result` / 随手 `.unwrap()` → 显式处理，门禁不会宽容。

## 四、折叠前提，具体化

`cp 最终版 → git add → commit --amend` 只在该文件于整个序列中被单次修改时成立：

```bash
git log --oneline <base>..HEAD -- src/<file>.rs | wc -l   # 必须为 1
```

大于 1 就说明这个修复该放在末条提交里，并在报告中写明映射为何被破坏。

## 五、测试永远看不见的缺口

某弹窗给菜单文案预留了容量，却从不 push 任何一条文案。形状测试全绿——因为断言比的是 24 个状态下的**元素个数**，空行照样算一行。人眼看是空菜单，测试看是完美。

所以：门禁全绿之后，亲手打开新增的输出路径看一眼。文字、图标、分隔符、副作用。把发现写进报告，让用户判断是否阻塞落地。

## 六、值得记下来的数字

把本次落地的测试通过数记下来。下次有人声称「测试全部通过」，你能拿真实数字对照，而不是对着一句形容词。
{% endif %}
