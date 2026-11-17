//! 生成产物的双向语言门禁。
//!
//! `skill_lang` 是全部模板层「面向人的文案」总开关：英文项目里所有文本文件
//! 都不得出现汉字，中文项目里所有面向人的文案都必须出现汉字。后者是容易漏掉
//! 的一半——一份忘了写双语分支的纯英文模板，在英文项目里「零汉字」检查能通过，
//! 却会让中文项目拿到英文文档。`doctor.d/README.md` 就是这么漏网的。
//!
//! 本模块把两半检查收进同一个审计器，既能扫「刚渲染出来的模板」（回归测试用），
//! 也能按 manifest 审计「已生成的项目」（命令行门禁用）。机器产物走白名单，
//! 白名单要有理由，新增例外前先确认它真的不承载面向人的文案。

use std::collections::BTreeSet;
use std::path::Path;

use crate::error::CoreError;
use crate::manifest::ProjectManifest;
use crate::render::is_text;

/// 期望的项目文案语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    En,
    #[default]
    Zh,
}

impl Lang {
    /// 从 `skill_lang` 读；缺省按中文（模板的既有默认）。
    pub fn from_skill_lang(value: Option<&str>) -> Self {
        match value {
            Some("en") => Lang::En,
            _ => Lang::Zh,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }
}

/// 一处语言违规。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// 相对项目根目录、统一用 `/` 的路径。
    pub path: String,
    pub lang: Lang,
    pub kind: FindingKind,
}

/// 违规种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// 英文项目的文本文件里出现了汉字。
    HanInEnglish,
    /// 中文项目的面向人文案里一个汉字都没有——通常是模板忘了写双语分支。
    NoHanInChinese,
}

impl FindingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingKind::HanInEnglish => "en 项目出现汉字",
            FindingKind::NoHanInChinese => "zh 项目文案缺中文",
        }
    }
}

/// 审计结果。
#[derive(Debug, Clone, Default)]
pub struct LangAudit {
    pub lang: Lang,
    /// 实际检查过的文本文件数。
    pub files_checked: usize,
    pub findings: Vec<Finding>,
}

