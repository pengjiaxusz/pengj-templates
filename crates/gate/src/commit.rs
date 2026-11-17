//! Conventional-commit validation.
//!
//! Replaces `commitlint` + `@commitlint/config-conventional`. The upstream
//! rule set was read from `commitlint --print-config` and every rule that can
//! reject a message is implemented here, so the gate is never looser than the
//! tool it replaces:
//!
//! | rule                     | level | implemented |
//! |--------------------------|-------|-------------|
//! | `header-trim`            | error | yes         |
//! | `header-max-length`      | error | yes         |
//! | `type-empty`             | error | yes         |
//! | `type-case`              | error | yes         |
//! | `type-enum`              | error | yes         |
//! | `scope-enum`             | error | yes         |
//! | `subject-empty`          | error | yes         |
//! | `subject-full-stop`      | error | yes         |
//! | `subject-language`       | error | yes         |
//! | `footer-max-line-length` | error | yes         |
//!
//! `body-leading-blank` and `footer-leading-blank` are warnings upstream and
//! `body-max-line-length` is disabled by the project config, so none of them
//! can reject a message and none of them are implemented here.
//!
//! The header grammar follows the upstream parser pattern
//! `/^(\w*)(?:\((.*)\))?!?: (.*)$/`: the colon must be followed by a space and
//! the type must be made of word characters. A header that does not match
//! leaves type and subject empty, which is what upstream reports.

use crate::config::{Config, SubjectLang};
use crate::lang_guard::is_han;

const CONVENTIONAL_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];
const DEFAULT_MAX_HEADER: usize = 100;
const DEFAULT_MIN_HAN: usize = 2;
const DEFAULT_MAX_FOOTER_LINE: usize = 100;
const MAX_UNQUOTED_HAN_IN_ENGLISH: usize = 4;

/// Upstream splits a scope on these delimiters and accepts the scope when
/// every part is whitelisted.
const SCOPE_DELIMITERS: [char; 3] = ['/', '\\', ','];

/// `commit.scopes` 的三种策略。
///
/// 未配置与空列表都归到 `None`：这是刻意收紧的默认值——没有显式声明 scope
/// 白名单的项目不接受任何 scope，"忘了配"不能再等于"全放行"。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopePolicy {
    /// 未配置或 `[]`：拒绝任何带 scope 的 header。
    None,
    /// `["*"]`：放行任意 scope。
    Any,
    /// 具体列表：只放行列出的 scope，或每个分段都在列表里的 scope。
    Whitelist(Vec<String>),
}

impl ScopePolicy {
    /// `*` 是"放行所有"的哨兵；空列表与未配置同为"拒绝所有"。
    pub fn from_config(config: &Config) -> ScopePolicy {
        match config.list("commit.scopes") {
            None => ScopePolicy::None,
            Some([]) => ScopePolicy::None,
            Some(items) if items.iter().any(|item| item == "*") => ScopePolicy::Any,
            Some(items) => ScopePolicy::Whitelist(items.to_vec()),
        }
    }

    /// `scope-enum` 的裁决：整串命中，或每个分段都命中即通过。
    pub fn allows(&self, scope: &str) -> bool {
        match self {
            ScopePolicy::Any => true,
            ScopePolicy::None => false,
            ScopePolicy::Whitelist(allowed) => {
                allowed.iter().any(|item| item == scope)
                    || scope
                        .split(SCOPE_DELIMITERS)
                        .map(str::trim)
                        .all(|part| allowed.iter().any(|item| item == part))
            }
        }
    }

    /// 拒绝时的提示前缀。`Any` 永不拒绝，这里只为穷尽匹配。
    pub fn rejection_hint(&self) -> String {
        match self {
            ScopePolicy::Any => String::new(),
            ScopePolicy::None => {
                "no scope is allowed (omit `(...)` or set commit.scopes = [\"*\"] in .pengj-templates/gate.toml)"
                    .to_string()
            }
            ScopePolicy::Whitelist(items) => {
                format!("the scope must be one of [{}]", items.join(", "))
            }
        }
    }
}

pub struct Rules {
    pub types: Vec<String>,
    pub scopes: ScopePolicy,
    pub subject_lang: SubjectLang,
    pub min_han: usize,
    pub max_header: usize,
    pub max_footer_line: usize,
}

