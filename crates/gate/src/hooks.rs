//! Install the gate as the repository's git hooks.
//!
//! Replaces `lefthook install` + the old `install-hooks.mjs`. Setting
//! `core.hooksPath` is the whole mechanism: git then runs the tracked
//! `.githooks/*` files directly, so the hooks are versioned, diffable and
//! survive a fresh clone without an install step for every tool version.
//!
//! The scope is deliberately per-repository: `core.hooksPath = .githooks`
//! inside the repository's own `.git/config`. Git has no versioned way to
//! activate hooks, so `pengj-templates-cli doctor --fix` (or `gate hooks install`)
//! is the single activation entry point.

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

/// 与提交门禁竞争 `core.hooksPath` 的工具残留告警。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRivalWarning {
    /// 竞品工具名称（如 `lefthook`、`husky`）
    pub tool: String,
    /// 检测到的残留旧钩子列表（如 `commit-msg`、`pre-commit`）
    pub leftovers: Vec<String>,
}

/// `install_in` 的结构化执行结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookInstallOutcome {
    /// 本次激活的钩子名称列表
    pub hooks: Vec<String>,
    /// 本次在 git 索引中补齐可执行位的钩子相对路径
    pub chmod_recorded: Vec<String>,
    /// 检测到的竞品残留告警
    pub rival_warnings: Vec<HookRivalWarning>,
    /// 非 git 工作树时置为 true 并跳过安装
    pub skipped_not_git: bool,
}

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

/// 在指定目录内激活提交门禁钩子，返回结构化结果且不向标准输出打印日志。
pub fn install_in(dir: &Path) -> Result<HookInstallOutcome, String> {
    if !crate::git::in_work_tree_in(dir) {
        return Ok(HookInstallOutcome {
            hooks: Vec::new(),
            chmod_recorded: Vec::new(),
            rival_warnings: Vec::new(),
            skipped_not_git: true,
        });
    }
    let root = crate::git::repo_root_in(dir)
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

    crate::git::run_in(&root, &["config", "core.hooksPath", ".githooks"])?;

    let configured =
        crate::git::probe_in(&root, &["config", "--get", "core.hooksPath"]).unwrap_or_default();
    if configured != ".githooks" {
        return Err(format!(
            "core.hooksPath is `{configured}` after installation, expected `.githooks`"
        ));
    }

    // Record the executable bit in the index. A fresh clone on Linux runs the
    // hooks straight from the working tree, and without the bit git skips
    // them silently, which turns the gate off without any error.
    let mut chmod_recorded = Vec::new();
    for hook in &hooks {
        let relative = format!(".githooks/{hook}");
        if let Some(mode) = crate::git::hook_mode_in(&root, &relative) {
            if !mode.starts_with("1007") {
                crate::git::run_in(&root, &["update-index", "--chmod=+x", &relative])?;
                let updated = crate::git::hook_mode_in(&root, &relative).unwrap_or_default();
                if !updated.starts_with("1007") {
                    return Err(format!(
                        "{relative} is recorded as {updated}; a Linux clone would skip it.\n  fix with: git update-index --chmod=+x {relative}"
                    ));
                }
                chmod_recorded.push(relative);
            }
        }
    }

    let rival_warnings = detect_rivals(&root);
    Ok(HookInstallOutcome {
        hooks: hooks.into_iter().map(str::to_string).collect(),
        chmod_recorded,
        rival_warnings,
        skipped_not_git: false,
    })
}

pub fn install() -> Result<(), String> {
    let outcome = install_in(Path::new("."))?;
    if outcome.skipped_not_git {
        println!("[gate] not a git work tree, skipping hook installation");
        return Ok(());
    }
    for relative in &outcome.chmod_recorded {
        println!("[gate] recorded the executable bit for {relative}");
    }
    println!(
        "[gate] commit gate installed: core.hooksPath=.githooks ({})",
        outcome.hooks.join(", ")
    );
    print_rival_warnings(&outcome.rival_warnings);
    Ok(())
}

/// 检测 `.githooks/` 下由其他钩子工具留下的 `.old` 残留文件。
pub fn detect_rivals(root: &Path) -> Vec<HookRivalWarning> {
    let hooks_dir = root.join(".githooks");
    let mut warnings = Vec::new();
    for (name, _) in RIVALS {
        let leftovers = ["commit-msg", "pre-commit"]
            .iter()
            .filter(|hook| hooks_dir.join(format!("{hook}.old")).exists())
            .map(|hook| (*hook).to_string())
            .collect::<Vec<_>>();
        if !leftovers.is_empty() {
            warnings.push(HookRivalWarning {
                tool: name.to_string(),
                leftovers,
            });
        }
    }
    warnings
}

fn print_rival_warnings(warnings: &[HookRivalWarning]) {
    for warning in warnings {
        let reason = RIVALS
            .iter()
            .find(|(name, _)| *name == warning.tool)
            .map(|(_, reason)| *reason)
            .unwrap_or("writes its own hooks into core.hooksPath and will replace the gate hooks");
        println!(
            "[gate] warning: {} left hooks behind ({}); {reason}.\n  \
             remove the leftovers and re-run `just doctor --fix`",
            warning.tool,
            warning.leftovers.join(", ")
        );
    }
}

/// Report the tools that would fight with the gate for `core.hooksPath`.
///
/// This is advisory: the gate does not need those tools to be absent, but
/// running both means two sets of hooks in the same directory and only one
/// of them wins.
pub fn report_rivals(root: &Path) {
    let warnings = detect_rivals(root);
    print_rival_warnings(&warnings);
}
