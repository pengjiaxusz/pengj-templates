//! 项目环境自检（doctor）：以 `.pengj-templates/manifest.json` 为唯一事实源，
//! 按项目实际启用的层与技能在进程内执行工具链、提交门禁、工作区链接、子仓库、
//! 共享编译目录检查及 `.pengj-templates/scripts/doctor.d/` 自定义扩展。
//!
//! 展示遵循「通过摘要、异常展开」：子检查全通过时仅展示一行摘要，存在警告或
//! 错误时缩进展开具体问题明细与修复建议；`--json` 输出始终保留完整结构。

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::error::{CoreError, Result};
use crate::i18n_audit::{contains_han, Lang};
use crate::manifest::ProjectManifest;
use crate::repo::{default_links, repo_init, repo_status, RepoItem, RepoStatus};
use crate::subrepo_check::{check_project_with_lang, CheckLevel};
use crate::worktree::{
    check_shared_target, conflicting_override, ensure_shared_target, SharedTargetState,
};

/// 自检运行选项
#[derive(Debug, Clone, Default)]
pub struct DoctorOptions {
    /// 本地修复模式：激活提交钩子、补齐工作区链接与共享编译目录（永不联网）
    pub fix: bool,
    /// 可选的输出语言覆盖（优先级高于 manifest.options.skill_lang）
    pub lang: Option<Lang>,
}

/// 单项检查的结论级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorLevel {
    /// 通过
    Ok,
    /// 警告（不阻断，退出码仍为 0）
    Warn,
    /// 错误（阻断，退出码为 1）
    Error,
}

/// 子检查明细条目
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorDetail {
    /// 明细级别
    pub level: DoctorLevel,
    /// 检查对象名称
    pub name: String,
    /// 明细说明
    pub message: String,
    /// 可操作的修复建议
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
}

/// 单项自检结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorItem {
    /// 稳定标识符（如 `cli`、`git`、`gate`、`repo`、`subrepo`、`worktree`、`custom:<name>`）
    pub id: String,
    /// 分类（如 `tool`、`gate`、`agent`、`subrepo`、`worktree`、`custom`）
    pub category: String,
    /// 结论级别
    pub level: DoctorLevel,
    /// 单行汇总说明
    pub summary: String,
    /// 子检查明细（为空时不序列化）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<DoctorDetail>,
    /// 自定义脚本的标准输出/错误输出（原样保留）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// 一次项目环境自检的完整报告
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    /// 项目根目录绝对路径
    pub root: String,
    /// 生效的输出语言（`zh` 或 `en`）
    pub lang: String,
    /// manifest 中记录的已启用层列表
    pub layers: Vec<String>,
    /// 是否通过（无错误即为 true，允许警告）
    pub ok: bool,
    /// 总检查项数量
    pub checks: usize,
    /// 错误数量
    pub errors: usize,
    /// 警告数量
    pub warnings: usize,
    /// 已执行的自定义扩展脚本数量
    pub custom_count: usize,
    /// 逐项检查结论
    pub items: Vec<DoctorItem>,
}

impl DoctorReport {
    /// 退出码：`errors > 0` 返回 `1`，否则返回 `0`（仅警告仍返回 `0`）
    pub fn exit_code(&self) -> i32 {
        if self.errors > 0 {
            1
        } else {
            0
        }
    }
}

/// 执行项目环境自检。
///
/// 以 `.pengj-templates/manifest.json` 为唯一事实源；缺失时返回 `CoreError::MissingManifest`。
pub fn run_doctor(project_dir: &Path, opts: &DoctorOptions) -> Result<DoctorReport> {
    let root = absolute(project_dir)?;
    let manifest = ProjectManifest::load(&root)?;
    let lang = opts.lang.unwrap_or_else(|| {
        Lang::from_skill_lang(manifest.options.get("skill_lang").and_then(|v| v.as_str()))
    });

    let mut items = Vec::new();

    // 1. pengj-templates-cli 自身（始终通过）
    items.push(DoctorItem {
        id: "cli".to_string(),
        category: "tool".to_string(),
        level: DoctorLevel::Ok,
        summary: format!("pengj-templates-cli {}", env!("CARGO_PKG_VERSION")),
        details: Vec::new(),
        output: None,
    });

    // 2. git（始终检查）
    items.push(check_git_tool(lang));

    // 3. just 层：just（以及 Windows 下的 pwsh）
    if has_layer(&manifest, "just") {
        items.push(check_just_tool(lang));
        #[cfg(windows)]
        items.push(check_pwsh_tool(lang));
    }

    // 4. rust / rust-workspace 层：cargo、rustc
    if has_layer(&manifest, "rust") || has_layer(&manifest, "rust-workspace") {
        items.push(check_cargo_tool(lang));
        items.push(check_rustc_tool(lang));
    }

    // 5. bque 层
    if has_layer(&manifest, "bque") {
        items.push(check_bque_tool(lang));
    }

    // 6. options.skills 含 mcp-bootstrap：node
    if has_skill(&manifest, "mcp-bootstrap") {
        items.push(check_node_tool(lang));
    }

    // 7. gate 层：提交门禁钩子
    if has_layer(&manifest, "gate") {
        items.push(check_gate(&root, opts.fix, lang));
    }

    // 8. agent 层且 .agents 存在：工作区链接与仓库初始化状态
    if has_layer(&manifest, "agent") && root.join(".agents").is_dir() {
        items.push(check_agent_repo(&root, opts.fix, lang));
    }

    // 9. subrepo 层：子仓库体检
    if has_layer(&manifest, "subrepo") {
        items.push(check_subrepo(&root, lang));
    }

    // 10. worktree 层：共享编译目录
    if has_layer(&manifest, "worktree") {
        items.push(check_worktree(&root, opts.fix, lang));
    }

    // 11. 自定义扩展 .pengj-templates/scripts/doctor.d/
    let custom_items = run_custom_scripts(&root, opts.fix, lang);
    let custom_count = custom_items.len();
    items.extend(custom_items);

    let checks = items.len();
    let errors = items
        .iter()
        .filter(|i| i.level == DoctorLevel::Error)
        .count();
    let warnings = items
        .iter()
        .filter(|i| i.level == DoctorLevel::Warn)
        .count();

    Ok(DoctorReport {
        root: display_path(&root),
        lang: lang.as_str().to_string(),
        layers: manifest.layers,
        ok: errors == 0,
        checks,
        errors,
        warnings,
        custom_count,
        items,
    })
}

