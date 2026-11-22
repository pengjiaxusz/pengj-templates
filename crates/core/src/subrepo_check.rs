//! 子仓库体检：把登记表里的每条登记翻译成「该对它做什么检查」
//!
//! 三件今天没有任何机制能查的事：
//!
//! 1. **嵌套工作区红线**——外部子仓库自带工作区定义、却没被宿主工作区排除时，
//!    宿主工作区会把它的成员当作自己的 path 成员，继承错误的包元数据，直接解析失败。
//!    检查要给出「把哪个路径加到排除清单」的可操作指引，而不是一句「配置有问题」。
//! 2. **子模块就绪状态**——子模块没初始化、或者实际指针与登记的上游分支已经漂移。
//! 3. **依赖模式自洽**——登记表里出现未知模式时报错，而不是静默跳过。
//!
//! 检查逻辑放在**单一实现**里（命令行工具），自检直接复用它——
//! 与提交门禁「只有一套实现」的既有原则一致。
//!
//! 退出码分级：`0` 全部通过（可含警告）、`1` 有失败项、`2` 只有警告项。
//! 「有失败」与「仅警告」必须能被调用方区分开，否则 CI 里无法选择是否阻断。

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{CoreError, Result};
use crate::i18n_audit::Lang;
use crate::subrepo::{DependencyMode, SubrepoEntry, SubrepoRegistry};

/// 单条检查的结论级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckLevel {
    /// 通过
    Pass,
    /// 警告：值得知道，但不该阻断
    Warn,
    /// 失败：必须处理
    Fail,
}

impl CheckLevel {
    /// 稳定的字面量（命令输出与 JSON 用）
    pub fn as_str(self) -> &'static str {
        match self {
            CheckLevel::Pass => "pass",
            CheckLevel::Warn => "warn",
            CheckLevel::Fail => "fail",
        }
    }
}

/// 一条检查结论
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckItem {
    /// 被检查的对象（子仓库名，或检查项本身的名字）
    pub name: String,
    /// 结论级别
    pub level: CheckLevel,
    /// 结论说明
    pub detail: String,
    /// 可操作的修复指引（没有则为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
}

impl CheckItem {
    fn pass(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: CheckLevel::Pass,
            detail: detail.into(),
            remedy: None,
        }
    }

    fn warn(name: impl Into<String>, detail: impl Into<String>, remedy: Option<String>) -> Self {
        Self {
            name: name.into(),
            level: CheckLevel::Warn,
            detail: detail.into(),
            remedy,
        }
    }

    fn fail(name: impl Into<String>, detail: impl Into<String>, remedy: Option<String>) -> Self {
        Self {
            name: name.into(),
            level: CheckLevel::Fail,
            detail: detail.into(),
            remedy,
        }
    }
}

/// 一次体检的完整报告
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckReport {
    /// 项目根
    pub root: String,
    /// 逐条结论
    pub items: Vec<CheckItem>,
}

impl CheckReport {
    /// 有失败项
    pub fn has_failures(&self) -> bool {
        self.items.iter().any(|i| i.level == CheckLevel::Fail)
    }

    /// 有警告项
    pub fn has_warnings(&self) -> bool {
        self.items.iter().any(|i| i.level == CheckLevel::Warn)
    }

    /// 退出码：`0` 全部通过，`1` 有失败，`2` 只有警告。
    ///
    /// 「有失败」与「仅警告」分两级，调用方才能决定是否阻断。
    pub fn exit_code(&self) -> i32 {
        if self.has_failures() {
            1
        } else if self.has_warnings() {
            2
        } else {
            0
        }
    }
}

/// 对给定项目做子仓库体检（默认中文输出）。
///
/// 登记表缺失时报错退出而不是「通过」：项目可能压根没勾子仓库层，
/// 但更可能是登记表被误删了——报「一切正常」比报错危险得多。
pub fn check_project(project_dir: &Path) -> Result<CheckReport> {
    check_project_with_lang(project_dir, Lang::Zh)
}

