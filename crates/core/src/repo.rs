//! 仓库级初始化：工作区链接、仓库级 Git 配置与子模块补缺。
//!
//! 这些操作与模板层无关，供 `just doctor --fix` 通过
//! `pengj-templates-cli repo init` 调用。不变式：
//! - 只补缺失项，不覆盖真实目录或指向别处的链接；
//! - 链接目标必须未跟踪且被忽略，避免污染 `git status`；
//! - 子模块只初始化未初始化的，不移动已有工作树；
//! - 重复执行无副作用。

use crate::error::{CoreError, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 一条工作区链接：`link` 与 `target` 都相对项目根目录。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LinkSpec {
    pub link: String,
    pub target: String,
}

impl LinkSpec {
    /// 解析 `<link>=<target>` 形式的命令行参数。
    pub fn parse(raw: &str) -> Result<Self> {
        let (link, target) = raw
            .split_once('=')
            .ok_or_else(|| CoreError::Git(format!("链接参数格式应为 <link>=<target>: {raw}")))?;
        let link = link.trim();
        let target = target.trim();
        if link.is_empty() || target.is_empty() {
            return Err(CoreError::Git(format!(
                "链接参数格式应为 <link>=<target>: {raw}"
            )));
        }
        Ok(Self {
            link: link.replace('\\', "/"),
            target: target.replace('\\', "/"),
        })
    }
}

/// 仓库内一项检查/修复的结果状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoStatus {
    Ok,
    Missing,
    Created,
    Conflict,
    Skipped,
}

/// 单项结果。
#[derive(Debug, Clone, Serialize)]
pub struct RepoItem {
    pub name: String,
    pub status: RepoStatus,
    pub detail: String,
}

/// 一次 `repo status` / `repo init` 的完整报告。
#[derive(Debug, Clone, Serialize)]
pub struct RepoReport {
    pub root: String,
    pub ok: bool,
    pub items: Vec<RepoItem>,
}

/// 默认工作区链接：仅在项目包含 `.agents` 时启用（agent 层产物）。
///
/// 分两类：
/// - `.trae` 整目录链：Trae 同时读 `rules/`、`skills/` 与 `mcp.json`，只链技能不够；
///   它写进 `.agents/` 的 `specs|plans|documents` 由模板随附的 `.agents/.gitignore` 挡住。
/// - `.claude/skills`、`.workbuddy/skills` 点路径链：这两个客户端目录里还有自己的状态
///   （Claude 的 settings、WorkBuddy 的 memory/_inbox），整目录链会把状态搬进受管目录。
pub fn default_links(root: &Path) -> Vec<LinkSpec> {
    if !root.join(".agents").is_dir() {
        return Vec::new();
    }
    vec![
        LinkSpec {
            link: ".trae".to_string(),
            target: ".agents".to_string(),
        },
        LinkSpec {
            link: ".claude/skills".to_string(),
            target: ".agents/skills".to_string(),
        },
        LinkSpec {
            link: ".workbuddy/skills".to_string(),
            target: ".agents/skills".to_string(),
        },
    ]
}

/// 只检查，不修改任何文件。
pub fn repo_status(root: &Path, links: &[LinkSpec]) -> Result<RepoReport> {
    let root = absolute(root)?;
    let mut items = Vec::new();
    let in_git = is_git_work_tree(&root);

    if in_git {
        check_git_config(&root, &mut items)?;
        check_submodules(&root, &mut items)?;
    } else {
        items.push(RepoItem {
            name: "git".to_string(),
            status: RepoStatus::Skipped,
            detail: "不是 git 工作树，跳过仓库级配置与子模块检查".to_string(),
        });
    }

    for spec in links {
        check_link(&root, spec, in_git, &mut items);
    }
    Ok(finish(&root, items))
}

/// 创建缺失项；冲突项保持原样并计入失败。
pub fn repo_init(root: &Path, links: &[LinkSpec], with_submodules: bool) -> Result<RepoReport> {
    let root = absolute(root)?;
    let mut items = Vec::new();
    let in_git = is_git_work_tree(&root);

    if in_git {
        ensure_git_config(&root, &mut items)?;
        if with_submodules {
            init_submodules(&root, &mut items)?;
        }
    } else {
        items.push(RepoItem {
            name: "git".to_string(),
            status: RepoStatus::Skipped,
            detail: "不是 git 工作树，跳过仓库级配置与子模块检查".to_string(),
        });
    }

    for spec in links {
        init_link(&root, spec, in_git, &mut items);
    }
    Ok(finish(&root, items))
}

