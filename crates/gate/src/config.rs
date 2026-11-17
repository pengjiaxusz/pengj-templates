//! `.pengj-templates/gate.toml` reader.
//!
//! The gate is copied verbatim into every generated project, so it must not
//! depend on the manifest format (`options.chinese_programming` and friends).
//! Instead the template renders the switches into a dedicated config file at
//! generation time; the gate only ever reads that file. The manifest can then
//! evolve without breaking old gates.
//!
//! The accepted syntax is a flat TOML subset: `[section]` headers, `key = value`
//! with a string, number, boolean or array-of-strings value, and `#` comments.
//! Unknown keys are ignored so an older binary keeps working against a newer
//! config, but malformed values are hard errors: a typo must never silently
//! disable a check.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = ".pengj-templates/gate.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    English,
    Chinese,
}

impl Mode {
    pub fn parse(raw: &str) -> Option<Mode> {
        match raw {
            "english" => Some(Mode::English),
            "chinese" => Some(Mode::Chinese),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::English => "english",
            Mode::Chinese => "chinese",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectLang {
    Zh,
    En,
}

impl SubjectLang {
    pub fn parse(raw: &str) -> Option<SubjectLang> {
        match raw {
            "zh" => Some(SubjectLang::Zh),
            "en" => Some(SubjectLang::En),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    List(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Str,
    Num,
    List,
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ValueKind::Str => "a string",
            ValueKind::Num => "a number",
            ValueKind::List => "an array of strings",
        };
        formatter.write_str(text)
    }
}

impl Value {
    fn has_kind(&self, kind: ValueKind) -> bool {
        matches!(
            (self, kind),
            (Value::Str(_), ValueKind::Str)
                | (Value::Num(_), ValueKind::Num)
                | (Value::List(_), ValueKind::List)
        )
    }

    fn kind_name(&self) -> &'static str {
        match self {
            Value::Str(_) => "a string",
            Value::Num(_) => "a number",
            Value::Bool(_) => "a boolean",
            Value::List(_) => "an array of strings",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    values: BTreeMap<String, Value>,
}

impl Config {
    pub fn mode(&self) -> Option<Mode> {
        self.values.get("mode").and_then(|value| match value {
            Value::Str(raw) => Mode::parse(raw),
            _ => None,
        })
    }

    pub fn number(&self, key: &str) -> Option<f64> {
        match self.values.get(key) {
            Some(Value::Num(raw)) => Some(*raw),
            _ => None,
        }
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(Value::Str(raw)) => Some(raw),
            _ => None,
        }
    }

    pub fn list(&self, key: &str) -> Option<&[String]> {
        match self.values.get(key) {
            Some(Value::List(items)) => Some(items),
            _ => None,
        }
    }

    pub fn is_disabled(&self, check: &str) -> bool {
        ["disabled", "project.disabled"].iter().any(|key| {
            self.list(key)
                .map(|items| items.iter().any(|item| item == check))
                .unwrap_or(false)
        })
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, c) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_string(raw: &str) -> Option<String> {
    let body = raw.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            other => out.push(other),
        }
    }
    Some(out)
}

fn parse_list(raw: &str, line: usize) -> Result<Vec<String>, String> {
    let body = raw
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(|| format!("line {line}: array value must be closed with `]`"))?;
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth_quote = false;
    for c in body.chars() {
        match c {
            '"' => {
                depth_quote = !depth_quote;
                current.push(c);
            }
            ',' if !depth_quote => {
                let item = current.trim();
                if !item.is_empty() {
                    items.push(
                        parse_string(item)
                            .ok_or_else(|| format!("line {line}: array items must be quoted"))?,
                    );
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let tail = current.trim();
    if !tail.is_empty() {
        items.push(
            parse_string(tail).ok_or_else(|| format!("line {line}: array items must be quoted"))?,
        );
    }
    Ok(items)
}

pub fn parse(text: &str) -> Result<Config, String> {
    let mut config = Config::default();
    let mut section = String::new();

    for (offset, raw_line) in text.lines().enumerate() {
        let line_no = offset + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            let name = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
                .ok_or_else(|| format!("line {line_no}: malformed section header"))?;
            section = name.trim().to_string();
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_no}: expected `key = value`"))?;
        let key = key.trim();
        let raw_value = raw_value.trim();
        let full_key = if section.is_empty() {
            key.to_string()
        } else {
            format!("{section}.{key}")
        };

        let value =
            if raw_value.starts_with('"') {
                Value::Str(parse_string(raw_value).ok_or_else(|| {
                    format!("line {line_no}: string value must be closed with `\"`")
                })?)
            } else if raw_value.starts_with('[') {
                Value::List(parse_list(raw_value, line_no)?)
            } else if raw_value == "true" || raw_value == "false" {
                Value::Bool(raw_value == "true")
            } else {
                Value::Num(
                    raw_value
                        .parse::<f64>()
                        .map_err(|_| format!("line {line_no}: `{raw_value}` is not a number"))?,
                )
            };
        if let Some(expected) = expected_kind(&full_key) {
            if !value.has_kind(expected) {
                return Err(format!(
                    "line {line_no}: `{full_key}` expects {expected}, found {}",
                    value.kind_name()
                ));
            }
        }
        config.values.insert(full_key, value);
    }

    Ok(config)
}

/// The value type each known key must have.
///
/// A key whose type is wrong is a hard error rather than a silent fallback:
/// `max_ratio = "high"` used to be swallowed and the default ratio applied,
/// which quietly loosened the language guard instead of failing loudly.
fn expected_kind(key: &str) -> Option<ValueKind> {
    match key {
        "mode" | "commit.subject_lang" => Some(ValueKind::Str),
        "lang.max_ratio"
        | "lang.max_ident_ratio"
        | "max_ratio"
        | "max_ident_ratio"
        | "commit.subject_min_han"
        | "commit.max_header"
        | "commit.max_footer_line" => Some(ValueKind::Num),
        "commit.types" | "commit.scopes" | "project.disabled" | "disabled" => Some(ValueKind::List),
        _ => None,
    }
}

/// Walk up from `start` looking for `.pengj-templates/gate.toml`, stopping at the repository
/// root marker. Returns the config and the path it was read from.
pub fn load(start: &Path) -> Result<(Config, Option<PathBuf>), String> {
    let mut dir = start.to_path_buf();
    for _ in 0..8 {
        let candidate = dir.join(CONFIG_FILE);
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate)
                .map_err(|error| format!("cannot read {}: {error}", candidate.display()))?;
            let config =
                parse(&text).map_err(|error| format!("{}: {error}", candidate.display()))?;
            return Ok((config, Some(candidate)));
        }
        if !dir.pop() {
            break;
        }
    }
    Ok((Config::default(), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sections_and_values() {
        let config = parse(
            r#"
# comment
mode = "chinese"

[lang]
max_ratio = 0.005

[commit]
types = ["feat", "fix"]
"#,
        )
        .unwrap();
        assert_eq!(config.mode(), Some(Mode::Chinese));
        assert_eq!(config.number("lang.max_ratio"), Some(0.005));
        assert_eq!(config.list("commit.types").unwrap(), ["feat", "fix"]);
    }

    #[test]
    fn trailing_comment_does_not_eat_quoted_hash() {
        let config = parse("mode = \"chinese\" # 中文项目\n").unwrap();
        assert_eq!(config.mode(), Some(Mode::Chinese));
    }

    #[test]
    fn hash_inside_string_is_preserved() {
        let config = parse("commit_types = [\"a#b\"]").unwrap();
        assert_eq!(config.list("commit_types").unwrap(), ["a#b"]);
    }

    #[test]
    fn unknown_keys_are_tolerated() {
        let config = parse("future_switch = true").unwrap();
        assert_eq!(config.number("nope"), None);
    }

    #[test]
    fn malformed_value_is_an_error() {
        assert!(parse("max_ratio = \"high\"").is_err());
        assert!(parse("max_ratio").is_err());
    }

    #[test]
    fn disabled_reads_project_section() {
        let config = parse("[project]\ndisabled = [\"lang-guard\"]").unwrap();
        assert!(config.is_disabled("lang-guard"));
        assert!(!config.is_disabled("commit"));
    }
}
