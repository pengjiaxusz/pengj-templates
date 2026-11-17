//! 破坏性模板变更的投递通道
//!
//! 模板里凡是「改了就让存量项目按旧方式出错」的变更，光写进 CHANGELOG 是没用的：
//! 下游项目不会有人去翻。`update` 是唯一一次必然发生的接触，所以把变更说明挂在
//! 它上面——拿 manifest 记录的版本和当前版本一比，版本区间内的条目全部打印出来。
//!
//! 约定：任何 `feat!` / `BREAKING CHANGE` 的模板变更，必须在 [`MIGRATIONS`] 里补
//! 一条，并与 CHANGELOG 的条目一一对应；否则它只到维护者为止。

/// 一条迁移提示：版本 + 发生了什么 + 需要做什么。
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationNote {
    /// 引入该变更的版本（与 CHANGELOG 的标题版本一致）
    pub version: &'static str,
    /// 变更本身：旧行为 -> 新行为
    pub summary: &'static str,
    /// 存量项目要做的动作；无需动作时写「无需改动」
    pub action: &'static str,
}

/// 全部迁移提示，按版本升序排列。
///
/// 只收录**语义反转 / 必然导致旧项目行为改变**的条目；能自动迁移的（层名别名、
/// 技能合并）不在此列——它们由引擎直接处理，不需要人看。
pub const MIGRATIONS: &[MigrationNote] = &[
    MigrationNote {
        version: "0.67.0",
        summary: "commit.scopes 的语义从「空列表 = 放行所有 scope」反转为「拒绝优先的三态策略」：\
                  缺省或 [] 拒绝所有 scope，[\"*\"] 放行任意 scope，列表则收窄为白名单",
        action: "没有 scope 白名单的项目请在 .gate.toml 设 commit.scopes = [\"*\"] 恢复旧行为；\
                 已经维护白名单的项目无需改动，但请留意 .gate.toml 里的旧说明已过期",
    },
    MigrationNote {
        version: "0.73.0",
        summary: "agent 层的默认工作区链接从 2 条变 3 条（新增 `.workbuddy/skills` → `.agents/skills`），\
                  且 doctor 在 gate/agent 层项目里把「缺 pengj-templates-cli」从警告改判为错误——\
                  存量项目 update 后第一次跑 doctor 会报错，直到链接补齐",
        action: "跑一次 `just doctor --fix`（或 `pengj-templates-cli repo init --dir .`）补出链接；\
                 机器上没有 CLI 时先跑 `just doctor --install-cli`（或其提示里的安装命令），\
                 重开终端后再跑一次 doctor",
    },
    MigrationNote {
        version: "0.74.0",
        summary: "模板产物从项目根收敛到 `.pengj-templates/`：清单 `.pengj-templates.json` → \
                  `.pengj-templates/manifest.json`、门禁配置 `.gate.toml` → `.pengj-templates/gate.toml`、\
                  自检脚本 `scripts/doctor*` → `.pengj-templates/scripts/`、配方模块 `just/*.just` → \
                  `.pengj-templates/just/`。根目录只保留 `justfile` 与 `.githooks/`",
        action: "先 `git mv .pengj-templates.json .pengj-templates/manifest.json`，再跑 update：\
                 新版把其余产物写到新路径并删除根上的旧副本；根 `scripts/` 里项目自有的脚本留在原处。\
                 `.gate.toml` 的配置值要手工搬进 `.pengj-templates/gate.toml`（该文件归项目所有，\
                 update 只刷新说明块、不覆盖值），搬完删掉根上的旧文件再跑一次 update 清账本",
    },
];

/// 当前工具的版本（crate 版本即发布版本）
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// 解析 `1.2.3` / `v1.2.3` / `1.2.3-beta.1` 形式的版本号。
///
/// 只取前三段数字，预发布与构建元数据丢弃；任何一段不是数字时返回 `None`，
/// 由调用方决定「版本不可比就不猜」。
fn parse_version(raw: &str) -> Option<(u64, u64, u64)> {
    let trimmed = raw.trim();
    let trimmed = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let core = trimmed
        .split(['-', '+'])
        .next()
        .unwrap_or(trimmed)
        .to_string();
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

/// 取版本区间 `(from, to]` 内的迁移提示。
///
/// - `to` 不可解析 -> 空（当前版本一定可解析，这属于构造错误而非运行时情况）
/// - `from` 不可解析 -> 空：不知道从哪儿来，就不做任何断言，避免吓人一跳
/// - `from >= to` -> 空（已是最新，或 manifest 被手工改大）
pub fn notes_between(from: &str, to: &str) -> Vec<MigrationNote> {
    let (Some(from), Some(to)) = (parse_version(from), parse_version(to)) else {
        return Vec::new();
    };
    if from >= to {
        return Vec::new();
    }
    MIGRATIONS
        .iter()
        .filter(|note| match parse_version(note.version) {
            Some(v) => v > from && v <= to,
            None => false,
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_prefixed_versions() {
        assert_eq!(parse_version("0.71.0"), Some((0, 71, 0)));
        assert_eq!(parse_version("v0.71.0"), Some((0, 71, 0)));
        assert_eq!(parse_version("0.18.0-beta.1"), Some((0, 18, 0)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("发布版"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn returns_notes_inside_the_interval_only() {
        let notes = notes_between("0.66.0", current_version());
        assert!(
            notes.iter().any(|n| n.version == "0.67.0"),
            "0.66 -> 当前 应包含 0.67.0 的破坏性变更: {notes:?}"
        );

        let after = notes_between("0.67.0", current_version());
        assert!(
            after.iter().all(|n| n.version != "0.67.0"),
            "区间是左开右闭，(from, to] 不含 from 本身"
        );
    }

    #[test]
    fn unknown_or_newer_versions_yield_nothing() {
        assert!(notes_between("不是版本号", current_version()).is_empty());
        assert!(notes_between(current_version(), current_version()).is_empty());
        assert!(notes_between("99.0.0", current_version()).is_empty());
    }

    /// 表的版本必须能解析、按升序排列、字段非空——这是给未来的自己设的路障。
    #[test]
    fn table_is_well_formed() {
        let mut last = (0u64, 0u64, 0u64);
        for note in MIGRATIONS {
            let v = parse_version(note.version).unwrap_or_else(|| {
                panic!("迁移表里的版本 {} 不可解析", note.version);
            });
            assert!(v > last, "迁移表必须按版本升序：{} 乱序", note.version);
            last = v;
            assert!(!note.summary.trim().is_empty());
            assert!(!note.action.trim().is_empty());
        }
    }
}
