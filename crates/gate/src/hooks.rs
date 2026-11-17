//! Install the gate as the repository's git hooks.
//!
//! Replaces `lefthook install` + the old `install-hooks.mjs`. Setting
//! `core.hooksPath` is the whole mechanism: git then runs the tracked
//! `.githooks/*` files directly, so the hooks are versioned, diffable and
//! survive a fresh clone without an install step for every tool version.
//!
//! Two scopes are supported:
//! - **per-repository** ([`install`]): `core.hooksPath = .githooks` inside the
//!   repository's own `.git/config`.
//! - **global** ([`install_global`]): `core.hooksPath` points at a dispatcher
//!   directory next to the CLI. Git reads that setting on every invocation,
//!   so a fresh clone is guarded with no extra step — and existing clones
//!   become guarded the moment the setting is written.

use std::path::{Path, PathBuf};

const HOOKS: [&str; 2] = ["pre-commit", "commit-msg"];

/// The marker every gate hook carries. A hook without it was written by
/// another tool and must not be overwritten.
const MARKER: &str = "gate hook";

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

/// The tools that are known to fight over `core.hooksPath`.
const RIVALS: [(&str, &str); 2] = [
    (
        "lefthook",
        "lefthook writes its own hooks into core.hooksPath and will replace the gate hooks",
    ),
    (
        "husky",
        "husky writes its own hooks into core.hooksPath and will replace the gate hooks",
    ),
];

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
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        if !text.contains(MARKER) {
            return Err(format!(
                "{} was not written by the gate.\n  \
                 Another tool owns this directory; installing would replace its hooks.\n  \
                 Remove the foreign hooks first, then run `just gate-hooks` again.",
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
    report_rivals(&root);
    Ok(())
}

/// Report the tools that would fight with the gate for `core.hooksPath`.
///
/// This is advisory: the gate does not need those tools to be absent, but
/// running both means two sets of hooks in the same directory and only one
/// of them wins.
pub fn report_rivals(root: &std::path::Path) {
    let hooks_dir = root.join(".githooks");
    for (name, reason) in RIVALS {
        let leftovers = ["commit-msg", "pre-commit"]
            .iter()
            .filter_map(|hook| {
                hooks_dir
                    .join(format!("{hook}.old"))
                    .exists()
                    .then_some(*hook)
            })
            .collect::<Vec<_>>();
        if !leftovers.is_empty() {
            println!(
                "[gate] warning: {name} left hooks behind ({}); {reason}.\n  \
                 remove the leftovers and re-run `just gate-hooks`",
                leftovers.join(", ")
            );
        }
    }
}

/// The global dispatcher directory, relative to the installation root.
const DISPATCH_DIR: &str = "hooks";

/// Locate the installation root — the directory holding `bin/` and `templates/`.
///
/// `PENGJ_INSTALL_DIR` wins when set; otherwise the root is derived from the
/// running binary (`<root>/bin/pengj-templates-cli`). A root without a
/// `templates/` directory is not an installation, so the lookup fails loudly
/// instead of scattering dispatcher files into an arbitrary place.
pub fn install_root() -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("PENGJ_INSTALL_DIR") {
        let dir = PathBuf::from(dir);
        if dir.join("templates").is_dir() {
            return Ok(dir);
        }
        return Err(format!(
            "PENGJ_INSTALL_DIR={} has no templates/ directory; it is not a pengj-templates installation",
            dir.display()
        ));
    }

    let exe = std::env::current_exe()
        .map_err(|error| format!("cannot locate the running executable: {error}"))?;
    // <root>/bin/pengj-templates-cli → <root>
    let root = exe
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "cannot derive the installation root from the executable path".to_string())?
        .to_path_buf();
    if !root.join("templates").is_dir() {
        return Err(format!(
            "{} is not a pengj-templates installation (no templates/ directory).\n  \
             set PENGJ_INSTALL_DIR to the installation root and run this again",
            root.display()
        ));
    }
    Ok(root)
}

/// Render a path the way git wants it in config: forward slashes, no escaping.
fn git_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The dispatcher script: git runs this, it forwards to the repository's own
/// tracked hook. A repository without `.githooks/` is left alone, so the
/// global setting never touches projects that do not use the gate.
fn dispatcher_script(hook: &str) -> String {
    format!(
        r#"#!/bin/sh
# pengj-templates global hook dispatcher - generated, do not edit.
#
# git finds this file through the global core.hooksPath. It forwards to the
# repository's own tracked hook under .githooks/, so the policy stays
# versioned and reviewable in the repository while the wiring lives here.
# A repository without .githooks/ is not managed by the gate: exit quietly.
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -n "$root" ] || exit 0
script="$root/.githooks/{hook}"
[ -f "$script" ] || exit 0
exec sh "$script" "$@"
"#
    )
}