/// 按指定语言对给定项目做子仓库体检。
///
/// `Lang::En` 时所有条目名称、详情与修复建议均为英文且不含汉字，供 `doctor` 全链路双语复用。
pub fn check_project_with_lang(project_dir: &Path, lang: Lang) -> Result<CheckReport> {
    let root = absolute(project_dir, lang)?;
    let registry = SubrepoRegistry::load(&root)?.ok_or_else(|| {
        let message = match lang {
            Lang::Zh => format!(
                "未找到子仓库登记表: {}（该项目可能未勾选「子仓库」层）",
                crate::subrepo::SUBREPO_REGISTRY_REL
            ),
            Lang::En => format!(
                "sub-repository registry not found: {} (the project may not have the subrepo layer enabled)",
                crate::subrepo::SUBREPO_REGISTRY_REL
            ),
        };
        CoreError::SubrepoRegistry(message)
    })?;

    let mut items = Vec::new();

    // 登记表为空：没有可检查的对象，但也不算失败——空清单是「还没登记」，
    // 不是「登记坏了」。给一条警告，让人知道体检没覆盖到东西。
    if registry.entries.is_empty() {
        let (name, detail, remedy) = match lang {
            Lang::Zh => (
                "登记表",
                "登记表里没有任何子仓库条目，本次体检没有可检查的对象",
                format!(
                    "在 {} 里登记 [[subrepo]] 条目后再跑一次",
                    crate::subrepo::SUBREPO_REGISTRY_REL
                ),
            ),
            Lang::En => (
                "registry",
                "no sub-repository entries registered; nothing to check",
                format!(
                    "add [[subrepo]] entries to {} and run the check again",
                    crate::subrepo::SUBREPO_REGISTRY_REL
                ),
            ),
        };
        items.push(CheckItem::warn(name, detail, Some(remedy)));
        return Ok(CheckReport {
            root: display_path(&root),
            items,
        });
    }

    // 宿主工作区的排除清单：嵌套工作区红线要比对的就是它
    let host_excludes = HostWorkspace::detect(&root);

    for entry in &registry.entries {
        check_entry(&root, entry, &host_excludes, lang, &mut items);
    }

    Ok(CheckReport {
        root: display_path(&root),
        items,
    })
}

/// 宿主工作区的排除清单
struct HostWorkspace {
    /// 宿主工作区定义文件相对项目根的路径（没有则 None）
    definition: Option<String>,
    /// 被排除的路径（归一化为正斜杠、相对项目根）
    excludes: Vec<String>,
}

impl HostWorkspace {
    /// 探测宿主工作区定义与它的排除清单。
    ///
    /// 优先看 `Cargo.toml` 的 `[workspace] exclude`：Rust 工作区把外部仓库的成员
    /// 当作自己的 path 成员正是这条红线最常见的触发场景。
    /// 没有工作区定义的项目不判红线——没地方排除，也就没有红线可违反。
    fn detect(root: &Path) -> Self {
        let cargo = root.join("Cargo.toml");
        if !cargo.is_file() {
            return Self {
                definition: None,
                excludes: Vec::new(),
            };
        }
        let Ok(text) = std::fs::read_to_string(&cargo) else {
            return Self {
                definition: None,
                excludes: Vec::new(),
            };
        };
        let Ok(value) = text.parse::<toml::Value>() else {
            return Self {
                definition: None,
                excludes: Vec::new(),
            };
        };
        // 只有声明了 [workspace] 的 Cargo.toml 才是「宿主工作区」
        if value.get("workspace").is_none() {
            return Self {
                definition: None,
                excludes: Vec::new(),
            };
        }
        let excludes = value
            .get("workspace")
            .and_then(|w| w.get("exclude"))
            .and_then(|e| e.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(normalize_rel_path)
                    .collect()
            })
            .unwrap_or_default();
        Self {
            definition: Some("Cargo.toml".to_string()),
            excludes,
        }
    }

    /// 某条路径是否已被宿主工作区排除
    fn excludes(&self, path: &str) -> bool {
        let target = normalize_rel_path(path);
        self.excludes.iter().any(|excluded| {
            // 排除项可能写成目录（`submodules/foo`）或目录下的内容（`submodules/foo/`）
            let dir = excluded.trim_end_matches('/');
            target == dir || target.starts_with(&format!("{dir}/"))
        })
    }
}

