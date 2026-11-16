//! The commit corpus: a frozen set of messages and the verdict each one must
//! get.
//!
//! The verdicts were produced by `commitlint` itself (the rule set came from
//! `commitlint --print-config`), so this file pins the gate to the tool it
//! replaces. Keeping the corpus here means the behaviour stays locked even
//! after the node toolchain is gone.

use pengj_gate::{commit, config};
use std::path::Path;

const CORPUS: &str = include_str!("commit_corpus.jsonl");

/// One corpus entry: the message and whether it must be accepted.
struct Case {
    name: String,
    message: String,
    accepted: bool,
}

/// Parse the JSONL corpus without pulling in a JSON dependency.
///
/// The corpus is generated, so it only needs the three shapes it actually
/// uses: a string `name`, a string `msg` and a boolean `accept`.
fn parse_corpus(text: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let name = read_string_field(line, "name").expect("every case needs a name");
        let message = read_string_field(line, "msg").expect("every case needs a message");
        let accepted = read_bool_field(line, "accept").unwrap_or(false);
        cases.push(Case {
            name,
            message,
            accepted,
        });
    }
    cases
}

/// Read a JSON string field, resolving the escapes the corpus uses.
fn read_string_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => break,
            },
            other => out.push(other),
        }
    }
    None
}

fn read_bool_field(line: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\":");
    let start = line.find(&needle)? + needle.len();
    let rest = line[start..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn rules() -> commit::Rules {
    commit::Rules {
        types: [
            "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore",
            "revert",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect(),
        scopes: [
            "core",
            "cli",
            "agent",
            "app",
            "mcp",
            "ci",
            "templates",
            "main",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect(),
        subject_lang: config::SubjectLang::Zh,
        min_han: 2,
        max_header: 100,
        max_footer_line: 100,
    }
}

/// The gate must accept exactly the messages the corpus marks as accepted.
#[test]
fn the_gate_matches_the_commitlint_verdicts() {
    let cases = parse_corpus(CORPUS);
    assert!(!cases.is_empty(), "the corpus must not be empty");
    let mut failures = Vec::new();
    for case in &cases {
        let issues = commit::validate(&case.message, &rules());
        let accepted = issues.is_empty();
        if accepted != case.accepted {
            failures.push(format!(
                "{}: expected {}, got {} ({})",
                case.name,
                if case.accepted { "accept" } else { "reject" },
                if accepted { "accept" } else { "reject" },
                issues.join("; ")
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "the gate disagrees with the commitlint verdicts:\n{}",
        failures.join("\n")
    );
}

/// The corpus must keep covering the rules that can reject a message.
#[test]
fn the_corpus_covers_every_rule() {
    let cases = parse_corpus(CORPUS);
    let mut seen: Vec<&str> = Vec::new();
    for case in &cases {
        if case.accepted {
            continue;
        }
        for issue in commit::validate(&case.message, &rules()) {
            for rule in [
                "header-trim",
                "header-max-length",
                "type-empty",
                "type-case",
                "type-enum",
                "scope-enum",
                "subject-empty",
                "subject-full-stop",
                "subject-language",
                "footer-max-line-length",
            ] {
                if issue.contains(rule) && !seen.contains(&rule) {
                    seen.push(rule);
                }
            }
        }
    }
    for rule in [
        "header-trim",
        "header-max-length",
        "type-empty",
        "type-case",
        "type-enum",
        "scope-enum",
        "subject-empty",
        "subject-full-stop",
        "subject-language",
        "footer-max-line-length",
    ] {
        assert!(
            seen.contains(&rule),
            "the corpus does not cover the {rule} rule"
        );
    }
}

/// The corpus file must stay where the test expects it.
#[test]
fn the_corpus_file_is_readable() {
    assert!(Path::new("tests/commit_corpus.jsonl").is_file());
}