/// 将自检报告格式化为面向人的终端文本。
///
/// 通过项只打印一行摘要；警告/错误项缩进展开明细与修复建议；自定义脚本有输出时原样打印。
pub fn format_doctor_report(report: &DoctorReport, lang: Lang) -> String {
    let mut out = String::new();

    for item in &report.items {
        if item.category == "custom" {
            if let Some(script_output) = &item.output {
                if !script_output.is_empty() {
                    out.push_str(script_output);
                    if !script_output.ends_with('\n') {
                        out.push('\n');
                    }
                }
            }
            let tag = match item.level {
                DoctorLevel::Ok => "ok   ",
                DoctorLevel::Warn => "warn ",
                DoctorLevel::Error => "error",
            };
            let _ = writeln!(out, "[doctor] custom {tag} {}", item.summary);
            continue;
        }

        let tag = match item.level {
            DoctorLevel::Ok => "ok   ",
            DoctorLevel::Warn => "warn ",
            DoctorLevel::Error => "error",
        };
        let _ = writeln!(out, "[doctor] {tag} {}", item.summary);

        if item.level != DoctorLevel::Ok && !item.details.is_empty() {
            let mut shown_any = false;
            for detail in &item.details {
                if detail.level == DoctorLevel::Ok {
                    continue;
                }
                shown_any = true;
                write_detail(&mut out, detail, lang);
            }
            if !shown_any {
                for detail in &item.details {
                    write_detail(&mut out, detail, lang);
                }
            }
        }
    }

    if report.custom_count == 0 {
        let ext = custom_script_glob();
        match lang {
            Lang::Zh => {
                let _ = writeln!(
                    out,
                    "[doctor] 自定义检查：无（在 .pengj-templates/scripts/doctor.d/ 下添加 {ext} 脚本即可扩展本检查）"
                );
            }
            Lang::En => {
                let _ = writeln!(
                    out,
                    "[doctor] custom: none (add {ext} scripts to .pengj-templates/scripts/doctor.d/ to extend this check)"
                );
            }
        }
    }

    match lang {
        Lang::Zh => {
            let _ = writeln!(
                out,
                "[doctor] 共 {} 项检查：{} 个错误，{} 个警告",
                report.checks, report.errors, report.warnings
            );
        }
        Lang::En => {
            let _ = writeln!(
                out,
                "[doctor] {} checks: {} errors, {} warnings",
                report.checks, report.errors, report.warnings
            );
        }
    }

    out
}

fn write_detail(out: &mut String, detail: &DoctorDetail, lang: Lang) {
    let label = match (lang, detail.level) {
        (Lang::Zh, DoctorLevel::Ok) => "通过",
        (Lang::Zh, DoctorLevel::Warn) => "警告",
        (Lang::Zh, DoctorLevel::Error) => "错误",
        (Lang::En, DoctorLevel::Ok) => "ok",
        (Lang::En, DoctorLevel::Warn) => "warn",
        (Lang::En, DoctorLevel::Error) => "error",
    };
    let _ = writeln!(out, "  - [{label}] {}: {}", detail.name, detail.message);
    if let Some(remedy) = &detail.remedy {
        let _ = writeln!(out, "      -> {remedy}");
    }
}

fn custom_script_glob() -> &'static str {
    #[cfg(windows)]
    {
        "*.ps1"
    }
    #[cfg(not(windows))]
    {
        "*.sh"
    }
}

// ---------------------------------------------------------------------------
// 层/技能与基础工具探测
// ---------------------------------------------------------------------------

fn has_layer(manifest: &ProjectManifest, name: &str) -> bool {
    manifest.layers.iter().any(|l| l == name)
}

fn has_skill(manifest: &ProjectManifest, name: &str) -> bool {
    manifest
        .options
        .get("skills")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|item| item.as_str() == Some(name)))
        .unwrap_or(false)
}