/// 检查单条登记
fn check_entry(
    root: &Path,
    entry: &SubrepoEntry,
    host: &HostWorkspace,
    lang: Lang,
    items: &mut Vec<CheckItem>,
) {
    // --- 嵌套工作区红线 ---
    if entry.has_workspace {
        match &host.definition {
            None => {
                let detail = match lang {
                    Lang::Zh => "该子仓库自带工作区定义，但宿主项目没有工作区定义，无需排除",
                    Lang::En => {
                        "this sub-repository defines its own workspace, but the host project has no workspace; no exclusion needed"
                    }
                };
                items.push(CheckItem::pass(&entry.name, detail));
            }
            Some(definition) => {
                if host.excludes(&entry.path) {
                    let detail = match lang {
                        Lang::Zh => {
                            format!("该子仓库自带工作区定义，已被 {definition} 的工作区排除清单覆盖")
                        }
                        Lang::En => format!(
                            "this sub-repository defines its own workspace and is excluded in {definition}"
                        ),
                    };
                    items.push(CheckItem::pass(&entry.name, detail));
                } else {
                    let (detail, remedy) = match lang {
                        Lang::Zh => (
                            format!(
                                "该子仓库自带工作区定义，但未出现在 {definition} 的工作区排除清单里——\
                                 宿主工作区会把它的成员当作自己的 path 成员，继承错误的包元数据并解析失败"
                            ),
                            format!(
                                "把 \"{}\" 加到 {definition} 的 [workspace] exclude 列表里",
                                entry.path
                            ),
                        ),
                        Lang::En => (
                            format!(
                                "this sub-repository defines its own workspace, but is missing from [workspace] exclude in {definition} — \
                                 the host workspace will treat its members as its own path members and fail to resolve"
                            ),
                            format!(
                                "add \"{}\" to [workspace] exclude in {definition}",
                                entry.path
                            ),
                        ),
                    };
                    items.push(CheckItem::fail(&entry.name, detail, Some(remedy)));
                }
            }
        }
    }

    // --- 依赖模式相关的检查 ---
    match entry.mode {
        DependencyMode::Submodule => check_submodule(root, entry, lang, items),
        DependencyMode::Fetch => {
            let detail = match lang {
                Lang::Zh => "由宿主构建系统按版本拉取，版本锁定在构建系统里，无需指针检查",
                Lang::En => {
                    "fetched by version via the host build system; no git pointer check needed"
                }
            };
            items.push(CheckItem::pass(&entry.name, detail));
        }
        DependencyMode::Local => check_local(root, entry, lang, items),
    }
}

