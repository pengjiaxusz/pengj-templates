//! 工作树创建：分支 + 目录一次做完，并把子模块补齐
//!
//! 现状痛点：新开一个工作树后，子模块不会自动补齐——要手工跑一遍初始化命令
//! 才能编译。工作树层负责工作树的日常操作，子仓库的初始化由**既有的仓库初始化
//! 入口**承接：两者各自调用它，不互相依赖。
//!
//! 不变式：
//! - 目录与分支按既有约定命名（主仓同级、前缀一致）；
//! - 目标工作树已存在时**明确报错**，不静默跳过也不破坏现有目录；
//! - 子模块只初始化未初始化的（复用仓库初始化入口，不另写一套）。

use std::path::{Path, PathBuf};

use crate::error::{CoreError, Result};

/// 新建工作树的报告
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeReport {
    /// 工作树目录（绝对路径）
    pub path: PathBuf,
    /// 分支名
    pub branch: String,
    /// 共享编译目录（多工作树共享同一份）
    pub shared_target: PathBuf,
    /// 子模块补齐的结果（条目名 -> 是否就绪）
    pub submodules: Vec<(String, bool)>,
}

/// 工作树命名约定：主仓同级、前缀一致。
///
/// 主仓目录名 `foo`、工作树名 `bar` → 目录 `foo-bar`、分支 `bar`。
/// 前缀与主仓一致，多个工作树并排放在主仓旁边，一眼能看出它们属于同一个仓库。
pub fn worktree_dir_name(main_repo_name: &str, worktree_name: &str) -> String {
    format!("{main_repo_name}-{worktree_name}")
}

/// 在 `main_repo` 旁边创建工作树 `name`。
///
/// `name` 同时作为分支名与目录后缀。目标目录已存在时报错——静默跳过会让人以为
/// 新工作树建好了，实际上拿到的还是旧的那份。
pub fn create_worktree(main_repo: &Path, name: &str) -> Result<WorktreeReport> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::Git("工作树名称不能为空".to_string()));
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CoreError::Git(format!(
            "工作树名称不能包含路径分隔符或 `..`: {name}"
        )));
    }

    let main_repo = main_repo.canonicalize().map_err(|error| {
        CoreError::Git(format!("无法访问主仓 {}: {error}", main_repo.display()))
    })?;
    // Windows 的 canonicalize 会带回 `\\?\` 逐字前缀，git 拿它建目录会失败，
    // 所以对外暴露与传给 git 的路径都统一剥掉前缀。
    let main_repo = strip_verbatim(&main_repo);
    // 从链接工作树里新建时，落点仍应是**主工作树**旁边，用主仓目录名做前缀；
    // 否则从 `my-app-a` 新建 `b` 会得到 `my-app-a-b`，名字会随层级越滚越长。
    let main_repo = main_worktree(&main_repo).unwrap_or(main_repo);

    let main_name = main_repo
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string());
    let dir_name = worktree_dir_name(&main_name, name);
    let target = main_repo
        .parent()
        .ok_or_else(|| CoreError::Git("无法确定主仓的父目录".to_string()))?
        .join(&dir_name);

    // 幂等性要求：已存在就明确报错，不静默跳过也不破坏现有目录
    if target.exists() {
        return Err(CoreError::Git(format!(
            "工作树目录已存在: {}（请换个名字，或先手动清理它）",
            target.display()
        )));
    }

    // 分支已存在时也报错：复用已有分支会把工作树接到不明状态上
    let branch_exists = git(
        &main_repo,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{name}"),
        ],
    )
    .map(|output| output.status.success())
    .unwrap_or(false);
    if branch_exists {
        return Err(CoreError::Git(format!(
            "分支已存在: {name}（请换个名字，或先手动删除该分支）"
        )));
    }

    git_stdout(
        &main_repo,
        &["worktree", "add", "-b", name, &target.to_string_lossy()],
    )?;

    // 共享编译目录：新建工作树后接线到位，无需手工配置
    let shared = crate::worktree::ensure_shared_target(&target)?;

    // 子模块补齐：复用既有的仓库初始化入口，不另写一套子模块初始化逻辑
    let submodules = init_submodules(&target);

    Ok(WorktreeReport {
        path: target,
        branch: name.to_string(),
        shared_target: shared.0.path,
        submodules,
    })
}

/// 补齐工作树里的子模块（只初始化未初始化的）
fn init_submodules(worktree: &Path) -> Vec<(String, bool)> {
    // 仓库初始化入口负责子模块补缺；它同时会做别的事（工作区链接、git 配置），
    // 在新建的工作树里这些同样是需要的，所以直接复用整个入口。
    let links = crate::repo::default_links(worktree);
    match crate::repo::repo_init(worktree, &links, true) {
        Ok(report) => report
            .items
            .into_iter()
            .filter(|item| item.name.starts_with("submodule"))
            .map(|item| {
                let ready = !matches!(
                    item.status,
                    crate::repo::RepoStatus::Missing | crate::repo::RepoStatus::Conflict
                );
                (item.name, ready)
            })
            .collect(),
        // 补齐失败不撤销已建好的工作树：目录与分支已经就位，
        // 子模块可以事后单独再试一次。
        Err(_) => Vec::new(),
    }
}

