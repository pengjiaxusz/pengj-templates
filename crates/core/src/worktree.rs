//! 共享编译目录：所有工作树都编译到**主工作树的 `target/`**
//!
//! 现状痛点：每个工作树各持一份编译产物目录，依赖被重复编译、磁盘被数十 GB 的
//! 重复产物撑爆。手写方案的根本问题是**状态无法被任何自动机制校验**——链接没建、
//! 被误删或悄悄退化时，没有任何环节会报警。
//!
//! 落点选主工作树的 `target/`，而不是平台数据目录下的独立目录：
//!
//! - **产物跟着项目走**：项目在哪块盘、搬到哪里，产物都在同一棵树里，按项目就能
//!   看清与清理；不往 C 盘用户目录里堆几十 GB。
//! - **主工作树零配置**：它的默认 `target/` 就是共享目录，裸跑 `cargo build`
//!   也落在同一处，不需要配方也能命中复用。
//! - **链接工作树**通过构建配方注入的 `CARGO_TARGET_DIR` 指过去。
//!
//! 代价（必须写进文档，不能靠使用者记忆）：在主工作树里 `cargo clean` 或
//! `git clean -xdf` 会清掉**所有**工作树的中间产物。这是「产物落在项目内」的
//! 必然取舍，换来的是零新增目录、`.gitignore` 天然覆盖、同盘移动不拆散产物。
//!
//! **推导逻辑只有一份**（本模块），构建配方与自检读同一个值，不得各写一套。
//! 这也是「不在 `.cargo/config.toml` 写 `target-dir`」的原因：那个文件走模板的
//! 结构化合并，模板受管块与用户自定义区若出现同名表头会判冲突、整文件不予写盘，
//! 用户的构建配置会静默停在旧值。
//!
//! 主工作树的判定用 `git worktree list --porcelain` 的第一条：同一仓库的所有工作树
//! 都会得到它，因此跨工作树稳定；不同仓库各有自己的主工作树，天然按仓库分桶。
//! 不在 git 仓库里时退回自身 `target/`——单目录项目没有「别的工作树」可分。

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{CoreError, Result};

/// 编译产物目录名（主工作树里就是 Cargo 的默认位置）
pub const TARGET_DIR_NAME: &str = "target";

/// 共享编译目录的推导结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedTarget {
    /// 共享目录的绝对路径（主工作树的 `target/`）
    pub path: PathBuf,
    /// 主工作树的根：共享目录的持有者，也是清理操作的作用域
    pub main_worktree: PathBuf,
}

/// 推导共享编译目录：**主工作树的 `target/`**。
///
/// 从任意一个链接工作树进来都得到同一个结果——这是复用的前提。不落在平台数据
/// 目录，也不按哈希分桶：主工作树本身就是桶。
pub fn shared_target_dir(project_dir: &Path) -> SharedTarget {
    let main_worktree = main_worktree(project_dir).unwrap_or_else(|| project_dir.to_path_buf());
    SharedTarget {
        path: main_worktree.join(TARGET_DIR_NAME),
        main_worktree,
    }
}

/// 共享编译目录的健康状况
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SharedTargetState {
    /// 目录存在且可写
    Ready,
    /// 目录不存在（修复模式下可创建）
    Missing,
    /// 目录存在但不可写
    NotWritable,
}

impl SharedTargetState {
    /// 稳定的字面量（命令输出与 JSON 用）
    pub fn as_str(self) -> &'static str {
        match self {
            SharedTargetState::Ready => "ready",
            SharedTargetState::Missing => "missing",
            SharedTargetState::NotWritable => "not-writable",
        }
    }

    /// 面向人的说明（命令输出用）
    pub fn describe(self) -> &'static str {
        match self {
            SharedTargetState::Ready => "就绪（存在且可写）",
            SharedTargetState::Missing => "不存在（可用修复模式创建）",
            SharedTargetState::NotWritable => "存在但不可写",
        }
    }
}

/// 校验共享编译目录：存在、可写。
pub fn check_shared_target(project_dir: &Path) -> (SharedTarget, SharedTargetState) {
    let target = shared_target_dir(project_dir);
    let state = if !target.path.exists() {
        SharedTargetState::Missing
    } else if !is_writable_dir(&target.path) {
        SharedTargetState::NotWritable
    } else {
        SharedTargetState::Ready
    };
    (target, state)
}

