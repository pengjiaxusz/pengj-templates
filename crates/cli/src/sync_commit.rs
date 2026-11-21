//! `update` 的收口步骤：精确暂存 → 生成提交信息 → 提交 → 推送。
//!
//! `template-sync` 技能里的 `sync-template.ps1` 原本自己干这几件事，代价是同一套
//! 语义有两份实现：scope 白名单、主题语言、暂存范围各写一遍，改一处漏一处。
//! 现在落盘动作只有这一份实现，ps1 退化成「批量巡检 + 汇总表 + 转调 CLI」。
//!
//! 不变式：
//! - 只暂存本次报告点名的路径与 manifest，绝不 `git add .`——工作区里可能还躺着
//!   并行会话或用户自己没提交的改动；
//! - 提交信息按门禁自己的规则推导（`commit.scopes` / `commit.subject_lang` 都从
//!   `.pengj-templates/gate.toml` 读），生成的信息当场就能过 `commit-msg` 钩子；
//! - 暂存后没有差异就跳过提交与推送，不制造空提交，也不把别处的脏改动推上去。

use std::path::Path;

use anyhow::anyhow;
use pengj_core::{UpdateReport, MANIFEST_FILE};
use pengj_gate::commit::Rules;
use pengj_gate::config::{self, SubjectLang};
use pengj_gate::git;

/// 收口选项：来自 `--commit` / `--push` / `--message`。
#[derive(Debug, Clone, Default)]
pub struct FinishOptions {
    /// 精确暂存并提交本次同步
    pub commit: bool,
    /// 提交后立即推送（调用方保证 push 蕴含 commit）
    pub push: bool,
    /// 覆盖自动推导的提交信息
    pub message: Option<String>,
}

/// 收口结果，供 CLI 打印。
#[derive(Debug, Clone)]
pub struct FinishReport {
    /// 本次提交覆盖的文件数；0 表示没有差异、跳过了提交
    pub staged: usize,
    /// 实际使用的提交信息
    pub message: String,
    /// 是否已完成推送
    pub pushed: bool,
}

/// 提交主题里愿意使用的 scope，按优先级排列：门禁放行的第一个胜出。
///
/// `templates` 是本仓库自己的白名单用词，`template` / `ai-skill` 是存量项目
/// （含 `sync-template.ps1` 时代生成的提交历史）里的用词。
const SCOPE_CANDIDATES: [&str; 3] = ["templates", "template", "ai-skill"];

const SUBJECT_ZH: &str = "同步分层模板更新";
const SUBJECT_EN: &str = "sync layered template updates";

/// 按 `--commit` / `--push` 完成收口；未开启时直接返回 `None`。
pub fn finish(
    dir: &Path,
    report: &UpdateReport,
    options: &FinishOptions,
) -> anyhow::Result<Option<FinishReport>> {
    if !options.commit && !options.push {
        return Ok(None);
    }
    if git::probe_in(dir, &["rev-parse", "--is-inside-work-tree"]).is_none() {
        anyhow::bail!(
            "{} 不是 git 工作树，--commit / --push 无处落脚",
            dir.display()
        );
    }

    let message = match &options.message {
        Some(message) => message.clone(),
        None => commit_message(dir)?,
    };

    let paths = touched_paths(report);
    stage(dir, &paths)?;
    let staged = staged_file_count(dir)?;
    if staged == 0 {
        return Ok(Some(FinishReport {
            staged: 0,
            message,
            pushed: false,
        }));
    }

    git::run_in(dir, &["commit", "-m", &message]).map_err(|error| {
        anyhow!("{error}\n提示：提交被拒通常是门禁或钩子的裁决，请先修好再重跑")
    })?;

    let mut pushed = false;
    if options.push {
        git::run_in(dir, &["push"]).map_err(|error| {
            anyhow!("推送失败：{error}\n提交已经落地，修好远端问题后单独 `git push` 即可")
        })?;
        pushed = true;
    }

    Ok(Some(FinishReport {
        staged,
        message,
        pushed,
    }))
}

/// 本次同步真正动过的路径 + manifest。
///
/// 冲突与被冻结的文件不在其中：它们没有被写过，暂存进来只会把别处的改动带进提交。
fn touched_paths(report: &UpdateReport) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    paths.extend(report.updated.iter().cloned());
    paths.extend(report.created.iter().cloned());
    paths.extend(report.removed.iter().cloned());
    paths.extend(report.docs_synced.iter().cloned());
    paths.push(MANIFEST_FILE.to_string());
    paths.sort();
    paths.dedup();
    paths
}

/// 精确暂存：路径表交给 git，删除的文件由 `-A` 记账。
fn stage(dir: &Path, paths: &[String]) -> anyhow::Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    // git 的 pathspec 只认正斜杠；报告里的相对路径在 Windows 上带反斜杠。
    let normalized: Vec<String> = paths.iter().map(|path| path.replace('\\', "/")).collect();
    let mut args: Vec<&str> = vec!["add", "-A", "--"];
    args.extend(normalized.iter().map(String::as_str));
    git::run_in(dir, &args).map_err(|error| anyhow!("暂存同步产物失败: {error}"))?;
    Ok(())
}

