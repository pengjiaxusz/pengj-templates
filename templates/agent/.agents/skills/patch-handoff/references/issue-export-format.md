{% if options["skill_lang"] == "en" %}
# Export Format & Front Matter Contract

Pull writes this layout; push accepts the same per-issue files. One format, both directions.

## 1. Bundle Layout

```
<out>/
├── README.md                     # index table + topology tree (templates/index.md + templates/topology.md)
├── issues/
│   └── NNNN-<kind>-<slug>.md     # one per issue (templates/issue.md)
└── data/
    ├── issues.json               # raw gh issue list payload
    └── relations.json            # per issue: parent / sub_issues / blocked_by / blocking / comments
```

`-Zip` compresses the same tree with the output directory name as the archive root.

## 2. Front Matter

Scalars and inline arrays only — no nested maps, no block sequences. Push parses exactly this subset.

| Field | Required on push | Meaning |
| :--- | :--- | :--- |
| `key` | yes | Local identity used by `parent` / `blocked_by`. Pull writes the issue number; hand-written plans may use any stable string (`S1-01`, `T-spec`). |
| `title` | yes | Issue title. |
| `labels` | no | `[a, b]`. Labels must already exist on the repo or push fails. |
| `state` | no | `open` / `closed`; informational on pull, ignored by push. |
| `kind` | no | `epic` / `ticket` / `issue`, or a name from `-KindRules`. Informational. |
| `parent` | no | Key of the parent issue. Becomes a sub-issue link. |
| `blocked_by` | no | `[key, key]`. Becomes native `blocked_by` edges. |
| `blocks` | no | Inverse view, computed on pull. Ignored by push. |
| `number` | no | Real issue number, filled by push and by pull. |
| `url` | no | Canonical URL. |
| `created_at` / `updated_at` | no | ISO timestamps. |

Unknown keys are preserved verbatim in the file and ignored by push.

## 3. Kind Resolution

Order of precedence:

1. `-KindRules "spec=^Spec\s,ticket=^S\d-\d\d"` — first matching regex on the title wins.
2. No rule matches: `epic` if the issue has sub-issues, else `ticket` if it has a parent, else `issue`.

Kind only affects the file name and the index column; nothing in the API depends on it.

## 4. Templates

`templates/` holds three plain Markdown files with `{% raw %}{{PLACEHOLDER}}{% endraw %}` slots. Copy any of them next to your bundle and pass `-Templates <dir>` to override the built-in set; no code changes.

| Template | Placeholders |
| :--- | :--- |
| `issue.md` | `{% raw %}{{FRONT_MATTER}}{% endraw %}` `{% raw %}{{TITLE}}{% endraw %}` `{% raw %}{{META_TABLE}}{% endraw %}` `{% raw %}{{BODY}}{% endraw %}` `{% raw %}{{COMMENTS}}{% endraw %}` |
| `index.md` | `{% raw %}{{REPO}}{% endraw %}` `{% raw %}{{COUNT}}{% endraw %}` `{% raw %}{{RANGE}}{% endraw %}` `{% raw %}{{GENERATED_AT}}{% endraw %}` `{% raw %}{{TABLE}}{% endraw %}` `{% raw %}{{TOPOLOGY}}{% endraw %}` `{% raw %}{{NOTES}}{% endraw %}` |
| `topology.md` | `{% raw %}{{TREE}}{% endraw %}` `{% raw %}{{NOTE}}{% endraw %}` |

Missing placeholders are left empty rather than failing the render.

## 5. Resume State

Push writes `<mapping>.state.json` next to the mapping file:

```json
{ "pending_creates": ["S3-02"], "pending_parents": ["S3-02"], "pending_edges": [["S3-04", "S3-01"]] }
```

`-Resume` reads it and continues. `-DryRun` never writes state.
{% else %}
# 产物格式与 front matter 契约

读侧按这个布局写盘，写侧接受同一套单篇文件。一个格式，两个方向。

## 1. 包结构

```
<out>/
├── README.md                     # 索引表 + 拓扑树（templates/index.md + templates/topology.md）
├── issues/
│   └── NNNN-<kind>-<slug>.md     # 每单一份（templates/issue.md）
└── data/
    ├── issues.json               # gh issue list 原始载荷
    └── relations.json            # 每单的 parent / sub_issues / blocked_by / blocking / comments
```

`-Zip` 把同一棵树压缩，压缩包以输出目录名作根。

## 2. Front Matter

只支持标量与行内数组，不支持嵌套 map 与块序列。写侧就按这个子集解析。

| 字段 | 推送时必需 | 含义 |
| :--- | :--- | :--- |
| `key` | 是 | 本地身份，供 `parent` / `blocked_by` 引用。读侧写的是 issue 编号；手写计划可用任意稳定串（`S1-01`、`T-spec`）。 |
| `title` | 是 | issue 标题。 |
| `labels` | 否 | `[a, b]`。标签必须已在仓库存在，否则推送失败。 |
| `state` | 否 | `open` / `closed`；读侧记录，写侧忽略。 |
| `kind` | 否 | `epic` / `ticket` / `issue`，或 `-KindRules` 给出的名字。仅供展示。 |
| `parent` | 否 | 父单的 key，会挂成 sub-issue。 |
| `blocked_by` | 否 | `[key, key]`，连成原生 `blocked_by` 边。 |
| `blocks` | 否 | 反向视图，读侧计算。写侧忽略。 |
| `number` | 否 | 真实编号，读写两侧都会填。 |
| `url` | 否 | 规范链接。 |
| `created_at` / `updated_at` | 否 | ISO 时间戳。 |

不认识的键原样保留，写侧忽略。

## 3. kind 判定

优先级：

1. `-KindRules "spec=^Spec\s,ticket=^S\d-\d\d"` —— 标题上第一条命中的正则胜出。
2. 没有规则命中：有子单为 `epic`，有父单为 `ticket`，否则 `issue`。

kind 只影响文件名与索引列，不参与任何 API 调用。

## 4. 模板

`templates/` 下是三份纯 Markdown，占位符写作 `{% raw %}{{PLACEHOLDER}}{% endraw %}`。把任意一份复制到你的包旁边、用 `-Templates <dir>` 指过去即可覆盖内置版式，不用改代码。

| 模板 | 占位符 |
| :--- | :--- |
| `issue.md` | `{% raw %}{{FRONT_MATTER}}{% endraw %}` `{% raw %}{{TITLE}}{% endraw %}` `{% raw %}{{META_TABLE}}{% endraw %}` `{% raw %}{{BODY}}{% endraw %}` `{% raw %}{{COMMENTS}}{% endraw %}` |
| `index.md` | `{% raw %}{{REPO}}{% endraw %}` `{% raw %}{{COUNT}}{% endraw %}` `{% raw %}{{RANGE}}{% endraw %}` `{% raw %}{{GENERATED_AT}}{% endraw %}` `{% raw %}{{TABLE}}{% endraw %}` `{% raw %}{{TOPOLOGY}}{% endraw %}` `{% raw %}{{NOTES}}{% endraw %}` |
| `topology.md` | `{% raw %}{{TREE}}{% endraw %}` `{% raw %}{{NOTE}}{% endraw %}` |

缺占位符时留空，不会让渲染失败。

## 5. 续传状态

写侧在映射文件旁写 `<mapping>.state.json`：

```json
{ "pending_creates": ["S3-02"], "pending_parents": ["S3-02"], "pending_edges": [["S3-04", "S3-01"]] }
```

`-Resume` 读它继续。`-DryRun` 永不写状态。
{% endif %}