impl LangAudit {
    pub fn has_violations(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// 汉字（CJK 统一表意文字）检测。
pub fn contains_han(text: &str) -> bool {
    text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// 机器产物：不承载面向人的文案，中文模式下允许零汉字。
///
/// 每一条都对应项目里一条明确的约定（见 AGENTS.md 的多语言章节）：
/// - `*.json`：manifest、编辑器配置、ruleset 等机器配置；
/// - `.agents/skills/*/scripts/*`：技能脚本恒为英文，不随 `skill_lang` 变化；
/// - `Cargo.toml` / `rust-toolchain.toml`：构建配置，注释可有可无；
/// - `src/main.rs`：种子程序，示例输出不本地化。
///
/// 注意这里是 `.agents/skills/` 下的 `scripts/`，不含项目根的 `scripts/doctor.d/`——
/// 后者的 README 是给人看的文档，必须参与检查。
pub fn is_machine_owned(path: &str) -> bool {
    let p = path.replace('\\', "/");
    if p.ends_with(".json") {
        return true;
    }
    if p.starts_with(".agents/skills/") && p.contains("/scripts/") {
        return true;
    }
    matches!(
        p.as_str(),
        "Cargo.toml" | "rust-toolchain.toml" | "src/main.rs"
    )
}

/// 检查单个文本文件。返回 `None` 表示合规。
fn check_file(path: &str, text: &str, lang: Lang) -> Option<Finding> {
    let han = contains_han(text);
    let violating = match lang {
        Lang::En => han,
        Lang::Zh => !han && !is_machine_owned(path),
    };
    violating.then(|| Finding {
        path: path.to_string(),
        lang,
        kind: match lang {
            Lang::En => FindingKind::HanInEnglish,
            Lang::Zh => FindingKind::NoHanInChinese,
        },
    })
}

/// 递归收集目录下的所有文件路径（相对 `root`，统一用 `/`）。
fn collect_files(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // 生成产物里不该有这些，但审计已生成项目时它们常见且体积巨大。
            if matches!(name.as_ref(), ".git" | "target" | "node_modules") {
                continue;
            }
            collect_files(&path, root, out);
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
}

/// 扫描一个目录里的文本文件，按 `lang` 判定语言是否对齐。
///
/// 用于模板回归测试：渲染完直接把整个项目目录丢进来，连机器产物也一并检查
/// （英文项目零汉字是硬要求，机器产物同样不能带汉字）。
pub fn audit_generated(dir: &Path, lang: Lang) -> LangAudit {
    let mut paths = Vec::new();
    collect_files(dir, dir, &mut paths);
    let mut report = LangAudit {
        lang,
        ..LangAudit::default()
    };
    for rel in paths {
        let Ok(bytes) = std::fs::read(dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR)))
        else {
            continue;
        };
        if !is_text(&bytes) {
            continue;
        }
        report.files_checked += 1;
        let text = String::from_utf8_lossy(&bytes);
        if let Some(finding) = check_file(&rel, &text, lang) {
            report.findings.push(finding);
        }
    }
    report.findings.sort_by(|a, b| a.path.cmp(&b.path));
    report
}

/// 按 manifest 审计已生成的项目：只检查模板托管的文件，不动用户自己的文件。
///
/// 语言从项目选项 `skill_lang` 推断（缺省中文），可用 `--lang` 覆盖。清单里
/// 的 Windows 反斜杠会先归一化，保证同一份项目在 Windows / Unix 上判定一致。
pub fn audit_project_lang(dir: &Path, override_lang: Option<Lang>) -> Result<LangAudit, CoreError> {
    let manifest = ProjectManifest::load(dir)?;
    let skill_lang = manifest.options.get("skill_lang").and_then(|v| v.as_str());
    let lang = override_lang.unwrap_or_else(|| Lang::from_skill_lang(skill_lang));

    let mut report = LangAudit {
        lang,
        ..LangAudit::default()
    };
    let mut seen = BTreeSet::new();
    for rel in manifest.files.keys() {
        let normalized = rel.replace('\\', "/");
        if !seen.insert(normalized.clone()) {
            continue;
        }
        let Ok(bytes) = std::fs::read(dir.join(rel)) else {
            // 用户可能删掉了某个托管文件，这属于 audit 命令的职责，不在这里重复报。
            continue;
        };
        if !is_text(&bytes) {
            continue;
        }
        report.files_checked += 1;
        let text = String::from_utf8_lossy(&bytes);
        if let Some(finding) = check_file(&normalized, &text, lang) {
            report.findings.push(finding);
        }
    }
    report.findings.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(report)
}

/// 人类可读的诊断行。
pub fn format_findings(report: &LangAudit) -> Vec<String> {
    report
        .findings
        .iter()
        .map(|f| format!("  [{}] {}: {}", f.lang.as_str(), f.kind.as_str(), f.path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_owned_allowlist_is_narrow() {
        // 白名单命中：脚本、配置、种子代码。
        for path in [
            ".agents/skills/git-workflow/scripts/sync-branch.ps1",
            ".agents\\skills\\mcp-bootstrap\\scripts\\ensure-mcp.mjs",
            ".vscode/settings.json",
            ".pengj-templates.json",
            "Cargo.toml",
            "rust-toolchain.toml",
            "src/main.rs",
        ] {
            assert!(is_machine_owned(path), "{path} 应为机器产物");
        }
        // 关键：doctor.d 的 README 是文档，不能进白名单，否则会漏掉本次这类 bug。
        for path in [
            "scripts/doctor.d/README.md",
            "scripts/doctor.sh",
            ".gate.toml",
            "AGENTS.md",
            "just/rust.just",
        ] {
            assert!(!is_machine_owned(path), "{path} 不应被当成机器产物");
        }
    }

    #[test]
    fn both_directions_are_checked() {
        let doc = "# 自定义环境检查\n使用 sh 执行。";
        assert!(check_file("scripts/doctor.d/README.md", doc, Lang::Zh).is_none());
        assert_eq!(
            check_file("scripts/doctor.d/README.md", doc, Lang::En),
            Some(Finding {
                path: "scripts/doctor.d/README.md".into(),
                lang: Lang::En,
                kind: FindingKind::HanInEnglish,
            })
        );

        // 本次漏网的形状：纯英文文档，英文检查通过、中文检查必须拦下。
        let english_only = "# Custom environment checks\nRun with sh.";
        assert!(check_file("scripts/doctor.d/README.md", english_only, Lang::En).is_none());
        assert_eq!(
            check_file("scripts/doctor.d/README.md", english_only, Lang::Zh).map(|f| f.kind),
            Some(FindingKind::NoHanInChinese)
        );

        // 机器产物在中文模式下不因零汉字报错。
        assert!(check_file(".vscode/settings.json", "{}", Lang::Zh).is_none());
        assert!(check_file("src/main.rs", "fn main() {}", Lang::Zh).is_none());
    }

    #[test]
    fn audit_generated_walks_real_files() {
        let root = std::env::temp_dir().join(format!("pg-i18n-audit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("scripts/doctor.d")).unwrap();
        std::fs::create_dir_all(root.join(".vscode")).unwrap();
        std::fs::write(root.join("scripts/doctor.d/README.md"), "# 自定义环境检查").unwrap();
        std::fs::write(root.join(".vscode/settings.json"), "{}").unwrap();

        let zh = audit_generated(&root, Lang::Zh);
        assert_eq!(zh.files_checked, 2);
        assert!(!zh.has_violations(), "{:?}", zh.findings);

        std::fs::write(
            root.join("scripts/doctor.d/README.md"),
            "# Custom environment checks",
        )
        .unwrap();
        let zh = audit_generated(&root, Lang::Zh);
        assert_eq!(zh.findings.len(), 1);
        assert_eq!(zh.findings[0].path, "scripts/doctor.d/README.md");

        let en = audit_generated(&root, Lang::En);
        assert!(!en.has_violations(), "{:?}", en.findings);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn audit_project_scopes_to_manifest_and_infers_lang() {
        let root = std::env::temp_dir().join(format!("pg-i18n-project-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("AGENTS.md"), "# 编码规范").unwrap();
        // 用户自己的英文文件不在 manifest 里，不该被审计。
        std::fs::write(root.join("my-notes.md"), "# My English notes").unwrap();
        let manifest = serde_json::json!({
            "tool": "pengj-templates",
            "version": "0.0.0",
            "project_name": "demo",
            "layers": ["common"],
            "options": { "skill_lang": "zh" },
            "generated_at": "now",
            "files": { "AGENTS.md": "x", "my-notes.md": "y" }
        });
        std::fs::write(
            root.join(crate::manifest::MANIFEST_FILE),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        // my-notes.md 在清单里就会被判违规——manifest 即托管边界。
        let report = audit_project_lang(&root, None).unwrap();
        assert_eq!(report.lang, Lang::Zh);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].path, "my-notes.md");

        let report = audit_project_lang(&root, Some(Lang::En)).unwrap();
        assert_eq!(report.lang, Lang::En);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].path, "AGENTS.md");
        let _ = std::fs::remove_dir_all(&root);
    }
}
