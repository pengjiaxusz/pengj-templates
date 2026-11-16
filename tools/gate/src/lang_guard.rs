//! Code-language guard.
//!
//! Ported rule-for-rule from the former `check-lang.mjs`, including its
//! per-file ratio tolerance, its `lang-ok:` / `lang-allow:` certification
//! markers and its distinction between prose and machine vocabulary.
//!
//! Two modes:
//! - `english`: identifiers must be Latin, copy must not contain a foreign
//!   script (CJK, kana, hangul, Cyrillic, Hebrew, Arabic, Devanagari).
//! - `chinese`: copy must contain Han characters, identifiers are free (the
//!   Chinese-programming convention keeps English anchors on purpose), and
//!   only human prose is judged so machine tokens stay English.

use crate::config::Mode;
use crate::scanner::{self, Token};
use std::collections::BTreeMap;
use std::path::Path;

const SKIP_DIRS: [&str; 6] = [
    "target",
    ".git",
    "node_modules",
    "third-party",
    "generated",
    "submodules",
];
const MAX_PRINTED: usize = 80;

const RUST_KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "macro_rules",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
    "while",
    "yield",
];

const PRIMITIVES: &[&str] = &[
    "bool", "char", "str", "usize", "isize", "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32",
    "u64", "u128", "f32", "f64",
];

pub(crate) fn is_han(c: char) -> bool {
    matches!(c as u32, 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff)
}

fn is_cjk(c: char) -> bool {
    matches!(
        c as u32,
        0x3040..=0x30ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff
    )
}

fn is_foreign_script(c: char) -> bool {
    matches!(
        c as u32,
        0x0400..=0x04ff
            | 0x0590..=0x05ff
            | 0x0600..=0x06ff
            | 0x0900..=0x097f
            | 0x2e80..=0x2fff
            | 0x3000..=0x303f
            | 0x3040..=0x30ff
            | 0x3400..=0x4dbf
            | 0x4e00..=0x9fff
            | 0xac00..=0xd7af
            | 0xf900..=0xfaff
            | 0xfe30..=0xfe4f
            | 0xff00..=0xffef
    )
}

fn is_non_han_script(c: char) -> bool {
    matches!(
        c as u32,
        0x0400..=0x04ff
            | 0x0590..=0x05ff
            | 0x0600..=0x06ff
            | 0x0900..=0x097f
            | 0x3040..=0x30ff
            | 0xac00..=0xd7af
    )
}

fn has_language_letters(text: &str) -> bool {
    text.chars()
        .any(|c| c.is_ascii_alphabetic() || is_cjk(c) || is_foreign_script(c))
}

fn has_foreign(text: &str) -> bool {
    text.chars().any(is_foreign_script)
}

fn has_han(text: &str) -> bool {
    text.chars().any(is_han)
}