/// 找到主工作树的根：`git worktree list --porcelain` 的第一条就是它。
///
/// 从任意一个链接工作树里新建，落点都应是主工作树旁边。取不到时返回 `None`，
/// 调用方退回「按给定目录当主仓」——这是单工作树仓库的常态。
fn main_worktree(dir: &Path) -> Option<PathBuf> {
    let output = git(dir, &["worktree", "list", "--porcelain"]).ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let path = text
        .lines()
        .find_map(|line| line.strip_prefix("worktree "))
        .map(str::trim)
        .filter(|p| !p.is_empty())?;
    // git 的 porcelain 输出用正斜杠；不归一的话 Windows 上拼出来的路径会一半
    // 正斜杠一半反斜杠，虽然能用，但打印出来像 bug。
    #[cfg(windows)]
    let path = path.replace('/', "\\");
    #[cfg(not(windows))]
    let path = path.to_string();
    let resolved = PathBuf::from(path);
    Some(strip_verbatim(&resolved))
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(strip_verbatim(root))
        .args(args)
        .output()
        .map_err(|error| CoreError::Git(format!("无法执行 git: {error}")))
}

/// 剥掉 Windows 的 `\\?\` 逐字路径前缀。
///
/// `canonicalize()` 会给绝对路径加上这个前缀，git 遇到它会在建目录时失败
/// （`could not create leading directories of '//?/C:/...'`），所以凡是交给
/// git 的路径都要先剥掉。
fn strip_verbatim(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path.to_path_buf()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn git_init(dir: &Path) {
        std::fs::create_dir_all(dir).unwrap();
        let ok = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(args)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} 失败");
        };
        ok(&["init", "-q"]);
        ok(&["config", "user.email", "test@example.com"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(dir.join("a.txt"), b"hi").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-qm", "init"]);
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-worktree-{}-{}-{}",
            tag,
            std::process::id(),
            line!()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn 目录命名遵循主仓同级前缀一致的约定() {
        assert_eq!(worktree_dir_name("my-app", "feature-a"), "my-app-feature-a");
        assert_eq!(worktree_dir_name("repo", "x"), "repo-x");
    }

    #[test]
    fn 创建成功时报出路径与分支() {
        let base = temp_dir("create");
        let main = base.join("my-app");
        git_init(&main);

        let report = create_worktree(&main, "feature-a").expect("应能创建工作树");
        assert_eq!(report.branch, "feature-a");
        assert!(
            report.path.is_dir(),
            "工作树目录应存在: {}",
            report.path.display()
        );
        // Windows 上临时目录可能是 8.3 短名（DEVELO~1），canonicalize 会展开成
        // 长名，所以比较父目录前先各自规范化。
        let worktree_parent = report.path.parent().unwrap().canonicalize().unwrap();
        let main_parent = main.parent().unwrap().canonicalize().unwrap();
        assert_eq!(worktree_parent, main_parent, "工作树应与主仓同级");
        assert!(
            report
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("my-app-"),
            "工作树目录名应带主仓前缀: {}",
            report.path.display()
        );
        // 共享编译目录接线到位
        assert!(
            !report.shared_target.to_string_lossy().is_empty(),
            "共享编译目录应已推导"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn 重复创建时明确报错而不是静默跳过() {
        let base = temp_dir("dup");
        let main = base.join("my-app");
        git_init(&main);

        create_worktree(&main, "feature-a").expect("首次应成功");
        let error = create_worktree(&main, "feature-a").expect_err("重复创建必须报错");
        assert!(
            error.to_string().contains("已存在"),
            "报错应点明是已存在: {error}"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn 分支已存在时报错() {
        let base = temp_dir("branch");
        let main = base.join("my-app");
        git_init(&main);
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(&main)
            .args(["branch", "taken"])
            .status()
            .unwrap();
        assert!(status.success());

        let error = create_worktree(&main, "taken").expect_err("分支已存在必须报错");
        assert!(
            error.to_string().contains("分支已存在"),
            "报错应点明是分支已存在: {error}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn 非法名称被拒绝() {
        let base = temp_dir("bad");
        let main = base.join("my-app");
        git_init(&main);

        assert!(create_worktree(&main, "").is_err(), "空名必须报错");
        assert!(
            create_worktree(&main, "../escape").is_err(),
            "含路径分隔符的名字必须报错"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn 子模块补齐复用仓库初始化入口() {
        // 没有 .gitmodules 时不报错，条目为空即可
        let base = temp_dir("sub");
        let main = base.join("my-app");
        git_init(&main);
        let report = create_worktree(&main, "wt").expect("应能创建工作树");
        // 无子模块时不应报出未就绪的子模块
        assert!(
            report.submodules.iter().all(|(_, ready)| *ready),
            "无子模块时不该报未就绪: {:?}",
            report.submodules
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn 从链接工作树新建时仍落在主仓旁边() {
        let base = temp_dir("from-linked");
        let main = base.join("my-app");
        git_init(&main);

        let first = create_worktree(&main, "a").expect("首次应成功");
        let second = create_worktree(&first.path, "b").expect("从链接工作树也应能新建");

        assert_eq!(second.branch, "b");
        assert_eq!(
            second.path.file_name().unwrap().to_string_lossy(),
            "my-app-b",
            "从链接工作树新建时不得把工作树名再叠一层: {}",
            second.path.display()
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