fn probe_tool_version(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().unwrap_or("").trim();
    if !line.is_empty() {
        return Some(line.to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let err_line = stderr.lines().next().unwrap_or("").trim();
    Some(err_line.to_string())
}

fn check_git_tool(lang: Lang) -> DoctorItem {
    match probe_tool_version("git", &["--version"]) {
        Some(ver) => {
            let trimmed = ver.strip_prefix("git version ").unwrap_or(&ver).trim();
            DoctorItem {
                id: "git".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Ok,
                summary: format!("git {trimmed}"),
                details: Vec::new(),
                output: None,
            }
        }
        None => DoctorItem {
            id: "git".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Error,
            summary: match lang {
                Lang::Zh => "未找到 git".to_string(),
                Lang::En => "git not found".to_string(),
            },
            details: Vec::new(),
            output: None,
        },
    }
}

fn check_just_tool(lang: Lang) -> DoctorItem {
    match probe_tool_version("just", &["--version"]) {
        Some(ver) => {
            let trimmed = ver.strip_prefix("just ").unwrap_or(&ver).trim();
            DoctorItem {
                id: "just".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Ok,
                summary: format!("just {trimmed}"),
                details: Vec::new(),
                output: None,
            }
        }
        None => DoctorItem {
            id: "just".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Error,
            summary: match lang {
                Lang::Zh => {
                    "未找到 just；安装地址：https://github.com/casey/just#installation".to_string()
                }
                Lang::En => "just not found; install: https://github.com/casey/just#installation"
                    .to_string(),
            },
            details: Vec::new(),
            output: None,
        },
    }
}

#[cfg(windows)]
fn check_pwsh_tool(lang: Lang) -> DoctorItem {
    match probe_tool_version("pwsh", &["--version"]) {
        Some(ver) => DoctorItem {
            id: "pwsh".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Ok,
            summary: format!("pwsh {}", ver.trim()),
            details: Vec::new(),
            output: None,
        },
        None => DoctorItem {
            id: "pwsh".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Error,
            summary: match lang {
                Lang::Zh => "未找到 pwsh；Windows 上的 just 配方需要 PowerShell 7".to_string(),
                Lang::En => {
                    "pwsh not found; the just recipes use PowerShell 7 on Windows".to_string()
                }
            },
            details: Vec::new(),
            output: None,
        },
    }
}

fn check_cargo_tool(lang: Lang) -> DoctorItem {
    match probe_tool_version("cargo", &["--version"]) {
        Some(ver) => {
            let trimmed = ver.strip_prefix("cargo ").unwrap_or(&ver).trim();
            DoctorItem {
                id: "cargo".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Ok,
                summary: format!("cargo {trimmed}"),
                details: Vec::new(),
                output: None,
            }
        }
        None => DoctorItem {
            id: "cargo".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Error,
            summary: match lang {
                Lang::Zh => "未找到 cargo；请安装 Rust 工具链".to_string(),
                Lang::En => "cargo not found; install a Rust toolchain".to_string(),
            },
            details: Vec::new(),
            output: None,
        },
    }
}

fn check_rustc_tool(lang: Lang) -> DoctorItem {
    match probe_tool_version("rustc", &["--version"]) {
        Some(ver) => {
            let trimmed = ver.strip_prefix("rustc ").unwrap_or(&ver).trim();
            DoctorItem {
                id: "rustc".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Ok,
                summary: format!("rustc {trimmed}"),
                details: Vec::new(),
                output: None,
            }
        }
        None => DoctorItem {
            id: "rustc".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Error,
            summary: match lang {
                Lang::Zh => "未找到 rustc；请安装 Rust 工具链".to_string(),
                Lang::En => "rustc not found; install a Rust toolchain".to_string(),
            },
            details: Vec::new(),
            output: None,
        },
    }
}

fn check_bque_tool(lang: Lang) -> DoctorItem {
    #[cfg(windows)]
    {
        match probe_tool_version("bque", &["--version"]) {
            Some(ver) => DoctorItem {
                id: "bque".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Ok,
                summary: format!("bque {}", ver.trim()),
                details: Vec::new(),
                output: None,
            },
            None => DoctorItem {
                id: "bque".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Warn,
                summary: match lang {
                    Lang::Zh => {
                        "未找到 bque；安装命令：irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex"
                            .to_string()
                    }
                    Lang::En => {
                        "bque not found; install: irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex"
                            .to_string()
                    }
                },
                details: Vec::new(),
                output: None,
            },
        }
    }
    #[cfg(not(windows))]
    {
        DoctorItem {
            id: "bque".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Ok,
            summary: match lang {
                Lang::Zh => "当前平台不需要 bque".to_string(),
                Lang::En => "bque not required on this platform".to_string(),
            },
            details: Vec::new(),
            output: None,
        }
    }
}

fn check_node_tool(lang: Lang) -> DoctorItem {
    match probe_tool_version("node", &["--version"]) {
        Some(ver) => DoctorItem {
            id: "node".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Ok,
            summary: format!("node {}", ver.trim()),
            details: Vec::new(),
            output: None,
        },
        None => DoctorItem {
            id: "node".to_string(),
            category: "tool".to_string(),
            level: DoctorLevel::Warn,
            summary: match lang {
                Lang::Zh => {
                    "未找到 node；mcp-bootstrap 技能需要它（请安装 Node.js，或从 options.skills 中移除该技能）"
                        .to_string()
                }
                Lang::En => {
                    "node not found; required by the mcp-bootstrap skill (install Node.js or drop that skill from options.skills)"
                        .to_string()
                }
            },
            details: Vec::new(),
            output: None,
        },
    }
}

// ---------------------------------------------------------------------------
// gate 层：提交门禁钩子检查与修复
// ---------------------------------------------------------------------------

fn check_gate(root: &Path, fix: bool, lang: Lang) -> DoctorItem {
    if !pengj_gate::git::in_work_tree_in(root) {
        return DoctorItem {
            id: "gate".to_string(),
            category: "gate".to_string(),
            level: DoctorLevel::Warn,
            summary: match lang {
                Lang::Zh => "当前不是 git 工作树；跳过提交钩子检查".to_string(),
                Lang::En => "not a git work tree; skipping commit hook check".to_string(),
            },
            details: Vec::new(),
            output: None,
        };
    }

    if fix {
        return fix_gate(root, lang);
    }

    let mut details = Vec::new();
    let hooks_dir = root.join(".githooks");

    for hook in ["pre-commit", "commit-msg"] {
        let path = hooks_dir.join(hook);
        if !path.is_file() {
            details.push(DoctorDetail {
                level: DoctorLevel::Error,
                name: format!(".githooks/{hook}"),
                message: match lang {
                    Lang::Zh => "必需的提交钩子文件缺失".to_string(),
                    Lang::En => "required commit hook file is missing".to_string(),
                },
                remedy: Some(match lang {
                    Lang::Zh => "运行 pengj-templates-cli update 补齐 .githooks 目录".to_string(),
                    Lang::En => "run `pengj-templates-cli update` to restore .githooks".to_string(),
                }),
            });
        } else {
            details.push(DoctorDetail {
                level: DoctorLevel::Ok,
                name: format!(".githooks/{hook}"),
                message: match lang {
                    Lang::Zh => "钩子文件存在".to_string(),
                    Lang::En => "hook file present".to_string(),
                },
                remedy: None,
            });
        }
    }

    let local_hooks =
        pengj_gate::git::probe_in(root, &["config", "--local", "--get", "core.hooksPath"])
            .unwrap_or_default();
    if local_hooks == ".githooks" {
        details.push(DoctorDetail {
            level: DoctorLevel::Ok,
            name: "core.hooksPath".to_string(),
            message: ".githooks".to_string(),
            remedy: None,
        });
    } else if !local_hooks.is_empty() {
        details.push(DoctorDetail {
            level: DoctorLevel::Warn,
            name: "core.hooksPath".to_string(),
            message: match lang {
                Lang::Zh => format!("当前值为 '{local_hooks}'，并非 .githooks；提交门禁未生效"),
                Lang::En => {
                    format!("set to '{local_hooks}', expected .githooks; commit gate is inactive")
                }
            },
            remedy: Some(match lang {
                Lang::Zh => {
                    "运行 just doctor --fix 或 pengj-templates-cli gate hooks install".to_string()
                }
                Lang::En => "run `just doctor --fix` or `pengj-templates-cli gate hooks install`"
                    .to_string(),
            }),
        });
    } else {
        details.push(DoctorDetail {
            level: DoctorLevel::Warn,
            name: "core.hooksPath".to_string(),
            message: match lang {
                Lang::Zh => "提交钩子未激活（未设置 core.hooksPath）".to_string(),
                Lang::En => "commit hooks are not active (core.hooksPath is unset)".to_string(),
            },
            remedy: Some(match lang {
                Lang::Zh => {
                    "运行 just doctor --fix 或 pengj-templates-cli doctor --fix".to_string()
                }
                Lang::En => {
                    "run `just doctor --fix` or `pengj-templates-cli doctor --fix`".to_string()
                }
            }),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for hook in ["pre-commit", "commit-msg", "pre-push"] {
            let path = hooks_dir.join(hook);
            if !path.is_file() {
                continue;
            }
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.permissions().mode() & 0o111 == 0 {
                    details.push(DoctorDetail {
                        level: DoctorLevel::Warn,
                        name: format!(".githooks/{hook}"),
                        message: match lang {
                            Lang::Zh => "文件缺少可执行权限（Unix 上 git 会静默跳过）".to_string(),
                            Lang::En => {
                                "file is not executable (git silently skips hooks without +x on Unix)"
                                    .to_string()
                            }
                        },
                        remedy: Some(match lang {
                            Lang::Zh => "带 --fix 运行 doctor 自动补齐权限".to_string(),
                            Lang::En => {
                                "run doctor with --fix to set executable permissions".to_string()
                            }
                        }),
                    });
                }
            }
        }
    }

    for rival in pengj_gate::hooks::detect_rivals(root) {
        details.push(rival_detail(&rival, lang));
    }

    let level = max_detail_level(&details);
    let summary = match (lang, level) {
        (Lang::Zh, DoctorLevel::Ok) => "提交钩子已激活（core.hooksPath=.githooks）".to_string(),
        (Lang::Zh, DoctorLevel::Warn) => "提交钩子存在告警（未激活或需修正）".to_string(),
        (Lang::Zh, DoctorLevel::Error) => {
            ".githooks 不完整；应包含 pre-commit 与 commit-msg".to_string()
        }
        (Lang::En, DoctorLevel::Ok) => "commit hooks active (core.hooksPath=.githooks)".to_string(),
        (Lang::En, DoctorLevel::Warn) => {
            "commit hooks need attention (inactive or requires fix)".to_string()
        }
        (Lang::En, DoctorLevel::Error) => {
            ".githooks is incomplete; expected pre-commit and commit-msg".to_string()
        }
    };

    DoctorItem {
        id: "gate".to_string(),
        category: "gate".to_string(),
        level,
        summary,
        details,
        output: None,
    }
}

fn fix_gate(root: &Path, lang: Lang) -> DoctorItem {
    match pengj_gate::hooks::install_in(root) {
        Ok(outcome) => {
            if outcome.skipped_not_git {
                return DoctorItem {
                    id: "gate".to_string(),
                    category: "gate".to_string(),
                    level: DoctorLevel::Warn,
                    summary: match lang {
                        Lang::Zh => "当前不是 git 工作树；跳过提交钩子检查".to_string(),
                        Lang::En => "not a git work tree; skipping commit hook check".to_string(),
                    },
                    details: Vec::new(),
                    output: None,
                };
            }

            let mut details: Vec<DoctorDetail> = outcome
                .hooks
                .iter()
                .map(|hook| DoctorDetail {
                    level: DoctorLevel::Ok,
                    name: format!(".githooks/{hook}"),
                    message: match lang {
                        Lang::Zh => "已激活".to_string(),
                        Lang::En => "activated".to_string(),
                    },
                    remedy: None,
                })
                .collect();

            for rival in &outcome.rival_warnings {
                details.push(rival_detail(rival, lang));
            }

            let has_rivals = !outcome.rival_warnings.is_empty();
            let level = if has_rivals {
                DoctorLevel::Warn
            } else {
                DoctorLevel::Ok
            };
            let summary = match (lang, has_rivals) {
                (Lang::Zh, false) => "提交钩子已激活（core.hooksPath=.githooks）".to_string(),
                (Lang::Zh, true) => {
                    "提交钩子已激活（core.hooksPath=.githooks），但检测到竞品残留".to_string()
                }
                (Lang::En, false) => {
                    "commit hooks activated (core.hooksPath=.githooks)".to_string()
                }
                (Lang::En, true) => {
                    "commit hooks activated (core.hooksPath=.githooks), but rival hook leftovers were detected"
                        .to_string()
                }
            };

            DoctorItem {
                id: "gate".to_string(),
                category: "gate".to_string(),
                level,
                summary,
                details,
                output: None,
            }
        }
        Err(err) => {
            let message = if lang == Lang::En && contains_han(&err) {
                "hook installation failed; inspect .githooks".to_string()
            } else {
                err
            };
            DoctorItem {
                id: "gate".to_string(),
                category: "gate".to_string(),
                level: DoctorLevel::Error,
                summary: match lang {
                    Lang::Zh => "无法激活提交钩子".to_string(),
                    Lang::En => "could not activate commit hooks".to_string(),
                },
                details: vec![DoctorDetail {
                    level: DoctorLevel::Error,
                    name: "gate".to_string(),
                    message,
                    remedy: Some(match lang {
                        Lang::Zh => {
                            "检查 .githooks 目录后重跑 pengj-templates-cli doctor --fix".to_string()
                        }
                        Lang::En => {
                            "inspect .githooks and re-run `pengj-templates-cli doctor --fix`"
                                .to_string()
                        }
                    }),
                }],
                output: None,
            }
        }
    }
}

fn rival_detail(rival: &pengj_gate::hooks::HookRivalWarning, lang: Lang) -> DoctorDetail {
    let joined = rival.leftovers.join(", ");
    DoctorDetail {
        level: DoctorLevel::Warn,
        name: rival.tool.clone(),
        message: match lang {
            Lang::Zh => format!("检测到竞品钩子残留（{joined}），可能与提交门禁冲突"),
            Lang::En => format!(
                "detected leftover hooks ({joined}) from {}, which may conflict with the commit gate",
                rival.tool
            ),
        },
        remedy: Some(match lang {
            Lang::Zh => "移除 .githooks/*.old 残留文件后重跑 just doctor --fix".to_string(),
            Lang::En => "remove the .githooks/*.old leftovers and re-run `just doctor --fix`"
                .to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// agent 层：工作区链接与仓库初始化
// ---------------------------------------------------------------------------

fn check_agent_repo(root: &Path, fix: bool, lang: Lang) -> DoctorItem {
    let links = default_links(root);
    let init_err = if fix {
        repo_init(root, &links, true).err()
    } else {
        None
    };

    let report = match repo_status(root, &links) {
        Ok(report) => report,
        Err(err) => {
            let msg = match lang {
                Lang::Zh => format!("检查工作区链接失败: {err}"),
                Lang::En => "failed to inspect agent workspace links".to_string(),
            };
            return DoctorItem {
                id: "repo".to_string(),
                category: "agent".to_string(),
                level: DoctorLevel::Error,
                summary: match lang {
                    Lang::Zh => "工作区链接检查失败".to_string(),
                    Lang::En => "workspace link check failed".to_string(),
                },
                details: vec![DoctorDetail {
                    level: DoctorLevel::Error,
                    name: "repo".to_string(),
                    message: msg,
                    remedy: Some(match lang {
                        Lang::Zh => "运行 pengj-templates-cli repo status --dir . 排查".to_string(),
                        Lang::En => {
                            "run `pengj-templates-cli repo status --dir .` to diagnose".to_string()
                        }
                    }),
                }],
                output: None,
            };
        }
    };

    let mut details: Vec<DoctorDetail> = report
        .items
        .iter()
        .map(|item| map_repo_item(item, lang))
        .collect();

    if let Some(err) = init_err {
        details.push(DoctorDetail {
            level: DoctorLevel::Error,
            name: "repo-init".to_string(),
            message: match lang {
                Lang::Zh => format!("自动初始化失败: {err}"),
                Lang::En => "automatic workspace initialization failed".to_string(),
            },
            remedy: Some(match lang {
                Lang::Zh => "运行 pengj-templates-cli repo init --dir . 查看详情".to_string(),
                Lang::En => "run `pengj-templates-cli repo init --dir .` for details".to_string(),
            }),
        });
    }

    let ok = report.ok && !details.iter().any(|d| d.level == DoctorLevel::Error);
    let level = if ok {
        DoctorLevel::Ok
    } else {
        DoctorLevel::Error
    };
    let summary = match (lang, ok, fix) {
        (Lang::Zh, true, _) => "工作区链接已就绪（agent 客户端）".to_string(),
        (Lang::Zh, false, true) => "无法完成工作区链接修复（agent 客户端）".to_string(),
        (Lang::Zh, false, false) => "工作区链接未就绪（agent 客户端）".to_string(),
        (Lang::En, true, _) => "workspace links ready (agent clients)".to_string(),
        (Lang::En, false, true) => "could not fix workspace links (agent clients)".to_string(),
        (Lang::En, false, false) => "workspace links are not ready (agent clients)".to_string(),
    };

    DoctorItem {
        id: "repo".to_string(),
        category: "agent".to_string(),
        level,
        summary,
        details,
        output: None,
    }
}

fn map_repo_item(item: &RepoItem, lang: Lang) -> DoctorDetail {
    let level = match item.status {
        RepoStatus::Ok | RepoStatus::Created | RepoStatus::Skipped => DoctorLevel::Ok,
        RepoStatus::Missing | RepoStatus::Conflict => DoctorLevel::Error,
    };

    let (message, remedy) = match lang {
        Lang::Zh => {
            let remedy = match item.status {
                RepoStatus::Missing => Some(
                    "运行 just doctor --fix 或 pengj-templates-cli repo init --dir .".to_string(),
                ),
                RepoStatus::Conflict => {
                    Some("手动处理冲突路径后重跑 just doctor --fix".to_string())
                }
                _ => None,
            };
            (item.detail.clone(), remedy)
        }
        Lang::En => {
            let (msg, rem) = match item.status {
                RepoStatus::Ok => ("ready".to_string(), None),
                RepoStatus::Created => ("created".to_string(), None),
                RepoStatus::Skipped => (
                    "skipped (not a git work tree or not applicable)".to_string(),
                    None,
                ),
                RepoStatus::Missing => (
                    "missing workspace link or repository configuration".to_string(),
                    Some(
                        "run `just doctor --fix` or `pengj-templates-cli repo init --dir .`"
                            .to_string(),
                    ),
                ),
                RepoStatus::Conflict => (
                    "conflict: path already exists and points elsewhere".to_string(),
                    Some(
                        "resolve the conflicting path manually, then re-run `just doctor --fix`"
                            .to_string(),
                    ),
                ),
            };
            (msg, rem)
        }
    };

    DoctorDetail {
        level,
        name: item.name.clone(),
        message,
        remedy,
    }
}

// ---------------------------------------------------------------------------
// subrepo 层：子仓库体检
// ---------------------------------------------------------------------------

fn check_subrepo(root: &Path, lang: Lang) -> DoctorItem {
    match check_project_with_lang(root, lang) {
        Ok(report) => {
            let details: Vec<DoctorDetail> = report
                .items
                .into_iter()
                .map(|item| DoctorDetail {
                    level: match item.level {
                        CheckLevel::Pass => DoctorLevel::Ok,
                        CheckLevel::Warn => DoctorLevel::Warn,
                        CheckLevel::Fail => DoctorLevel::Error,
                    },
                    name: item.name,
                    message: item.detail,
                    remedy: item.remedy,
                })
                .collect();

            let level = max_detail_level(&details);
            let summary = match (lang, level) {
                (Lang::Zh, DoctorLevel::Ok) => "子仓库体检通过".to_string(),
                (Lang::Zh, DoctorLevel::Warn) => "子仓库体检有警告".to_string(),
                (Lang::Zh, DoctorLevel::Error) => "子仓库体检未通过".to_string(),
                (Lang::En, DoctorLevel::Ok) => "sub-repository health check passed".to_string(),
                (Lang::En, DoctorLevel::Warn) => {
                    "sub-repository health check reported warnings".to_string()
                }
                (Lang::En, DoctorLevel::Error) => "sub-repository health check failed".to_string(),
            };

            DoctorItem {
                id: "subrepo".to_string(),
                category: "subrepo".to_string(),
                level,
                summary,
                details,
                output: None,
            }
        }
        Err(err) => {
            let err_text = err.to_string();
            let message = if lang == Lang::En && contains_han(&err_text) {
                format!(
                    "failed to load or parse {}",
                    crate::subrepo::SUBREPO_REGISTRY_REL
                )
            } else {
                err_text
            };
            let remedy = match lang {
                Lang::Zh => format!(
                    "确认 {} 存在且为合法 TOML，再运行 pengj-templates-cli subrepo check --dir .",
                    crate::subrepo::SUBREPO_REGISTRY_REL
                ),
                Lang::En => format!(
                    "ensure {} exists and is valid TOML, then run `pengj-templates-cli subrepo check --dir .`",
                    crate::subrepo::SUBREPO_REGISTRY_REL
                ),
            };
            DoctorItem {
                id: "subrepo".to_string(),
                category: "subrepo".to_string(),
                level: DoctorLevel::Error,
                summary: match lang {
                    Lang::Zh => "子仓库体检未通过（登记表缺失或无效）".to_string(),
                    Lang::En => "sub-repository health check failed (missing or invalid registry)"
                        .to_string(),
                },
                details: vec![DoctorDetail {
                    level: DoctorLevel::Error,
                    name: crate::subrepo::SUBREPO_REGISTRY_REL.to_string(),
                    message,
                    remedy: Some(remedy),
                }],
                output: None,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// worktree 层：共享编译目录检查与修复
// ---------------------------------------------------------------------------

fn check_worktree(root: &Path, fix: bool, lang: Lang) -> DoctorItem {
    let checked = if fix {
        ensure_shared_target(root)
    } else {
        Ok(check_shared_target(root))
    };

    let (target, state) = match checked {
        Ok(pair) => pair,
        Err(err) => {
            let err_text = err.to_string();
            let message = if lang == Lang::En && contains_han(&err_text) {
                "failed to create shared build directory".to_string()
            } else {
                err_text
            };
            return DoctorItem {
                id: "worktree".to_string(),
                category: "worktree".to_string(),
                level: DoctorLevel::Error,
                summary: match lang {
                    Lang::Zh => "无法创建共享编译目录".to_string(),
                    Lang::En => "could not create shared build directory".to_string(),
                },
                details: vec![DoctorDetail {
                    level: DoctorLevel::Error,
                    name: "shared-target".to_string(),
                    message,
                    remedy: Some(match lang {
                        Lang::Zh => "检查主工作树目录权限后重跑 just doctor --fix".to_string(),
                        Lang::En => {
                            "check main worktree directory permissions and re-run `just doctor --fix`"
                                .to_string()
                        }
                    }),
                }],
                output: None,
            };
        }
    };

    let override_path = conflicting_override(&target);
    let target_display = display_path(&target.path);
    let mut details = Vec::new();

    match state {
        SharedTargetState::Ready => {
            details.push(DoctorDetail {
                level: DoctorLevel::Ok,
                name: "shared-target".to_string(),
                message: match lang {
                    Lang::Zh => format!("共享编译目录就绪：{target_display}"),
                    Lang::En => format!("shared build directory ready: {target_display}"),
                },
                remedy: None,
            });
        }
        SharedTargetState::Missing => {
            details.push(DoctorDetail {
                level: DoctorLevel::Error,
                name: "shared-target".to_string(),
                message: match lang {
                    Lang::Zh => format!("共享编译目录不存在：{target_display}"),
                    Lang::En => format!("shared build directory is missing: {target_display}"),
                },
                remedy: Some(match lang {
                    Lang::Zh => {
                        "运行 just doctor --fix 或 pengj-templates-cli worktree check --dir . --fix"
                            .to_string()
                    }
                    Lang::En => {
                        "run `just doctor --fix` or `pengj-templates-cli worktree check --dir . --fix`"
                            .to_string()
                    }
                }),
            });
        }
        SharedTargetState::NotWritable => {
            details.push(DoctorDetail {
                level: DoctorLevel::Error,
                name: "shared-target".to_string(),
                message: match lang {
                    Lang::Zh => format!("共享编译目录存在但不可写：{target_display}"),
                    Lang::En => format!("shared build directory is not writable: {target_display}"),
                },
                remedy: Some(match lang {
                    Lang::Zh => format!("检查目录权限：{target_display}"),
                    Lang::En => format!("check directory permissions for: {target_display}"),
                }),
            });
        }
    }

    if let Some(other) = &override_path {
        details.push(DoctorDetail {
            level: DoctorLevel::Warn,
            name: "CARGO_TARGET_DIR".to_string(),
            message: match lang {
                Lang::Zh => format!(
                    "环境变量 CARGO_TARGET_DIR 指向 {other}，与共享编译目录 {target_display} 不一致"
                ),
                Lang::En => format!(
                    "CARGO_TARGET_DIR points to {other}, which differs from the shared build directory {target_display}"
                ),
            },
            remedy: Some(match lang {
                Lang::Zh => "请经由 just 配方构建，或清除外部 CARGO_TARGET_DIR 环境变量".to_string(),
                Lang::En => {
                    "build via `just` recipes or unset the external CARGO_TARGET_DIR override"
                        .to_string()
                }
            }),
        });
    }

    let level = max_detail_level(&details);
    let summary = match (lang, level) {
        (Lang::Zh, DoctorLevel::Ok) => "共享编译目录就绪（各工作树复用同一份编译目录）".to_string(),
        (Lang::Zh, DoctorLevel::Warn) => {
            "共享编译目录就绪，但环境变量 CARGO_TARGET_DIR 指向别处".to_string()
        }
        (Lang::Zh, DoctorLevel::Error) => "共享编译目录缺失或不可写".to_string(),
        (Lang::En, DoctorLevel::Ok) => {
            "shared build directory ready (worktrees reuse one build directory)".to_string()
        }
        (Lang::En, DoctorLevel::Warn) => {
            "shared build directory is ready, but CARGO_TARGET_DIR points elsewhere".to_string()
        }
        (Lang::En, DoctorLevel::Error) => {
            "shared build directory is missing or not writable".to_string()
        }
    };

    DoctorItem {
        id: "worktree".to_string(),
        category: "worktree".to_string(),
        level,
        summary,
        details,
        output: None,
    }
}

// ---------------------------------------------------------------------------
// 自定义扩展：.pengj-templates/scripts/doctor.d/
// ---------------------------------------------------------------------------

fn run_custom_scripts(root: &Path, fix: bool, lang: Lang) -> Vec<DoctorItem> {
    let dir = root.join(".pengj-templates/scripts/doctor.d");
    if !dir.is_dir() {
        return Vec::new();
    }

    #[cfg(windows)]
    let ext = "ps1";
    #[cfg(not(windows))]
    let ext = "sh";

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut scripts: Vec<(String, PathBuf)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let file_ext = path.extension()?.to_str()?;
            if !file_ext.eq_ignore_ascii_case(ext) {
                return None;
            }
            let name = path.file_name()?.to_str()?.to_string();
            Some((name, path))
        })
        .collect();

    scripts.sort_by(|a, b| a.0.cmp(&b.0));

    let root_str = display_path(root);
    let mut results = Vec::with_capacity(scripts.len());

    for (name, script_path) in scripts {
        #[cfg(windows)]
        let mut cmd = {
            let mut c = Command::new("pwsh");
            c.args(["-NoProfile", "-File"]).arg(&script_path);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.arg(&script_path);
            c
        };

        cmd.current_dir(root).env("DOCTOR_ROOT", &root_str);
        if fix {
            cmd.env("DOCTOR_FIX", "1");
        } else {
            cmd.env_remove("DOCTOR_FIX");
        }

        match cmd.output() {
            Ok(out) => {
                let mut combined = String::new();
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                combined.push_str(&stdout);
                if !stderr.is_empty() {
                    if !combined.is_empty() && !combined.ends_with('\n') {
                        combined.push('\n');
                    }
                    combined.push_str(&stderr);
                }
                let output = if combined.is_empty() {
                    None
                } else {
                    Some(combined)
                };
                let level = if out.status.success() {
                    DoctorLevel::Ok
                } else {
                    DoctorLevel::Error
                };
                results.push(DoctorItem {
                    id: format!("custom:{name}"),
                    category: "custom".to_string(),
                    level,
                    summary: name,
                    details: Vec::new(),
                    output,
                });
            }
            Err(err) => {
                let msg = match lang {
                    Lang::Zh => format!("无法启动自定义脚本 {name}: {err}"),
                    Lang::En => format!("failed to launch custom script {name}: {err}"),
                };
                results.push(DoctorItem {
                    id: format!("custom:{name}"),
                    category: "custom".to_string(),
                    level: DoctorLevel::Error,
                    summary: name.clone(),
                    details: vec![DoctorDetail {
                        level: DoctorLevel::Error,
                        name,
                        message: msg,
                        remedy: None,
                    }],
                    output: None,
                });
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// 通用辅助函数
// ---------------------------------------------------------------------------

fn max_detail_level(details: &[DoctorDetail]) -> DoctorLevel {
    if details.iter().any(|d| d.level == DoctorLevel::Error) {
        DoctorLevel::Error
    } else if details.iter().any(|d| d.level == DoctorLevel::Warn) {
        DoctorLevel::Warn
    } else {
        DoctorLevel::Ok
    }
}

fn absolute(root: &Path) -> Result<PathBuf> {
    let canonical = root
        .canonicalize()
        .map_err(|error| CoreError::Git(format!("无法访问目录 {}: {error}", root.display())))?;
    Ok(crate::worktree::strip_verbatim(&canonical))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-doctor-{}-{}-{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建临时目录失败");
        dir
    }

    fn write_manifest(dir: &Path, layers: &[&str], options: BTreeMap<String, serde_json::Value>) {
        let manifest = ProjectManifest {
            tool: "pengj-templates".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_name: "demo".to_string(),
            layers: layers.iter().map(|s| (*s).to_string()).collect(),
            options,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            files: BTreeMap::new(),
        };
        manifest.save(dir).expect("写入 manifest 失败");
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("创建父目录失败");
        }
        std::fs::write(path, content).expect("写文件失败");
    }

    fn init_git_repo(dir: &Path) -> bool {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .arg("init")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn 缺失manifest时返回missing_manifest错误() {
        let dir = temp_dir("missing-manifest");
        let err = run_doctor(&dir, &DoctorOptions::default()).expect_err("缺失 manifest 必须报错");
        assert!(
            matches!(err, CoreError::MissingManifest(_)),
            "预期 MissingManifest，实际为: {err:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 按层与技能裁剪检查项() {
        let dir = temp_dir("layer-pruning");
        write_manifest(&dir, &["common"], BTreeMap::new());
        let report = run_doctor(&dir, &DoctorOptions::default()).expect("common 层应可自检");
        let ids: Vec<&str> = report.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"cli"));
        assert!(ids.contains(&"git"));
        assert!(!ids.contains(&"just"));
        assert!(!ids.contains(&"cargo"));
        assert!(!ids.contains(&"rustc"));
        assert!(!ids.contains(&"bque"));
        assert!(!ids.contains(&"node"));
        assert!(!ids.contains(&"gate"));
        assert!(!ids.contains(&"repo"));
        assert!(!ids.contains(&"subrepo"));
        assert!(!ids.contains(&"worktree"));

        // 启用 mcp-bootstrap 技能与 bque 层后才出现对应检查项
        let mut opts = BTreeMap::new();
        opts.insert("skills".to_string(), serde_json::json!(["mcp-bootstrap"]));
        write_manifest(&dir, &["common", "bque"], opts);
        let report2 = run_doctor(&dir, &DoctorOptions::default()).expect("应可自检");
        let ids2: Vec<&str> = report2.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids2.contains(&"bque"));
        assert!(ids2.contains(&"node"));
        assert!(!ids2.contains(&"cargo"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 语言优先级与英文模式零汉字() {
        let dir = temp_dir("lang-priority");
        let mut opts = BTreeMap::new();
        opts.insert("skill_lang".to_string(), serde_json::json!("en"));
        write_manifest(
            &dir,
            &["common", "gate", "agent", "subrepo", "worktree"],
            opts,
        );
        std::fs::create_dir_all(dir.join(".agents/skills")).expect("创建 .agents 失败");

        // manifest 设为 en，未显式覆盖 -> 应为 en 且全链路零汉字
        let report_en = run_doctor(&dir, &DoctorOptions::default()).expect("应可自检");
        assert_eq!(report_en.lang, "en");
        let rendered_en = format_doctor_report(&report_en, Lang::En);
        assert!(
            !contains_han(&rendered_en),
            "英文模式渲染输出不应含汉字:\n{rendered_en}"
        );
        for item in &report_en.items {
            assert!(
                !contains_han(&item.summary),
                "en summary 含汉字: {}",
                item.summary
            );
            for d in &item.details {
                assert!(!contains_han(&d.name), "en detail.name 含汉字: {}", d.name);
                assert!(
                    !contains_han(&d.message),
                    "en detail.message 含汉字: {}",
                    d.message
                );
                if let Some(r) = &d.remedy {
                    assert!(!contains_han(r), "en detail.remedy 含汉字: {r}");
                }
            }
        }

        // --lang zh 覆盖 manifest 的 en
        let report_zh = run_doctor(
            &dir,
            &DoctorOptions {
                fix: false,
                lang: Some(Lang::Zh),
            },
        )
        .expect("覆盖 zh 应可自检");
        assert_eq!(report_zh.lang, "zh");
        let rendered_zh = format_doctor_report(&report_zh, Lang::Zh);
        assert!(contains_han(&rendered_zh), "中文模式输出应含汉字");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate层只读告警且fix后激活钩子() {
        let dir = temp_dir("gate-fix");
        if !init_git_repo(&dir) {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }
        write_manifest(&dir, &["common", "gate"], BTreeMap::new());
        write_file(
            &dir.join(".githooks/pre-commit"),
            "#!/bin/sh\n# gate hook\nexit 0\n",
        );
        write_file(
            &dir.join(".githooks/commit-msg"),
            "#!/bin/sh\n# gate hook\nexit 0\n",
        );

        // 只读模式：未激活应记 Warn，不修改 git config
        let readonly = run_doctor(&dir, &DoctorOptions::default()).expect("只读自检应成功");
        let gate_item = readonly
            .items
            .iter()
            .find(|i| i.id == "gate")
            .expect("应包含 gate 项");
        assert_eq!(gate_item.level, DoctorLevel::Warn);
        assert_eq!(readonly.exit_code(), 0, "仅警告退出码应为 0");

        // --fix 模式：激活 core.hooksPath=.githooks
        let fixed = run_doctor(
            &dir,
            &DoctorOptions {
                fix: true,
                lang: None,
            },
        )
        .expect("修复自检应成功");
        let fixed_gate = fixed
            .items
            .iter()
            .find(|i| i.id == "gate")
            .expect("应包含 gate 项");
        assert_eq!(fixed_gate.level, DoctorLevel::Ok);
        let configured =
            pengj_gate::git::probe_in(&dir, &["config", "--local", "--get", "core.hooksPath"]);
        assert_eq!(configured.as_deref(), Some(".githooks"));

        // 放入竞品残留 .old 后应产生 Warn
        write_file(&dir.join(".githooks/pre-commit.old"), "# old\n");
        let with_rival = run_doctor(&dir, &DoctorOptions::default()).expect("竞品自检应成功");
        let rival_gate = with_rival
            .items
            .iter()
            .find(|i| i.id == "gate")
            .expect("应包含 gate 项");
        assert_eq!(rival_gate.level, DoctorLevel::Warn);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn agent层工作区链接只读报错并支持英文展开() {
        let dir = temp_dir("agent-repo");
        write_manifest(&dir, &["common", "agent"], BTreeMap::new());
        std::fs::create_dir_all(dir.join(".agents/skills")).expect("创建 .agents 失败");

        let report = run_doctor(
            &dir,
            &DoctorOptions {
                fix: false,
                lang: Some(Lang::En),
            },
        )
        .expect("agent 自检应成功");
        let repo_item = report
            .items
            .iter()
            .find(|i| i.id == "repo")
            .expect("存在 .agents 时应包含 repo 项");
        assert_eq!(repo_item.level, DoctorLevel::Error);
        assert!(!repo_item.details.is_empty());
        let text = format_doctor_report(&report, Lang::En);
        assert!(!contains_han(&text), "英文输出不应含汉字:\n{text}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn worktree层缺失报错且fix创建目录() {
        let dir = temp_dir("worktree-fix");
        write_manifest(&dir, &["common", "worktree"], BTreeMap::new());

        let before = run_doctor(&dir, &DoctorOptions::default()).expect("只读自检应成功");
        let wt_before = before
            .items
            .iter()
            .find(|i| i.id == "worktree")
            .expect("应包含 worktree 项");
        assert_eq!(wt_before.level, DoctorLevel::Error);
        assert_eq!(before.exit_code(), 1);

        let after = run_doctor(
            &dir,
            &DoctorOptions {
                fix: true,
                lang: None,
            },
        )
        .expect("修复自检应成功");
        let wt_after = after
            .items
            .iter()
            .find(|i| i.id == "worktree")
            .expect("应包含 worktree 项");
        assert_ne!(
            wt_after.level,
            DoctorLevel::Error,
            "fix 后共享编译目录应已创建: {:?}",
            wt_after
        );
        assert!(dir.join("target").is_dir());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn doctor_d脚本按字典序执行并注入环境变量() {
        let dir = temp_dir("doctor-d");
        write_manifest(&dir, &["common"], BTreeMap::new());
        let scripts_dir = dir.join(".pengj-templates/scripts/doctor.d");
        write_file(
            &scripts_dir.join("02-second.sh"),
            "#!/bin/sh\necho \"second fix=${DOCTOR_FIX:-unset}\"\nexit 1\n",
        );
        write_file(
            &scripts_dir.join("01-first.sh"),
            "#!/bin/sh\necho \"first root=$DOCTOR_ROOT fix=${DOCTOR_FIX:-unset}\"\nexit 0\n",
        );

        let readonly = run_doctor(&dir, &DoctorOptions::default()).expect("只读运行应成功");
        assert_eq!(readonly.custom_count, 2);
        let custom_items: Vec<&DoctorItem> = readonly
            .items
            .iter()
            .filter(|i| i.category == "custom")
            .collect();
        assert_eq!(custom_items.len(), 2);
        assert_eq!(custom_items[0].summary, "01-first.sh");
        assert_eq!(custom_items[0].level, DoctorLevel::Ok);
        assert_eq!(custom_items[1].summary, "02-second.sh");
        assert_eq!(custom_items[1].level, DoctorLevel::Error);
        assert_eq!(readonly.exit_code(), 1);

        let first_out = custom_items[0].output.as_deref().unwrap_or("");
        assert!(first_out.contains(&readonly.root), "应注入 DOCTOR_ROOT");
        assert!(
            first_out.contains("fix=unset"),
            "只读模式不应注入 DOCTOR_FIX"
        );

        let fixed = run_doctor(
            &dir,
            &DoctorOptions {
                fix: true,
                lang: None,
            },
        )
        .expect("fix 运行应成功");
        let first_fixed = fixed
            .items
            .iter()
            .find(|i| i.id == "custom:01-first.sh")
            .expect("应有 01-first.sh");
        assert!(
            first_fixed
                .output
                .as_deref()
                .unwrap_or("")
                .contains("fix=1"),
            "--fix 应注入 DOCTOR_FIX=1"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 退出码规则与json序列化键名全英文() {
        let report_warn = DoctorReport {
            root: "/tmp/demo".to_string(),
            lang: "en".to_string(),
            layers: vec!["common".to_string()],
            ok: true,
            checks: 2,
            errors: 0,
            warnings: 1,
            custom_count: 0,
            items: vec![DoctorItem {
                id: "node".to_string(),
                category: "tool".to_string(),
                level: DoctorLevel::Warn,
                summary: "node not found".to_string(),
                details: Vec::new(),
                output: None,
            }],
        };
        assert_eq!(report_warn.exit_code(), 0, "仅警告应返回 0");

        let report_err = DoctorReport {
            ok: false,
            errors: 1,
            ..report_warn.clone()
        };
        assert_eq!(report_err.exit_code(), 1, "存在错误应返回 1");

        let json_value = serde_json::to_value(&report_warn).expect("JSON 序列化应成功");
        let obj = json_value.as_object().expect("顶层应为 JSON 对象");
        for key in obj.keys() {
            assert!(!contains_han(key), "JSON 键名不应含汉字: {key}");
        }
        assert_eq!(obj.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(obj.get("warnings").and_then(|v| v.as_u64()), Some(1));
    }
}
