---
name: commit
description: >-
  模板仓库提交：提交/amend 前 MUST 先按本 skill 的「提交前完整性检查」扫 diff 自主判定（构建验证 / 文档同步 / 格式与命名），
  再按约定式提交写中文 message 并拆分无关改动；scope 需要新增时按本 skill 流程自主更新 commitlint.config.js。
  支持切发 pre-release（beta/preview/rc）：用户提出"切 beta/preview/rc/预发布"时自动按 §5b 推算并写入 Release-As。
  每次提交成功后 MUST 立即 push。
  Triggers: commit, 提交, amend, push, commit message, conventional commits, Commitlint, 拆分提交, scope, 切 beta, 切 preview, 切 rc, 预发布, prerelease, Release-As.
---

# commit 提交流程

中文提交信息 + PowerShell 兼容执行。除 type/scope 外一律中文。**提交后立即 push**（防止本机故障丢失最新提交）。

## 流程

### 1. 收集提交上下文（标准 git 命令，跨平台）

在仓库根目录运行，一次性拿到全部分区信息，便于拆分提交与完整性检查：

```powershell
git status
git diff --stat
git diff --cached --stat
```

需要看完整内容时再补：

```powershell
git diff          # 未暂存完整 diff
git diff --cached # 已暂存完整 diff
```

### 2. 提交前完整性检查（扫 diff 自主判定）

扫上一步的 status / diff / 未跟踪列表，按**原则三问**判定命中哪些检查——**判定以 diff 实际内容为准**：

**原则三问**（每次提交都问，只看 diff）：

| # | 问题 | 命中 → 要查 |
| --- | --- | --- |
| 1 | 是否改动构建/依赖？（`Cargo.toml`、`.cargo/config.toml`、`justfile`、`package.json`、`pnpm-workspace.yaml`、新依赖） | **构建验证** |
| 2 | 是否触及已文档化模块/契约，或新增/修改了配置、技能、脚本？ | **文档与 AGENTS.md 同步** |
| 3 | 是否新增/修改了公开 API、类型、命名，或改动需要新的 commit scope？ | **格式与命名 + scope 增补** |

**判定表（常见触发条件，速查参考）**

| diff 里看到 | 命中的检查 |
| --- | --- |
| 改了 `Cargo.toml` / `.cargo/` / `justfile` / pnpm 相关 | 构建验证 |
| 新增/修改了模块结构、`AGENTS.md`、`.agents/` 技能、配置 | 文档与 AGENTS.md 同步 |
| 改了公开 API/类型/命名，或需新 scope | 格式与命名 + `commitlint.config.js` scope-enum |
| 纯测试、纯格式化、纯注释、配置微调、修 bug 不改行为 | **无（快速路径）** |

**命中后的核对与补齐**（缺则补，补完才算通过）：
- **构建验证**：在仓库根目录 `just build`（或按 `AGENTS.md` 指定命令）验证通过；禁止裸 `cargo build`。
- **文档同步**：核对 `AGENTS.md` 是否准确，新增模块/配置/技能补说明。
- **格式与命名**：`just fmt`；scope 白名单缺失时按本 skill 流程更新 `commitlint.config.js`。

*注：文档/杂项更新可放独立提交（如 `docs(scope)` / `chore(agent)`），但工作树里必须全部补齐。*

### 3. 判断是否拆分提交

一律不相关领域拆多次提交（例如：功能改动与文档改动不要混合提交；构建配置改动与业务逻辑分开提交）。

### 4. 选 type 与 scope 并撰写 commit message

`type(scope): 中文标题`

#### type 速记
`feat`新能力 `fix`修错 `docs`文档 `style`格式命名 `refactor`重构不改行为 `perf`性能 `test`测试 `build`构建依赖 `ci`CI `chore`杂项 `revert`回滚

#### scope 规则
- 读 `commitlint.config.js` 的 `scope-enum` 白名单；文件不存在时跳过。
- 无合适 scope 则省略（例如 `chore: ...`）。

### 5. 执行提交与 Push（PowerShell 兼容）

```powershell
git commit -m "fix: 中文标题" -m "正文。`n`n变更：`n- 点1`n- 点2"
git push
```

- **必须使用 PowerShell 兼容语法**：每个 `-m` 一段，正文内 `` `n `` 换行。禁止使用 bash heredoc（`<<'EOF'`）。
- 提交成功后**必须立即执行 `git push`**。
- 若远端有更新，先 `git pull --rebase` 再 `git push`。

### 5b. 可选：切发 pre-release（beta / preview / rc）

默认**发稳定版**（无需任何额外操作，release-please 自动 bump）。仅当用户明确要求"切 beta / preview / rc / 预发布"时才走本节。

1. **读当前版本号**：从 `.github/release-please-manifest.json` 的 `"."` 字段读取当前版本（如 `0.5.0`）——这是 release-please 的权威版本源，**不要**从 `Cargo.toml`/`package.json`/`tauri.conf.json` 读（它们可能滞后）。
2. **推算目标 pre-release 号**（`n` 为当前 `minor`，`p` 为当前 `patch`）：
   - 稳定版 → beta / preview / rc：取 `major.(n+1).0-<type>.1`（如 `0.5.0` → `0.6.0-beta.1`）。
   - 已在同阶段 pre-release（如 `0.6.0-beta.2`）且用户指同一类型：不加 `Release-As`，release-please 会自动 +1（`beta.2`→`beta.3`）。
   - 切到**另一类型**（如 `beta`→`rc`）：取 `major.minor.patch-<新type>.1`（如 `0.6.0-beta.3` → `0.6.0-rc.1`）。
   - 用户显式报了版本号：直接用用户给的号。
3. **写提交信息时**在正文末尾追加 footer 行（PowerShell 多 `-m` 传一段）：
   `Release-As: <推算的 pre-release 号>`
4. 后续流程不变（push 后 release-please 会据此创建 pre-release PR，GitHub 自动标 **Prerelease**）。

*注：`Release-As:` 只在"切换通道"那一次需要；同阶段之后的普通 `feat`/`fix` 会自动递增 pre-release 号，无需再带。*

### 6. Amend

仅当用户明确要求且符合「只改刚提交、未 push、无他人依赖」时。Amend 前仍跑「提交前完整性检查」；amend 成功后同样立即 push。

## 质量要求

- 标题中文短句、不加句号、不空泛。
- 提交失败重试时带上 `git add`。
- 收尾确认 `git status` 干净且本地无未推送提交。