/// 检查子模块的就绪状态与指针漂移
fn check_submodule(root: &Path, entry: &SubrepoEntry, lang: Lang, items: &mut Vec<CheckItem>) {
    let target = root.join(&entry.path);

    // 未初始化：目录不存在，或目录存在但不是 git 工作树
    if !target.exists() {
        let (detail, remedy) = match lang {
            Lang::Zh => (
                format!("子模块未初始化：{} 不存在", entry.path),
                format!(
                    "运行 git submodule update --init --recursive -- {}",
                    entry.path
                ),
            ),
            Lang::En => (
                format!(
                    "submodule is not initialized: {} does not exist",
                    entry.path
                ),
                format!(
                    "run: git submodule update --init --recursive -- {}",
                    entry.path
                ),
            ),
        };
        items.push(CheckItem::fail(&entry.name, detail, Some(remedy)));
        return;
    }

    let status = git_output(root, &["submodule", "status", "--", &entry.path]);
    let Some(status) = status else {
        let (detail, remedy) = match lang {
            Lang::Zh => (
                "无法读取子模块状态（当前目录不是 git 工作树，或 git 不可用）",
                "在 git 工作树里重跑一次体检",
            ),
            Lang::En => (
                "cannot read submodule status (not a git work tree, or git is unavailable)",
                "re-run the check inside a git work tree",
            ),
        };
        items.push(CheckItem::warn(
            &entry.name,
            detail,
            Some(remedy.to_string()),
        ));
        return;
    };

    // git submodule status 的首字符标记状态：
    // `-` 未初始化、`+` 指针与索引不一致、`U` 冲突、空格表示正常。
    let Some(line) = status.lines().find(|l| !l.trim().is_empty()) else {
        let (detail, remedy) = match lang {
            Lang::Zh => (
                format!("子模块 {} 没有出现在 git submodule status 里", entry.path),
                format!(
                    "确认它已登记进 .gitmodules：git submodule update --init --recursive -- {}",
                    entry.path
                ),
            ),
            Lang::En => (
                format!("submodule {} is not listed in git submodule status", entry.path),
                format!(
                    "ensure it is registered in .gitmodules: git submodule update --init --recursive -- {}",
                    entry.path
                ),
            ),
        };
        items.push(CheckItem::warn(&entry.name, detail, Some(remedy)));
        return;
    };

    let (marker, rest) = line.split_at(1);
    match marker {
        "-" => {
            let (detail, remedy) = match lang {
                Lang::Zh => (
                    format!("子模块未初始化：{}", rest.trim()),
                    format!(
                        "运行 git submodule update --init --recursive -- {}",
                        entry.path
                    ),
                ),
                Lang::En => (
                    format!("submodule is not initialized: {}", rest.trim()),
                    format!(
                        "run: git submodule update --init --recursive -- {}",
                        entry.path
                    ),
                ),
            };
            items.push(CheckItem::fail(&entry.name, detail, Some(remedy)));
            return;
        }
        "U" => {
            let (detail, remedy) = match lang {
                Lang::Zh => (
                    format!("子模块存在合并冲突：{}", rest.trim()),
                    format!("在 {} 里解决冲突后重新提交", entry.path),
                ),
                Lang::En => (
                    format!("submodule has merge conflicts: {}", rest.trim()),
                    format!("resolve conflicts inside {} and commit again", entry.path),
                ),
            };
            items.push(CheckItem::fail(&entry.name, detail, Some(remedy)));
            return;
        }
        "+" => {
            let (detail, remedy) = match lang {
                Lang::Zh => (
                    format!(
                        "子模块指针与索引不一致：{}（本地检出与仓库记录不是同一个提交）",
                        rest.trim()
                    ),
                    format!(
                        "确认要保留哪个指针：git -C {} checkout <目标提交>，或 git add {} 记录当前指针",
                        entry.path, entry.path
                    ),
                ),
                Lang::En => (
                    format!(
                        "submodule pointer differs from index: {} (checked-out commit differs from recorded commit)",
                        rest.trim()
                    ),
                    format!(
                        "decide which commit to keep: git -C {} checkout <commit>, or git add {} to record the current commit",
                        entry.path, entry.path
                    ),
                ),
            };
            items.push(CheckItem::warn(&entry.name, detail, Some(remedy)));
            return;
        }
        _ => {}
    }

    // 指针漂移：实际检出的提交与登记的上游跟踪分支对不上
    let upstream = entry.upstream.as_deref().unwrap_or("");
    if upstream.is_empty() {
        let (detail, remedy) = match lang {
            Lang::Zh => (
                "登记表里没有登记 upstream，无法核对指针漂移",
                "在登记表里补上 upstream（上游跟踪分支）",
            ),
            Lang::En => (
                "upstream is not set in the registry; cannot check pointer drift",
                "add upstream (tracking branch) to the registry entry",
            ),
        };
        items.push(CheckItem::warn(
            &entry.name,
            detail,
            Some(remedy.to_string()),
        ));
        return;
    }

    // upstream 记的是「远程/分支」（如 origin/main），指针比对要落到具体提交上。
    // 远程引用在本地可能不存在（没 fetch 过），此时只报告存在性，不猜结论。
    let head = git_output(&target, &["rev-parse", "HEAD"]);
    let upstream_sha = git_output(&target, &["rev-parse", "--verify", "--quiet", upstream]);
    match (head, upstream_sha) {
        (Some(head), Some(upstream_sha)) => {
            let head = head.trim();
            let upstream_sha = upstream_sha.trim();
            if head == upstream_sha {
                let detail = match lang {
                    Lang::Zh => format!("子模块已就绪，指针与上游 {} 一致", upstream),
                    Lang::En => format!("submodule is ready and matches upstream {}", upstream),
                };
                items.push(CheckItem::pass(&entry.name, detail));
            } else {
                let (detail, remedy) = match lang {
                    Lang::Zh => (
                        format!(
                            "子模块指针与登记的上游 {} 已经漂移：本地 {}，上游 {}",
                            upstream,
                            short_sha(head),
                            short_sha(upstream_sha)
                        ),
                        format!(
                            "确认是否需要同步：git -C {} fetch && git -C {} log --oneline {}..{}",
                            entry.path, entry.path, upstream, upstream
                        ),
                    ),
                    Lang::En => (
                        format!(
                            "submodule pointer drifted from upstream {}: local {}, upstream {}",
                            upstream,
                            short_sha(head),
                            short_sha(upstream_sha)
                        ),
                        format!(
                            "check if sync is needed: git -C {} fetch && git -C {} log --oneline {}..{}",
                            entry.path, entry.path, upstream, upstream
                        ),
                    ),
                };
                items.push(CheckItem::warn(&entry.name, detail, Some(remedy)));
            }
        }
        (Some(_), None) => {
            let (detail, remedy) = match lang {
                Lang::Zh => (
                    format!(
                        "本地没有上游引用 {}，无法核对指针漂移（可能还没 fetch 过）",
                        upstream
                    ),
                    format!("先 fetch 再体检：git -C {} fetch", entry.path),
                ),
                Lang::En => (
                    format!(
                        "local upstream ref {} not found; cannot check pointer drift (try fetching first)",
                        upstream
                    ),
                    format!("fetch first and re-run: git -C {} fetch", entry.path),
                ),
            };
            items.push(CheckItem::warn(&entry.name, detail, Some(remedy)));
        }
        _ => {
            let detail = match lang {
                Lang::Zh => "无法读取子模块的提交号，无法核对指针漂移",
                Lang::En => "cannot read submodule commit hash; unable to check pointer drift",
            };
            items.push(CheckItem::warn(&entry.name, detail, None));
        }
    }
}

