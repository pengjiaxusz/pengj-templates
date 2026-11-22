//! Thin wrapper around the git CLI.
//!
//! The gate never reimplements git: hooks, index and pathspec semantics change
//! between versions, and every developer already has git.

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run git inside `dir`.
///
/// The gate must not depend on the caller's working directory: the CLI can
/// be invoked from anywhere, and tests run in parallel with a different
/// repository each.
pub fn run_in(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|error| format!("cannot run git in {}: {error}", dir.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run git inside `dir` and keep stdout, treating a non-zero status as "no
/// answer" instead of an error.
pub fn probe_in(dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run git and keep stdout, but treat a non-zero status as "no answer" instead
/// of an error. Used for probes such as `rev-parse --git-dir`.
pub fn probe(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn in_work_tree() -> bool {
    probe(&["rev-parse", "--git-dir"]).is_some()
}

/// Check whether `dir` is inside a git work tree.
pub fn in_work_tree_in(dir: &Path) -> bool {
    probe_in(dir, &["rev-parse", "--git-dir"]).is_some()
}

pub fn toplevel() -> Option<String> {
    probe(&["rev-parse", "--show-toplevel"])
}

/// Files staged for this commit, restricted to Rust sources.
///
/// `--diff-filter=ACMR` drops deletions: a hook cannot format a file that the
/// same commit removes.
pub fn staged_rust_files() -> Result<Vec<String>, String> {
    if !in_work_tree() {
        return Ok(Vec::new());
    }
    let output = run(&[
        "diff",
        "--cached",
        "--name-only",
        "--diff-filter=ACMR",
        "--",
        "*.rs",
    ])?;
    Ok(output
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect())
}

pub fn hook_mode(path: &str) -> Option<String> {
    let output = run(&["ls-files", "-s", "--", path]).ok()?;
    let mode = output.split_whitespace().next()?.to_string();
    Some(mode)
}

/// Return the git index mode (e.g. `100644` / `100755`) of `path` inside `dir`.
pub fn hook_mode_in(dir: &Path, path: &str) -> Option<String> {
    let output = run_in(dir, &["ls-files", "-s", "--", path]).ok()?;
    let mode = output.split_whitespace().next()?.to_string();
    Some(mode)
}

pub fn repo_root() -> Option<PathBuf> {
    toplevel().map(|root| Path::new(&root).to_path_buf())
}

/// Resolve the git repository root containing `dir`.
pub fn repo_root_in(dir: &Path) -> Option<PathBuf> {
    probe_in(dir, &["rev-parse", "--show-toplevel"]).map(|root| Path::new(&root).to_path_buf())
}
