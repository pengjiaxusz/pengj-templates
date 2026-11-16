//! `rustfmt` on staged files, then re-stage what changed.
//!
//! This replaces lefthook's `stage_fixed: true`. Note the same caveat lefthook
//! had: when a file is partially staged, formatting works on the worktree copy
//! and `git add` therefore stages the unstaged hunks too. Review before you
//! commit a partially staged file.

use std::path::Path;
use std::process::Command;

/// Find the edition that applies to a file by reading the nearest ancestor
/// `Cargo.toml`. Plain `rustfmt` defaults to 2015, which mis-parses modern
/// syntax, so the edition has to be passed explicitly.
fn edition_for(path: &Path) -> String {
    let mut dir = path.parent();
    while let Some(current) = dir {
        let manifest = current.join("Cargo.toml");
        if let Ok(text) = std::fs::read_to_string(&manifest) {
            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("edition") {
                    continue;
                }
                if let Some(value) = trimmed.split('=').nth(1) {
                    let value = value.trim().trim_matches('"');
                    if value.len() == 4 && value.chars().all(|c| c.is_ascii_digit()) {
                        return value.to_string();
                    }
                }
            }
        }
        dir = current.parent();
    }
    "2021".to_string()
}

pub fn run(files: &[String]) -> Result<(), String> {
    let existing: Vec<&String> = files
        .iter()
        .filter(|file| Path::new(file.as_str()).is_file())
        .collect();
    if existing.is_empty() {
        return Ok(());
    }

    for file in &existing {
        let edition = edition_for(Path::new(file.as_str()));
        let output = Command::new("rustfmt")
            .arg("--edition")
            .arg(&edition)
            .arg(file.as_str())
            .output()
            .map_err(|error| {
                format!(
                    "cannot run rustfmt: {error}\n  install the component with `rustup component add rustfmt`"
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "rustfmt failed on {file}:\n{}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }

    let mut args: Vec<&str> = vec!["add", "--"];
    args.extend(existing.iter().map(|file| file.as_str()));
    crate::git::run(&args)?;
    Ok(())
}