/// 创建缺失的共享编译目录。
///
/// 只补缺、不覆盖：目录已存在时**不动它**（哪怕它不是空的），符合仓库初始化入口
/// 既有的「只补缺、不覆盖」不变式。
pub fn ensure_shared_target(project_dir: &Path) -> Result<(SharedTarget, SharedTargetState)> {
    let (target, state) = check_shared_target(project_dir);
    if state != SharedTargetState::Missing {
        return Ok((target, state));
    }
    std::fs::create_dir_all(&target.path).map_err(|error| {
        CoreError::Git(format!(
            "无法创建共享编译目录 {}: {error}",
            target.path.display()
        ))
    })?;
    Ok((target, SharedTargetState::Ready))
}

/// 环境里是否存在指向别处的 `CARGO_TARGET_DIR`。
///
/// 「共享目录当前指向正确」不只要求目录存在：用户可能在 shell 里全局设了一个
/// `CARGO_TARGET_DIR`，那样**裸跑 cargo** 的工作树会落到另一个目录，配方再正确也
/// 只是把它盖回去。返回覆盖值（与本机推导值不一致时）供调用方提示。
pub fn conflicting_override(target: &SharedTarget) -> Option<String> {
    let value = std::env::var_os("CARGO_TARGET_DIR")?;
    let text = value.to_string_lossy().to_string();
    if text.trim().is_empty() {
        return None;
    }
    let given = PathBuf::from(&text);
    if paths_equal(&given, &target.path) {
        None
    } else {
        Some(text)
    }
}

/// 两条路径是否指向同一处。
///
/// **两侧一起**规范化：只规范化其中一侧会制造假警报——Windows 的
/// `canonicalize` 返回带 `\\?\` 前缀的字面量，与未规范化的推导值永不相等，
/// 于是「用户把环境变量指向本机推导的同一个目录」会被判成指向别处。
/// 规范化失败（目录还不存在）时退回按分隔符归一的字面量比较。
fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => same_path(a, b),
    }
}

/// 两条路径是否指向同一处（规范化失败时的退路：按分隔符归一后比较）
fn same_path(a: &Path, b: &Path) -> bool {
    let left = a.to_string_lossy().replace('\\', "/");
    let right = b.to_string_lossy().replace('\\', "/");
    left.trim_end_matches('/') == right.trim_end_matches('/')
}

/// 找到主工作树的根。
///
/// 从任意一个链接工作树进来都得到同一个值，这正是「所有工作树共享一份产物」的
/// 前提。取不到时返回 `None`，调用方退回给定目录本身——单目录项目或非 git 目录
/// 的常态，此时它的 `target/` 就是唯一的那份。
///
/// 判定分两种情况，不能只信 `git worktree list`：
///
/// - **自己是主工作树**（`git-dir` == `git-common-dir`）：直接取 `--show-toplevel`。
///   子模块正是这种形态（`.git` 是指向 `../.git/modules/...` 的文件），此时
///   `worktree list` 会把 **gitdir** 当成主工作树报出来——拿它当产物目录会把
///   编译产物写进 `.git/modules/` 里，这在 unbound-ui 上是实测到的真实失败。
/// - **自己是链接工作树**：`worktree list --porcelain` 的第一条就是主工作树。
pub fn main_worktree(dir: &Path) -> Option<PathBuf> {
    let git_dirs = git_stdout(
        dir,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-dir",
            "--git-common-dir",
        ],
    )?;
    let mut lines = git_dirs.lines().map(str::trim).filter(|l| !l.is_empty());
    let git_dir = PathBuf::from(normalize_git_path(lines.next()?));
    let common_dir = PathBuf::from(normalize_git_path(lines.next()?));

    if paths_equal(&git_dir, &common_dir) {
        return toplevel(dir);
    }
    first_worktree_entry(dir).or_else(|| toplevel(dir))
}

/// `git rev-parse --show-toplevel`（当前检出目录的根）
fn toplevel(dir: &Path) -> Option<PathBuf> {
    let text = git_stdout(dir, &["rev-parse", "--show-toplevel"])?;
    let path = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    Some(strip_verbatim(Path::new(&normalize_git_path(path))))
}