impl Rules {
    pub fn from_config(config: &Config) -> Rules {
        let types = config
            .list("commit.types")
            .map(|items| items.to_vec())
            .unwrap_or_else(|| {
                CONVENTIONAL_TYPES
                    .iter()
                    .map(|item| item.to_string())
                    .collect()
            });
        let scopes = ScopePolicy::from_config(config);
        let subject_lang = config
            .string("commit.subject_lang")
            .and_then(SubjectLang::parse)
            .unwrap_or(SubjectLang::Zh);
        let min_han = config
            .number("commit.subject_min_han")
            .map(|value| value.max(0.0) as usize)
            .unwrap_or(DEFAULT_MIN_HAN);
        let max_header = config
            .number("commit.max_header")
            .map(|value| value.max(1.0) as usize)
            .unwrap_or(DEFAULT_MAX_HEADER);
        let max_footer_line = config
            .number("commit.max_footer_line")
            .map(|value| value.max(1.0) as usize)
            .unwrap_or(DEFAULT_MAX_FOOTER_LINE);
        Rules {
            types,
            scopes,
            subject_lang,
            min_han,
            max_header,
            max_footer_line,
        }
    }
}

/// The message split the way the upstream parser splits it.
pub struct Parsed {
    /// The raw first line, whitespace included.
    pub header: String,
    /// `None` when the header did not match the grammar.
    pub kind: Option<String>,
    pub scope: Option<String>,
    pub subject: Option<String>,
    /// The trailer lines, if the message has any.
    pub footer: Vec<String>,
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Match `/^(\w*)(?:\((.*)\))?!?: (.*)$/`.
///
/// The pattern is greedy, so the type takes as many word characters as it can
/// and the scope takes as much as it can; both fall back to shorter matches
/// when the rest of the header does not line up.
fn match_header(line: &str) -> (Option<String>, Option<String>, Option<String>) {
    let bytes = line.as_bytes();
    let type_len = bytes.iter().take_while(|byte| is_word_byte(**byte)).count();

    for split in (0..=type_len).rev() {
        let rest = &line[split..];
        if rest.starts_with('(') {
            // Greedy scope: try the last closing parenthesis first.
            let closers: Vec<usize> = rest
                .char_indices()
                .filter(|(_, c)| *c == ')')
                .map(|(index, _)| index)
                .collect();
            for closer in closers.into_iter().rev() {
                if let Some(header) =
                    finish_header(line, split, Some(&rest[1..closer]), &rest[closer + 1..])
                {
                    return header;
                }
            }
        }
        if let Some(header) = finish_header(line, split, None, rest) {
            return header;
        }
    }
    (None, None, None)
}

/// Consume the optional `!` and the mandatory `: `, then read the subject.
fn finish_header(
    line: &str,
    type_end: usize,
    scope: Option<&str>,
    rest: &str,
) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let rest = rest.strip_prefix('!').unwrap_or(rest);
    let subject = rest.strip_prefix(": ")?;
    let kind = &line[..type_end];
    Some((
        empty_to_none(kind),
        scope.and_then(empty_to_none),
        empty_to_none(subject),
    ))
}

