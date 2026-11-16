//! Install the gate as the repository's git hooks.
//!
//! Replaces `lefthook install` + the old `install-hooks.mjs`. Setting
//! `core.hooksPath` is the whole mechanism: git then runs the tracked
//! `.githooks/*` files directly, so the hooks are versioned, diffable and
//! survive a fresh clone without an install step for every tool version.

use std::path::Path;

const HOOKS: [&str; 2] = ["pre-commit", "commit-msg"];

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| format!("cannot stat {}: {error}", path.display()))?
        .permissions();
    let mode = permissions.mode() | 0o755;
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
        .map_err(|error| format!("cannot chmod {}: {error}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    // Git for Windows runs hooks without an executable bit; the mode recorded
    // in the index is what a Linux clone will honour.
    Ok(())
}

pub fn install() -> Result<(), String> {
    if !crate::git::in_work_tree() {
        println!("[gate] not a git work tree, skipping hook installation");
        return Ok(());
    }
    let root = crate::git::repo_root()
        .ok_or_else(|| "cannot determine the repository root".to_string())?;

    for hook in HOOKS {
        let path = root.join(".githooks").join(hook);
        if !path.is_file() {
            return Err(format!(
                "{} is missing; the .githooks directory is incomplete",
                path.display()
            ));
        }
        make_executable(&path)?;
    }

    crate::git::run(&["config", "core.hooksPath", ".githooks"])?;

    let configured = crate::git::probe(&["config", "--get", "core.hooksPath"]).unwrap_or_default();
    if configured != ".githooks" {
        return Err(format!(
            "core.hooksPath is `{configured}` after installation, expected `.githooks`"
        ));
    }

    // Record the executable bit in the index. A fresh clone on Linux runs the
    // hooks straight from the working tree, and without the bit git skips
    // them silently, which turns the gate off without any error.
    for hook in HOOKS {
        let relative = format!(".githooks/{hook}");
        if let Some(mode) = crate::git::hook_mode(&relative) {
            if !mode.starts_with("1007") {
                crate::git::run(&["update-index", "--chmod=+x", &relative])?;
                let updated = crate::git::hook_mode(&relative).unwrap_or_default();
                if !updated.starts_with("1007") {
                    return Err(format!(
                        "{relative} is recorded as {updated}; a Linux clone would skip it.\n  fix with: git update-index --chmod=+x {relative}"
                    ));
                }
                println!("[gate] recorded the executable bit for {relative}");
            }
        }
    }

    println!(
        "[gate] commit gate installed: core.hooksPath=.githooks ({})",
        HOOKS.join(", ")
    );
    Ok(())
}