/// Install the gate globally: every repository on this machine picks it up.
///
/// Git reads `core.hooksPath` on every invocation, so this covers both fresh
/// clones and repositories that already exist. The repository-level
/// `core.hooksPath` (if any) still wins — git prefers local over global.
pub fn install_global() -> Result<(), String> {
    let root = install_root()?;
    let dir = root.join(DISPATCH_DIR);

    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    for hook in HOOKS {
        let path = dir.join(hook);
        std::fs::write(&path, dispatcher_script(hook))
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        make_executable(&path)?;
    }

    let target = git_path(&dir);
    let existing = crate::git::probe(&["config", "--global", "--get", "core.hooksPath"])
        .filter(|value| !value.is_empty());
    if let Some(current) = existing.as_deref() {
        if current != target {
            // Somebody else owns the global hooks path (lefthook, husky, a
            // shared team directory). Overwriting it would silently disable
            // their hooks, so stop and let the user decide.
            return Err(format!(
                "the global core.hooksPath is already set to `{current}`.\n  \
                 it belongs to another hook setup; overriding it would disable those hooks.\n  \
                 remove it first (`git config --global --unset core.hooksPath`) \
                 and run this again, or keep it and use the per-repository \
                 `just gate-hooks` instead"
            ));
        }
        println!("[gate] global core.hooksPath already points at {target}");
    } else {
        crate::git::run(&["config", "--global", "core.hooksPath", &target])?;
    }

    let configured =
        crate::git::probe(&["config", "--global", "--get", "core.hooksPath"]).unwrap_or_default();
    if configured != target {
        return Err(format!(
            "the global core.hooksPath is `{configured}` after installation, expected `{target}`"
        ));
    }

    println!(
        "[gate] global commit gate installed: core.hooksPath={target}\n  \
         every repository now forwards to its own .githooks/ - fresh clones are guarded with no extra step.\n  \
         remove it with: pengj-templates-cli gate hooks uninstall --global"
    );
    Ok(())
}

/// Remove the global dispatcher and the global `core.hooksPath` setting.
pub fn uninstall_global() -> Result<(), String> {
    let root = install_root()?;
    let dir = root.join(DISPATCH_DIR);
    let target = git_path(&dir);

    if let Some(current) = crate::git::probe(&["config", "--global", "--get", "core.hooksPath"])
        .filter(|v| !v.is_empty())
    {
        if current == target {
            crate::git::run(&["config", "--global", "--unset", "core.hooksPath"])?;
            println!("[gate] removed the global core.hooksPath setting");
        } else {
            println!("[gate] the global core.hooksPath is `{current}`, not ours - left untouched");
        }
    } else {
        println!("[gate] no global core.hooksPath is set");
    }

    if dir.is_dir() {
        std::fs::remove_dir_all(&dir)
            .map_err(|error| format!("cannot remove {}: {error}", dir.display()))?;
        println!("[gate] removed the dispatcher directory {}", dir.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatcher_forwards_to_the_repository_hook() {
        let script = dispatcher_script("commit-msg");
        // The dispatcher resolves the repository root and hands over to the
        // tracked hook; a repository without one must be left alone.
        assert!(script.contains("git rev-parse --show-toplevel"));
        assert!(script.contains(".githooks/commit-msg"));
        assert!(script.contains("[ -f \"$script\" ] || exit 0"));
        // The tracked hook is invoked through `sh` so a clone without the
        // executable bit recorded in the index still runs it.
        assert!(script.contains("exec sh \"$script\" \"$@\""));
    }

    #[test]
    fn dispatcher_names_the_hook_it_dispatches() {
        assert!(dispatcher_script("pre-commit").contains(".githooks/pre-commit"));
        assert!(dispatcher_script("commit-msg").contains(".githooks/commit-msg"));
    }

    #[test]
    fn git_path_uses_forward_slashes() {
        // git stores the path verbatim; backslashes would break the shell
        // dispatcher on Windows.
        let path = PathBuf::from(r"C:\Users\dev\AppData\Local\pengj-templates\hooks");
        assert_eq!(
            git_path(&path),
            "C:/Users/dev/AppData/Local/pengj-templates/hooks"
        );
    }
}