/// Upstream turns an empty capture into `null`.
fn empty_to_none(text: &str) -> Option<String> {
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Drop the lines git and commitlint drop: comments and GPG signatures.
///
/// Upstream trims leading and trailing newlines before it splits the message,
/// so a blank first line never becomes the header.
fn message_lines(raw: &str) -> Vec<&str> {
    let trimmed = raw.trim_matches(['\r', '\n']);
    trimmed
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter(|line| !line.trim_start().starts_with("gpg:"))
        .collect()
}

/// Upstream's footer grammar: `^(?:BREAKING CHANGE|[\w-]+)(?::\s+|\s+#).+`
fn is_footer_token(line: &str) -> bool {
    let trimmed = line.trim_start();
    let token_len = trimmed
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .unwrap_or(trimmed.len());
    let token = &trimmed[..token_len];
    if token.is_empty() {
        return false;
    }
    let tail = &trimmed[token_len..];
    if let Some(after) = tail.strip_prefix(':') {
        // `: ` followed by a non-empty value.
        return after.starts_with(' ') && !after.trim().is_empty();
    }
    if let Some(after) = tail.strip_prefix(' ') {
        // ` #123` is a reference, not a footer token.
        return after.starts_with('#') && after.trim().len() > 1;
    }
    false
}

/// Upstream's note grammar: `^(?:\*\s+)?(BREAKING CHANGE|BREAKING-CHANGE):\s*`
fn is_note(line: &str) -> bool {
    let trimmed = line
        .trim_start()
        .strip_prefix("* ")
        .unwrap_or(line.trim_start());
    let lowered = trimmed.to_ascii_lowercase();
    lowered.starts_with("breaking change:") || lowered.starts_with("breaking-change:")
}

/// The trailer lines: everything from the first note or footer token onwards.
fn collect_footer(lines: &[&str]) -> Vec<String> {
    let start = lines
        .iter()
        .position(|line| is_note(line) || is_footer_token(line));
    match start {
        Some(index) => lines[index..].iter().map(|line| line.to_string()).collect(),
        None => Vec::new(),
    }
}

/// Parse a raw commit message the way the upstream parser does.
pub fn parse(raw: &str) -> Parsed {
    let lines = message_lines(raw);
    let header = lines.first().copied().unwrap_or("").to_string();
    let (kind, scope, subject) = match_header(&header);
    let footer = collect_footer(&lines[1.min(lines.len())..]);
    Parsed {
        header,
        kind,
        scope,
        subject,
        footer,
    }
}

/// JavaScript's `String.length` counts UTF-16 code units.
fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

fn count_han(text: &str) -> bool {
    text.chars().any(is_han)
}

/// Remove the contents of quoted literals, the way the English rule does.
fn strip_quoted(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, '\'' | '"' | '`') {
            if let Some(offset) = chars[i + 1..].iter().position(|&x| x == c) {
                out.push(' ');
                i += offset + 2;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Upstream ignores merge commits, reverts, fixups and version bumps.
fn is_ignored(message: &str) -> bool {
    let first_line = message.lines().next().unwrap_or("");
    if is_semver_bump(first_line) {
        return true;
    }
    message.lines().any(|line| {
        line.starts_with("Merge pull request")
            || line.starts_with("Merge branch")
            || line.starts_with("Merge remote-tracking branch")
            || line.starts_with("Merge tag")
            || line.starts_with("Merged ")
            || line.starts_with("Automatic merge")
            || line.starts_with("Auto-merged")
            || line.starts_with("Reapply ")
            || line.starts_with("Revert ")
            || line.starts_with("revert ")
            || line.starts_with("amend!")
            || line.starts_with("fixup!")
            || line.starts_with("squash!")
            || (line.starts_with("Merge ") && line.contains(" into "))
    })
}

/// Upstream ignores `chore: 1.2.3`-style version bumps.
///
/// Upstream strips a leading `chore:` or `chore(scope):`, then removes any
/// `[skip ci]` marker, and checks whether the remainder is a valid semantic
/// version. The marker is removed with a plain replace, so it may sit
/// anywhere in the line.
fn is_semver_bump(line: &str) -> bool {
    let mut rest = line.trim_start();
    if let Some(after) = rest.strip_prefix("chore") {
        rest = after;
        if rest.starts_with('(') {
            match rest.find(')') {
                Some(index) => rest = &rest[index + 1..],
                None => return false,
            }
        }
        rest = rest.strip_prefix(':').unwrap_or(rest);
    }
    let rest = remove_markers(rest);
    is_valid_semver(rest.trim())
}

/// Remove the `[skip ci]`-style markers upstream strips before its semver
/// check. Upstream replaces them wherever they appear, not just at the start.
fn remove_markers(text: &str) -> String {
    let mut out = text.to_string();
    for marker in ["[skip ci]", "[ci skip]", "(skip ci)", "(ci skip)"] {
        if let Some(index) = out.to_ascii_lowercase().find(marker) {
            out.replace_range(index..index + marker.len(), "");
        }
    }
    out
}

/// A minimal semantic-version check, matching `semver.valid`.
fn is_valid_semver(text: &str) -> bool {
    if text.is_empty() || text.starts_with('v') || text.starts_with('=') {
        return false;
    }
    let (rest, build) = match text.split_once('+') {
        Some((head, tail)) => (head, Some(tail)),
        None => (text, None),
    };
    let (core, prerelease) = match rest.split_once('-') {
        Some((head, tail)) => (head, Some(tail)),
        None => (rest, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    for part in &parts {
        if !is_numeric_identifier(part, false) {
            return false;
        }
    }
    if let Some(prerelease) = prerelease {
        if prerelease.is_empty() {
            return false;
        }
        for id in prerelease.split('.') {
            if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
            if id.chars().all(|c| c.is_ascii_digit()) && !is_numeric_identifier(id, false) {
                return false;
            }
        }
    }
    if let Some(build) = build {
        if build.is_empty() {
            return false;
        }
        for id in build.split('.') {
            if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
        }
    }
    true
}

fn is_numeric_identifier(text: &str, allow_leading_zero: bool) -> bool {
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if !allow_leading_zero && text.len() > 1 && text.starts_with('0') {
        return false;
    }
    true
}

/// The `type-case` rule: lower-case, ignoring quoted parts.
fn type_is_lower_case(kind: &str) -> bool {
    let stripped = strip_quoted(kind);
    let stripped = stripped.trim();
    if stripped.is_empty() || stripped.starts_with(|c: char| c.is_ascii_digit()) {
        return true;
    }
    stripped.chars().all(|c| !c.is_uppercase())
}

/// The `subject-full-stop` rule, checked against the raw header.
fn subject_has_full_stop(header: &str) -> bool {
    let units: Vec<u16> = header.encode_utf16().collect();
    if let Some(colon) = units.iter().position(|unit| *unit == b':' as u16) {
        if colon > 0 && colon == units.len() - 1 {
            return false;
        }
    }
    let dot = b'.' as u16;
    if units.len() >= 3 && units[units.len() - 3..] == [dot, dot, dot] {
        return false;
    }
    units.last() == Some(&dot)
}

/// A footer line is allowed to be long when it carries a URL.
fn footer_line_is_allowed(line: &str, max: usize) -> bool {
    if has_url(line) {
        return true;
    }
    utf16_len(line) <= max
}

fn has_url(line: &str) -> bool {
    match line.find("http://") {
        Some(index) => index + 7 < line.len(),
        None => match line.find("https://") {
            Some(index) => index + 8 < line.len(),
            None => false,
        },
    }
}

/// Returns the list of violations; empty means the message is acceptable.
pub fn validate(raw: &str, rules: &Rules) -> Vec<String> {
    let message = raw.trim_end_matches(['\r', '\n']);
    if message.trim().is_empty() {
        return vec!["the commit message is empty".to_string()];
    }
    if is_ignored(message) {
        return Vec::new();
    }

    let parsed = parse(message);
    let mut issues = Vec::new();

    // `header-trim`: the raw header may not start or end with whitespace.
    if parsed.header.starts_with(' ') || parsed.header.ends_with(' ') {
        issues.push("the header must not start or end with whitespace (header-trim)".to_string());
    }
    // `header-max-length`
    if utf16_len(&parsed.header) > rules.max_header {
        issues.push(format!(
            "the header must be at most {} characters, found {} (header-max-length)",
            rules.max_header,
            utf16_len(&parsed.header)
        ));
    }

    // A header that does not match the grammar leaves type and subject empty,
    // which upstream reports as two separate errors.
    let kind = parsed.kind.as_deref().unwrap_or("");
    if kind.is_empty() {
        issues.push("the type may not be empty (type-empty)".to_string());
    } else {
        if !type_is_lower_case(kind) {
            issues.push(format!(
                "the type must be lower-case, found `{kind}` (type-case)"
            ));
        }
        if !rules.types.iter().any(|item| item == kind) {
            issues.push(format!(
                "the type must be one of [{}], found `{kind}` (type-enum)",
                rules.types.join(", ")
            ));
        }
    }

    // `scope-enum`: an absent scope is always fine; a present one is judged
    // by the configured policy (none / any / whitelist).
    if let Some(scope) = parsed.scope.as_deref() {
        if !rules.scopes.allows(scope) {
            issues.push(format!(
                "{}, found `{scope}` (scope-enum)",
                rules.scopes.rejection_hint()
            ));
        }
    }

    let subject = parsed.subject.as_deref().unwrap_or("");
    if subject.is_empty() {
        issues.push("the subject may not be empty (subject-empty)".to_string());
    } else {
        if subject_has_full_stop(&parsed.header) {
            issues.push("the subject may not end with a full stop (subject-full-stop)".to_string());
        }
        issues.extend(check_subject_language(message, &parsed, rules));
    }

    // `footer-max-line-length`
    for line in &parsed.footer {
        if !footer_line_is_allowed(line, rules.max_footer_line) {
            issues.push(format!(
                "the footer's lines must not be longer than {} characters (footer-max-line-length)",
                rules.max_footer_line
            ));
            break;
        }
    }

    issues
}

fn check_subject_language(message: &str, parsed: &Parsed, rules: &Rules) -> Vec<String> {
    let subject = parsed.subject.as_deref().unwrap_or("");
    let exempt = message.starts_with("revert:")
        || message.starts_with("Revert \"")
        || parsed.scope.as_deref() == Some("main");
    if exempt {
        return Vec::new();
    }

    match rules.subject_lang {
        SubjectLang::Zh => {
            let found = subject.chars().filter(|c| is_han(*c)).count();
            if found >= rules.min_han {
                return Vec::new();
            }
            vec![format!(
                "the subject must be written in Chinese (found {found} Han characters, at least {} required) (subject-language).\n    wrong: feat(cli): add new flags\n    right: feat(cli): 新增参数解析支持",
                rules.min_han
            )]
        }
        SubjectLang::En => {
            let stripped = strip_quoted(subject);
            let stripped = stripped.trim();
            if !count_han(stripped) {
                return Vec::new();
            }
            let found = stripped.chars().filter(|c| is_han(*c)).count();
            let starts_with_han = stripped.starts_with(|c: char| is_han(c));
            if !starts_with_han && found <= MAX_UNQUOTED_HAN_IN_ENGLISH {
                return Vec::new();
            }
            vec![format!(
                "the subject must be written in English (found {found} unquoted Han characters) (subject-language).\n    Quote a literal term if you must reference it, e.g. 'some-term'.\n    header: {}",
                parsed.header
            )]
        }
    }
}

/// Read a message file, validate it and report. Returns the exit code.
pub fn validate_file(path: &str) -> Result<u8, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read the commit message at {path}: {error}"))?;
    let (config, _) = crate::config::load(&std::env::current_dir().unwrap_or_default())?;
    if config.is_disabled("commit") {
        return Ok(0);
    }
    let rules = Rules::from_config(&config);
    let issues = validate(&raw, &rules);
    if issues.is_empty() {
        return Ok(0);
    }
    eprintln!("[gate] the commit message was rejected:");
    for issue in &issues {
        eprintln!("  - {issue}");
    }
    let header = raw
        .trim_end_matches(['\r', '\n'])
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    eprintln!("  header: {header}");
    eprintln!(
        "\n[gate] the rule set lives in .pengj-templates/gate.toml under [commit]; the convention is documented in the git-workflow skill."
    );
    Ok(1)
}

/// The outcome of checking a range of commits.
#[derive(Debug, Clone)]
pub struct CommitCheckReport {
    /// The range that was walked, for the message.
    pub range: String,
    /// How many commits were checked.
    pub checked: usize,
    /// One entry per rejected commit: the sha, its subject and the issues.
    pub rejected: Vec<RejectedCommit>,
}

/// A commit whose message did not pass the rules.
#[derive(Debug, Clone)]
pub struct RejectedCommit {
    pub sha: String,
    pub subject: String,
    pub issues: Vec<String>,
}

/// The refs a CI run usually branches from, in the order they are tried.
const BASE_CANDIDATES: [&str; 4] = ["origin/main", "origin/master", "main", "master"];

/// Pick the base ref: the explicit one, else the first branch that exists.
fn resolve_base(dir: &std::path::Path, explicit: Option<&str>) -> String {
    if let Some(base) = explicit {
        return base.to_string();
    }
    for candidate in BASE_CANDIDATES {
        if crate::git::probe_in(dir, &["rev-parse", "--verify", "--quiet", candidate]).is_some() {
            return candidate.to_string();
        }
    }
    "HEAD~1".to_string()
}

/// Validate every commit message in a range.
///
/// This is the server-side counterpart of the `commit-msg` hook: it needs no
/// local setup at all, so a pull request is guarded even when the author
/// never installed the CLI.
pub fn check_commits(base: Option<&str>, head: Option<&str>) -> Result<CommitCheckReport, String> {
    let cwd = std::env::current_dir().unwrap_or_default();
    check_commits_in(&cwd, base, head)
}

/// Validate every commit message in a range, working inside `dir`.
///
/// The repository root is resolved once and every git call runs there, so the
/// result never depends on where the command was invoked from.
pub fn check_commits_in(
    dir: &std::path::Path,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<CommitCheckReport, String> {
    let root = crate::git::probe_in(dir, &["rev-parse", "--show-toplevel"])
        .filter(|root| !root.is_empty())
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "not a git work tree; check-commits needs a repository".to_string())?;

    let (config, _) = crate::config::load(&root)?;
    if config.is_disabled("commit") {
        return Ok(CommitCheckReport {
            range: String::new(),
            checked: 0,
            rejected: Vec::new(),
        });
    }
    let rules = Rules::from_config(&config);

    let base_ref = resolve_base(&root, base);
    if crate::git::probe_in(&root, &["rev-parse", "--verify", "--quiet", &base_ref]).is_none() {
        return Err(format!(
            "the base ref `{base_ref}` does not exist in this repository.\n  \
             the commit range cannot be walked — fetch the full history first \
             (`git fetch --unshallow`) or pass a reachable --base"
        ));
    }
    let head_ref = head.unwrap_or("HEAD").to_string();
    let range = format!("{base_ref}..{head_ref}");
    let listing = crate::git::run_in(&root, &["rev-list", "--reverse", &range])?;
    let shas: Vec<String> = listing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    let mut rejected = Vec::new();
    for sha in &shas {
        let message = crate::git::run_in(&root, &["show", "-s", "--format=%B", sha])?;
        let issues = validate(&message, &rules);
        if issues.is_empty() {
            continue;
        }
        let subject = message.lines().next().unwrap_or("").trim().to_string();
        rejected.push(RejectedCommit {
            sha: sha.clone(),
            subject,
            issues,
        });
    }

    Ok(CommitCheckReport {
        range,
        checked: shas.len(),
        rejected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules_zh() -> Rules {
        Rules {
            types: CONVENTIONAL_TYPES
                .iter()
                .map(|item| item.to_string())
                .collect(),
            scopes: ScopePolicy::Whitelist(vec!["core".into(), "cli".into()]),
            subject_lang: SubjectLang::Zh,
            min_han: 2,
            max_header: 100,
            max_footer_line: 100,
        }
    }

    #[test]
    fn accepts_a_well_formed_chinese_header() {
        assert!(validate("feat(cli): 新增参数解析支持", &rules_zh()).is_empty());
    }

    #[test]
    fn rejects_an_english_subject_for_a_chinese_project() {
        let issues = validate("feat(cli): add new flags", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("Chinese")));
    }

    #[test]
    fn rejects_an_unknown_type() {
        let issues = validate("feature(cli): 新增参数解析", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("type-enum")));
    }

    #[test]
    fn rejects_an_unknown_scope() {
        let issues = validate("feat(nope): 新增参数解析", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("scope-enum")));
    }

    #[test]
    fn scope_is_optional() {
        assert!(validate("docs: 补充文档说明", &rules_zh()).is_empty());
    }

    #[test]
    fn an_empty_scope_counts_as_no_scope() {
        assert!(validate("feat(): 新增参数解析支持", &rules_zh()).is_empty());
    }

    #[test]
    fn a_scope_may_list_several_whitelisted_parts() {
        assert!(validate("feat(core/cli): 新增参数解析支持", &rules_zh()).is_empty());
    }

    #[test]
    fn rejects_a_trailing_full_stop() {
        let issues = validate("docs: 补充文档说明.", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("full stop")));
    }

    #[test]
    fn merge_and_fixup_commits_are_ignored() {
        assert!(validate("Merge branch 'main' into feat", &rules_zh()).is_empty());
        assert!(validate("fixup! docs: 补充文档说明", &rules_zh()).is_empty());
    }

    #[test]
    fn version_bumps_are_ignored() {
        assert!(validate("chore: 1.2.3", &rules_zh()).is_empty());
    }

    #[test]
    fn english_projects_reject_chinese_subjects() {
        let rules = Rules {
            subject_lang: SubjectLang::En,
            ..rules_zh()
        };
        assert!(validate("feat(cli): add new flags", &rules).is_empty());
        assert!(!validate("feat(cli): 新增参数解析支持", &rules).is_empty());
    }

    #[test]
    fn english_projects_tolerate_a_quoted_term() {
        let rules = Rules {
            subject_lang: SubjectLang::En,
            ..rules_zh()
        };
        assert!(validate("fix(core): support '中文别名' alias", &rules).is_empty());
        // A subject that starts with Han is rejected even when the count is
        // within the tolerance.
        assert!(!validate("fix(core): 支持 legacy 别名", &rules).is_empty());
    }

    #[test]
    fn comment_lines_are_dropped() {
        let raw =
            "feat(cli): 新增参数解析支持\n\n# Please enter the commit message\n# On branch main\n";
        assert!(validate(raw, &rules_zh()).is_empty());
    }

    #[test]
    fn empty_message_is_rejected() {
        assert!(!validate("", &rules_zh()).is_empty());
        assert!(!validate("   \n", &rules_zh()).is_empty());
    }

    #[test]
    fn a_header_without_a_space_after_the_colon_is_rejected() {
        let issues = validate("feat(cli):no space here", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("type-empty")));
        assert!(issues.iter().any(|issue| issue.contains("subject-empty")));
    }

    #[test]
    fn a_type_with_a_dash_is_rejected() {
        let issues = validate("feat-x(cli): 新增参数解析支持", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("type-empty")));
    }

    #[test]
    fn a_long_footer_is_rejected() {
        let long = "x".repeat(120);
        let raw = format!("feat(cli): 新增参数解析支持\n\nBREAKING CHANGE: {long}");
        let issues = validate(&raw, &rules_zh());
        assert!(issues
            .iter()
            .any(|issue| issue.contains("footer-max-line-length")));
    }

    #[test]
    fn a_footer_line_with_a_url_is_allowed_to_be_long() {
        let long = "x".repeat(120);
        let raw = format!("feat(cli): 新增参数解析支持\n\nCloses: https://example.com/{long}");
        assert!(validate(&raw, &rules_zh()).is_empty());
    }

    #[test]
    fn a_trailing_space_in_the_header_is_rejected() {
        let issues = validate("feat(cli): 新增参数解析支持 ", &rules_zh());
        assert!(issues.iter().any(|issue| issue.contains("header-trim")));
    }

    #[test]
    fn a_breaking_change_marker_is_accepted() {
        assert!(validate("feat(cli)!: 新增参数解析支持", &rules_zh()).is_empty());
    }

    #[test]
    fn scope_policy_from_config_has_three_states() {
        let absent = Rules::from_config(&crate::config::parse("").unwrap());
        assert_eq!(absent.scopes, ScopePolicy::None);

        let empty = Rules::from_config(&crate::config::parse("[commit]\nscopes = []").unwrap());
        assert_eq!(empty.scopes, ScopePolicy::None);

        let any = Rules::from_config(&crate::config::parse("[commit]\nscopes = [\"*\"]").unwrap());
        assert_eq!(any.scopes, ScopePolicy::Any);

        let list = Rules::from_config(
            &crate::config::parse("[commit]\nscopes = [\"core\", \"cli\"]").unwrap(),
        );
        assert_eq!(
            list.scopes,
            ScopePolicy::Whitelist(vec!["core".into(), "cli".into()])
        );
    }

    #[test]
    fn no_scope_policy_rejects_every_scope_but_allows_omitting_it() {
        let rules = Rules {
            scopes: ScopePolicy::None,
            ..rules_zh()
        };
        assert!(validate("feat(core): 新增参数解析支持", &rules)
            .iter()
            .any(|issue| issue.contains("scope-enum")));
        // 不带 scope 的 header 始终合法。
        assert!(validate("feat: 新增参数解析支持", &rules).is_empty());
    }

    #[test]
    fn wildcard_policy_accepts_any_scope() {
        let rules = Rules {
            scopes: ScopePolicy::Any,
            ..rules_zh()
        };
        assert!(validate("feat(whatever): 新增参数解析支持", &rules).is_empty());
        assert!(validate("feat(core/nope): 新增参数解析支持", &rules).is_empty());
    }
}