fn staged_file_count(dir: &Path) -> anyhow::Result<usize> {
    let output = git::run_in(dir, &["diff", "--cached", "--name-only"])
        .map_err(|error| anyhow!("读取暂存区失败: {error}"))?;
    Ok(output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count())
}

/// 推导提交信息：type 固定 `chore`，scope 取门禁放行的候选，主题语言跟门禁一致。
///
/// 项目没有 gate 层（没有 `gate.toml`）时退回 manifest 的 `commit_zh`，因为那时
/// 没有任何门禁能替我们判定语言，只能沿用生成时的选择。
pub fn commit_message(dir: &Path) -> anyhow::Result<String> {
    let (config, config_path) = config::load(dir).map_err(|error| anyhow!(error))?;
    let rules = Rules::from_config(&config);
    let compose = compose_message(
        &rules,
        subject_lang_fallback(dir, &rules, config_path.is_some()),
    );
    Ok(compose)
}

/// 主题语言：门禁配置优先；没有配置文件时按 manifest 的 `commit_zh` 兜底。
fn subject_lang_fallback(dir: &Path, rules: &Rules, has_config: bool) -> SubjectLang {
    if has_config {
        return rules.subject_lang;
    }
    let commit_zh = pengj_core::ProjectManifest::load(dir)
        .ok()
        .and_then(|manifest| manifest.options.get("commit_zh").and_then(|v| v.as_bool()))
        .unwrap_or(true);
    if commit_zh {
        SubjectLang::Zh
    } else {
        SubjectLang::En
    }
}

/// 拼装提交主题（纯函数，便于测试）。
fn compose_message(rules: &Rules, lang: SubjectLang) -> String {
    let subject = match lang {
        SubjectLang::Zh => SUBJECT_ZH,
        SubjectLang::En => SUBJECT_EN,
    };
    match SCOPE_CANDIDATES
        .iter()
        .find(|scope| rules.scopes.allows(scope))
    {
        Some(scope) => format!("chore({scope}): {subject}"),
        // 门禁不接受任何 scope 时必须省略括号，否则 commit-msg 直接拒绝
        None => format!("chore: {subject}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(raw: &str) -> Rules {
        Rules::from_config(&config::parse(raw).expect("测试配置应可解析"))
    }

    #[test]
    fn wildcard_scope_uses_templates() {
        let rules = rules("commit.scopes = [\"*\"]\n");
        assert_eq!(
            compose_message(&rules, SubjectLang::Zh),
            "chore(templates): 同步分层模板更新"
        );
    }

    #[test]
    fn whitelist_picks_first_allowed_candidate() {
        let rules = rules("commit.scopes = [\"cli\", \"ai-skill\"]\n");
        assert_eq!(
            compose_message(&rules, SubjectLang::En),
            "chore(ai-skill): sync layered template updates"
        );
    }

    #[test]
    fn whitelist_without_candidates_drops_scope() {
        // 门禁只放行 cli：带任何候选 scope 都会被拒，必须退回无 scope 形态
        let rules = rules("commit.scopes = [\"cli\"]\ncommit.subject_lang = \"zh\"\n");
        assert_eq!(
            compose_message(&rules, SubjectLang::Zh),
            "chore: 同步分层模板更新"
        );
    }

    #[test]
    fn empty_scope_list_drops_scope() {
        let rules = rules("commit.scopes = []\n");
        assert_eq!(
            compose_message(&rules, SubjectLang::Zh),
            "chore: 同步分层模板更新"
        );
    }

    #[test]
    fn subject_language_follows_gate_config() {
        let rules = rules("commit.subject_lang = \"en\"\n");
        let message = compose_message(&rules, rules.subject_lang);
        assert_eq!(message, "chore: sync layered template updates");
    }

    #[test]
    fn touched_paths_cover_report_and_manifest() {
        let report = UpdateReport {
            project_name: "demo".to_string(),
            layers: Vec::new(),
            updated: vec!["AGENTS.md".to_string()],
            created: vec![".pengj-templates\\just\\bque.just".to_string()],
            conflicted: Vec::new(),
            removed: vec![".agents/skills/issue-sync/SKILL.md".to_string()],
            needs_review: vec!["justfile".to_string()],
            dropped_skills: Vec::new(),
            docs_synced: vec![".pengj-templates/gate.toml".to_string()],
            migrations: Vec::new(),
            unchanged: 0,
        };
        let paths = touched_paths(&report);
        assert!(paths.contains(&"AGENTS.md".to_string()));
        assert!(paths.contains(&".pengj-templates\\just\\bque.just".to_string()));
        assert!(paths.contains(&".agents/skills/issue-sync/SKILL.md".to_string()));
        assert!(paths.contains(&MANIFEST_FILE.to_string()));
        // 只是提示复核、没被写过的文件不能进暂存，否则会把别处的改动带进提交
        assert!(!paths.contains(&"justfile".to_string()));
    }
}
