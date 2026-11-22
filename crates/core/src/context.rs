use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// 模板渲染时的上下文变量
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// 项目名（用户输入原样）
    pub project_name: String,
    /// 项目名转 kebab-case（用于 Cargo/package name 等）
    pub project_slug: String,
    /// 当前年份
    pub year: i32,
    /// 实际启用的层（按合并顺序）
    pub layers: Vec<String>,
    /// 用户生成时选定的选项（如 edition、use_sccache、use_lld）。
    /// 会随 manifest 持久化，`update` 时按同一批选项重新渲染模板。
    pub options: BTreeMap<String, serde_json::Value>,
    /// 目标项目是否包含子模块（存在非空 `.gitmodules` 或启用了 `subrepo` 层）
    pub has_submodules: bool,
    /// 规范化后的仓库 HTTPS URL（来自 `git remote get-url origin`，无或非远程地址时为空串）
    pub repo_url: String,
}

impl RenderContext {
    pub fn new(
        project_name: &str,
        layers: Vec<String>,
        options: BTreeMap<String, serde_json::Value>,
    ) -> Self {
        let has_submodules = layers.iter().any(|l| l == "subrepo");
        Self {
            project_name: project_name.to_string(),
            project_slug: slugify(project_name),
            year: chrono_lite_year(),
            layers,
            options,
            has_submodules,
            repo_url: String::new(),
        }
    }

    /// 结合目标项目目录自动探测 `has_submodules` 与 `repo_url`
    pub fn with_project_dir(
        project_name: &str,
        layers: Vec<String>,
        options: BTreeMap<String, serde_json::Value>,
        project_dir: &Path,
    ) -> Self {
        let has_submodules = detect_has_submodules(project_dir, &layers);
        let repo_url = detect_repo_url(project_dir);
        Self {
            project_name: project_name.to_string(),
            project_slug: slugify(project_name),
            year: chrono_lite_year(),
            layers,
            options,
            has_submodules,
            repo_url,
        }
    }
}

/// 判断目标项目是否包含子模块：存在非空 `.gitmodules` 或 `layers` 包含 `"subrepo"`。
pub fn detect_has_submodules(project_dir: &Path, layers: &[String]) -> bool {
    if layers.iter().any(|l| l == "subrepo") {
        return true;
    }
    let gitmodules = project_dir.join(".gitmodules");
    std::fs::read_to_string(gitmodules)
        .map(|content| !content.trim().is_empty())
        .unwrap_or(false)
}

/// 读取目标项目 `git remote get-url origin` 并规范化为干净的 HTTPS URL。
/// 无 `origin`、非 Git 仓库或本地路径时返回空字符串。
pub fn detect_repo_url(project_dir: &Path) -> String {
    if !project_dir.is_dir() {
        return String::new();
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .args(["remote", "get-url", "origin"])
        .output();
    let Ok(out) = output else {
        return String::new();
    };
    if !out.status.success() {
        return String::new();
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    normalize_repo_url(raw.trim())
}

/// 将常见 Git remote URL（SSH SCP 风格、`ssh://`、`git://`、`https://`、`http://`）
/// 规范化为干净的 HTTPS URL，并剥离任何内嵌凭证（`user:token@`）。
/// 非远程 URL（空串或本地文件系统路径）返回空字符串。
pub fn normalize_repo_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(rest) = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
    {
        return format_clean_https(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("ssh://") {
        return format_clean_https_strip_default_ssh_port(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("git://") {
        return format_clean_https(rest);
    }

    // SCP 风格 SSH：`git@github.com:owner/repo.git`
    if !trimmed.contains("://") {
        if let Some((user_host, path)) = trimmed.split_once(':') {
            if let Some((_, host)) = user_host.rsplit_once('@') {
                if !host.is_empty() && !host.contains('/') && !path.is_empty() {
                    let clean_path = path.trim_start_matches('/');
                    if !clean_path.is_empty() {
                        return format!("https://{host}/{clean_path}");
                    }
                }
            }
        }
    }

    String::new()
}

fn format_clean_https(rest: &str) -> String {
    let Some((authority, path)) = rest.split_once('/') else {
        return String::new();
    };
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let clean_path = path.trim_start_matches('/');
    if host_port.is_empty() || clean_path.is_empty() {
        return String::new();
    }
    format!("https://{host_port}/{clean_path}")
}

fn format_clean_https_strip_default_ssh_port(rest: &str) -> String {
    let Some((authority, path)) = rest.split_once('/') else {
        return String::new();
    };
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = host_port.strip_suffix(":22").unwrap_or(host_port);
    let clean_path = path.trim_start_matches('/');
    if host.is_empty() || clean_path.is_empty() {
        return String::new();
    }
    format!("https://{host}/{clean_path}")
}

/// 转 kebab-case slug：非 ASCII 字母数字一律转 `-`，连续 `-` 合并
/// 纯 CJK 等场景结果为空时回退为 `project`
pub fn slugify(s: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !slug.is_empty() && !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}

/// 当前年份（本地时区）
fn chrono_lite_year() -> i32 {
    // 不引入 chrono，直接取本地时间
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 粗略换算 UTC 年份（够用；±1 年误差可忽略）
    (1970 + secs / 31_557_600) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-ctx-{prefix}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn normalize_repo_url_handles_ssh_https_and_credentials() {
        assert_eq!(
            normalize_repo_url("git@github.com:owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            normalize_repo_url("ssh://git@github.com/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            normalize_repo_url("ssh://git@github.com:22/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            normalize_repo_url("git://github.com/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            normalize_repo_url("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            normalize_repo_url("https://x-access-token:secret123@github.com/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
        assert_eq!(
            normalize_repo_url("http://user:pass@example.com/group/proj.git"),
            "https://example.com/group/proj.git"
        );
        assert_eq!(normalize_repo_url(""), "");
        assert_eq!(normalize_repo_url("/local/path/to/repo"), "");
        assert_eq!(normalize_repo_url("file:///local/path/to/repo"), "");
    }

    #[test]
    fn detect_has_submodules_checks_gitmodules_and_subrepo_layer() {
        let dir = temp_dir("submodules");
        let common = vec!["common".to_string()];
        let with_subrepo = vec!["common".to_string(), "subrepo".to_string()];

        assert!(!detect_has_submodules(&dir, &common));
        assert!(detect_has_submodules(&dir, &with_subrepo));

        // 空白 .gitmodules 不算启用子模块
        std::fs::write(dir.join(".gitmodules"), "   \n\t\n").unwrap();
        assert!(!detect_has_submodules(&dir, &common));

        // 非空 .gitmodules 视为包含子模块
        std::fs::write(
            dir.join(".gitmodules"),
            "[submodule \"foo\"]\n\tpath = foo\n\turl = https://github.com/a/b.git\n",
        )
        .unwrap();
        assert!(detect_has_submodules(&dir, &common));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_repo_url_reads_and_normalizes_origin_remote() {
        let dir = temp_dir("remote");
        // 非 git 仓库返回空串
        assert_eq!(detect_repo_url(&dir), "");

        let status = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .arg("init")
            .status()
            .unwrap();
        assert!(status.success());
        // 未配置 origin 返回空串
        assert_eq!(detect_repo_url(&dir), "");

        let status = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(["remote", "add", "origin", "git@github.com:acme/demo.git"])
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(detect_repo_url(&dir), "https://github.com/acme/demo.git");

        let status = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args([
                "remote",
                "set-url",
                "origin",
                "https://user:tok@github.com/acme/demo.git",
            ])
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(detect_repo_url(&dir), "https://github.com/acme/demo.git");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
