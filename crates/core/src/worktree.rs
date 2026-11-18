//! 共享编译目录：多工作树共享同一份编译产物
//!
//! 现状痛点：每个工作树各持一份编译产物目录，依赖被重复编译、磁盘被数十 GB 的
//! 重复产物撑爆。手写方案的根本问题是**状态无法被任何自动机制校验**——链接没建、
//! 被误删或悄悄退化时，没有任何环节会报警。
//!
//! 机制选择：由构建配方统一注入 `CARGO_TARGET_DIR`，指向**仓库外**的共享目录。
//! 目录位置按机器动态推导（平台数据目录 + 项目标识），不是模板里的静态路径。
//!
//! **推导逻辑只有一份**（本模块），构建配方与自检读同一个值，不得各写一套。
//! 这也是「不在 `.cargo/config.toml` 写 `target-dir`」的原因：那个文件走模板的
//! 结构化合并，模板受管块与用户自定义区若出现同名表头会判冲突、整文件不予写盘，
//! 用户的构建配置会静默停在旧值。
//!
//! 跨工作树稳定的项目标识：同一仓库的所有工作树共享同一个 git `common-dir`，
//! 用它的哈希做标识 —— 不同仓库不会撞，同仓库不同工作树必然一致。

use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{CoreError, Result};

/// 共享编译目录在平台数据目录下的子目录名
pub const SHARED_TARGET_DIR_NAME: &str = "shared-target";

/// 共享编译目录的推导结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedTarget {
    /// 共享目录的绝对路径
    pub path: PathBuf,
    /// 项目标识（用于人工辨认；不含盘符、用户名等机器相关字面量）
    pub project_key: String,
}

/// 推导项目标识。
///
/// 优先用 git 的 `common-dir`：同一仓库的多个工作树共享它，因此标识天然跨工作树稳定；
/// 不同仓库各用自己的 `common-dir`，不会互相覆盖产物。
///
/// 不在 git 工作树里时退回到目录名 + 路径哈希：仍然稳定且不泄漏机器信息。
pub fn project_key(project_dir: &Path) -> String {
    if let Some(common) = git_common_dir(project_dir) {
        return format!("repo-{}", short_hash(&common.to_string_lossy()));
    }
    let name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    let slug = crate::context::slugify(&name);
    format!("{}-{}", slug, short_hash(&project_dir.to_string_lossy()))
}

/// 推导共享编译目录。
///
/// 位置 = 平台数据目录 / `shared-target` / 项目标识。
/// 平台数据目录取不到时返回错误——**不静默退回项目内的 `target/`**：
/// 悄悄退回独立目录正是这套机制要消灭的失败模式。
pub fn shared_target_dir(project_dir: &Path) -> Result<SharedTarget> {
    let base = crate::engine::platform_data_dir().ok_or_else(|| {
        CoreError::Git(
            "无法确定平台数据目录，无法推导共享编译目录（Windows 用 LOCALAPPDATA，\
             Unix 用 XDG_DATA_HOME 或 HOME）"
                .to_string(),
        )
    })?;
    let project_key = project_key(project_dir);
    let path = base
        .join(SHARED_TARGET_DIR_NAME)
        .join(&project_key)
        .join(target_triple());
    Ok(SharedTarget { path, project_key })
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
pub fn check_shared_target(project_dir: &Path) -> Result<(SharedTarget, SharedTargetState)> {
    let target = shared_target_dir(project_dir)?;
    let state = if !target.path.exists() {
        SharedTargetState::Missing
    } else if !is_writable_dir(&target.path) {
        SharedTargetState::NotWritable
    } else {
        SharedTargetState::Ready
    };
    Ok((target, state))
}

/// 创建缺失的共享编译目录。
///
/// 只补缺、不覆盖：目录已存在时**不动它**（哪怕它不是空的），符合仓库初始化入口
/// 既有的「只补缺、不覆盖」不变式。
pub fn ensure_shared_target(project_dir: &Path) -> Result<(SharedTarget, SharedTargetState)> {
    let (target, state) = check_shared_target(project_dir)?;
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
    let same = given
        .canonicalize()
        .map(|resolved| resolved == target.path)
        .unwrap_or_else(|_| same_path(&given, &target.path));
    if same {
        None
    } else {
        Some(text)
    }
}

/// 两条路径是否指向同一处（规范化失败时的退路：按分隔符归一后比较）
fn same_path(a: &Path, b: &Path) -> bool {
    let left = a.to_string_lossy().replace('\\', "/");
    let right = b.to_string_lossy().replace('\\', "/");
    left.trim_end_matches('/') == right.trim_end_matches('/')
}

/// 目标三元组（架构 + 供应商 + 操作系统）。
///
/// 共享目录按三元组分桶：不同架构的产物本来就不能共用，放一起会互相覆盖。
fn target_triple() -> String {
    format!(
        "{}-{}-{}",
        std::env::consts::ARCH,
        std::env::consts::FAMILY,
        std::env::consts::OS
    )
}

/// 取 git 的 `common-dir`（同一仓库的所有工作树共享它）
fn git_common_dir(project_dir: &Path) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(PathBuf::from(text))
}

/// 短哈希（8 位十六进制）
fn short_hash(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().take(4).map(|b| format!("{b:02x}")).collect()
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

    #[test]
    fn 推导结果落在平台数据目录下且不含机器字面量() {
        let dir = temp_dir("derive");
        let target = shared_target_dir(&dir).expect("应能推导共享目录");
        let text = target.path.to_string_lossy().to_string();

        // 位置在平台数据目录之下
        let base = crate::engine::platform_data_dir().expect("应能取到平台数据目录");
        assert!(
            target.path.starts_with(&base),
            "共享目录应在平台数据目录之下: {text}"
        );
        assert!(
            target
                .path
                .to_string_lossy()
                .contains(SHARED_TARGET_DIR_NAME),
            "共享目录应带约定的子目录名: {text}"
        );

        // 标识里不含盘符、用户名等机器相关字面量
        assert!(
            !target.project_key.contains('\\') && !target.project_key.contains('/'),
            "项目标识不应含路径分隔符: {}",
            target.project_key
        );
        assert!(
            !target.project_key.contains(':'),
            "项目标识不应含盘符: {}",
            target.project_key
        );
        assert!(
            target
                .project_key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "项目标识只应是 ASCII 字母数字与连字符: {}",
            target.project_key
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 非工作树项目的标识来自目录名且稳定() {
        let dir = temp_dir("plain");
        let first = project_key(&dir);
        let second = project_key(&dir);
        assert_eq!(first, second, "同一目录的标识必须稳定");
        assert!(first.contains("plain-"), "标识应带目录名: {first}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 缺失的共享目录被报告为缺失且修复后变为就绪() {
        let dir = temp_dir("missing");
        // 共享目录落在平台数据目录里，跨测试运行会残留；先清掉才能断言「尚未创建」
        let target = shared_target_dir(&dir).expect("应能推导共享目录");
        let _ = std::fs::remove_dir_all(&target.path);
        let (_, state) = check_shared_target(&dir).expect("应能校验");
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
}