fn absolute(root: &Path) -> Result<PathBuf> {
    root.canonicalize()
        .map_err(|error| CoreError::Git(format!("无法访问目录 {}: {error}", root.display())))
}

fn finish(root: &Path, items: Vec<RepoItem>) -> RepoReport {
    let ok = !items
        .iter()
        .any(|item| matches!(item.status, RepoStatus::Missing | RepoStatus::Conflict));
    RepoReport {
        root: display_path(root),
        ok,
        items,
    }
}

fn display_path(path: &Path) -> String {
    let text = path.display().to_string();
    #[cfg(windows)]
    {
        text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
    }
    #[cfg(not(windows))]
    {
        text
    }
}

fn item(name: &str, status: RepoStatus, detail: impl Into<String>) -> RepoItem {
    RepoItem {
        name: name.to_string(),
        status,
        detail: detail.into(),
    }
}

// ---------------------------------------------------------------------------
// git 基础
// ---------------------------------------------------------------------------

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| CoreError::Git(format!("无法执行 git: {error}")))
}

fn git_stdout(root: &Path, args: &[&str]) -> Result<String> {
    let output = git(root, args)?;
    if !output.status.success() {
        return Err(CoreError::Git(format!(
            "git {} 失败: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn is_git_work_tree(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn config_get(root: &Path, key: &str) -> Option<String> {
    let output = git(root, &["config", "--local", "--get", key]).ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn config_set(root: &Path, key: &str, value: &str) -> Result<()> {
    git_stdout(root, &["config", "--local", key, value]).map(|_| ())
}

fn is_tracked(root: &Path, rel: &str) -> bool {
    git(root, &["ls-files", "--", rel])
        .map(|output| {
            output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
        })
        .unwrap_or(false)
}

fn is_ignored(root: &Path, rel: &str) -> bool {
    git(root, &["check-ignore", "-q", "--", rel])
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// 仓库级 Git 配置
// ---------------------------------------------------------------------------

fn check_git_config(root: &Path, items: &mut Vec<RepoItem>) -> Result<()> {
    if root.join(".gitmodules").is_file() {
        match config_get(root, "submodule.recurse").as_deref() {
            Some("true") => items.push(item("git:submodule.recurse", RepoStatus::Ok, "已开启")),
            _ => items.push(item(
                "git:submodule.recurse",
                RepoStatus::Missing,
                "未开启；运行 repo init 修复",
            )),
        }
    }

    if has_tracked_symlinks(root)? {
        match config_get(root, "core.symlinks").as_deref() {
            Some("true") => items.push(item("git:core.symlinks", RepoStatus::Ok, "已开启")),
            _ => items.push(item(
                "git:core.symlinks",
                RepoStatus::Missing,
                "仓库跟踪符号链接但未开启；运行 repo init 修复",
            )),
        }
    }
    Ok(())
}

fn ensure_git_config(root: &Path, items: &mut Vec<RepoItem>) -> Result<()> {
    if root.join(".gitmodules").is_file()
        && config_get(root, "submodule.recurse").as_deref() != Some("true")
    {
        config_set(root, "submodule.recurse", "true")?;
        items.push(item("git:submodule.recurse", RepoStatus::Created, "已开启"));
    }

    if has_tracked_symlinks(root)? && config_get(root, "core.symlinks").as_deref() != Some("true") {
        config_set(root, "core.symlinks", "true")?;
        items.push(item("git:core.symlinks", RepoStatus::Created, "已开启"));
    }
    Ok(())
}

fn has_tracked_symlinks(root: &Path) -> Result<bool> {
    let listing = git_stdout(root, &["ls-files", "-s"])?;
    Ok(listing.lines().any(|line| line.starts_with("120000 ")))
}

// ---------------------------------------------------------------------------
// 子模块
// ---------------------------------------------------------------------------

fn missing_submodules(root: &Path) -> Result<Vec<String>> {
    if !root.join(".gitmodules").is_file() {
        return Ok(Vec::new());
    }
    let listing = git_stdout(root, &["submodule", "status"])?;
    let mut paths = Vec::new();
    for line in listing.lines() {
        // 未初始化的子模块行以 `-` 开头；已初始化的是 ` `、`+` 或 `U`。
        if let Some(rest) = line.strip_prefix('-') {
            let mut fields = rest.split_whitespace();
            let _sha = fields.next();
            if let Some(path) = fields.next() {
                paths.push(path.to_string());
            }
        }
    }
    Ok(paths)
}

fn check_submodules(root: &Path, items: &mut Vec<RepoItem>) -> Result<()> {
    let missing = missing_submodules(root)?;
    if missing.is_empty() {
        return Ok(());
    }
    items.push(item(
        "submodules",
        RepoStatus::Missing,
        format!("{} 个子模块未初始化；运行 repo init 修复", missing.len()),
    ));
    Ok(())
}

fn init_submodules(root: &Path, items: &mut Vec<RepoItem>) -> Result<()> {
    let missing = missing_submodules(root)?;
    if missing.is_empty() {
        return Ok(());
    }
    let mut args = vec!["submodule", "update", "--init", "--"];
    args.extend(missing.iter().map(String::as_str));
    git_stdout(root, &args)?;
    items.push(item(
        "submodules",
        RepoStatus::Created,
        format!("已初始化 {} 个子模块", missing.len()),
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// 工作区链接
// ---------------------------------------------------------------------------

fn check_link(root: &Path, spec: &LinkSpec, in_git: bool, items: &mut Vec<RepoItem>) {
    let target = root.join(&spec.target);
    if !target.exists() {
        items.push(item(
            &spec.link,
            RepoStatus::Conflict,
            format!("链接目标不存在: {}", spec.target),
        ));
        return;
    }

    let link = root.join(&spec.link);
    match std::fs::symlink_metadata(&link) {
        Ok(_) => match (link.canonicalize(), target.canonicalize()) {
            (Ok(actual), Ok(expected)) if actual == expected => items.push(item(
                &spec.link,
                RepoStatus::Ok,
                format!("-> {}", spec.target),
            )),
            _ => items.push(item(
                &spec.link,
                RepoStatus::Conflict,
                format!("已存在且未指向 {}，不会覆盖", spec.target),
            )),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if in_git && !is_ignored(root, &spec.link) {
                items.push(item(
                    &spec.link,
                    RepoStatus::Conflict,
                    "未被 .gitignore 忽略；先加入忽略规则再创建链接",
                ));
            } else {
                items.push(item(
                    &spec.link,
                    RepoStatus::Missing,
                    format!("未创建 -> {}", spec.target),
                ));
            }
        }
        Err(error) => items.push(item(
            &spec.link,
            RepoStatus::Conflict,
            format!("无法读取: {error}"),
        )),
    }
}

fn init_link(root: &Path, spec: &LinkSpec, in_git: bool, items: &mut Vec<RepoItem>) {
    let target = root.join(&spec.target);
    if !target.exists() {
        items.push(item(
            &spec.link,
            RepoStatus::Conflict,
            format!("链接目标不存在: {}", spec.target),
        ));
        return;
    }

    let link = root.join(&spec.link);
    match std::fs::symlink_metadata(&link) {
        Ok(_) => match (link.canonicalize(), target.canonicalize()) {
            (Ok(actual), Ok(expected)) if actual == expected => items.push(item(
                &spec.link,
                RepoStatus::Ok,
                format!("-> {}", spec.target),
            )),
            _ => items.push(item(
                &spec.link,
                RepoStatus::Conflict,
                format!("已存在且未指向 {}，不会覆盖", spec.target),
            )),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if in_git && !is_ignored(root, &spec.link) {
                items.push(item(
                    &spec.link,
                    RepoStatus::Conflict,
                    "未被 .gitignore 忽略；先加入忽略规则再创建链接",
                ));
                return;
            }
            if in_git && is_tracked(root, &spec.link) {
                items.push(item(
                    &spec.link,
                    RepoStatus::Conflict,
                    "路径已被 git 跟踪，不会覆盖",
                ));
                return;
            }
            if let Some(parent) = link.parent() {
                if let Err(error) = std::fs::create_dir_all(parent) {
                    items.push(item(
                        &spec.link,
                        RepoStatus::Conflict,
                        format!("无法创建父目录 {}: {error}", parent.display()),
                    ));
                    return;
                }
            }
            match create_dir_link(&link, &target) {
                Ok(()) => items.push(item(
                    &spec.link,
                    RepoStatus::Created,
                    format!("-> {}", spec.target),
                )),
                Err(error) => items.push(item(
                    &spec.link,
                    RepoStatus::Conflict,
                    format!("创建链接失败: {error}"),
                )),
            }
        }
        Err(error) => items.push(item(
            &spec.link,
            RepoStatus::Conflict,
            format!("无法读取: {error}"),
        )),
    }
}

#[cfg(unix)]
fn create_dir_link(link: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_dir_link(link: &Path, target: &Path) -> std::io::Result<()> {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => Ok(()),
        Err(_) => {
            // 没有创建符号链接的权限时回落到目录联接（Junction）。
            let status = Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .status()?;
            if status.success() {
                Ok(())
            } else {
                Err(std::io::Error::other("mklink /J 失败"))
            }
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn create_dir_link(_link: &Path, _target: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "当前平台不支持目录链接",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_root(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root =
            std::env::temp_dir().join(format!("pengj-repo-{}-{}-{}", name, std::process::id(), id));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("创建临时目录");
        root
    }

    fn git_init(root: &Path) {
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .expect("执行 git init");
        assert!(status.success(), "git init 失败");
    }

    #[test]
    fn default_links_require_agents_directory() {
        let root = temp_root("defaults");
        assert!(default_links(&root).is_empty());
        fs::create_dir_all(root.join(".agents/skills")).unwrap();
        let links = default_links(&root);
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].link, ".trae");
        assert_eq!(links[1].link, ".claude/skills");
        assert_eq!(links[2].link, ".workbuddy/skills");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn init_creates_links_and_is_idempotent() {
        let root = temp_root("init");
        fs::create_dir_all(root.join(".agents/skills")).unwrap();
        fs::write(
            root.join(".gitignore"),
            "/.trae\n/.claude/skills\n/.workbuddy\n",
        )
        .unwrap();
        git_init(&root);

        let links = default_links(&root);
        let report = repo_init(&root, &links, false).unwrap();
        assert!(report.ok, "{report:?}");
        assert!(root.join(".trae").exists());
        assert!(root.join(".claude/skills").exists());
        // `.workbuddy` 事先不存在：父目录要由 init 自己补出来。
        assert!(root.join(".workbuddy/skills").exists());

        let again = repo_init(&root, &links, false).unwrap();
        assert!(again.ok, "{again:?}");
        assert!(again
            .items
            .iter()
            .filter(|item| item.name.starts_with('.'))
            .all(|item| item.status == RepoStatus::Ok));
        assert!(repo_status(&root, &links).unwrap().ok);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn init_never_overwrites_a_real_directory() {
        let root = temp_root("conflict");
        fs::create_dir_all(root.join(".agents/skills")).unwrap();
        fs::create_dir_all(root.join(".trae")).unwrap();
        fs::write(root.join(".trae/keep.txt"), "keep").unwrap();
        fs::write(
            root.join(".gitignore"),
            "/.trae\n/.claude/skills\n/.workbuddy\n",
        )
        .unwrap();
        git_init(&root);

        let links = default_links(&root);
        let report = repo_init(&root, &links, false).unwrap();
        assert!(!report.ok);
        assert!(root.join(".trae/keep.txt").exists());
        assert!(report
            .items
            .iter()
            .any(|item| item.name == ".trae" && item.status == RepoStatus::Conflict));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn init_keeps_client_state_when_linking_into_it() {
        let root = temp_root("client-state");
        fs::create_dir_all(root.join(".agents/skills")).unwrap();
        // 客户端自己已经在 `.workbuddy` 里写了状态，链接只能补在里面。
        fs::create_dir_all(root.join(".workbuddy/memory")).unwrap();
        fs::write(root.join(".workbuddy/memory/MEMORY.md"), "keep").unwrap();
        fs::write(
            root.join(".gitignore"),
            "/.trae\n/.claude/skills\n/.workbuddy\n",
        )
        .unwrap();
        git_init(&root);

        let report = repo_init(&root, &default_links(&root), false).unwrap();
        assert!(report.ok, "{report:?}");
        assert!(root.join(".workbuddy/skills").exists());
        assert!(root.join(".workbuddy/memory/MEMORY.md").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn init_refuses_unignored_links() {
        let root = temp_root("unignored");
        fs::create_dir_all(root.join(".agents/skills")).unwrap();
        fs::write(root.join(".gitignore"), "").unwrap();
        git_init(&root);

        let report = repo_init(&root, &default_links(&root), false).unwrap();
        assert!(!report.ok);
        assert!(!root.join(".trae").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn parses_link_spec() {
        let spec = LinkSpec::parse(".codex/skills=.agents/skills").unwrap();
        assert_eq!(spec.link, ".codex/skills");
        assert_eq!(spec.target, ".agents/skills");
        assert!(LinkSpec::parse("missing-separator").is_err());
    }
}
