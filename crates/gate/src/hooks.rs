//! Install the gate as the repository's git hooks.
//!
//! Replaces `lefthook install` + the old `install-hooks.mjs`. Setting
//! `core.hooksPath` is the whole mechanism: git then runs the tracked
//! `.githooks/*` files directly, so the hooks are versioned, diffable and
//! survive a fresh clone without an install step for every tool version.
//!
//! The scope is deliberately per-repository: `core.hooksPath = .githooks`
//! inside the repository's own `.git/config`. Git has no versioned way to
//! activate hooks, so the project's environment check (`.pengj-templates/scripts/doctor.*`)
//! is the single activation entry point — it calls this command when the CLI
//! is available and falls back to plain `git config` when it is not.

use std::path::Path;

/// 必须存在的钩子：缺一个就说明 `.githooks` 不完整，安装直接失败。
const REQUIRED_HOOKS: [&str; 2] = ["pre-commit", "commit-msg"];

/// 有文件就一并装上的钩子。
///
/// `pre-push` 晚于另外两个出现：新 CLI 掉进一个还没 `update` 过的旧项目时，`.githooks`
/// 里当然没有它。这种情况跳过而不是报错——安装器的职责是「把仓库里有的钩子激活」，
/// 不是替模板补齐文件；文件由 `update` 送上门。
const OPTIONAL_HOOKS: [&str; 1] = ["pre-push"];

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

/// 同一个路径上已经躺着一个不是 gate 写的钩子。
///
/// 装下去就等于替别的工具接管它的钩子，所以这里一律硬失败，而不是覆盖后再解释。
fn foreign_hook_message(path: &Path) -> String {
    format!(
        "{} was not written by the gate.\n  \
         Another tool owns this directory; installing would replace its hooks.\n  \
         Remove the foreign hooks first, then run `just doctor --fix` \
         (or `pengj-templates-cli gate hooks install`) again.",
        path.display()
    )
}

pub fn install() -> Result<(), String> {
    if !crate::git::in_work_tree() {
        println!("[gate] not a git work tree, skipping hook installation");
        return Ok(());
    }
    let root = crate::git::repo_root()
        .ok_or_else(|| "cannot determine the repository root".to_string())?;

    // 先判定哪些钩子要装：必需的缺一不可，可选的（`pre-push`）只在这份 `.githooks`
    // 真的带了它时才纳入。
    let mut hooks: Vec<&str> = Vec::new();
    for hook in REQUIRED_HOOKS {
        let path = root.join(".githooks").join(hook);
        if !path.is_file() {
            return Err(format!(
                "{} is missing; the .githooks directory is incomplete",
                path.display()
            ));
        }
        if !std::fs::read_to_string(&path)
            .unwrap_or_default()
            .contains(MARKER)
        {
            return Err(foreign_hook_message(&path));
        }
        hooks.push(hook);
    }
    for hook in OPTIONAL_HOOKS {
        let path = root.join(".githooks").join(hook);
        if !path.is_file() {
            continue;
        }
        // 可选只针对「文件不存在」，不针对「文件被别的工具占了」——后者仍是硬错误。
        if !std::fs::read_to_string(&path)
            .unwrap_or_default()
            .contains(MARKER)
        {
            return Err(foreign_hook_message(&path));
        }
        hooks.push(hook);
    }
    for hook in &hooks {
        make_executable(&root.join(".githooks").join(hook))?;
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
    for hook in &hooks {
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
        hooks.join(", ")
    );
    report_rivals(&root);
    Ok(())
}

/// Report the tools that would fight with the gate for `core.hooksPath`.
///
/// This is advisory: the gate does not need those tools to be absent, but
/// running both means two sets of hooks in the same directory and only one
/// of them wins.
pub fn report_rivals(root: &Path) {
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
                 remove the leftovers and re-run `just doctor --fix`",
                leftovers.join(", ")
            );
        }
    }
}