/// 检查本地同级目录模式的存在性
fn check_local(root: &Path, entry: &SubrepoEntry, lang: Lang, items: &mut Vec<CheckItem>) {
    let local_dir = entry.local_dir.as_deref().unwrap_or("");
    if local_dir.is_empty() {
        let (detail, remedy) = match lang {
            Lang::Zh => (
                "登记表里没有登记 local_dir，无法检查本地目录",
                "在登记表里补上 local_dir（本地同级目录）",
            ),
            Lang::En => (
                "local_dir is not set in the registry; cannot check local directory",
                "add local_dir (sibling directory path) to the registry entry",
            ),
        };
        items.push(CheckItem::fail(
            &entry.name,
            detail,
            Some(remedy.to_string()),
        ));
        return;
    }
    let target = root.join(local_dir);
    if target.is_dir() {
        let detail = match lang {
            Lang::Zh => format!("本地同级目录存在：{}", local_dir),
            Lang::En => format!("local sibling directory exists: {}", local_dir),
        };
        items.push(CheckItem::pass(&entry.name, detail));
    } else {
        let (detail, remedy) = match lang {
            Lang::Zh => (
                format!("本地同级目录不存在：{}", local_dir),
                format!("准备该目录，或把它检出到 {}", display_path(&target)),
            ),
            Lang::En => (
                format!("local sibling directory is missing: {}", local_dir),
                format!(
                    "prepare this directory or check it out at {}",
                    display_path(&target)
                ),
            ),
        };
        items.push(CheckItem::fail(&entry.name, detail, Some(remedy)));
    }
}

/// 执行 git 命令并取标准输出；失败返回 None
fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 取短提交号（便于人读）
fn short_sha(sha: &str) -> &str {
    let trimmed = sha.trim();
    if trimmed.len() > 8 {
        &trimmed[..8]
    } else {
        trimmed
    }
}

