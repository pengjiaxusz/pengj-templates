# 将环境自检（doctor）与钩子激活收编进 CLI 二进制

> 本 spec 由 `/grill-with-docs` + `/to-spec` 产出。配套文档：[`GLOSSARY.md`](https://github.com/pengjiaxusz/pengj-templates/blob/main/GLOSSARY.md)、[`docs/adr/0005-doctor-in-cli-binary.md`](https://github.com/pengjiaxusz/pengj-templates/blob/main/docs/adr/0005-doctor-in-cli-binary.md)

## 问题陈述（Problem Statement）

当前 `pengj-templates` 的项目环境自检依赖模板 `common` 层在每个下游项目里生成两份 330+ 行的脚本（Unix 的 POSIX shell 脚本与 Windows 的 PowerShell 7 脚本），并配合 `gate` 层附带的独立钩子安装脚本一起工作。随着项目演进，这套「模板脚本 + CLI 子命令」的双层结构带来了明显摩擦：

- **双平台双语重复维护**：每增加一个层或检查项，都要同时在两份脚本里各写一遍逻辑与中英双语分支，极易发生平台漂移。
- **进程往返与诊断信息折损**：工作区链接、子仓库体检、共享编译目录本来都由 `pengj-templates-cli` 实现，但自检脚本只能把它们当作外部子进程拉起并丢弃标准输出，仅凭退出码打印一行笼统结论——遇到失败或警告时，用户还得再手工重跑对应子命令才能看到真正的问题与修复指引。
- **钩子激活存在两套实现与多余依赖**：提交门禁内部已经具备完整的 Rust 钩子安装逻辑（含 foreign hook 防覆盖、可选 `pre-push`、索引可执行位与竞品残留检测），但自检脚本的 `--fix` 绕开了它去调用独立 shell 脚本，导致 Windows 环境下反而额外依赖 `sh`。
- **零依赖前提已不成立**：凡是使用本模板项目的机器，跑提交门禁、工作区链接、子仓库体检或共享编译目录本来就必须安装 `pengj-templates-cli`。为了极少触发的「CLI 尚未安装时用 `--install-cli` 下载」而长期维护 660+ 行模板脚本，投入产出比失衡。

从用户视角看：**既然机器上本来就要装 `pengj-templates-cli`，自检应该就是一个内置、跨平台、开箱即用且能直接把问题说清楚的 CLI 命令，而不是堆在每个项目里的几百行样板脚本。**

## 解决方案（Solution）

将环境自检与钩子激活全面收编进 `pengj-templates-cli`：

- **新增内置子命令**：提供 `pengj-templates-cli doctor [--dir <项目>] [--fix] [--lang zh|en] [--json]`，不依赖模板目录，以项目内的 `.pengj-templates/manifest.json` 为唯一事实源，按已启用的层与技能精确运行对应检查。
- **进程内复用核心能力**：基础工具检查（`git`、`just`、Windows 下的 `pwsh`、`cargo`/`rustc`、`bque`、`node`）以及 `gate` 钩子激活、`agent` 工作区链接、`subrepo` 子仓库体检、`worktree` 共享编译目录检查全部在 CLI 进程内直接执行；通过时输出一行摘要，出现失败或警告时直接展开具体问题条目与修复建议。
- **全链路中英双语**：语言按 `--lang` 覆盖 > manifest 中固化的 `skill_lang` > 默认中文判定；`en` 模式下包含子检查展开明细在内的全部面向人输出零汉字，`--json` 字段名恒为英文。
- **彻底清理模板冗余脚本**：`common` 层不再产出两份自检脚本，`gate` 层不再产出独立钩子安装脚本与对应裸激活配方；`justfile` 托管块里的 `doctor` 配方跨平台合并为一条直调命令。
- **保留项目自定义扩展点**：原样保留 `.pengj-templates/scripts/doctor.d/` 契约，由 CLI 在内置检查完成后按文件名排序执行项目自定义脚本并汇总退出码。
- **显式投递破坏性迁移提示**：在迁移表中登记本次变更，存量项目执行 `update` 时自动清理未改动的旧脚本并收到清晰的一次性指引。

## 用户故事（User Stories）

1. 作为使用本模板的开发者，我希望运行 `just doctor` 或 `pengj-templates-cli doctor` 时直接由二进制完成全部环境自检，以便在 Windows 与 Unix 上获得完全一致的行为，不再受 shell / PowerShell 脚本差异困扰。
2. 作为新克隆项目的开发者，我希望运行 `just doctor --fix` 一次性完成提交钩子激活、agent 工作区链接创建与共享编译目录准备，以便不必记住多个子命令。
3. 作为 Windows 开发者，我希望 `just doctor --fix` 激活提交钩子时不再依赖本机存在 `sh`，以便在纯 PowerShell 环境下也能顺利完成初始化。
4. 作为项目维护者，我希望 `doctor` 自动读取项目 manifest 中的层清单与技能清单，只检查当前项目实际用到的工具与能力，以便未勾选 Rust、子仓库或工作树层的项目不会收到无关报错。
5. 作为项目维护者，我希望当子仓库体检、工作区链接或共享编译目录出现失败或警告时，`doctor` 直接在报告中展开具体出错项与修复建议，以便我无需再手工重跑第二条命令排错。
6. 作为项目维护者，我希望全部子检查都通过时输出保持简洁（每类一行摘要），以便日常自检报告一眼扫清、不被冗余细节淹没。
7. 作为英文项目（`skill_lang = "en"`）的维护者，我希望运行 `just doctor` 时所有面向人的输出（包括展开的子检查明细与修复建议）全部为英文且不含汉字，以便国外协作者或英文 CI 日志阅读无障碍。
8. 作为开发者，我希望必要时能通过 `--lang zh|en` 临时覆盖 `doctor` 的输出语言，以便在不修改 manifest 的前提下切换语言查看报告。
9. 作为自动化脚本或 CI 维护者，我希望通过 `pengj-templates-cli doctor --json` 获取稳定、全英文键名的结构化检查报告（含子检查完整明细），以便程序化消费结果。
10. 作为项目维护者，我希望可选工具缺失（如 `bque` 或 `mcp-bootstrap` 所需的 `node`）仅记为警告且 `doctor` 退出码仍为 `0`，只有存在错误时退出码才为 `1`，以便警告不会误伤 CI 或初始化流程。
11. 作为项目维护者，我希望继续通过在 `.pengj-templates/scripts/doctor.d/` 下放置自定义脚本来扩展项目专属检查（如 cmake、SDK），且沿用 `DOCTOR_ROOT` 与 `DOCTOR_FIX=1` 环境变量契约，以便存量自定义检查零改动兼容。
12. 作为未纳管目录的误操作者，我希望在缺少 `.pengj-templates/manifest.json` 的目录运行 `doctor` 时收到明确报错与先执行 `adopt` 的提示，而不是得到一份基于猜测的假通过报告。
13. 作为存量项目维护者，我希望在升级模板并运行 `update` 后，自动清理未修改过的旧自检脚本与旧钩子安装脚本，并在终端看到一条清晰的迁移提示，以便我知道旧入口已收编进 `just doctor` / `pengj-templates-cli doctor`。
14. 作为模板维护者，我希望不再在模板里维护双平台双语自检脚本与独立钩子安装脚本，以便后续新增层或检查项时只需在单一 Rust 实现中修改一次。

## 实现决策（Implementation Decisions）

### CLI 契约与退出语义

- 新增顶层子命令 `doctor`，参数为：
  - `--dir <路径>`（默认当前目录 `.`）
  - `--fix`（本地修复模式；永不联网）
  - `--lang <zh|en>`（可选，覆盖输出语言）
  - `--json`（输出结构化 JSON，键名恒为英文）
- 不依赖 `--templates` 参数（与 `repo`、`subrepo`、`worktree`、`check-lang` 一致，属离线项目级命令）。
- 退出码语义沿用旧版：`0` 表示无错误（允许存在警告），`1` 表示存在至少一项错误。所有提前退出走既有的统一退出包装，不丢失 CLI 后台更新提示。
- 新机器前置安装 `pengj-templates-cli` 成为显式约定，`--install-cli` 参数彻底退役，不再由 `doctor` 发起外网下载。

### 事实源与按层裁剪

- 以 `.pengj-templates/manifest.json` 为唯一事实源：读取 `layers` 与 `options["skills"]` 决定启用哪些内置检查；目录缺少 manifest 时直接返回错误并提示先执行 `adopt`。
- 内置检查项按层与技能精确裁剪：
  - **CLI 自身**：始终报告当前运行中的 `pengj-templates-cli` 版本为通过。
  - **基础工具**：`git`（始终，缺失为错误）；`just`（含 `just` 层，缺失为错误）；Windows 下的 `pwsh`（含 `just` 层，缺失为错误）；`cargo` 与 `rustc`（含 `rust` 或 `rust-workspace` 层，缺失为错误）；`bque`（含 `bque` 层：Windows 缺失为警告，非 Windows 标为不适用/通过）；`node`（`options.skills` 含 `mcp-bootstrap`，缺失为警告）。
  - **提交门禁（`gate` 层）**：校验位于 git 工作树内、`.githooks` 含必需钩子、`core.hooksPath` 指向 `.githooks` 以及 Unix 下的可执行位；`--fix` 时直接进程内调用门禁模块的钩子安装能力（覆盖必需钩子与可选 `pre-push`、拒绝覆盖非门禁钩子、补齐索引可执行位并提示竞品残留）。
  - **工作区链接（`agent` 层）**：当 `.agents` 目录存在时，进程内调用仓库初始化模块的状态校验；`--fix` 时先执行初始化再复核。
  - **子仓库体检（`subrepo` 层）**：进程内调用子仓库体检模块；失败映射为错误，仅警告映射为警告。
  - **共享编译目录（`worktree` 层）**：进程内调用工作树共享编译目录校验（`--fix` 时自动创建目录）；目录缺失或不可写映射为错误，目录就绪但外部 `CARGO_TARGET_DIR` 覆盖指向别处映射为警告。

### 进程内复用与展示粒度

- 自检核心编排与报告数据结构下沉到核心库（对外暴露可序列化报告类型），CLI 仅负责参数解析、语言渲染与退出码映射，与 `repo`、`subrepo`、`worktree`、`i18n_audit` 保持同一分层。
- 文本展示采用「通过摘要、异常展开」策略：
  - 子检查（`gate` / `repo` / `subrepo` / `worktree`）全部通过时，仅输出一行汇总。
  - 存在失败或警告时，在该汇总行下方缩进展开具体问题条目与修复建议。
- `--json` 输出始终包含完整的结构化检查项列表与子检查明细，不因通过与否而省略结构。

### 全链路中英双语

- 输出语言判定优先级：`--lang zh|en` > `manifest.options.skill_lang` > 默认 `zh`；传入非法语言值时报错退出。
- `doctor` 外壳（标题、状态标签、汇总行、基础工具提示）以及展开的 `gate`、`repo`、`subrepo`、`worktree` 摘要、明细与修复建议全部支持 `zh`/`en` 双语，确保 `en` 模式下所有面向人的文本输出零汉字。
- 保留各子模块既有中文接口的兼容性，通过结构化状态映射或补齐双语渲染方法实现，不在模板层残留双语脚本。

### 自定义扩展点（`doctor.d/`）

- 原样保留 `.pengj-templates/scripts/doctor.d/` 目录与契约：
  - 在全部内置检查执行完毕后扫描该目录，按文件名字典序执行（Unix 经 `sh` 执行 `*.sh`，Windows 经 `pwsh -NoProfile -File` 执行 `*.ps1`）。
  - 工作目录固定为项目根，注入环境变量 `DOCTOR_ROOT=<项目根>`，带 `--fix` 时注入 `DOCTOR_FIX=1`（未带 `--fix` 时确保不透传残留的 `DOCTOR_FIX`）。
  - 退出码 `0` 记通过，非 `0` 记错误；脚本输出按原样透出；无自定义脚本时打印一行提示。
- `common` 层继续产出 `.pengj-templates/scripts/doctor.d/README.md`（双语），更新其说明使其指向 `pengj-templates-cli doctor` / `just doctor` 而非旧脚本。

### 模板清理、配方收口与破坏性迁移

- `common` 层移除两份自检脚本产物。
- `gate` 层移除独立钩子安装脚本，以及配方模块中对应的裸激活配方（裸激活统一引导至 `pengj-templates-cli gate hooks install` 或 `just doctor --fix`）。
- `just` 层的根 `justfile` 托管块将 `[unix]` 与 `[windows]` 两条 `doctor` 配方合并为一条跨平台直调命令：`doctor *ARGS` 直接透传参数给 `pengj-templates-cli doctor --dir .`。
- 本次属于 `feat!` 破坏性模板变更：在迁移表（`MIGRATIONS`）中补一条迁移提示，说明旧脚本与裸激活配方已收编进 `pengj-templates-cli doctor`，存量项目 `update` 会自动删除未改动旧脚本，手动调用入口改为 `just doctor [--fix]` 或 `pengj-templates-cli doctor [--fix]`。
- 同步更新本仓库自身的托管 `justfile`、规范文档与双语 README。

## 测试决策（Testing Decisions）

### 好测试的标准

只测外部可观察行为：通过真实的目录状态、manifest 配置、命令输出文本/JSON 与退出码断言行为；不测内部私有辅助函数，不镜像实现细节。双语测试直接复用汉字检测逻辑断言 `en` 输出零汉字、`zh` 输出含中文。

### 测试接缝（复用现有，不新建）

继续复用仓库内联的 `mod tests` 接缝，不引入额外测试框架或 dev-dependencies：

1. **引擎端到端接缝（现有主接缝）**：定位真实模板目录 → 生成 / 更新到临时目录 → 断言旧脚本（两份自检脚本与独立钩子安装脚本）不再产出、`justfile` 合并为单条跨平台 `doctor` 配方、`gate` 配方模块不含旧裸激活配方、存量项目 `update` 自动清理未改动旧脚本并产出迁移提示，且全层双语门禁通过。
2. **自检核心模块内联接缝**：在临时目录构造最小 manifest 与真实 git 仓库/假文件，直接调用核心自检入口与双语渲染入口，覆盖：
   - 缺失 manifest 时报错；
   - 按层与技能裁剪检查项（仅 `common` 不查 Rust / gate / agent / subrepo / worktree；勾选对应层才触发对应检查）；
   - `--fix` 修复路径（提交钩子激活与可执行位、工作区链接创建、共享编译目录创建，以及非 `--fix` 下只报告不修改）；
   - 子检查异常展开明细与通过摘要两种形态；
   - `doctor.d/` 自定义脚本按序执行、`DOCTOR_ROOT` / `DOCTOR_FIX` 环境变量注入与非零退出码计入错误；
   - 退出码规则（仅警告返回 `0`，有错误返回 `1`）；
   - 语言切换优先级（`--lang` > `manifest.options.skill_lang` > 默认 `zh`），断言 `en` 模式全链路输出（含展开明细）零汉字，`--json` 字段名全英文。
3. **迁移表接缝（现有）**：复用迁移模块既有测试，验证新增迁移条目版本可解析、严格升序、摘要与动作非空。

### 测试先例参考

- 模板产物存在性、反向断言、更新清理与全层双语审计：引擎模块现有端到端用例。
- 仓库初始化只补缺不覆盖、子仓库体检失败/警告分级、共享编译目录推导与修复：各自核心模块的现有内联用例。
- 迁移表格式与版本区间断言：迁移模块的现有内联用例。

## 范围之外（Out of Scope）

- **在 `doctor` 内重新实现 CLI 自动下载安装（`--install-cli`）**：CLI 安装属于前置引导，由官方安装脚本承担，不再塞回 `doctor`。
- **无 manifest 目录的启发式探测**：不猜项目用了哪些层；未纳管项目必须先 `adopt`。
- **改动 `.pengj-templates/scripts/doctor.d/` 的目录路径或改为声明式配置**：保持原契约以维持存量项目零成本兼容。
- **全面改造 `pengj-templates-cli` 其余所有子命令（如 `audit`、`create`、`update`）为双语输出**：本次双语范围仅覆盖 `doctor` 及其展开的子检查展示，不顺手重写全 CLI 的 i18n。
- **把 `doctor` 的工具检查扩展为自动安装缺失工具（如自动装 Rust / just / node）**：`--fix` 严格遵守「永不联网、只修本地仓库状态」边界。

## 补充说明（Further Notes）

- **架构决策记录**：本次「收编进二进制、放弃零依赖脚本与 `--install-cli`」的完整权衡与后果已记录在 `docs/adr/0005-doctor-in-cli-binary.md`。
- **领域术语表**：本次未引入新的领域概念，`GLOSSARY.md` 保持不变。
- **与既有规范的对齐**：实施完成后需顺手清理 `AGENTS.md` 与 `templates/README.md` 中仍提及旧自检脚本、`--install-cli` 与 `.githooks/install.sh` 的过时描述，确保文档与实现单一事实源一致。
