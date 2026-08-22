use std::collections::BTreeMap;

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
}

impl RenderContext {
    pub fn new(
        project_name: &str,
        layers: Vec<String>,
        options: BTreeMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_name: project_name.to_string(),
            project_slug: slugify(project_name),
            year: chrono_lite_year(),
            layers,
            options,
        }
    }
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
