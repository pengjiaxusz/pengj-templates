//! 路径展开与配置目录定位
//!
//! 配方里的路径统一写成 `~`、`$VAR`、`%VAR%` 形式，落盘前再展开为绝对路径，
//! 这样同一份配方可以跨机器、跨平台复用。

use std::path::PathBuf;

use crate::error::{McpError, Result};

/// 用户主目录（Windows 优先 USERPROFILE，其余平台回落 HOME）
pub fn home_dir() -> Result<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| {
            McpError::NoConfigDir("无法定位用户主目录（USERPROFILE/HOME 均为空）".into())
        })
}

/// Windows 的 `%APPDATA%`（其余平台回落到 `$XDG_CONFIG_HOME` 或 `~/.config`）
pub fn appdata_dir() -> Result<PathBuf> {
    if let Some(v) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(v));
    }
    if let Some(v) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(v));
    }
    Ok(home_dir()?.join(".config"))
}

/// 本工具的私有数据目录：`~/.pengj-mcp`
pub fn config_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".pengj-mcp"))
}

/// 展开 `~`、`$VAR`、`${VAR}`、`%VAR%`；无法识别的变量原样保留，便于报错时看出问题
pub fn expand(raw: &str) -> PathBuf {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0usize;

    if chars.first() == Some(&'~') {
        if let Ok(home) = home_dir() {
            out.push_str(&home.to_string_lossy());
            i = 1;
        }
    }

    while i < chars.len() {
        let c = chars[i];
        if c == '$' {
            if let (Some(name), next) = read_var(&chars, i + 1, false) {
                push_or_literal(&mut out, &name, &chars[i..next]);
                i = next;
                continue;
            }
        } else if c == '%' {
            if let (Some(name), next) = read_var(&chars, i + 1, true) {
                push_or_literal(&mut out, &name, &chars[i..next]);
                i = next;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }

    // Windows 下把正斜杠归一化，避免展开后出现 `C:\a\b/qrenderdoc/x` 这类混合写法
    if cfg!(windows) {
        PathBuf::from(out.replace('/', "\\"))
    } else {
        PathBuf::from(out)
    }
}

fn push_or_literal(out: &mut String, name: &str, literal: &[char]) {
    match std::env::var_os(name) {
        Some(v) => out.push_str(&v.to_string_lossy()),
        None => out.extend(literal),
    }
}

/// 读取变量名：`%NAME%` / `$NAME` / `${NAME}`
fn read_var(chars: &[char], start: usize, percent: bool) -> (Option<String>, usize) {
    let braced = !percent && chars.get(start) == Some(&'{');
    let mut j = if braced { start + 1 } else { start };
    let begin = j;

    while j < chars.len() {
        let c = chars[j];
        let keep = if percent {
            c != '%'
        } else if braced {
            c != '}'
        } else {
            c.is_ascii_alphanumeric() || c == '_'
        };
        if !keep {
            break;
        }
        j += 1;
    }

    if j == begin {
        return (None, start);
    }
    let name: String = chars[begin..j].iter().collect();
    let next = match chars.get(j) {
        Some(c) if percent && *c == '%' => j + 1,
        Some(c) if braced && *c == '}' => j + 1,
        _ if !percent && !braced => j,
        _ => return (None, start),
    };
    (Some(name), next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 展开波浪号得到主目录() {
        let p = expand("~/.pengj-mcp");
        assert!(p.is_absolute(), "应展开为绝对路径: {}", p.display());
        assert!(p.ends_with(".pengj-mcp"));
    }

    #[test]
    fn 展开百分号与美元变量() {
        let raw_percent = "%APPDATA%/qrenderdoc/UI.config";
        let got = expand(raw_percent);
        match std::env::var_os("APPDATA") {
            Some(v) => assert_eq!(got, PathBuf::from(v).join("qrenderdoc/UI.config")),
            None => assert_eq!(got, PathBuf::from(raw_percent)),
        }

        let raw_dollar = "$HOME/x";
        let got = expand(raw_dollar);
        match std::env::var_os("HOME") {
            Some(v) => assert_eq!(got, PathBuf::from(v).join("x")),
            None => assert_eq!(got, PathBuf::from(raw_dollar)),
        }
    }

    #[test]
    fn 未知变量原样保留() {
        let got = expand("${PENGJ_NOT_SET_XYZ}/a");
        assert_eq!(got, PathBuf::from("${PENGJ_NOT_SET_XYZ}/a"));
    }

    #[test]
    fn 单独的美元与百分号不算变量() {
        assert_eq!(expand("a$"), PathBuf::from("a$"));
        assert_eq!(expand("100%"), PathBuf::from("100%"));
    }
}