/// `git worktree list --porcelain` 的第一条（主工作树）
fn first_worktree_entry(dir: &Path) -> Option<PathBuf> {
    let text = git_stdout(dir, &["worktree", "list", "--porcelain"])?;
    let path = text
        .lines()
        .find_map(|line| line.strip_prefix("worktree "))
        .map(str::trim)
        .filter(|p| !p.is_empty())?;
    Some(strip_verbatim(Path::new(&normalize_git_path(path))))
}

/// 执行 git 并取标准输出；失败或为空时返回 `None`（调用方退回保守结果）
fn git_stdout(dir: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// git 的输出一律用正斜杠；不归一的话 Windows 上拼出来的路径会一半正斜杠一半
/// 反斜杠，虽然能用，但打印出来像 bug。
fn normalize_git_path(path: &str) -> String {
    #[cfg(windows)]
    {
        path.replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path.to_string()
    }
}

/// 剥掉 Windows 的 `\\?\` 逐字路径前缀。
///
/// `canonicalize()` 会给绝对路径加上这个前缀，git 遇到它会在建目录时失败
/// （`could not create leading directories of '//?/C:/...'`），所以凡是交给
/// git 的路径都要先剥掉。
pub(crate) fn strip_verbatim(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path.to_path_buf()
}

/// 目录是否可写
fn is_writable_dir(path: &Path) -> bool {
    // 目录不存在时不算不可写（那是「缺失」而非「不可写」）
    if !path.is_dir() {
        return false;
    }
    let probe = path.join(".pengj-write-probe");
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-shared-target-{}-{}-{}",
            tag,
            std::process::id(),
            line!()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn git_ok(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} 失败");
    }

    /// 非 git 目录没有「别的工作树」：共享目录就是它自己的 `target/`
    #[test]
    fn 非仓库目录的共享目录就是自身target() {
        let dir = temp_dir("plain");
        let target = shared_target_dir(&dir);
        assert_eq!(target.path, dir.join(TARGET_DIR_NAME));
        assert_eq!(target.main_worktree, dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 复用的前提：同一仓库的链接工作树必须推出**同一个**共享目录
    #[test]
    fn 链接工作树与主工作树推出同一个共享目录() {
        let root = temp_dir("wt-main");
        git_ok(&root, &["init", "-q"]);
        git_ok(&root, &["config", "user.email", "test@example.com"]);
        git_ok(&root, &["config", "user.name", "test"]);
        std::fs::write(root.join("README.md"), b"x").unwrap();
        git_ok(&root, &["add", "."]);
        git_ok(&root, &["commit", "-q", "-m", "init"]);

        let linked = root.parent().unwrap().join(format!(
            "pengj-shared-target-wt-linked-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&linked);
        git_ok(
            &root,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "probe-wt",
                &linked.to_string_lossy(),
            ],
        );

        let main = shared_target_dir(&root);
        let from_linked = shared_target_dir(&linked);
        // Windows 上临时目录可能是短路径（`DEVELO~1`），字面量比较会误判；
        // 建出目录后两侧都能规范化，比较才是「是否同一处」的真语义。
        std::fs::create_dir_all(&main.path).unwrap();
        assert!(
            paths_equal(&main.path, &from_linked.path),
            "两个工作树必须指向同一个共享目录: {} vs {}",
            main.path.display(),
            from_linked.path.display()
        );
        assert!(
            paths_equal(&main.path, &root.join(TARGET_DIR_NAME)),
            "共享目录应是主工作树的 target: {}",
            main.path.display()
        );

        git_ok(
            &root,
            &["worktree", "remove", "--force", &linked.to_string_lossy()],
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 子模块/独立 git 目录形态：`.git` 是指向外部 gitdir 的文件，
    /// `worktree list` 会把 **gitdir** 当成主工作树报出来。产物绝不能落到
    /// `.git/modules/...` 里——共享目录必须是检出目录自己的 `target/`。
    #[test]
    fn gitdir文件形态取检出目录而不是git目录() {
        let base = temp_dir("gitdir");
        let work = base.join("work");
        let gitdir = base.join("modules").join("sub");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&gitdir).unwrap();
        git_ok(
            &base,
            &[
                "init",
                "-q",
                "--separate-git-dir",
                &gitdir.to_string_lossy(),
                &work.to_string_lossy(),
            ],
        );
        // 先确认这条 git 行为确实是我们要防的那个坑，否则测试失去意义
        let listed = first_worktree_entry(&work).expect("应能取到 worktree list 首条");
        assert!(
            paths_equal(&listed, &gitdir),
            "前提不成立：git 已不再把 gitdir 当主工作树报出来: {}",
            listed.display()
        );

        let target = shared_target_dir(&work);
        // 建出目录后两侧都能规范化（临时目录在 Windows 上可能是短路径）
        std::fs::create_dir_all(&target.path).unwrap();
        assert!(
            paths_equal(&target.path, &work.join(TARGET_DIR_NAME)),
            "共享目录应落在检出目录下: {}",
            target.path.display()
        );
        assert!(
            !target.path.starts_with(&gitdir),
            "共享目录不得落进 git 目录: {}",
            target.path.display()
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn 缺失的共享目录被报告为缺失且修复后变为就绪() {
        let dir = temp_dir("missing");
        let target = shared_target_dir(&dir);
        let _ = std::fs::remove_dir_all(&target.path);
        let (_, state) = check_shared_target(&dir);
        assert_eq!(state, SharedTargetState::Missing, "尚未创建时应当是缺失");

        let (target, state) = ensure_shared_target(&dir).expect("应能创建");
        assert_eq!(state, SharedTargetState::Ready, "创建后应当就绪");
        assert!(
            target.path.is_dir(),
            "共享目录应已建好: {}",
            target.path.display()
        );

        // 只补缺、不覆盖：再跑一次不报错，目录仍在
        let (again, state) = ensure_shared_target(&dir).expect("重复执行应无副作用");
        assert_eq!(state, SharedTargetState::Ready);
        assert_eq!(again.path, target.path);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 就绪的共享目录不会被覆盖() {
        let dir = temp_dir("ready");
        let (target, _) = ensure_shared_target(&dir).expect("应能创建");
        // 在共享目录里放一个标记文件，模拟用户已有产物
        let marker = target.path.join("keep-me.txt");
        std::fs::write(&marker, b"mine").unwrap();

        let (_, state) = ensure_shared_target(&dir).expect("重复执行应无副作用");
        assert_eq!(state, SharedTargetState::Ready);
        assert!(marker.is_file(), "已有内容不得被清理: {}", marker.display());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 状态字面量与说明一一对应() {
        for state in [
            SharedTargetState::Ready,
            SharedTargetState::Missing,
            SharedTargetState::NotWritable,
        ] {
            assert!(!state.as_str().is_empty());
            assert!(!state.describe().is_empty());
        }
        assert_eq!(SharedTargetState::NotWritable.as_str(), "not-writable");
    }

    #[test]
    fn 路径比较容忍分隔符差异与尾斜杠() {
        let left = PathBuf::from("C:\\work\\shared-target");
        let right = PathBuf::from("C:/work/shared-target/");
        // 不存在的路径无法 canonicalize，走退路比较
        assert!(same_path(&left, &right));
        assert!(!same_path(&left, &PathBuf::from("C:/work/other")));
    }

    #[test]
    fn 同一位置的规范化字面量判为相同() {
        let dir = temp_dir("equal");
        // Windows 上 canonicalize 会加 `\\?\` 前缀；两侧只规范化一侧就会误判「指向别处」
        let canonical = dir.canonicalize().expect("临时目录应可规范化");
        assert!(
            paths_equal(&dir, &canonical),
            "同一目录的两种字面量必须判等: {} vs {}",
            dir.display(),
            canonical.display()
        );
        // 反方向同样要判等：真实调用里「环境变量值」与「推导值」谁带前缀并不固定
        assert!(
            paths_equal(&canonical, &dir),
            "同一目录的两种字面量必须判等（反向）: {} vs {}",
            canonical.display(),
            dir.display()
        );
        assert!(paths_equal(&dir, &dir));

        let other = temp_dir("equal-other");
        assert!(
            !paths_equal(&dir, &other),
            "不同目录不得判等: {} vs {}",
            dir.display(),
            other.display()
        );
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&other);
    }
}