/// 归一化相对路径：正斜杠、去掉前导 `./` 与尾部 `/`
fn normalize_rel_path(path: &str) -> String {
    let mut text = path.trim().replace('\\', "/");
    while let Some(rest) = text.strip_prefix("./") {
        text = rest.to_string();
    }
    text.trim_end_matches('/').to_string()
}

fn absolute(root: &Path, lang: Lang) -> Result<PathBuf> {
    root.canonicalize().map_err(|error| {
        let message = match lang {
            Lang::Zh => format!("无法访问目录 {}: {error}", root.display()),
            Lang::En => format!("cannot access directory {}: {error}", root.display()),
        };
        CoreError::Git(message)
    })
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

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-subrepo-check-{}-{}-{}",
            tag,
            std::process::id(),
            line!()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn 缺失登记表时报错而不是报通过() {
        let dir = temp_dir("missing");
        let error = check_project(&dir).expect_err("登记表缺失必须报错");
        assert!(
            error.to_string().contains("登记表"),
            "报错应点明是登记表的问题: {error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 空登记表给出警告而不是失败() {
        let dir = temp_dir("empty");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "# 只有说明\n",
        );
        let report = check_project(&dir).expect("空登记表应能体检");
        assert!(!report.has_failures(), "空登记表不该判失败");
        assert!(report.has_warnings(), "空登记表应给出警告");
        assert_eq!(report.exit_code(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 嵌套工作区红线命中时报错并给出排除指引() {
        let dir = temp_dir("redline");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"ui\"\npath = \"submodules/ui\"\nmode = \"fetch\"\nhas_workspace = true\n",
        );
        // 宿主是工作区，但排除清单里没有这条
        write(
            &dir.join("Cargo.toml"),
            "[workspace]\nmembers = []\nexclude = [\"other\"]\n",
        );
        let report = check_project(&dir).expect("应能体检");
        assert!(report.has_failures(), "红线命中应判失败");
        let item = report
            .items
            .iter()
            .find(|i| i.level == CheckLevel::Fail)
            .expect("应有失败项");
        assert!(
            item.detail.contains("子模块") || item.detail.contains("工作区"),
            "失败说明应点明工作区: {}",
            item.detail
        );
        let remedy = item.remedy.as_ref().expect("红线失败必须给出修复指引");
        assert!(
            remedy.contains("submodules/ui") && remedy.contains("exclude"),
            "指引应说明把哪个路径加到排除清单: {remedy}"
        );
        assert_eq!(report.exit_code(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 嵌套工作区红线未命中时通过() {
        let dir = temp_dir("redline-ok");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"ui\"\npath = \"submodules/ui\"\nmode = \"fetch\"\nhas_workspace = true\n",
        );
        write(
            &dir.join("Cargo.toml"),
            "[workspace]\nmembers = []\nexclude = [\"submodules/ui\"]\n",
        );
        let report = check_project(&dir).expect("应能体检");
        assert!(
            !report.has_failures(),
            "已排除时不该判失败: {:?}",
            report.items
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 排除清单写成目录形式也认() {
        let dir = temp_dir("redline-dir");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"ui\"\npath = \"submodules/ui/crates\"\nmode = \"fetch\"\nhas_workspace = true\n",
        );
        write(
            &dir.join("Cargo.toml"),
            "[workspace]\nexclude = [\"submodules/ui/\"]\n",
        );
        let report = check_project(&dir).expect("应能体检");
        assert!(
            !report.has_failures(),
            "目录形式的排除项应覆盖其下的路径: {:?}",
            report.items
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 没有工作区定义的项目不判红线() {
        let dir = temp_dir("no-workspace");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"ui\"\npath = \"submodules/ui\"\nmode = \"fetch\"\nhas_workspace = true\n",
        );
        let report = check_project(&dir).expect("应能体检");
        assert!(!report.has_failures(), "没有宿主工作区就没有红线可违反");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 子模块未初始化时报告条目名与补齐命令() {
        let dir = temp_dir("submodule-missing");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"engine\"\npath = \"submodules/engine\"\nmode = \"submodule\"\nupstream = \"origin/main\"\n",
        );
        let report = check_project(&dir).expect("应能体检");
        assert!(report.has_failures(), "未初始化的子模块应判失败");
        let item = report
            .items
            .iter()
            .find(|i| i.level == CheckLevel::Fail)
            .expect("应有失败项");
        assert_eq!(item.name, "engine", "失败项应指明是哪个条目");
        assert!(item.detail.contains("未初始化"), "说明应点明未初始化");
        let remedy = item.remedy.as_ref().expect("必须给出补齐命令");
        assert!(
            remedy.contains("git submodule update --init"),
            "指引应给出补齐命令: {remedy}"
        );
        assert_eq!(report.exit_code(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 本地目录缺失时判失败() {
        let dir = temp_dir("local-missing");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"pe\"\npath = \"external/pe\"\nmode = \"local\"\nlocal_dir = \"../pe\"\n",
        );
        let report = check_project(&dir).expect("应能体检");
        assert!(report.has_failures(), "本地目录不存在应判失败");
        let item = report
            .items
            .iter()
            .find(|i| i.level == CheckLevel::Fail)
            .expect("应有失败项");
        assert!(
            item.detail.contains("本地同级目录"),
            "说明应点明是本地目录问题"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 本地目录存在时通过() {
        let dir = temp_dir("local-ok");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"pe\"\npath = \"external/pe\"\nmode = \"local\"\nlocal_dir = \"external/pe\"\n",
        );
        std::fs::create_dir_all(dir.join("external/pe")).unwrap();
        let report = check_project(&dir).expect("应能体检");
        assert!(
            !report.has_failures(),
            "本地目录存在应通过: {:?}",
            report.items
        );
        assert_eq!(report.exit_code(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 未知依赖模式在体检时报错而不是静默跳过() {
        let dir = temp_dir("bad-mode");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"x\"\npath = \"submodules/x\"\nmode = \"vendor-drop\"\n",
        );
        let error = check_project(&dir).expect_err("未知模式必须报错");
        assert!(
            error.to_string().contains("vendor-drop"),
            "报错应点明未知值: {error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 退出码分级区分失败与仅警告() {
        let dir = temp_dir("levels");
        // 只有警告：登记表为空
        write(&dir.join(crate::subrepo::SUBREPO_REGISTRY_REL), "# 空\n");
        assert_eq!(check_project(&dir).unwrap().exit_code(), 2, "仅警告应为 2");

        // 有失败：本地目录不存在
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"pe\"\npath = \"external/pe\"\nmode = \"local\"\nlocal_dir = \"external/pe\"\n",
        );
        assert_eq!(check_project(&dir).unwrap().exit_code(), 1, "有失败应为 1");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 路径归一化处理斜杠与前后缀() {
        assert_eq!(normalize_rel_path("./submodules/ui/"), "submodules/ui");
        assert_eq!(normalize_rel_path("submodules\\ui"), "submodules/ui");
        assert_eq!(normalize_rel_path("  submodules/ui  "), "submodules/ui");
    }

    #[test]
    fn 英文模式下体检明细与修复建议零汉字() {
        let dir = temp_dir("en-lang");
        write(
            &dir.join(crate::subrepo::SUBREPO_REGISTRY_REL),
            "[[subrepo]]\nname = \"ui\"\npath = \"submodules/ui\"\nmode = \"submodule\"\nupstream = \"origin/main\"\nhas_workspace = true\n",
        );
        write(
            &dir.join("Cargo.toml"),
            "[workspace]\nmembers = []\nexclude = []\n",
        );
        let report = check_project_with_lang(&dir, Lang::En).expect("英文模式应能体检");
        assert!(report.has_failures());
        for item in &report.items {
            assert!(
                !crate::contains_han(&item.name),
                "name 不应含汉字: {}",
                item.name
            );
            assert!(
                !crate::contains_han(&item.detail),
                "detail 不应含汉字: {}",
                item.detail
            );
            if let Some(remedy) = &item.remedy {
                assert!(!crate::contains_han(remedy), "remedy 不应含汉字: {remedy}");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
