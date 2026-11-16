//! Rust source scanner: comments, string literals and identifiers.
//!
//! This is a byte-faithful port of the former `check-lang.mjs` scanner. It is
//! deliberately not a real lexer: the guard only needs to know which string
//! literals and identifiers a file contains, plus the certification markers
//! that exempt them. Comments are skipped, so a Chinese doc comment never
//! counts as copy.

use std::collections::BTreeMap;

pub const MARKER_FILE: &str = "lang-allow:";
pub const MARKER_LINE: &str = "lang-ok:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Default)]
pub struct Scan {
    pub identifiers: Vec<Token>,
    pub strings: Vec<Token>,
    /// `Some(reason)` when the file carries a `lang-allow:` marker.
    pub file_allow: Option<String>,
    /// Per-line exemptions declared with `lang-ok:`.
    pub line_ok: BTreeMap<usize, String>,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (c as u32) >= 0x80
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || (c as u32) >= 0x80
}

fn note_file(out: &mut Scan, text: &str) {
    let Some(index) = text.find(MARKER_FILE) else {
        return;
    };
    if out.file_allow.is_none() {
        out.file_allow = Some(text[index + MARKER_FILE.len()..].trim().to_string());
    }
}

fn note_line(out: &mut Scan, text: &str, at_line: usize) {
    let Some(index) = text.find(MARKER_LINE) else {
        return;
    };
    out.line_ok
        .entry(at_line)
        .or_insert_with(|| text[index + MARKER_LINE.len()..].trim().to_string());
}

fn count_newlines(chars: &[char]) -> usize {
    chars.iter().filter(|&&c| c == '\n').count()
}

/// Match a raw string opener at `i`: `r"..."`, `r#"..."#`, `br#"..."`, `cr#"..."`.
///
/// Returns `(body_start, closer_len)`. The `closer_len` includes the closing
/// quote, so the caller can skip the whole literal.
fn match_raw(chars: &[char], i: usize) -> Option<(usize, usize)> {
    if i > 0 && is_ident_char(chars[i - 1]) {
        return None;
    }
    let mut j = i;
    if matches!(chars.get(j), Some(&'b') | Some(&'c')) {
        j += 1;
    }
    if chars.get(j) != Some(&'r') {
        return None;
    }
    j += 1;
    let mut hashes = 0usize;
    while chars.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    if chars.get(j) != Some(&'"') {
        return None;
    }
    Some((j + 1, hashes + 1))
}

/// Match a char literal at `i` (`'a'`, `'\n'`, `'\\'`). Returns its length.
///
/// A lone quote (`'static`) does not match, which is what keeps lifetimes from
/// being mistaken for literals.
fn match_char_literal(chars: &[char], i: usize) -> Option<usize> {
    let next = *chars.get(i + 1)?;
    if next == '\\' {
        chars.get(i + 2)?;
        if chars.get(i + 3) == Some(&'\'') {
            return Some(4);
        }
        return None;
    }
    if next == '\'' {
        return None;
    }
    if chars.get(i + 2) == Some(&'\'') {
        return Some(3);
    }
    None
}

pub fn scan(source: &str) -> Scan {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut out = Scan::default();
    let mut line = 1usize;
    let mut i = 0usize;

    while i < n {
        let c = chars[i];
        let next = if i + 1 < n { chars[i + 1] } else { '\0' };

        if c == '/' && next == '/' {
            let end = chars[i..]
                .iter()
                .position(|&ch| ch == '\n')
                .map(|offset| i + offset)
                .unwrap_or(n);
            let text: String = chars[i..end].iter().collect();
            note_file(&mut out, &text);
            note_line(&mut out, &text, line);
            i = end;
            continue;
        }

        if c == '/' && next == '*' {
            let start = i;
            let mut depth = 0i32;
            let mut j = i;
            while j < n {
                if chars[j] == '/' && j + 1 < n && chars[j + 1] == '*' {
                    depth += 1;
                    j += 2;
                    continue;
                }
                if chars[j] == '*' && j + 1 < n && chars[j + 1] == '/' {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                j += 1;
            }
            let end = j.min(n);
            let text: String = chars[start..end].iter().collect();
            note_file(&mut out, &text);
            note_line(&mut out, &text, line);
            line += count_newlines(&chars[start..end]);
            i = end;
            continue;
        }

        if let Some((body_start, closer_len)) = match_raw(&chars, i) {
            let hashes = closer_len - 1;
            let mut k = body_start;
            let mut end = n;
            while k < n {
                if chars[k] == '"' && (0..hashes).all(|h| chars.get(k + 1 + h) == Some(&'#')) {
                    end = k;
                    break;
                }
                k += 1;
            }
            let body: String = chars[body_start..end].iter().collect();
            out.strings.push(Token { line, text: body });
            let stop = if end == n { n } else { end + closer_len };
            line += count_newlines(&chars[i..stop]);
            i = stop;
            continue;
        }

        if c == '"' {
            let start = i;
            let mut j = i + 1;
            let mut body = String::new();
            while j < n {
                if chars[j] == '\\' {
                    body.push(chars[j]);
                    if j + 1 < n {
                        body.push(chars[j + 1]);
                    }
                    j += 2;
                    continue;
                }
                if chars[j] == '"' {
                    break;
                }
                body.push(chars[j]);
                j += 1;
            }
            out.strings.push(Token { line, text: body });
            let stop = (j + 1).min(n);
            line += count_newlines(&chars[start..stop]);
            i = stop;
            continue;
        }

        if c == '\'' {
            if let Some(len) = match_char_literal(&chars, i) {
                out.strings.push(Token {
                    line,
                    text: chars[i..i + len].iter().collect(),
                });
                line += count_newlines(&chars[i..i + len]);
                i += len;
                continue;
            }
            i += 1;
            continue;
        }

        if is_ident_start(c) {
            let start = i;
            let mut j = i;
            while j < n && is_ident_char(chars[j]) {
                j += 1;
            }
            out.identifiers.push(Token {
                line,
                text: chars[start..j].iter().collect(),
            });
            i = j;
            continue;
        }

        if c == '\n' {
            line += 1;
        }
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetimes_are_not_char_literals() {
        let scanned = scan("fn f() -> &'static str { \"x\" }");
        assert_eq!(scanned.strings.len(), 1);
        assert_eq!(scanned.strings[0].text, "x");
    }

    #[test]
    fn escaped_quotes_stay_inside_the_literal() {
        let scanned = scan(r#"let s = "a\"b"; let t = "c";"#);
        assert_eq!(scanned.strings.len(), 2);
        assert_eq!(scanned.strings[0].text, r#"a\"b"#);
    }

    #[test]
    fn nested_block_comments_are_skipped() {
        let scanned = scan("/* outer /* inner */ still comment */ let x = \"y\";");
        assert_eq!(scanned.strings.len(), 1);
        assert_eq!(scanned.strings[0].line, 1);
    }

    #[test]
    fn markers_are_collected_from_comments_only() {
        let scanned = scan("// lang-allow: fixture\nlet s = \"a\";");
        assert_eq!(scanned.file_allow.as_deref(), Some("fixture"));
        let line_ok = scan("let s = \"a\"; // lang-ok: token");
        assert_eq!(line_ok.line_ok.get(&1).map(String::as_str), Some("token"));
    }

    #[test]
    fn line_numbers_survive_multiline_literals() {
        let scanned = scan("let a = \"x\ny\";\nlet b = \"z\";");
        assert_eq!(scanned.strings[1].line, 3);
    }
}