/// Drop `\n`, `\t`, `\x41` and friends: an escape sequence is punctuation, not
/// vocabulary. Mirrors the original single-character escape class, where the
/// longer `\xNN` / `\u{...}` alternatives were unreachable.
fn strip_escapes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\'
            && i + 1 < chars.len()
            && matches!(
                chars[i + 1],
                'n' | 'r' | 't' | '0' | '\\' | '\'' | '"' | 'x' | 'u' | 'U'
            )
        {
            i += 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Drop `{...}` format placeholders: what matters is the prose around them.
fn strip_placeholders(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            if let Some(offset) = chars[i + 1..].iter().position(|&c| c == '}' || c == '{') {
                if chars[i + 1 + offset] == '}' {
                    i += offset + 2;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn has_code_syntax(c: char) -> bool {
    matches!(
        c,
        ';' | '{' | '}' | '[' | ']' | '(' | ')' | '=' | '<' | '>' | '|' | '\\' | '/' | '\'' | '"'
    )
}

fn is_hex_blob(text: &str) -> bool {
    let body = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    body.len() >= 6 && body.chars().all(|c| c.is_ascii_hexdigit())
}

fn ascii_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            current.push(c);
        } else if !current.is_empty() {
            if current.len() >= 2 {
                words.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    if current.len() >= 2 {
        words.push(current);
    }
    words
}

/// True when an English literal reads like a sentence a human wrote.
///
/// Chinese mode needs this because "does this ASCII string need translating?"
/// is genuinely ambiguous: `"total_cases"`, `"-spirv"` and `"{unique}.hlsl"`
/// are not copy, while "Detour already initialized" is.
fn looks_like_prose(text: &str) -> bool {
    let stripped = strip_placeholders(&strip_escapes(text));
    if !stripped.chars().any(char::is_whitespace) {
        return false;
    }
    if stripped.chars().any(has_code_syntax) {
        return false;
    }
    if stripped.contains("::") || stripped.contains("->") {
        return false;
    }
    if is_hex_blob(stripped.trim()) {
        return false;
    }
    let words = ascii_words(&stripped);
    if words.len() < 2 {
        return false;
    }
    let tokens: Vec<&str> = stripped.split_whitespace().collect();
    if !tokens.is_empty() && tokens.iter().all(|token| token.starts_with('-')) {
        return false;
    }
    words.iter().any(|word| {
        word.chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
    })
}

/// True when the literal is human-readable copy that must follow the project
/// language. Machine vocabulary stays in the denominator either way: it is
/// still a string literal, and shrinking the denominator would silently tighten
/// every file's budget.
fn is_copy(text: &str, mode: Mode) -> bool {
    let without_escapes = strip_escapes(text);
    if !has_language_letters(&strip_placeholders(&without_escapes)) {
        return false;
    }
    match mode {
        Mode::English => true,
        Mode::Chinese => {
            if without_escapes.chars().any(is_non_han_script) {
                return true;
            }
            looks_like_prose(&without_escapes)
        }
    }
}

#[derive(Debug)]
pub struct Report {
    pub file: String,
    pub file_allow: Option<String>,
    pub line_ok: BTreeMap<usize, String>,
    pub identifiers: usize,
    pub string_items: usize,
    pub ident_violations: Vec<Token>,
    pub string_violations: Vec<Token>,
}

/// Classify one file. Exempt lines are dropped from both numerator and
/// denominator: a certified fixture should not make the rest of the file look
/// cleaner than it is.
pub fn classify(file: &str, source: &str, mode: Mode) -> Report {
    let scan = scanner::scan(source);
    let mut report = Report {
        file: file.to_string(),
        file_allow: scan.file_allow,
        line_ok: scan.line_ok,
        identifiers: 0,
        string_items: 0,
        ident_violations: Vec::new(),
        string_violations: Vec::new(),
    };
    if report.file_allow.is_some() {
        return report;
    }

    for token in &scan.identifiers {
        if report.line_ok.contains_key(&token.line) {
            continue;
        }
        if RUST_KEYWORDS.contains(&token.text.as_str()) || PRIMITIVES.contains(&token.text.as_str())
        {
            continue;
        }
        report.identifiers += 1;
        if mode == Mode::English && has_foreign(&token.text) {
            report.ident_violations.push(token.clone());
        }
    }

    for token in &scan.strings {
        if report.line_ok.contains_key(&token.line) {
            continue;
        }
        if !has_language_letters(&token.text) {
            continue;
        }
        report.string_items += 1;
        if !is_copy(&token.text, mode) {
            continue;
        }
        match mode {
            Mode::English => {
                if has_foreign(&token.text) {
                    report.string_violations.push(token.clone());
                }
            }
            Mode::Chinese => {
                if !has_han(&token.text) && token.text.chars().any(char::is_alphabetic) {
                    report.string_violations.push(token.clone());
                }
            }
        }
    }

    report
}

fn allowance(items: usize, ratio: f64) -> usize {
    if items == 0 {
        return 0;
    }
    // Guard against binary float noise: 200 * 0.01 must be exactly 2.
    (items as f64 * ratio + 1e-9).floor() as usize
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn walk(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            if SKIP_DIRS.iter().any(|skip| *skip == name) {
                continue;
            }
            walk(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(normalize(&path));
        }
    }
}

fn collect_targets(files: &[String]) -> Vec<String> {
    if !files.is_empty() {
        return files
            .iter()
            .filter(|file| file.ends_with(".rs"))
            .map(|file| file.replace('\\', "/"))
            .collect();
    }
    // A full audit walks the repository root and lets SKIP_DIRS prune build
    // output, dependency copies, generated code and submodules. Repository
    // layouts vary, so hard-coded roots would audit nothing in a project that
    // does not match them.
    let mut out = Vec::new();
    walk(Path::new("."), &mut out);
    out
}

pub struct AuditOptions {
    pub files: Vec<String>,
    pub mode: Mode,
    pub mode_source: String,
    pub max_ratio: f64,
    pub max_ident_ratio: f64,
    pub json: bool,
    pub list: bool,
}

fn format_percent(value: f64) -> String {
    if value == 0.0 {
        "0%".to_string()
    } else {
        format!("{:.2}%", value * 100.0)
    }
}

fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

struct Failure {
    report: Report,
    ident_budget: usize,
    string_budget: usize,
}

/// Run the guard. Returns the process exit code (0 pass, 1 violations).
pub fn audit(options: &AuditOptions) -> i32 {
    let targets = collect_targets(&options.files);
    let mut failures = Vec::new();
    let mut marker_issues = Vec::new();

    for file in &targets {
        let Ok(source) = std::fs::read_to_string(file) else {
            // A hook can hand over files deleted in the same commit.
            continue;
        };
        let report = classify(file, &source, options.mode);
        if let Some(reason) = &report.file_allow {
            if reason.is_empty() {
                marker_issues.push(format!("{file}: {} has no reason", scanner::MARKER_FILE));
            }
        }
        for (line, reason) in &report.line_ok {
            if reason.is_empty() {
                marker_issues.push(format!(
                    "{file}:{line}: {} has no reason",
                    scanner::MARKER_LINE
                ));
            }
        }
        let ident_budget = allowance(report.identifiers, options.max_ident_ratio);
        let string_budget = allowance(report.string_items, options.max_ratio);
        if report.ident_violations.len() > ident_budget
            || report.string_violations.len() > string_budget
        {
            failures.push(Failure {
                report,
                ident_budget,
                string_budget,
            });
        }
    }

    if options.json {
        let files: Vec<String> = failures
            .iter()
            .map(|failure| {
                format!(
                    "    {{\"file\": \"{}\", \"identifiers\": {}, \"identViolations\": {}, \"strings\": {}, \"stringViolations\": {}, \"certified\": {}}}",
                    json_escape(&failure.report.file),
                    failure.report.identifiers,
                    failure.report.ident_violations.len(),
                    failure.report.string_items,
                    failure.report.string_violations.len(),
                    failure.report.file_allow.is_some()
                )
            })
            .collect();
        let markers: Vec<String> = marker_issues
            .iter()
            .map(|issue| format!("    \"{}\"", json_escape(issue)))
            .collect();
        println!(
            "{{\n  \"mode\": \"{}\",\n  \"modeSource\": \"{}\",\n  \"maxRatio\": {},\n  \"maxIdentRatio\": {},\n  \"scanned\": {},\n  \"failing\": {},\n  \"files\": [\n{}\n  ],\n  \"markers\": [\n{}\n  ]\n}}",
            options.mode.as_str(),
            json_escape(&options.mode_source),
            options.max_ratio,
            options.max_ident_ratio,
            targets.len(),
            failures.len(),
            files.join(",\n"),
            markers.join(",\n")
        );
        return if failures.is_empty() && marker_issues.is_empty() {
            0
        } else {
            1
        };
    }

    if failures.is_empty() && marker_issues.is_empty() {
        println!(
            "[lang-guard] passed: {} Rust file(s), mode={}.",
            targets.len(),
            options.mode.as_str()
        );
        return 0;
    }

    if !failures.is_empty() {
        eprintln!(
            "[lang-guard] {} file(s) exceed the tolerance (mode={}, strings<={}, identifiers<={}):",
            failures.len(),
            options.mode.as_str(),
            format_percent(options.max_ratio),
            format_percent(options.max_ident_ratio)
        );
        for failure in &failures {
            let report = &failure.report;
            eprintln!(
                "  {}: identifiers {}/{} (budget {}), strings {}/{} (budget {})",
                report.file,
                report.ident_violations.len(),
                report.identifiers,
                failure.ident_budget,
                report.string_violations.len(),
                report.string_items,
                failure.string_budget
            );
            let shown: Vec<&Token> = if options.list {
                report
                    .ident_violations
                    .iter()
                    .chain(report.string_violations.iter())
                    .collect()
            } else {
                report
                    .ident_violations
                    .iter()
                    .take(3)
                    .chain(report.string_violations.iter().take(3))
                    .collect()
            };
            let total = report.ident_violations.len() + report.string_violations.len();
            let printed = shown.len().min(MAX_PRINTED);
            for hit in shown.iter().take(printed) {
                let text: String = hit.text.chars().take(80).collect();
                eprintln!("    {}:{}: \"{}\"", report.file, hit.line, text);
            }
            if total > printed {
                eprintln!("    ... {} more (re-run with --list)", total - printed);
            }
        }
    }

    if !marker_issues.is_empty() {
        eprintln!("[lang-guard] invalid certification markers:");
        for issue in &marker_issues {
            eprintln!("  {issue}");
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "\n[lang-guard] Rules: identifiers and copy must use the project language; keep genuine exceptions to <={} per file, or certify them with `{} <reason>` / `{} <reason>`.",
            format_percent(options.max_ratio),
            scanner::MARKER_LINE,
            scanner::MARKER_FILE
        );
    } else {
        eprintln!(
            "[lang-guard] `{}` and `{}` require a reason after the colon.",
            scanner::MARKER_FILE,
            scanner::MARKER_LINE
        );
    }
    1
}

/// The self-test table is shared by `cargo test` and `gate selftest` so the two
/// can never drift apart.
pub fn self_test_cases() -> Vec<(&'static str, bool)> {
    let mut cases: Vec<(&'static str, bool)> = Vec::new();

    let english_sample = [
        "fn title() -> &'static str { \"Hello world\" }",
        "fn \u{6807}\u{9898}() -> &'static str { \"\u{4f60}\u{597d}\" }",
        "let x = \"\u{4f60}\u{597d}\"; // lang-ok: CJK input sample",
        "// \u{4e2d}\u{6587}\u{6ce8}\u{91ca}\u{4e0d}\u{7b97}\u{8fdd}\u{89c4}",
    ]
    .join("\n");
    let english = classify("sample.rs", &english_sample, Mode::English);
    cases.push((
        "english: non-ASCII identifier is caught",
        english.ident_violations.len() == 1
            && english.ident_violations[0].text == "\u{6807}\u{9898}",
    ));
    cases.push((
        "english: certified line is excluded",
        english.string_violations.len() == 1,
    ));
    cases.push((
        "english: certified string leaves the denominator",
        english.string_items == 2,
    ));

    let chinese_sample = [
        "fn title() { let label = \"\u{4fdd}\u{5b58}\"; }",
        "fn other() { let label = \"Save changes\"; }",
        "fn tag() { let label = \"UTF-8\"; } // lang-ok: technical token",
    ]
    .join("\n");
    let chinese = classify("sample.rs", &chinese_sample, Mode::Chinese);
    cases.push((
        "chinese: English copy is a violation",
        chinese.string_violations.len() == 1,
    ));
    cases.push((
        "chinese: English identifiers are allowed",
        chinese.ident_violations.is_empty(),
    ));
    cases.push((
        "chinese: certified technical token is excluded",
        chinese.string_items == 2,
    ));

    let kana = classify("s.rs", "let s = \"\u{3059}\u{308b}\";", Mode::Chinese);
    cases.push((
        "chinese: kana-only copy still needs Han characters",
        kana.string_violations.len() == 1,
    ));

    let placeholders = classify(
        "s.rs",
        "let s = \"{:#x}\"; let t = \"{:08x}\";",
        Mode::Chinese,
    );
    cases.push((
        "format placeholders are never copy",
        placeholders.string_violations.is_empty(),
    ));
    cases.push((
        "machine strings still count toward the denominator",
        placeholders.string_items == 2,
    ));

    let tokens = classify(
        "s.rs",
        "let a = \"total_cases\"; let b = \"D3D11CreateDevice\"; let c = \"chahu.dll\";",
        Mode::Chinese,
    );
    cases.push((
        "machine vocabulary is not copy",
        tokens.string_violations.is_empty(),
    ));

    let embedded = classify(
        "s.rs",
        "let hlsl = \"float4 main() : SV_Target {\n    return 0;\n}\";",
        Mode::Chinese,
    );
    cases.push((
        "embedded source code is not copy",
        embedded.string_violations.is_empty(),
    ));

    let prose = classify(
        "s.rs",
        "let s = \"Failed to open file: {err}\";",
        Mode::Chinese,
    );
    cases.push((
        "prose around a placeholder is still copy",
        prose.string_violations.len() == 1,
    ));

    let allowed = classify(
        "s.rs",
        "// lang-allow: i18n table\nfn f() { let _ = \"\u{4f60}\u{597d}\"; }",
        Mode::English,
    );
    cases.push((
        "file certification bypasses the scan",
        allowed.file_allow.is_some() && allowed.string_violations.is_empty(),
    ));

    let raw = classify(
        "s.rs",
        "let s = r#\"\u{4f60}\u{597d}\"#; let b = br#\"ok\"#; let c = \"a\";",
        Mode::English,
    );
    cases.push(("raw strings are scanned", raw.string_violations.len() == 1));

    let noise = classify(
        "s.rs",
        "let a = \"{}\"; let b = \"->\"; let c = \"123\";",
        Mode::English,
    );
    cases.push(("non-language strings are ignored", noise.string_items == 0));

    let typography = classify(
        "s.rs",
        "let s = \"{} \u{2014} {} items\"; let q = \"caf\u{e9}\";",
        Mode::English,
    );
    cases.push((
        "typographic punctuation and Latin accents are legal English",
        typography.string_violations.is_empty(),
    ));

    let cyrillic = classify(
        "s.rs",
        "let s = \"\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}\";",
        Mode::English,
    );
    cases.push((
        "non-Latin scripts are caught",
        cyrillic.string_violations.len() == 1,
    ));

    cases.push((
        "allowance floors partial items",
        allowance(120, 0.01) == 1 && allowance(99, 0.01) == 0,
    ));

    cases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_table_passes() {
        let failed: Vec<_> = self_test_cases()
            .into_iter()
            .filter(|(_, ok)| !ok)
            .map(|(name, _)| name)
            .collect();
        assert!(failed.is_empty(), "failed cases: {failed:?}");
    }

    #[test]
    fn doc_comments_are_not_copy() {
        let report = classify("s.rs", "// 中文注释\nlet x = 1;", Mode::English);
        assert_eq!(report.string_items, 0);
        assert_eq!(report.identifiers, 1);
    }

    #[test]
    fn full_audit_skips_build_directories() {
        let root = std::env::temp_dir().join(format!("gate-walk-{}", std::process::id()));
        let target = root.join("target");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(root.join("a.rs"), "fn a() {}").unwrap();
        std::fs::write(target.join("b.rs"), "fn b() {}").unwrap();
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(&root).unwrap();
        let files = collect_targets(&[]);
        std::env::set_current_dir(previous).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.rs"));
    }
}
