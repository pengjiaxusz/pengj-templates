use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::context::RenderContext;
use crate::error::{CoreError, Result};
use crate::layer::{LayerInfo, LayerMeta, SkillInfo, LAYER_META_FILE};
use crate::manifest::ProjectManifest;
use crate::render::{is_text, render_text};

/// 模板根目录（磁盘上的 `templates/` 目录）。
///
/// 模板不再编译期嵌入二进制，而是运行时按该目录读取：
/// 增删层/文件无需重新编译，改完模板直接 re-run 即可。
#[derive(Debug, Clone)]
pub struct Templates {
    root: PathBuf,
}

impl Templates {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 递归收集某层目录下的所有文件，返回 层内相对路径（相对层根）-> 原始字节
    fn collect_files(&self, layer: &str) -> Result<Vec<(PathBuf, Vec<u8>)>> {
        let base = self.root.join(layer);
        let mut out = Vec::new();
        read_recursive(&base, &base, &mut out)?;
        Ok(out)
    }

    /// 各层元数据：层 id -> LayerMeta（模板根的直接子目录中带 layer.toml 的）
    fn layer_metas(&self) -> Result<BTreeMap<String, LayerMeta>> {
        let mut metas = BTreeMap::new();
        let dir = std::fs::read_dir(&self.root)?;
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let meta_path = path.join(LAYER_META_FILE);
            if !meta_path.exists() {
                continue;
            }
            let meta = parse_layer_meta(&std::fs::read(&meta_path)?)?;
            let id = path.file_name().unwrap().to_string_lossy().into_owned();
            metas.insert(id, meta);
        }
        Ok(metas)
    }

    /// 列出所有可用层（模板根的直接子目录中带 layer.toml 的）
    pub fn list_layers(&self) -> Result<Vec<LayerInfo>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let meta_path = path.join(LAYER_META_FILE);
            if !meta_path.exists() {
                continue;
            }
            let meta = parse_layer_meta(&std::fs::read(&meta_path)?)?;
            let id = path.file_name().unwrap().to_string_lossy().into_owned();
            let files = self.collect_files(&id)?;
            let file_count = files
                .iter()
                .filter(|(p, _)| p != Path::new(LAYER_META_FILE))
                .count();
            out.push(LayerInfo {
                id,
                name: meta.name,
                description: meta.description,
                depends: meta.depends,
                file_count,
            });
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    /// 列出各层可用的技能（`.agents/skills/<name>/SKILL.md`）
    ///
    /// 描述从 SKILL.md 的 frontmatter 解析：先按 zh 语言渲染模板，再取 `description`
    /// 字段（支持 `description: >` / `description: >-` 折叠块）。解析失败时回退为空
    /// 字符串，不阻断整个列表。
    pub fn list_skills(&self) -> Result<Vec<SkillInfo>> {
        let ctx = dummy_skill_ctx();
        let mut out: BTreeMap<String, String> = BTreeMap::new();
        for entry in std::fs::read_dir(&self.root)? {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            let skills_root = path.join(SKILLS_ROOT);
            if !skills_root.is_dir() {
                continue;
            }
            for sk in std::fs::read_dir(&skills_root)? {
                let sk_path = sk?.path();
                if !sk_path.is_dir() {
                    continue;
                }
                let Some(name) = sk_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                else {
                    continue;
                };
                let md = sk_path.join("SKILL.md");
                if !md.exists() {
                    continue;
                }
                let desc = std::fs::read_to_string(&md)
                    .ok()
                    .and_then(|text| render_text(&text, &ctx).ok())
                    .map(|rendered| parse_skill_description(&rendered))
                    .unwrap_or_default();
                out.insert(name, desc);
            }
        }
        Ok(out
            .into_iter()
            .map(|(name, description)| SkillInfo { name, description })
            .collect())
    }

    /// 解析选中的层，返回按依赖顺序排列的完整层列表（先依赖、后自己）
    pub fn resolve_layers(&self, selected: &[String]) -> Result<Vec<String>> {
        let metas = self.layer_metas()?;

        for id in selected {
            if !metas.contains_key(id) {
                return Err(CoreError::UnknownLayer(id.clone()));
            }
        }

        fn visit(
            id: &str,
            metas: &BTreeMap<String, LayerMeta>,
            order: &mut Vec<String>,
            visiting: &mut Vec<String>,
            visited: &mut Vec<String>,
        ) -> Result<()> {
            if visited.iter().any(|v| v == id) {
                return Ok(());
            }
            if visiting.iter().any(|v| v == id) {
                let mut cycle = visiting.clone();
                cycle.push(id.to_string());
                return Err(CoreError::CircularDependency(cycle));
            }
            visiting.push(id.to_string());
            let deps = metas.get(id).map(|m| m.depends.clone()).unwrap_or_default();
            for dep in &deps {
                if !metas.contains_key(dep) {
                    return Err(CoreError::UnknownLayer(dep.clone()));
                }
                visit(dep, metas, order, visiting, visited)?;
            }
            visiting.pop();
            visited.push(id.to_string());
            order.push(id.to_string());
            Ok(())
        }

        let mut order = Vec::new();
        let mut visiting = Vec::new();
        let mut visited = Vec::new();
        for id in selected {
            visit(id, &metas, &mut order, &mut visiting, &mut visited)?;
        }
        Ok(order)
    }
}

/// 解析 layer.toml（字节 -> 结构体）
fn parse_layer_meta(bytes: &[u8]) -> Result<LayerMeta> {
    let text = std::str::from_utf8(bytes).map_err(|e| CoreError::Render(e.to_string()))?;
    Ok(toml::from_str(text)?)
}

/// 递归读取 `base` 下（从 `dir` 开始）所有文件，写入相对 `base` 的路径与内容
fn read_recursive(base: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
        if path.is_dir() {
            read_recursive(base, &path, out)?;
        } else {
            out.push((rel, std::fs::read(&path)?));
        }
    }
    Ok(())
}

/// 定位模板根目录：env `PENGJ_TEMPLATES` > 当前目录 `templates/` > 可执行文件旁 `templates/`
///
/// 当前目录优先：开发时在仓库根运行即用最新模板；可执行文件旁用于安装打包后的随附模板。
pub fn default_templates_dir() -> Result<PathBuf> {
    if let Ok(p) = env::var("PENGJ_TEMPLATES") {
        let dir = PathBuf::from(p);
        if dir.is_dir() {
            return Ok(dir);
        }
        return Err(CoreError::TemplateRoot(dir.display().to_string()));
    }
    let cwd_dir = PathBuf::from("templates");
    if cwd_dir.is_dir() {
        return Ok(cwd_dir);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let dir = exe_dir.join("templates");
            if dir.is_dir() {
                return Ok(dir);
            }
        }
    }
    Err(CoreError::TemplateRoot(
        "（未找到模板目录，可设置 PENGJ_TEMPLATES）".to_string(),
    ))
}

/// 需要按层累加拼接而非后层覆盖的文件（如 `.gitignore`、`.gitattributes`）。
/// 每个层贡献一段，按依赖顺序拼接，每段前带来源层注释。
const ACCUMULATE_FILES: &[&str] = &[".gitignore", ".gitattributes"];

/// 技能目录：`.agents/skills/<name>/`。技能文件受 `options["skills"]` 过滤：
/// 未在列表中的技能整目录跳过（选项缺失时包含全部，向后兼容）。
const SKILLS_ROOT: &str = ".agents/skills";

/// 从选项解析选中的技能集合；`None` 表示未指定（包含全部技能，向后兼容）
fn selected_skills(options: &BTreeMap<String, serde_json::Value>) -> Option<BTreeSet<String>> {
    let arr = options.get("skills")?.as_array()?;
    Some(
        arr.iter()
            .filter_map(|x| x.as_str())
            .map(|s| s.to_string())
            .collect(),
    )
}

/// 判断相对路径是否属于 `.agents/skills/<name>/...`，是则返回技能名。
/// 跨平台按路径组件判断（Windows 相对路径分隔符为 `\`）。
fn skill_name_of(rel: &Path) -> Option<String> {
    let comps: Vec<Component> = rel
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();
    let is_skills_root = matches!(
        (comps.first(), comps.get(1)),
        (Some(Component::Normal(a)), Some(Component::Normal(b)))
            if *a == ".agents" && *b == "skills"
    );
    if !is_skills_root {
        return None;
    }
    match comps.get(2) {
        Some(Component::Normal(name)) => Some(name.to_string_lossy().into_owned()),
        _ => None,
    }
}

/// 是否为技能主文档 `<技能名>/SKILL.md`（相对 `.agents/skills/` 的直接子文件）。
///
/// 技能主文档有特殊的纳管策略：存量项目的 SKILL.md 常见「整文件全自定义」的
/// 旧形态（无托管块），纳管/更新时执行「接管」——框架插入 frontmatter 之后、
/// 原文整体下移为纳管过渡区（见 [`take_over_legacy_skill`]）。
fn is_skill_doc(rel: &Path) -> bool {
    rel.file_name().is_some_and(|n| n == "SKILL.md") && skill_name_of(rel).is_some()
}

/// 把文本拆为 `(frontmatter 含闭合行, 其余正文)`。
/// 非 `---` 开头或缺少闭合标记时返回 `("", 全文)`。假定 LF 行尾。
fn split_frontmatter(text: &str) -> (&str, &str) {
    let Some(rest) = text.strip_prefix("---") else {
        return ("", text);
    };
    // 找闭合 `---` 所在行（其前必为换行）
    let Some(rel_close) = rest.find("\n---") else {
        return ("", text);
    };
    let close_nl = 3 + rel_close;
    let body_start = text[close_nl + 1..]
        .find('\n')
        .map(|p| close_nl + 1 + p + 1)
        .unwrap_or(text.len());
    (&text[..body_start], &text[body_start..])
}

/// 接管存量全自定义技能主文档：
/// 1. 以模板渲染结果**整页为准**——frontmatter（含 description）与托管框架都
///    用模板的，直接覆盖用户自己的 frontmatter（如 commit 这类技能只在提交时
///    触发，description 无项目差异，统一由模板维护）
/// 2. 原正文（剥去其 frontmatter）整体下移到「纳管过渡区」注释之后——短暂
///    双流程，由用户把领域差异合并进上方骨架后删除过渡区
fn take_over_legacy_skill(disk_bytes: &[u8], rendered_bytes: &[u8]) -> Vec<u8> {
    let disk = String::from_utf8_lossy(disk_bytes);
    let rendered = String::from_utf8_lossy(rendered_bytes);
    // 用户 frontmatter 丢弃：description 等元信息以模板为准
    let (_, body) = split_frontmatter(&disk);

    let mut out = String::with_capacity(disk.len() + rendered.len() + 320);
    out.push_str(rendered.trim_end());
    out.push_str(
        "\n\n<!-- ⚠️ 纳管过渡区：以下为接管前的原始技能正文（暂时双流程）。\
         请把其中的领域差异合并进上方「项目专属提交流程与红线」，然后删除本段至文末。 -->\n\n",
    );
    let trimmed = body.trim();
    if !trimmed.is_empty() {
        out.push_str(trimmed);
        out.push('\n');
    }
    out.into_bytes()
}

/// 从 SKILL.md 的 frontmatter（`---` 与 `---` 之间）解析 `description` 字段。
/// 支持 `description: 值` 与 `description: >` / `>-` 折叠块。
fn parse_skill_description(text: &str) -> String {
    let Some(rest) = text.strip_prefix("---") else {
        return String::new();
    };
    let Some(end) = rest.find("\n---") else {
        return String::new();
    };
    let fm = &rest[..end];
    let mut lines = fm.lines();
    while let Some(line) = lines.next() {
        let Some(v) = line.trim_start().strip_prefix("description:") else {
            continue;
        };
        let v = v.trim();
        if v.is_empty() || matches!(v, ">" | ">-" | "|" | "|-") {
            let mut parts = Vec::new();
            for l2 in lines.by_ref() {
                if l2.trim().is_empty() {
                    continue;
                }
                if !l2.starts_with(' ') && !l2.starts_with('\t') {
                    break;
                }
                parts.push(l2.trim().to_string());
            }
            return parts.join(" ");
        }
        return v.to_string();
    }
    String::new()
}

/// 渲染技能模板做描述提取时的占位上下文（zh 语言、默认选项）
fn dummy_skill_ctx() -> RenderContext {
    let mut options = BTreeMap::new();
    for (k, v) in [
        ("skill_lang", serde_json::json!("zh")),
        ("commit_zh", serde_json::json!(true)),
        ("chinese_programming", serde_json::json!(false)),
        ("edition", serde_json::json!("2021")),
        ("channel", serde_json::json!("stable")),
        ("use_sccache", serde_json::json!(true)),
        ("use_lld", serde_json::json!(true)),
    ] {
        options.insert(k.to_string(), v);
    }
    RenderContext::new("project", vec!["agent".to_string()], options)
}

/// 需要做结构化 JSON 并集合并的文件。
/// 典型用途如 `package.json`：
/// - 生成时：各层按依赖顺序并集合并（后层覆盖同名依赖，用户自加项不冲突）
/// - 更新/纳管时：依赖类字段的同名包**版本以模板为准**（含模板未定版的 `latest`；
///   实际解析版本由项目 lockfile 锁定，升级走 `pnpm update --latest`）；
///   脚本与其余字段仍以用户为底、模板只补缺失，
///   用户自己加的库与其余标量（name/version 等）同样原样保留
const MERGE_JSON_FILES: &[&str] = &["package.json"];

/// package.json 里「并集合并」的字段（依赖与脚本）
const MERGE_JSON_UNION_KEYS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "scripts",
];

/// 并集字段中的「依赖类」字段：更新/纳管时同名包版本以模板为准（脚本除外）。
/// 用户显式钉住旧版本的需求让位于「依赖集合由模板统一维护」。
const MERGE_JSON_DEP_KEYS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

/// 模板渲染出的 VS Code 配置文件在项目内的相对路径。
///
/// `update_project` 在同步普通文件之外，还会把其中的 fileNesting 等配置合并写入
/// 项目根目录下的 `*.code-workspace` 文件，保证多文件夹工作区与单文件夹使用一致
/// 的资源管理器嵌套显示。
const VSCODE_SETTINGS_REL: &str = ".vscode/settings.json";

/// 模板文件分三类收集：普通（后层覆盖）、拼接（.gitignore）、JSON 并集（package.json）
///
/// 字段保持私有（模块内读取）；结构体本身 `pub` 是因为 `sync_workspace_file` 的公开
/// 签名需要引用它（供 CLI / Tauri 调用）。
pub struct FileMap {
    /// 普通文件：路径 -> 最终原始字节（未渲染，后层覆盖前层）
    normal: BTreeMap<PathBuf, Vec<u8>>,
    /// 拼接累加文件：路径 -> [(层名, 该层原始字节)]，保持依赖顺序
    concat: BTreeMap<PathBuf, Vec<(String, Vec<u8>)>>,
    /// JSON 并集文件：路径 -> [(层名, 该层原始字节)]，保持依赖顺序
    json: BTreeMap<PathBuf, Vec<(String, Vec<u8>)>>,
}

/// 按合并顺序收集模板文件，按类型分发到对应桶
impl Templates {
    fn build_file_map(
        &self,
        ordered: &[String],
        options: &BTreeMap<String, serde_json::Value>,
    ) -> Result<FileMap> {
        let mut fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
        // 技能过滤：未选中的技能整目录跳过（选项缺失时包含全部）
        let skills = selected_skills(options);
        for layer_id in ordered {
            let base = self.root.join(layer_id);
            if !base.is_dir() {
                return Err(CoreError::UnknownLayer(layer_id.clone()));
            }
            let mut files = Vec::new();
            read_recursive(&base, &base, &mut files)?;
            for (rel, bytes) in files.drain(..) {
                if rel == Path::new(LAYER_META_FILE) {
                    continue;
                }
                if let Some(name) = skill_name_of(&rel) {
                    if let Some(allowed) = &skills {
                        if !allowed.contains(&name) {
                            continue;
                        }
                    }
                }
                let rel_str = rel.to_str().unwrap_or("");
                if ACCUMULATE_FILES.contains(&rel_str) {
                    fm.concat
                        .entry(rel)
                        .or_default()
                        .push((layer_id.clone(), bytes));
                } else if MERGE_JSON_FILES.contains(&rel_str) {
                    fm.json
                        .entry(rel)
                        .or_default()
                        .push((layer_id.clone(), bytes));
                } else {
                    fm.normal.insert(rel, bytes);
                }
            }
        }
        Ok(fm)
    }

    /// 收集选中各层声明的更新黑名单（相对项目根路径）
    fn collect_update_ignores(&self, ordered: &[String]) -> Result<BTreeSet<PathBuf>> {
        let mut set = BTreeSet::new();
        for layer_id in ordered {
            let meta_path = self.root.join(layer_id).join(LAYER_META_FILE);
            let Ok(bytes) = std::fs::read(&meta_path) else {
                continue;
            };
            let meta = parse_layer_meta(&bytes)?;
            for p in meta.update_ignore {
                set.insert(PathBuf::from(p));
            }
        }
        Ok(set)
    }
}

/// 把 `incoming` 并按规则并入 `base`（JSON 并集合并）。
/// - 依赖类字段：生成与更新/纳管时同名包版本**一律以模板为准**
/// - 脚本字段：更新/纳管时同名键保留用户的（脚本内容是用户定制面），缺失才补
/// - 其余字段：仅当 `overwrite_other` 为真（生成时）或 base 缺该项时写入
fn merge_json(base: &mut serde_json::Value, incoming: &serde_json::Value, overwrite_other: bool) {
    let Some(obj) = incoming.as_object() else {
        if overwrite_other {
            *base = incoming.clone();
        }
        return;
    };
    for (k, v) in obj {
        if MERGE_JSON_UNION_KEYS.contains(&k.as_str()) {
            let base_map = base.get_mut(k).and_then(|x| x.as_object_mut());
            match base_map {
                Some(bm) => {
                    if let Some(iv) = v.as_object() {
                        let is_dep = MERGE_JSON_DEP_KEYS.contains(&k.as_str());
                        for (ik, ivv) in iv {
                            if overwrite_other || is_dep {
                                // 生成：层间覆盖；更新/纳管：依赖版本一律跟随模板
                                bm.insert(ik.clone(), ivv.clone());
                            } else {
                                // 更新语义：用户已有该键 → 保留用户值
                                bm.entry(ik.clone()).or_insert(ivv.clone());
                            }
                        }
                    }
                }
                None => {
                    base[k] = v.clone();
                }
            }
        } else if overwrite_other || base.get(k).is_none() {
            base[k] = v.clone();
        }
    }
}

/// 渲染整个文件映射：普通文件直接渲染；`.gitignore` 拼接各层；`package.json` 做 JSON 并集合并
///
/// 注意：普通文件的受管块合并不在这里做——adopt/update 需要结合磁盘状态区分
/// 「替换既有块」「追加进 legacy 文件」「TOML 结构化合并」并上报 needs_review，
/// 由两个入口各自调用 [`merge_managed_block`] / TOML 合并完成；generate 面向空目录，
/// 直接写渲染结果即可。
fn render_file_map(
    fm: &FileMap,
    ctx: &RenderContext,
    project_dir: &Path,
) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut out: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();

    for (rel, bytes) in &fm.normal {
        let rendered = render_bytes(bytes, ctx)?;
        out.insert(rel.clone(), rendered);
    }

    for (rel, parts) in &fm.concat {
        let mut merged: Vec<u8> = Vec::new();
        for (layer_id, bytes) in parts {
            merged.extend(format!("# --- {} 层 ---\n\n", layer_id).as_bytes());
            merged.extend(render_bytes(bytes, ctx)?);
            merged.push(b'\n');
        }
        out.insert(rel.clone(), merged);
    }

    for (rel, parts) in &fm.json {
        let path = project_dir.join(rel);
        // 文件已存在（更新）则以其为底，保留用户字段；否则（生成）以空为底、模板全量写出
        let overwrite_other = !path.exists();
        let mut base = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        for (_, bytes) in parts {
            let rendered = render_bytes(bytes, ctx)?;
            let incoming: serde_json::Value = serde_json::from_slice(&rendered)?;
            merge_json(&mut base, &incoming, overwrite_other);
        }
        let mut s = serde_json::to_string_pretty(&base)?;
        s.push('\n');
        out.insert(rel.clone(), s.into_bytes());
    }

    Ok(out)
}

/// 受管块文本合并方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagedTextMergeKind {
    /// 目标文件不存在、渲染结果非文本或不含受管块：原样使用渲染结果
    Fresh,
    /// 磁盘已有同风格托管块：仅替换块区间，块外内容保留
    Replaced,
    /// 磁盘是无托管块的既有文件（legacy）：模板托管块被追加到文件末尾
    Appended,
}

/// 受管块文本合并结果
pub(crate) struct ManagedTextMerge {
    /// 合并后的完整文件内容
    pub bytes: Vec<u8>,
    /// 本次合并采用的方式
    pub kind: ManagedTextMergeKind,
}

/// 受管块文本合并（纯文本版）：把 `incoming_text` 中的托管块并入 `target_text`。
///
/// - 目标含同风格托管块 → 仅替换该区间（`Replaced`）
/// - 其余情况 → 托管块追加到末尾（`Appended`）
fn merge_managed_block_texts(target_text: &str, incoming_text: &str) -> ManagedTextMerge {
    let Some(incoming) = crate::block::extract_managed_block(incoming_text) else {
        return ManagedTextMerge {
            bytes: incoming_text.as_bytes().to_vec(),
            kind: ManagedTextMergeKind::Fresh,
        };
    };
    let kind = match crate::block::extract_managed_block(target_text) {
        Some(t) if t.style == incoming.style => ManagedTextMergeKind::Replaced,
        _ => ManagedTextMergeKind::Appended,
    };
    let out = crate::block::replace_managed_block(target_text, incoming_text);
    ManagedTextMerge {
        bytes: out.into_bytes(),
        kind,
    }
}

/// 受管块原位合并：把模板渲染结果写回磁盘前，若目标文件已存在、渲染结果是文本且
/// 含受管块（`PENGJ_TEMPLATE_START`/`END`），则以磁盘文件为底、仅替换块区间，
/// 保留块外的用户自定义内容。
///
/// 规则：
/// - 目标文件不存在、渲染结果非文本、或渲染结果不含受管块 -> 原样返回渲染结果（`Fresh`）
/// - 磁盘文件读取失败（权限等）-> 回退为渲染结果（`Fresh`）
/// - 磁盘已有同风格受管块 -> 仅替换块区间（`Replaced`）
/// - 磁盘是无块的 legacy 文件 -> 追加到末尾（`Appended`，调用方应标记 needs_review）
///
/// 磁盘内容 UTF-8 校验失败时按 lossy 处理，不 panic
fn merge_managed_block(project_dir: &Path, rel: &Path, rendered: &[u8]) -> ManagedTextMerge {
    let rendered_text = String::from_utf8_lossy(rendered);
    if crate::block::extract_managed_block(&rendered_text).is_none() {
        return ManagedTextMerge {
            bytes: rendered.to_vec(),
            kind: ManagedTextMergeKind::Fresh,
        };
    }
    let path = project_dir.join(rel);
    if !path.is_file() {
        return ManagedTextMerge {
            bytes: rendered.to_vec(),
            kind: ManagedTextMergeKind::Fresh,
        };
    }
    let Ok(disk_bytes) = std::fs::read(&path) else {
        return ManagedTextMerge {
            bytes: rendered.to_vec(),
            kind: ManagedTextMergeKind::Fresh,
        };
    };
    // 磁盘文件非文本时不做合并，避免把二进制内容按 lossy 文本处理后回写导致损坏
    if !is_text(&disk_bytes) {
        return ManagedTextMerge {
            bytes: rendered.to_vec(),
            kind: ManagedTextMergeKind::Fresh,
        };
    }
    let disk_text = String::from_utf8_lossy(&disk_bytes).into_owned();
    merge_managed_block_texts(&disk_text, &rendered_text)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn render_bytes(bytes: &[u8], ctx: &RenderContext) -> Result<Vec<u8>> {
    if is_text(bytes) {
        Ok(render_text(&String::from_utf8_lossy(bytes), ctx)?.into_bytes())
    } else {
        Ok(bytes.to_vec())
    }
}

fn write_file(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

fn validate_project_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(CoreError::InvalidProjectName("项目名不能为空".to_string()));
    }
    if name == "." || name == ".." || name.contains(['/', '\\', ':', '<', '>', '"', '|', '?', '*'])
    {
        return Err(CoreError::InvalidProjectName(name.to_string()));
    }
    Ok(())
}

// ---------- 生成 ----------

#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerateReport {
    pub project_dir: String,
    pub layers: Vec<String>,
    pub files: Vec<String>,
}

/// 在 parent_dir 下生成 <project_name>/ 目录
///
/// `options`：生成时用户选定的选项（如 edition、use_sccache），会持久化到 manifest，
/// 供后续 `update` 用同一批选项重渲染。
pub fn generate(
    templates: &Templates,
    project_name: &str,
    selected: &[String],
    options: BTreeMap<String, serde_json::Value>,
    parent_dir: &Path,
) -> Result<GenerateReport> {
    validate_project_name(project_name)?;
    let ordered = templates.resolve_layers(selected)?;
    let target = parent_dir.join(project_name);

    if target.exists() {
        let non_empty = std::fs::read_dir(&target)?.next().is_some();
        if non_empty {
            return Err(CoreError::DirExists(target.display().to_string()));
        }
    } else {
        std::fs::create_dir_all(&target)?;
    }

    let ctx = RenderContext::new(project_name, ordered.clone(), options.clone());
    let fm = templates.build_file_map(&ordered, &options)?;
    let bytes_map = render_file_map(&fm, &ctx, &target)?;
    let mut manifest = ProjectManifest {
        tool: "pengj-templates".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        project_name: project_name.to_string(),
        layers: ordered.clone(),
        options,
        generated_at: now_rfc3339(),
        files: BTreeMap::new(),
    };

    let mut written = Vec::new();
    for (rel, bytes) in &bytes_map {
        let path = target.join(rel);
        write_file(&path, bytes)?;
        let rel_str = rel.to_string_lossy().into_owned();
        manifest.files.insert(rel_str.clone(), sha256_hex(bytes));
        written.push(rel_str);
    }

    manifest.save(&target)?;

    Ok(GenerateReport {
        project_dir: target.display().to_string(),
        layers: ordered,
        files: written,
    })
}

// ---------- 存量纳管 (Adopt) ----------

/// `commitlint.config.js` 接线到 `commitlint.base.js` 的结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitlintWireOutcome {
    /// 本次完成自动接线（config 文件已改写）
    Wired,
    /// config 已引用 base，无需处理
    AlreadyWired,
    /// 项目没有 commitlint.config.js，无需接线
    NoConfig,
    /// 无法安全改写的形态（非 ESM、`export default` 后不是对象字面量等），
    /// 需要人工接线
    ManualRequired,
}

/// 把既有 `commitlint.config.js` 自动接线到 `commitlint.base.js`。
///
/// 仅保守处理可安全改写的 ESM 对象字面量形态（首个 `export default` 后紧跟 `{`）：
/// - 顶部补 `import base from './commitlint.base.js';`
/// - 原 `export default` 改名为 `const pengjUserConfig =`
/// - 末尾追加 `export default { ...base, ...pengjUserConfig, rules: { ...base.rules,
///   ...pengjUserConfig.rules } }` —— base 规则铺底、项目专属规则同名覆盖，
///   此后模板对 base 的更新才能真正生效
/// - 接线的同时做**去重**：与 base 等价（规范化比较）的 rules 条目及其他成员
///   （如相同的 `extends`）从用户配置中删除，只保留真正的项目差异
///
/// 其余形态（CJS `module.exports`、工厂函数等）一律不动文件，返回
/// [`CommitlintWireOutcome::ManualRequired`] 由调用方给出手动指引。
fn try_wire_commitlint_base(project_dir: &Path) -> Result<CommitlintWireOutcome> {
    let path = project_dir.join("commitlint.config.js");
    if !path.is_file() {
        return Ok(CommitlintWireOutcome::NoConfig);
    }
    let text = String::from_utf8_lossy(&std::fs::read(&path)?).into_owned();
    if text.contains("commitlint.base") {
        return Ok(CommitlintWireOutcome::AlreadyWired);
    }
    let Some(idx) = text.find("export default") else {
        return Ok(CommitlintWireOutcome::ManualRequired);
    };
    // `export default` 后必须直接跟对象字面量（跳过空白），否则改写不安全
    let rest = text[idx + "export default".len()..]
        .trim_start()
        .chars()
        .next();
    if rest != Some('{') {
        return Ok(CommitlintWireOutcome::ManualRequired);
    }

    let mut out = String::with_capacity(text.len() + 256);
    out.push_str("import base from './commitlint.base.js';\n");
    if !text.starts_with(['\n', '\r']) {
        out.push('\n');
    }
    out.push_str(&text.replacen("export default", "const pengjUserConfig =", 1));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(
        "\nexport default {\n  ...base,\n  ...pengjUserConfig,\n  rules: {\n    ...base.rules,\n    ...pengjUserConfig.rules,\n  },\n};\n",
    );

    // 去重：无法安全比较时保持原样（保守）
    let out = dedupe_wired_config(project_dir, &out).unwrap_or(out);

    std::fs::write(&path, out)?;
    Ok(CommitlintWireOutcome::Wired)
}

/// 定位第一个平衡花括号对象 `{...}` 的 `(起始下标, 结束下标（含花括号）)`，
/// 字符串字面量整体消费。未找到配对时返回 `None`。
fn first_braced_object_span(text: &str) -> Option<(usize, usize)> {
    let b = text.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'{' {
            let first_open = i;
            let mut depth: usize = 0;
            let mut escaped = false;
            let mut in_string: Option<char> = None;
            while i < b.len() {
                let c = b[i] as char;
                if let Some(q) = in_string {
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == q {
                        in_string = None;
                    }
                } else {
                    match c {
                        '"' | '\'' => in_string = Some(c),
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some((first_open, i + 1));
                            }
                        }
                        _ => {}
                    }
                }
                i += 1;
            }
            return None;
        }
        i += 1;
    }
    None
}

/// 接线去重：把 `wired` 文本中与 base.js 等价的用户成员/规则条目删除。
/// 任一步无法安全解析时返回 `None`（调用方保持原文本不改动）。
fn dedupe_wired_config(project_dir: &Path, wired: &str) -> Option<String> {
    let base_text = std::fs::read_to_string(project_dir.join("commitlint.base.js")).ok()?;
    let (base_open, base_close) = first_braced_object_span(&base_text)?;
    let base_inner = base_text.get(base_open + 1..base_close - 1)?;
    // base 对象的成员名 -> 规范化全文（如 extends），以及 rules 条目名 -> 规范化值
    let mut base_member_norm: Vec<(String, String)> = Vec::new();
    let mut base_rules_norm: Vec<(String, String)> = Vec::new();
    for m in split_object_members(base_inner)? {
        if m.name == "rules" {
            let (open, close) = first_braced_object_span(&m.source)?;
            let rules_inner = m.source.get(open + 1..close - 1)?;
            for r in split_object_members(rules_inner)? {
                // 条目规范化只取值部分（去掉 `key:` 前缀），键名单独存
                let (_, value) = r.source.split_once(':')?;
                let norm_value = normalize_js_literal(value)?;
                base_rules_norm.push((r.name.clone(), norm_value));
            }
        } else {
            base_member_norm.push((m.name.clone(), m.normalized.clone()));
        }
    }

    // 用户侧 pengjUserConfig 对象
    let marker_pos = wired.find("const pengjUserConfig")?;
    let (obj_open, obj_close) = first_braced_object_span(&wired[marker_pos..])?;
    let obj_open = marker_pos + obj_open;
    let obj_close = marker_pos + obj_close;
    let user_inner = wired.get(obj_open + 1..obj_close - 1)?;
    let members = split_object_members(user_inner)?;

    let mut kept_sources: Vec<String> = Vec::new();
    let mut changed = false;
    for m in &members {
        if m.name == "rules" {
            // rules：条目级去重——与 base 同名且等价的条目删除，其余保留作覆盖
            let (open, close) = first_braced_object_span(&m.source)?;
            let rules_inner = m.source.get(open + 1..close - 1)?;
            let entries = split_object_members(rules_inner)?;
            let kept: Vec<&JsObjectEntry> = entries
                .iter()
                .filter(|e| {
                    let Some((_, value)) = e.source.split_once(':') else {
                        return true;
                    };
                    match normalize_js_literal(value) {
                        Some(nv) => !base_rules_norm
                            .iter()
                            .any(|(n, v)| *n == e.name && *v == nv),
                        None => true,
                    }
                })
                .collect();
            if kept.len() != entries.len() {
                changed = true;
            }
            let rebuilt_rules = if kept.is_empty() {
                "rules: {}".to_string()
            } else {
                format!(
                    "rules: {{\n    {}\n  }}",
                    kept.iter()
                        .map(|e| e.source.clone())
                        .collect::<Vec<_>>()
                        .join(",\n    ")
                )
            };
            kept_sources.push(rebuilt_rules);
        } else if base_member_norm
            .iter()
            .any(|(n, v)| *n == m.name && *v == m.normalized)
        {
            // 其他成员与 base 等价（如相同的 extends）：整员去重
            changed = true;
        } else {
            kept_sources.push(m.source.clone());
        }
    }
    if !changed {
        return None;
    }

    let rebuilt = format!(
        "{}const pengjUserConfig = {{\n  {}\n}}{}",
        &wired[..marker_pos],
        kept_sources.join(",\n  "),
        &wired[obj_close..]
    );
    Some(rebuilt)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AdoptReport {
    pub project_name: String,
    pub layers: Vec<String>,
    pub created: Vec<String>,
    pub adopted: Vec<String>,
    /// TOML 结构化合并发现同名键值冲突、未写盘的文件
    pub conflicted: Vec<ConflictInfo>,
    /// 模板内容被追加/并入既有 legacy 文件、建议人工复核去重的路径
    pub needs_review: Vec<String>,
    /// 需要用户手动完成的接线步骤（如既有 commitlint.config.js 接入 base）
    pub manual_steps: Vec<String>,
}

/// 纳管存量项目：为已有目录初始化 `.pengj-templates.json` manifest
///
/// 逻辑：
/// 1. 若项目根目录已存在 manifest，且没有指定 force，则返回错误。
/// 2. 解析所选层与渲染选项，在内存中渲染出完整的模板 FileMap。
/// 3. 对模板渲染出的每个文件：
///    - 若本地文件已存在：
///      - TOML 受管文件（`.cargo/config.toml`）：按表级并集 + 键级去重做结构化合并，
///        同键不同值报冲突不写盘；
///      - 其余含受管块（`PENGJ_TEMPLATE_START`/`END`）的文本文件：把模板受管块
///        注入/替换进磁盘文件，块外用户内容原样保留；追加进无块 legacy 文件时计入
///        `needs_review` 提示人工复核；
///      - 记录合并后（或模板渲染）哈希作为初始基线，计入 `adopted`。
///    - 若本地文件不存在：
///      - 写入新文件，记录新哈希，计入 `created`。
/// 4. 将层列表、持久化选项和文件哈希基线写入 `.pengj-templates.json`。
pub fn adopt_project(
    templates: &Templates,
    project_dir: &Path,
    selected: &[String],
    options: BTreeMap<String, serde_json::Value>,
    force: bool,
) -> Result<AdoptReport> {
    if !project_dir.exists() || !project_dir.is_dir() {
        return Err(CoreError::DirExists(format!(
            "目录不存在或不是目录: {}",
            project_dir.display()
        )));
    }

    let manifest_path = project_dir.join(crate::manifest::MANIFEST_FILE);
    if manifest_path.exists() && !force {
        return Err(CoreError::DirExists(format!(
            "目录已存在 {}，如需重新纳管请使用 force 选项",
            crate::manifest::MANIFEST_FILE
        )));
    }

    let project_name = project_dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());

    let ordered = templates.resolve_layers(selected)?;
    let ctx = RenderContext::new(&project_name, ordered.clone(), options.clone());
    let fm = templates.build_file_map(&ordered, &options)?;
    let bytes_map = render_file_map(&fm, &ctx, project_dir)?;
    let ignores = templates.collect_update_ignores(&ordered)?;

    let mut created = Vec::new();
    let mut adopted = Vec::new();
    let mut conflicted = Vec::new();
    let mut needs_review = Vec::new();
    let mut manifest_files = BTreeMap::new();

    for (rel, bytes) in &bytes_map {
        let rel_str = rel.to_string_lossy().into_owned();
        let target = project_dir.join(rel);

        if ignores.contains(rel) {
            if !target.exists() {
                write_file(&target, bytes)?;
                created.push(rel_str.clone());
            }
            continue;
        }

        // VS Code settings：项目根已有 `*.code-workspace` 时，模板配置并入工作区文件
        // 的 `settings` 节点，不再新建独立 `.vscode/settings.json`（工作区同步复用
        // update 同一语义；settings 相对路径计入 adopted 代表「配置已同步」）
        if rel == Path::new(VSCODE_SETTINGS_REL) {
            let workspaces = list_workspace_files(project_dir);
            if !workspaces.is_empty() {
                let mut ws_changed = false;
                for ws_path in workspaces {
                    ws_changed |= sync_workspace_file(&ws_path, &ctx, &fm, project_dir)?;
                }
                if ws_changed {
                    needs_review.push(format!(
                        "{rel_str}：VS Code 配置已并入 *.code-workspace 文件的 settings 节点，请复核"
                    ));
                }
                adopted.push(rel_str);
                continue;
            }
        }

        if target.exists() {
            let cur = std::fs::read(&target).unwrap_or_default();
            let rendered_has_block =
                crate::block::extract_managed_block(&String::from_utf8_lossy(bytes)).is_some();

            // JSON 并集文件（如 package.json）：bytes 已由 render_file_map 以磁盘为底、
            // 用户优先语义合并完成，这里直接写回，不让合并结果滞留到首次 update
            if fm.json.contains_key(rel) {
                if target.is_file() {
                    if bytes != &cur {
                        write_file(&target, bytes)?;
                    }
                } else {
                    write_file(&target, bytes)?;
                    created.push(rel_str.clone());
                }
                manifest_files.insert(rel_str.clone(), sha256_hex(bytes));
                adopted.push(rel_str);
                continue;
            }

            // 普通文件（fm.normal）且渲染结果含受管块：结合磁盘状态做合并注入。
            // 注意：concat 累加文件（.gitignore/.gitattributes）不走此分支，保持旧语义
            // （记录模板渲染哈希、不覆盖用户文件），避免多受管块拼接丢内容。
            if fm.normal.contains_key(rel)
                && target.is_file()
                && is_text(&cur)
                && rendered_has_block
            {
                if crate::toml_merge::is_toml_managed(rel) {
                    // TOML 结构化受管合并：表级并集、键级去重、冲突跳过
                    let disk_text = String::from_utf8_lossy(&cur);
                    match crate::toml_merge::merge_toml_managed(
                        &disk_text,
                        &String::from_utf8_lossy(bytes),
                    ) {
                        crate::toml_merge::TomlMergeOutcome::Merged(text) => {
                            let merged_bytes = text.into_bytes();
                            if merged_bytes != cur {
                                write_file(&target, &merged_bytes)?;
                            }
                            if !cur.is_empty()
                                && crate::block::extract_managed_block(&disk_text).is_none()
                            {
                                needs_review.push(format!(
                                    "{rel_str}：模板配置已结构化并入受管块，请复核与用户既有配置的等价性"
                                ));
                            }
                            manifest_files.insert(rel_str.clone(), sha256_hex(&merged_bytes));
                        }
                        crate::toml_merge::TomlMergeOutcome::Conflict(reason) => {
                            // 冲突不写盘；基线记模板渲染哈希，后续 update 保持冲突保护
                            conflicted.push(ConflictInfo {
                                path: rel_str.clone(),
                                reason,
                            });
                            manifest_files.insert(rel_str.clone(), sha256_hex(bytes));
                        }
                    }
                    adopted.push(rel_str);
                    continue;
                }

                let legacy_without_block =
                    crate::block::extract_managed_block(&String::from_utf8_lossy(&cur)).is_none();

                // 技能主文档为全自定义 legacy 形态（无托管块且非空）时执行「接管」：
                // 模板框架插入 frontmatter 之后，原全文整体下移到纳管过渡区（归用户）。
                // 短暂存在双流程，由用户把领域差异合并进项目专属区后删除过渡区；
                // 接管后文件含托管块、入托管清单，后续 update 走正常的块替换路径。
                if legacy_without_block
                    && is_skill_doc(rel)
                    && !String::from_utf8_lossy(&cur).trim().is_empty()
                {
                    let merged_bytes = take_over_legacy_skill(&cur, bytes);
                    write_file(&target, &merged_bytes)?;
                    manifest_files.insert(rel_str.clone(), sha256_hex(&merged_bytes));
                    needs_review.push(format!(
                        "{rel_str}：已接管——模板框架已插入 frontmatter 之后，原全文下移至纳管过渡区\
                         （暂时双流程）。请把领域差异合并进上方项目专属区后删除过渡区"
                    ));
                    adopted.push(rel_str);
                    continue;
                }

                let merged = merge_managed_block(project_dir, rel, bytes);
                if merged.bytes != cur {
                    write_file(&target, &merged.bytes)?;
                }
                if merged.kind == ManagedTextMergeKind::Appended
                    && legacy_without_block
                    && !cur.is_empty()
                {
                    needs_review.push(format!(
                        "{rel_str}：模板托管块已追加到既有文件末尾，请人工检查是否与原有内容重复"
                    ));
                }
                manifest_files.insert(rel_str.clone(), sha256_hex(&merged.bytes));
                adopted.push(rel_str);
                continue;
            }

            // 模板无受管块（或 concat/非文本文件）：manifest 记录模板渲染的哈希，
            // 这样后续 update 时，引擎能检测到 disk_sha != recorded_sha，从而触发
            // 冲突保护并跳过覆盖，绝不发生静默全量覆盖。
            manifest_files.insert(rel_str.clone(), sha256_hex(bytes));
            adopted.push(rel_str);
        } else {
            write_file(&target, bytes)?;
            manifest_files.insert(rel_str.clone(), sha256_hex(bytes));
            created.push(rel_str);
        }
    }

    // commitlint 自动接线：base 落地后（本轮新建或既有），把未接线的既有
    // commitlint.config.js 自动改写为继承 base；无法安全改写的形态保留手动指引
    let mut manual_steps = Vec::new();
    if project_dir.join("commitlint.base.js").is_file() {
        match try_wire_commitlint_base(project_dir)? {
            CommitlintWireOutcome::Wired => {
                needs_review.push(
                    "commitlint.config.js：已自动接入 commitlint.base.js（base 规则铺底、项目规则同名覆盖，等价重复项已去除），请复核规则优先级"
                        .to_string(),
                );
            }
            CommitlintWireOutcome::ManualRequired => {
                manual_steps.push(
                    "检测到既有 commitlint.config.js 未继承 commitlint.base.js：\
                     请在其顶部加 `import base from './commitlint.base.js';`，\
                     并改为 `export default { ...base, rules: { ...base.rules, /* 项目专属规则 */ } }`。\
                     接入后模板对 commitlint.base.js 的更新才会生效。"
                        .to_string(),
                );
            }
            CommitlintWireOutcome::AlreadyWired | CommitlintWireOutcome::NoConfig => {}
        }
    }

    let manifest = ProjectManifest {
        tool: "pengj-templates".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        project_name: project_name.clone(),
        layers: ordered.clone(),
        options,
        generated_at: now_rfc3339(),
        files: manifest_files,
    };
    manifest.save(project_dir)?;

    Ok(AdoptReport {
        project_name,
        layers: ordered,
        created,
        adopted,
        conflicted,
        needs_review,
        manual_steps,
    })
}

// ---------- 更新 ----------

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConflictInfo {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateReport {
    pub project_name: String,
    pub layers: Vec<String>,
    pub updated: Vec<String>,
    pub created: Vec<String>,
    pub conflicted: Vec<ConflictInfo>,
    pub removed: Vec<String>,
    /// 模板内容被追加/并入既有 legacy 文件、建议人工复核去重的路径
    pub needs_review: Vec<String>,
    pub unchanged: usize,
}

/// 按 manifest 记录把模板的最新内容同步到已生成的项目
///
/// 规则：
/// - 模板文件内容未变 -> 跳过
/// - 模板变了、本地文件未动过 -> 覆盖
/// - 模板变了、本地文件被改过 -> 冲突，跳过并上报
/// - 含受管块的文本文件 -> 合并注入（替换既有块 / 追加进 legacy 文件并报 needs_review）
/// - `.cargo/config.toml` 等 TOML 受管文件 -> 结构化合并（表级并集、键级去重、冲突跳过并上报）
/// - `package.json` -> JSON 并集合并，同名键以用户为准、模板只补缺失
/// - 模板新增文件 -> 创建（本地已有同名文件则跳过并上报）
/// - 模板删除文件 -> 不动本地文件，仅上报
/// - 层声明的 `update_ignore` 黑名单文件 -> 完全跳过（不覆盖/不冲突/不删除上报），归用户所有
pub fn update_project(templates: &Templates, project_dir: &Path) -> Result<UpdateReport> {
    let mut manifest = ProjectManifest::load(project_dir)?;
    let ordered = templates.resolve_layers(&manifest.layers)?;
    let ctx = RenderContext::new(
        &manifest.project_name,
        ordered.clone(),
        manifest.options.clone(),
    );
    let fm = templates.build_file_map(&ordered, &manifest.options)?;
    let bytes_map = render_file_map(&fm, &ctx, project_dir)?;
    let ignores = templates.collect_update_ignores(&ordered)?;

    let mut updated = Vec::new();
    let mut created = Vec::new();
    let mut conflicted = Vec::new();
    let mut needs_review = Vec::new();
    let mut unchanged = 0usize;
    let mut new_files: BTreeMap<String, String> = BTreeMap::new();
    // package.json 等 JSON 并集文件走独立合并分支，不参与普通「哈希/冲突」判定
    let json_merge_keys: BTreeSet<&Path> = fm.json.keys().map(|p| p.as_path()).collect();

    for (rel, bytes) in &bytes_map {
        // 黑名单文件：仅首次生成时写入，之后归用户所有，模板不再接管
        if ignores.contains(rel) {
            continue;
        }
        let rel_str = rel.to_string_lossy().into_owned();
        let new_sha = sha256_hex(bytes);

        let target = project_dir.join(rel);

        // JSON 并集文件：合并结果按构造保留了用户字段（同名键用户优先），直接写回，
        // 不再冲突跳过
        if json_merge_keys.contains(rel.as_path()) {
            let target_bytes = std::fs::read(&target).unwrap_or_default();
            if target_bytes == *bytes {
                unchanged += 1;
            } else {
                write_file(&target, bytes)?;
                updated.push(rel_str.clone());
            }
            new_files.insert(rel_str, new_sha);
            continue;
        }

        // `.vscode/settings.json`：增量合并（与 `.code-workspace` 同一语义），不走
        // SHA 冲突判定——模板的 fileNesting / rust-analyzer.clippy 并入用户文件，
        // 用户自定义规则不丢失且能收到模板新规则。
        // 项目根已有 `*.code-workspace` 且尚无独立 settings 文件时：配置并入工作区
        // 文件（循环末统一执行），不再新建 `.vscode/settings.json`。
        if rel == Path::new(VSCODE_SETTINGS_REL) {
            if !target.exists() && !list_workspace_files(project_dir).is_empty() {
                continue;
            }
            match sync_settings_file(&target, &ctx, &fm, project_dir)? {
                SettingsSyncOutcome::Updated => updated.push(rel_str.clone()),
                SettingsSyncOutcome::Created => created.push(rel_str.clone()),
                SettingsSyncOutcome::Unchanged => unchanged += 1,
            }
            // 记录写盘后（或现有）内容的 sha，避免下次误判冲突
            let disk_sha = std::fs::read(&target)
                .ok()
                .map(|b| sha256_hex(&b))
                .unwrap_or(new_sha);
            new_files.insert(rel_str, disk_sha);
            continue;
        }

        // TOML 结构化受管合并（如 `.cargo/config.toml`）：表级并集 + 键级去重 +
        // 冲突跳过。文本追加对 TOML 不安全——同名表重复定义是非法 TOML，会让 cargo
        // 直接解析失败，因此这类文件不走下面的通用受管块分支。
        if fm.normal.contains_key(rel) && crate::toml_merge::is_toml_managed(rel) {
            if !target.is_file() {
                // 项目尚无该文件：直接写渲染结果
                write_file(&target, bytes)?;
                new_files.insert(rel_str.clone(), new_sha);
                created.push(rel_str);
                continue;
            }
            let cur = std::fs::read(&target).unwrap_or_default();
            if is_text(&cur)
                && crate::block::extract_managed_block(&String::from_utf8_lossy(bytes)).is_some()
            {
                let disk_text = String::from_utf8_lossy(&cur);
                match crate::toml_merge::merge_toml_managed(
                    &disk_text,
                    &String::from_utf8_lossy(bytes),
                ) {
                    crate::toml_merge::TomlMergeOutcome::Merged(text) => {
                        let merged_bytes = text.into_bytes();
                        if merged_bytes == cur {
                            unchanged += 1;
                            new_files.insert(rel_str.clone(), sha256_hex(&cur));
                        } else {
                            write_file(&target, &merged_bytes)?;
                            new_files.insert(rel_str.clone(), sha256_hex(&merged_bytes));
                            updated.push(rel_str.clone());
                        }
                    }
                    crate::toml_merge::TomlMergeOutcome::Conflict(reason) => {
                        // 基线记模板渲染哈希：下次 update 检测到磁盘 != 基线时继续报
                        // 冲突，直到用户手工对齐为止；绝不静默覆盖用户的差异值
                        match manifest.files.get(&rel_str) {
                            Some(old_sha) => {
                                new_files.insert(rel_str.clone(), old_sha.clone());
                            }
                            None => {
                                new_files.insert(rel_str.clone(), new_sha);
                            }
                        }
                        conflicted.push(ConflictInfo {
                            path: rel_str,
                            reason,
                        });
                    }
                }
                continue;
            }
            // 磁盘内容非文本或渲染结果不含受管块：落入通用哈希/冲突判定
        }

        // 受管块文件：普通文件（fm.normal）且磁盘已有同名文件、模板渲染结果含受管块时，
        // 合并/注入受管块：磁盘已有同风格托管块则仅替换块区间（块外用户内容原样保留）；
        // 磁盘是无块的 legacy 文件则把托管块追加到末尾并计入 needs_review 提示人工复核。
        // 与磁盘一致则记为未变；合并成功不视为冲突。
        // 注意：concat 累加文件（.gitignore/.gitattributes）不走此分支——它们是逐层拼接、
        // 可含多个受管块，replace_managed_block 只能处理单个块，会丢内容，仍走哈希/冲突判定。
        if fm.normal.contains_key(rel) && target.is_file() {
            let cur = std::fs::read(&target).unwrap_or_default();
            if is_text(&cur)
                && crate::block::extract_managed_block(&String::from_utf8_lossy(bytes)).is_some()
            {
                let legacy_without_block =
                    crate::block::extract_managed_block(&String::from_utf8_lossy(&cur)).is_none();

                // 技能主文档为全自定义 legacy 形态：与 adopt 同策略执行「接管」——
                // 框架插入 frontmatter 之后、原全文下移为纳管过渡区（暂时双流程），
                // 接管后入托管清单，后续 update 走正常的块替换路径。
                if legacy_without_block
                    && is_skill_doc(rel)
                    && !String::from_utf8_lossy(&cur).trim().is_empty()
                {
                    let merged_bytes = take_over_legacy_skill(&cur, bytes);
                    write_file(&target, &merged_bytes)?;
                    new_files.insert(rel_str.clone(), sha256_hex(&merged_bytes));
                    updated.push(rel_str.clone());
                    needs_review.push(format!(
                        "{rel_str}：已接管——模板框架已插入 frontmatter 之后，原全文下移至纳管过渡区\
                         （暂时双流程）。请把领域差异合并进上方项目专属区后删除过渡区"
                    ));
                    continue;
                }

                let merged = merge_managed_block(project_dir, rel, bytes);
                if merged.bytes == cur {
                    unchanged += 1;
                    new_files.insert(rel_str.clone(), sha256_hex(&cur));
                } else {
                    write_file(&target, &merged.bytes)?;
                    new_files.insert(rel_str.clone(), sha256_hex(&merged.bytes));
                    updated.push(rel_str.clone());
                    if merged.kind == ManagedTextMergeKind::Appended
                        && legacy_without_block
                        && !cur.is_empty()
                    {
                        needs_review.push(format!(
                            "{rel_str}：模板托管块已追加到既有文件末尾，请人工检查是否与原有内容重复"
                        ));
                    }
                }
                continue;
            }
        }

        match manifest.files.get(&rel_str) {
            Some(old_sha) if *old_sha == new_sha => {
                unchanged += 1;
                new_files.insert(rel_str, new_sha);
            }
            Some(old_sha) => {
                let current = std::fs::read(&target).ok();
                match current {
                    None => {
                        write_file(&target, bytes)?;
                        new_files.insert(rel_str.clone(), new_sha);
                        created.push(rel_str);
                    }
                    Some(cur) if sha256_hex(&cur) == *old_sha => {
                        // 本地未改动，安全覆盖
                        write_file(&target, bytes)?;
                        new_files.insert(rel_str.clone(), new_sha);
                        updated.push(rel_str);
                    }
                    Some(_) => {
                        // 本地被用户改过且无受管块可合并，保持冲突跳过
                        new_files.insert(rel_str.clone(), old_sha.clone());
                        conflicted.push(ConflictInfo {
                            path: rel_str,
                            reason: "文件已被本地修改，跳过更新".to_string(),
                        });
                    }
                }
            }
            None => {
                // 模板新增的文件
                if target.exists() {
                    conflicted.push(ConflictInfo {
                        path: rel_str.clone(),
                        reason: "文件已存在但未被模板托管，跳过".to_string(),
                    });
                } else {
                    write_file(&target, bytes)?;
                    new_files.insert(rel_str.clone(), new_sha);
                    created.push(rel_str);
                }
            }
        }
    }

    let removed = manifest
        .files
        .keys()
        .filter(|p| !bytes_map.contains_key(Path::new(p)))
        // 黑名单文件不被模板接管，模板删除时不上报
        .filter(|p| !ignores.contains(&PathBuf::from(p)))
        .cloned()
        .collect::<Vec<_>>();

    // `.code-workspace` 工作区文件同步：无论 manifest 是否托管，都扫描项目根目录下
    // 的工作区文件，把模板的 fileNesting 配置合并进其 `settings` 节点。
    // 变更的文件计入 `updated`（复用原有字段，不新增报告结构）。
    for ws_path in list_workspace_files(project_dir) {
        if sync_workspace_file(&ws_path, &ctx, &fm, project_dir)? {
            if let Some(name) = ws_path.file_name() {
                updated.push(name.to_string_lossy().into_owned());
            }
        }
    }

    // commitlint 自动接线兜底：模板提供 base 而项目 config 尚未接线时补上
    //（adopt 已接线的会命中 AlreadyWired，保持幂等）
    if bytes_map.contains_key(Path::new("commitlint.base.js"))
        && try_wire_commitlint_base(project_dir)? == CommitlintWireOutcome::Wired
    {
        updated.push("commitlint.config.js".to_string());
        needs_review.push(
            "commitlint.config.js：已自动接入 commitlint.base.js（base 规则铺底、项目规则同名覆盖，等价重复项已去除），请复核规则优先级"
                .to_string(),
        );
    }

    manifest.files = new_files;
    manifest.save(project_dir)?;

    Ok(UpdateReport {
        project_name: manifest.project_name.clone(),
        layers: ordered,
        updated,
        created,
        conflicted,
        removed,
        needs_review,
        unchanged,
    })
}

// ---------- .code-workspace 同步 ----------

/// 扫描 `dir` 下一级（非递归）所有 `*.code-workspace` 文件，按文件名排序返回。
///
/// 供 CLI / GUI 在展示或批量处理工作区文件时复用。
pub fn list_workspace_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_workspace = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("code-workspace"));
        if is_workspace {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// 若 `obj` 中 `key` 的值与 `value` 不同则写入并返回 `true`（未变化返回 `false`）。
///
/// 用于 JSON 合并时判断内容是否真正变更，避免无意义的写盘。
fn json_set_if_changed(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: serde_json::Value,
) -> bool {
    if obj.get(key) == Some(&value) {
        return false;
    }
    obj.insert(key.to_string(), value);
    true
}

/// 取 VS Code settings 配置来源：模板渲染出的 `.vscode/settings.json`（经 RenderContext）
/// 优先；模板未提供时（未选 vscode 层）回退到磁盘上项目既有的 settings 文件。
/// 模板/磁盘文件缺失或 JSON 解析失败时返回空对象（`{}`），不阻断调用方。
fn vscode_settings_source(
    ctx: &RenderContext,
    fm: &FileMap,
    project_dir: &Path,
) -> Result<serde_json::Value> {
    let value = match fm.normal.get(Path::new(VSCODE_SETTINGS_REL)) {
        Some(bytes) => serde_json::from_slice(&render_bytes(bytes, ctx)?).unwrap_or_default(),
        None => std::fs::read_to_string(project_dir.join(VSCODE_SETTINGS_REL))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
    };
    Ok(value)
}

/// 把模板 settings 的 VS Code 配置合并进 `settings` 对象（settings.json 顶层
/// 或 `.code-workspace` 的 `settings` 节点共用同一语义）：
/// - `explorer.fileNesting.enabled` / `expand`：模板有则直接设值
/// - `explorer.fileNesting.patterns`：对象合并，模板值覆盖同名 key，其余 key 保留
/// - `rust-analyzer.cargo.checkOnSave.command = "clippy"`：仅当「中文编程 + rust 层」时写入
///
/// 其余 key 原样保留（用户的自定义设置不动）。返回是否发生变更。
fn merge_vscode_settings(
    settings: &mut serde_json::Map<String, serde_json::Value>,
    src: &serde_json::Map<String, serde_json::Value>,
    ctx: &RenderContext,
) -> bool {
    let mut changed = false;

    // fileNesting 开关：直接设值
    for key in [
        "explorer.fileNesting.enabled",
        "explorer.fileNesting.expand",
    ] {
        if let Some(v) = src.get(key) {
            changed |= json_set_if_changed(settings, key, v.clone());
        }
    }

    // patterns：对象合并，模板值覆盖同名 key，其余 key 保留
    if let Some(tpl_patterns) = src.get("explorer.fileNesting.patterns") {
        let patterns = settings
            .entry("explorer.fileNesting.patterns")
            .or_insert_with(|| serde_json::json!({}));
        match patterns.as_object_mut() {
            Some(pobj) => {
                if let Some(tpl_map) = tpl_patterns.as_object() {
                    for (k, v) in tpl_map {
                        changed |= json_set_if_changed(pobj, k, v.clone());
                    }
                }
            }
            None => {
                // 已有 patterns 但非对象：整体替换为模板值
                changed |= json_set_if_changed(
                    settings,
                    "explorer.fileNesting.patterns",
                    tpl_patterns.clone(),
                );
            }
        }
    }

    // rust-analyzer clippy：中文编程且含 rust 层时写入
    let rust_with_chinese = ctx
        .options
        .get("chinese_programming")
        .and_then(|v| v.as_bool())
        == Some(true)
        && ctx.layers.iter().any(|l| l == "rust");
    if rust_with_chinese {
        changed |= json_set_if_changed(
            settings,
            "rust-analyzer.cargo.checkOnSave.command",
            serde_json::json!("clippy"),
        );
    }

    changed
}

/// 把一个 `*.code-workspace` 文件与模板的 VS Code 配置同步。
///
/// 读取 workspace 的 JSON，把 `.vscode/settings.json` 渲染结果里的 fileNesting 配置
/// 写入其顶层 `settings` 节点（顶层 `folders` / `extensions` 等其它字段原样保留）：
/// - `explorer.fileNesting.enabled` / `expand`：按模板值直接设值
/// - `explorer.fileNesting.patterns`：对象合并，模板值覆盖同名 key，其余 key 保留
/// - `rust-analyzer.cargo.checkOnSave.command = "clippy"`：仅当「中文编程 + rust 层」时写入
///
/// 配置来源优先取模板渲染出的 `.vscode/settings.json`（模板优先）；模板未提供时
/// （未选 vscode 层）回退到磁盘上项目既有的 settings 文件，保证独立调用也能工作。
/// 仅在内容真正变化时写盘并返回 `true`；workspace 读取/JSON 解析失败时静默返回
/// `Ok(false)` 跳过，不阻断更新流程。
///
/// **JSONC 容错 + 原格式保留**：workspace 文件常带注释与尾随逗号（VS Code 官方
/// 允许），严格解析失败时剥离注释/尾逗号后再解析。写回时**统一只替换顶层
/// `settings` 节点区间**（按原文档缩进重排），folders、注释与其余排版逐字节保留，
/// 不会整文档重序列化破坏用户风格；文档没有 `settings` 成员时才回退整文档重写。
///
/// 注：不写入 `rust-analyzer.linkedProjects`。VS Code 的 rust-analyzer 默认 `[]`
/// 会自动发现工作区内的 Cargo 项目；显式列出反而会禁用自动发现，冗余且有害，
/// 因此模板与同步逻辑都不维护该字段。
pub fn sync_workspace_file(
    path: &Path,
    ctx: &RenderContext,
    fm: &FileMap,
    project_dir: &Path,
) -> Result<bool> {
    // 配置来源：模板渲染出的 `.vscode/settings.json`，缺失时回退磁盘文件
    let settings_value = vscode_settings_source(ctx, fm, project_dir)?;
    let Some(src) = settings_value.as_object() else {
        // 无可用配置来源（未选 vscode 层且磁盘无 settings）
        return Ok(false);
    };

    // 读取 workspace：失败即跳过（不 panic）；严格解析失败回退 JSONC 剥离后再解析
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let mut ws = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(v) => v,
        Err(_) => {
            let stripped = strip_jsonc_comments_and_trailing_commas(&text);
            match serde_json::from_str::<serde_json::Value>(&stripped) {
                Ok(v) => v,
                Err(_) => return Ok(false),
            }
        }
    };
    let Some(ws_obj) = ws.as_object_mut() else {
        return Ok(false);
    };
    // 确保顶层有 `settings` 对象；已存在但非对象时跳过（避免覆盖用户数据）
    let Some(settings) = ws_obj
        .entry("settings")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
    else {
        return Ok(false);
    };

    let changed = merge_vscode_settings(settings, src, ctx);

    if !changed {
        return Ok(false);
    }

    // 统一只替换顶层 `settings` 节点区间（按原文档缩进重排），其余原文
    // （folders、注释、格式）逐字节保留——严格 JSON 与 JSONC 一视同仁，
    // 避免整文档重序列化破坏用户的排版风格。
    let settings_text = serde_json::to_string_pretty(&ws["settings"])?;
    if let Some(final_text) = replace_top_level_member_value(&text, "settings", &settings_text) {
        std::fs::write(path, final_text)?;
        return Ok(true);
    }

    // 原文档没有顶层 `settings` 成员（合并是 entry 兜底造出来的）：回退整文档
    // 重序列化；含受管块时以原文件为底做原位替换，保留块外用户内容。
    let mut out = serde_json::to_string_pretty(&ws)?;
    out.push('\n');
    let final_text = match crate::block::extract_managed_block(&text) {
        Some(_) => {
            let replaced = crate::block::replace_managed_block(&text, &out);
            if replaced == text {
                out
            } else {
                replaced
            }
        }
        None => out,
    };
    std::fs::write(path, final_text)?;
    Ok(true)
}

/// 剥离 JSONC 的 `//` 行注释、`/* */` 块注释与尾随逗号（仅用于解析，不用于回写）。
/// 字符串字面量内的引号、反斜杠与注释记号一律原样保留。
fn strip_jsonc_comments_and_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '/' => match chars.peek() {
                Some('/') => {
                    while let Some(&n) = chars.peek() {
                        if n == '\n' {
                            break;
                        }
                        chars.next();
                    }
                }
                Some('*') => {
                    chars.next();
                    while let Some(n) = chars.next() {
                        if n == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => out.push('/'),
            },
            ',' => {
                // 尾随逗号：跳过空白后若紧跟 } 或 ] 则省略该逗号
                let mut lookahead = chars.clone();
                let next_significant = loop {
                    match lookahead.next() {
                        Some(' ' | '\t' | '\r' | '\n') => continue,
                        other => break other,
                    }
                };
                if next_significant != Some('}') && next_significant != Some(']') {
                    out.push(',');
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// 在 JSON 文本中定位顶层成员 `key` 的值区间并替换为 `replacement`，返回新全文。
///
/// 采用字符串感知的平衡花括号扫描：字符串字面量整体作为一个 token 消费，
/// 只在深度 1（根对象直下）匹配 `"key":`，值以 `{` 开始时扫描到与之配对的 `}`。
/// 其余成员的文本（含注释、格式）逐字节保留。未找到匹配或值不是对象时返回 `None`。
/// 在文本中定位顶层成员 `"key": <value>` 的区间（字符串感知的平衡括号扫描）。
/// 返回 `(键起始下标, 值起始下标, 值结束下标（不含）)`；未找到或值不是对象时返回 `None`。
fn find_top_level_member_span(text: &str, key: &str) -> Option<(usize, usize, usize)> {
    let target = format!("\"{key}\"");
    let b = text.as_bytes();
    let len = b.len();

    /// 从 `start`（引号处）读取完整字符串 token，返回收尾引号下标
    fn string_token_end(b: &[u8], start: usize) -> usize {
        let mut j = start + 1;
        let mut escaped = false;
        while j < b.len() {
            let c = b[j];
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                return j;
            }
            j += 1;
        }
        b.len()
    }

    let mut i = 0usize;
    let mut depth: usize = 0;
    while i < len {
        match b[i] {
            b'"' => {
                let close = string_token_end(b, i);
                if depth == 1
                    && text.get(i..=(close.min(len.saturating_sub(1)))) == Some(target.as_str())
                {
                    let mut k = close + 1;
                    while matches!(
                        b.get(k),
                        Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n')
                    ) {
                        k += 1;
                    }
                    if b.get(k) == Some(&b':') {
                        k += 1;
                        while matches!(
                            b.get(k),
                            Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n')
                        ) {
                            k += 1;
                        }
                        if b.get(k) == Some(&b'{') {
                            // 平衡扫描到配对的 '}'
                            let mut local_depth: usize = 0;
                            let mut m = k;
                            while m < len {
                                match b[m] {
                                    b'"' => m = string_token_end(b, m),
                                    b'{' => local_depth += 1,
                                    b'}' => {
                                        local_depth -= 1;
                                        if local_depth == 0 {
                                            return Some((i, k, m + 1));
                                        }
                                    }
                                    _ => {}
                                }
                                m += 1;
                            }
                            return None;
                        }
                    }
                }
                i = close + 1;
            }
            b'{' | b'[' => {
                depth += 1;
                i += 1;
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// 在 JSON 文本中定位顶层成员 `key` 的值区间并替换为 `replacement`，返回新全文。
///
/// 替换文本会按原文档的缩进重排：以 `key` 所在行的缩进为基准、以文档首层子成员的
/// 缩进宽度为单位，对 `replacement` 的各行重新缩进，避免破坏原文档的排版风格。
/// 其余成员的文本（含注释、格式）逐字节保留。未找到匹配或值不是对象时返回 `None`。
fn replace_top_level_member_value(text: &str, key: &str, replacement: &str) -> Option<String> {
    let (key_start, value_start, value_end) = find_top_level_member_span(text, key)?;
    let replacement = reindent_block(text, key_start, value_start, replacement);
    Some(format!(
        "{}{}{}",
        &text[..value_start],
        replacement,
        &text[value_end..]
    ))
}

/// 把多行替换文本按原文档缩进重排：
/// - 首行原样（它紧跟在 `"key": ` 之后）
/// - 其余行按其在替换文本中的相对深度（serde_json 固定 2 空格一级）加上
///   「key 行缩进 + 单位宽度」的前缀；单位宽度取文档首个根级子成员的缩进，
///   推断失败时回退 2 空格
fn reindent_block(text: &str, key_start: usize, value_start: usize, replacement: &str) -> String {
    // key 所在行缩进
    let line_start = text[..key_start].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let key_indent = &text[line_start..key_start];

    // 单位宽度：value 首行之后第一个子成员行的缩进减去 key 行缩进
    let unit = text
        .get(value_start..)
        .and_then(|rest| rest.find('\n'))
        .and_then(|nl| {
            text.get(value_start + nl + 1..).and_then(|tail| {
                let ind = tail.len() - tail.trim_start_matches([' ', '\t']).len();
                let non_ws = tail.trim_start_matches([' ', '\t']).chars().next()?;
                (non_ws != '}' && non_ws != ']').then_some(ind.saturating_sub(key_indent.len()))
            })
        })
        .filter(|&u| u > 0)
        .unwrap_or(2);

    let mut out = String::with_capacity(replacement.len() + 64);
    for (idx, line) in replacement.lines().enumerate() {
        if idx == 0 {
            out.push_str(line);
        } else {
            let trimmed = line.trim_start_matches(' ');
            let lead = line.len() - trimmed.len();
            let depth = lead / 2;
            out.push('\n');
            out.push_str(key_indent);
            for _ in 0..depth * unit {
                out.push(' ');
            }
            out.push_str(trimmed);
        }
    }
    out
}

/// 规范化 JS 对象字面量片段用于等价比较：去注释、单引号统一为双引号、
/// 去尾随逗号、压缩字符串外空白。含模板字符串等无法安全处理的构造时返回 `None`
/// （调用方应保守跳过去重）。
fn normalize_js_literal(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string: Option<char> = None; // None | Some('"') | Some('\'')
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if let Some(q) = in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                if q == '\'' {
                    // 结束的单引号已统一写成双引号
                    out.pop();
                    out.push('"');
                }
                in_string = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_string = Some(c);
                out.push('"');
            }
            '`' => return None,
            '/' => match chars.peek() {
                Some('/') => {
                    while let Some(&n) = chars.peek() {
                        if n == '\n' {
                            break;
                        }
                        chars.next();
                    }
                }
                Some('*') => {
                    chars.next();
                    while let Some(n) = chars.next() {
                        if n == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => out.push('/'),
            },
            ',' => {
                let next_significant = loop {
                    match chars.peek().copied() {
                        Some(' ' | '\t' | '\r' | '\n') => {
                            chars.next();
                        }
                        other => break other,
                    }
                };
                if next_significant != Some('}') && next_significant != Some(']') {
                    out.push(',');
                }
            }
            ' ' | '\t' | '\r' | '\n' => {}
            _ => out.push(c),
        }
    }
    // 引号统一后需保持成对结构：双引号数量为偶数才可信
    if !out.matches('"').count().is_multiple_of(2) {
        return None;
    }
    Some(out)
}

/// 对象字面量的一个顶层成员
struct JsObjectEntry {
    /// 成员名（去引号后的裸名）
    name: String,
    /// 成员在源文本中的原始切片（不含结尾逗号）
    source: String,
    /// 规范化形式（用于与模板侧比较是否等价）
    normalized: String,
}

/// 拆分对象字面量花括号内的顶层成员；无法安全解析时返回 `None`。
fn split_object_members(inner: &str) -> Option<Vec<JsObjectEntry>> {
    let b = inner.as_bytes();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut depth: usize = 0;
    let mut start: Option<usize> = None;
    let mut in_string: Option<char> = None;
    let mut escaped = false;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i] as char;
        if let Some(q) = in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                in_string = None;
            }
            i += 1;
            continue;
        }
        match c {
            '"' | '\'' => {
                in_string = Some(c);
                if start.is_none() {
                    start = Some(i);
                }
                i += 1;
            }
            '{' | '[' => {
                depth += 1;
                i += 1;
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            ',' if depth == 0 => {
                if let Some(s) = start.take() {
                    spans.push((s, i));
                }
                i += 1;
            }
            _ => {
                if !c.is_whitespace() && start.is_none() {
                    start = Some(i);
                }
                i += 1;
            }
        }
    }
    if let Some(s) = start.take() {
        spans.push((s, b.len()));
    }

    spans
        .into_iter()
        .map(|(s, e)| {
            let raw = inner.get(s..e)?.trim_end();
            let raw = raw.strip_suffix(',').unwrap_or(raw);
            let norm = normalize_js_literal(raw)?;
            // 成员名：`name:` 之前的键部分，去掉引号
            let colon = norm.find(':')?;
            let key_part = norm[..colon].trim();
            if key_part.is_empty() {
                return None;
            }
            Some(JsObjectEntry {
                name: key_part.trim_matches('"').to_string(),
                source: raw.to_string(),
                normalized: norm,
            })
        })
        .collect()
}

/// `.vscode/settings.json` 增量合并的结果分类（复用 `update_project` 的统计口径）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSyncOutcome {
    /// 内容已变更并写盘（文件原本存在）
    Updated,
    /// 文件原本不存在，本次创建
    Created,
    /// 无实质变更（含模板源缺失/非对象、磁盘 JSON 解析失败等保守跳过）
    Unchanged,
}

/// 把 `.vscode/settings.json` 与模板的 VS Code 配置增量合并。
///
/// 复用 `merge_vscode_settings` 的合并语义（与 `.code-workspace` 同步一致）：
/// - 配置来源：模板渲染出的 settings 优先，缺失时回退磁盘既有文件
/// - 模板源非对象 -> 保守跳过（`Unchanged`），不误伤用户文件
/// - 以磁盘现有 JSON 为底（不存在或解析失败视为 `{}`），模板的 fileNesting /
///   rust-analyzer.clippy 并入，用户其它 key 原样保留
/// - 仅真正变更才写盘；幂等（二次调用无变更返回 `Unchanged`）
pub fn sync_settings_file(
    path: &Path,
    ctx: &RenderContext,
    fm: &FileMap,
    project_dir: &Path,
) -> Result<SettingsSyncOutcome> {
    let settings_value = vscode_settings_source(ctx, fm, project_dir)?;
    let Some(src) = settings_value.as_object() else {
        // 模板源非对象：保守跳过
        return Ok(SettingsSyncOutcome::Unchanged);
    };

    let existed = path.exists();
    // 现有文件：不存在或 JSON 解析失败时视为空对象（保守，不覆盖用户数据）
    let disk: serde_json::Value = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let mut merged = if disk.is_object() {
        disk
    } else {
        serde_json::json!({})
    };
    let changed = match merged.as_object_mut() {
        Some(base) => merge_vscode_settings(base, src, ctx),
        None => false,
    };

    if existed && !changed {
        return Ok(SettingsSyncOutcome::Unchanged);
    }
    if !existed && merged.as_object().is_some_and(|o| o.is_empty()) {
        // 无任何可写内容（模板与磁盘均为空）：不创建空文件
        return Ok(SettingsSyncOutcome::Unchanged);
    }

    let mut out = serde_json::to_string_pretty(&merged)?;
    out.push('\n');
    write_file(path, out.as_bytes())?;
    Ok(if existed {
        SettingsSyncOutcome::Updated
    } else {
        SettingsSyncOutcome::Created
    })
}

// ---------- 时间 ----------

fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Howard Hinnant 的 civil_from_days 算法（公历日数 -> 年月日）
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_name_of_detects_skill_dirs() {
        // 正斜杠（Unix 风格相对路径）
        assert_eq!(
            skill_name_of(Path::new(".agents/skills/commit/SKILL.md")),
            Some("commit".to_string())
        );
        assert_eq!(
            skill_name_of(Path::new(".agents/skills/caveman/references/x.md")),
            Some("caveman".to_string())
        );
        // 反斜杠（Windows 相对路径，strip_prefix 产物）
        assert_eq!(
            skill_name_of(Path::new(r".agents\skills\grill-me\SKILL.md")),
            Some("grill-me".to_string())
        );
        // 非技能路径
        assert_eq!(skill_name_of(Path::new("AGENTS.md")), None);
        assert_eq!(skill_name_of(Path::new(".agents/AGENTS.md")), None);
        assert_eq!(
            skill_name_of(Path::new(".agents/skills")),
            None,
            "缺少技能名组件"
        );
        assert_eq!(
            skill_name_of(Path::new("other/.agents/skills/x/SKILL.md")),
            None
        );
    }

    #[test]
    fn selected_skills_parses_option() {
        let mut opts = BTreeMap::new();
        assert_eq!(selected_skills(&opts), None, "缺失选项 = 全部技能");

        opts.insert("skills".to_string(), serde_json::json!([]));
        assert_eq!(
            selected_skills(&opts),
            Some(BTreeSet::new()),
            "空数组 = 不生成任何技能"
        );

        opts.insert(
            "skills".to_string(),
            serde_json::json!(["commit", "caveman"]),
        );
        let set = selected_skills(&opts).unwrap();
        assert!(set.contains("commit") && set.contains("caveman") && !set.contains("grill-me"));
    }

    #[test]
    fn parse_skill_description_folded_block() {
        // 模拟 minijinja 渲染后的 frontmatter：`{% if %}` 行被移除留下空行
        let text = "---\nname: caveman\ndescription: >-\n\n  超压缩通信模式。省略废话。\n  Triggers: caveman, be brief\n\n---\n\n# Caveman 模式\n";
        assert_eq!(
            parse_skill_description(text),
            "超压缩通信模式。省略废话。 Triggers: caveman, be brief"
        );
    }

    #[test]
    fn parse_skill_description_inline_value() {
        let text =
            "---\nname: grill-me\ndescription: 就计划持续追问，直至达成共识。\n---\n\n正文\n";
        assert_eq!(
            parse_skill_description(text),
            "就计划持续追问，直至达成共识。"
        );
    }

    #[test]
    fn parse_skill_description_missing_or_empty() {
        assert_eq!(parse_skill_description("no frontmatter here"), "");
        assert_eq!(parse_skill_description("---\nname: x\n---\n"), "");
    }

    #[test]
    fn list_workspace_files_finds_sorted_code_workspaces() {
        let dir = std::env::temp_dir().join(format!("pengj-ws-list-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // 子目录中的工作区文件不参与（非递归）
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub").join("inner.code-workspace"), "{}").unwrap();
        std::fs::write(dir.join("b.code-workspace"), "{}").unwrap();
        std::fs::write(dir.join("a.CODE-WORKSPACE"), "{}").unwrap();
        std::fs::write(dir.join("not-ws.json"), "{}").unwrap();

        let names: Vec<String> = list_workspace_files(&dir)
            .into_iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        assert_eq!(names, vec!["a.CODE-WORKSPACE", "b.code-workspace"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 构造一个带 `.vscode/settings.json` 模板的 FileMap 与中文编程 + rust 的上下文
    fn ws_test_fixture() -> (FileMap, RenderContext) {
        let mut fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
        fm.normal.insert(
            PathBuf::from(".vscode/settings.json"),
            br#"{
  "explorer.fileNesting.enabled": true,
  "explorer.fileNesting.expand": false,
  "explorer.fileNesting.patterns": { "Cargo.toml": "tpl" }
}"#
            .to_vec(),
        );
        let mut options = BTreeMap::new();
        options.insert("chinese_programming".to_string(), serde_json::json!(true));
        let ctx = RenderContext::new(
            "proj",
            vec!["vscode".to_string(), "rust".to_string()],
            options,
        );
        (fm, ctx)
    }

    #[test]
    fn sync_workspace_file_merges_filenesting_template_priority() {
        let tmp = std::env::temp_dir().join(format!("pengj-ws-sync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let (fm, ctx) = ws_test_fixture();

        let ws = tmp.join("proj.code-workspace");
        std::fs::write(
            &ws,
            r#"{
  "folders": [{ "path": "." }],
  "settings": {
    "editor.fontSize": 14,
    "explorer.fileNesting.enabled": false,
    "explorer.fileNesting.patterns": { "Cargo.toml": "user", "README.md": "keep-me" }
  }
}"#,
        )
        .unwrap();

        // 首次同步发生变更，第二次幂等（不再写盘）
        assert!(sync_workspace_file(&ws, &ctx, &fm, &proj).unwrap());
        assert!(!sync_workspace_file(&ws, &ctx, &fm, &proj).unwrap());

        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ws).unwrap()).unwrap();
        let s = &parsed["settings"];
        assert_eq!(s["explorer.fileNesting.enabled"], serde_json::json!(true));
        assert_eq!(s["explorer.fileNesting.expand"], serde_json::json!(false));
        assert_eq!(s["editor.fontSize"], serde_json::json!(14), "用户标量保留");
        assert_eq!(
            s["explorer.fileNesting.patterns"]["Cargo.toml"],
            serde_json::json!("tpl"),
            "模板覆盖同名 key"
        );
        assert_eq!(
            s["explorer.fileNesting.patterns"]["README.md"],
            serde_json::json!("keep-me"),
            "用户其它 key 保留"
        );
        assert_eq!(
            s["rust-analyzer.cargo.checkOnSave.command"],
            serde_json::json!("clippy"),
            "中文编程 + rust 层写入 clippy"
        );
        // 顶层 folders / extensions 不被覆盖
        assert!(parsed["folders"].is_array());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn strip_jsonc_keeps_strings_and_drops_trailing_commas() {
        let src = "{\n  // 行注释\n  \"a\": \"http://x // 不是注释\", /* 块\n注释 */\n  \"b\": [1, 2,],\n  \"c\": {\"d\": true,}\n}\n";
        let out = strip_jsonc_comments_and_trailing_commas(src);
        let v: serde_json::Value = serde_json::from_str(&out).expect("剥离后必须是合法 JSON");
        assert_eq!(v["a"], "http://x // 不是注释", "字符串内记号原样保留");
        assert_eq!(v["b"], serde_json::json!([1, 2]));
        assert_eq!(v["c"]["d"], serde_json::json!(true));
    }

    #[test]
    fn sync_workspace_file_merges_jsonc_workspace_preserving_rest_of_file() {
        let tmp = std::env::temp_dir().join(format!("pengj-ws-jsonc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let (fm, ctx) = ws_test_fixture();

        // 模仿真实 VS Code 工程的 JSONC：尾随逗号 + 注释
        let ws = tmp.join("app.code-workspace");
        let original = "// 根级注释\n{\n  \"folders\": [\n    { \"path\": \".\" },\n  ],\n  \"settings\": {\n    \"editor.fontSize\": 14,\n    \"explorer.fileNesting.patterns\": {\n      \"README.md\": \"keep-me\",\n    },\n  },\n}\n";
        std::fs::write(&ws, original).unwrap();

        assert!(sync_workspace_file(&ws, &ctx, &fm, &proj).unwrap());
        let merged = std::fs::read_to_string(&ws).unwrap();
        // 其余部分逐字节保留：根注释与 folders 段
        assert!(merged.starts_with("// 根级注释"));
        assert!(merged.contains("{ \"path\": \".\" },"));
        assert!(merged.contains("\"editor.fontSize\": 14"), "用户设置保留");
        // settings 节点已合并模板内容
        let sanitized = strip_jsonc_comments_and_trailing_commas(&merged);
        let parsed: serde_json::Value =
            serde_json::from_str(&sanitized).expect("结果仍是合法 JSONC");
        let s = &parsed["settings"];
        assert_eq!(s["explorer.fileNesting.enabled"], serde_json::json!(true));
        assert_eq!(
            s["explorer.fileNesting.patterns"]["Cargo.toml"],
            serde_json::json!("tpl"),
            "模板 patterns 并入"
        );
        assert_eq!(
            s["explorer.fileNesting.patterns"]["README.md"],
            serde_json::json!("keep-me")
        );

        // 幂等：二次同步无变更、不写盘
        let before = std::fs::read_to_string(&ws).unwrap();
        assert!(!sync_workspace_file(&ws, &ctx, &fm, &proj).unwrap());
        assert_eq!(std::fs::read_to_string(&ws).unwrap(), before);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn sync_workspace_file_reindents_settings_to_document_style() {
        let tmp = std::env::temp_dir().join(format!("pengj-ws-indent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let (fm, ctx) = ws_test_fixture();

        // 4 空格缩进风格的文档
        let ws = tmp.join("app.code-workspace");
        std::fs::write(
            &ws,
            "{\n    \"folders\": [\n        { \"path\": \".\" }\n    ],\n    \"settings\": {\n        \"editor.fontSize\": 14\n    }\n}\n",
        )
        .unwrap();

        assert!(sync_workspace_file(&ws, &ctx, &fm, &proj).unwrap());
        let merged = std::fs::read_to_string(&ws).unwrap();
        // 合并结果仍可被严格解析，值正确
        let parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
        assert_eq!(
            parsed["settings"]["explorer.fileNesting.enabled"],
            serde_json::json!(true)
        );
        assert_eq!(parsed["folders"][0]["path"], serde_json::json!("."));
        // settings 子成员行使用「key 缩进(4) + 单位(4)」= 8 空格
        let enabled_line = merged
            .lines()
            .find(|l| l.contains("\"explorer.fileNesting.enabled\""))
            .expect("应包含合并进来的 fileNesting 开关");
        assert!(
            enabled_line.starts_with("        \"explorer.fileNesting.enabled\""),
            "settings 子成员应按 4 空格单位重排，实际: {enabled_line:?}"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn sync_workspace_file_skips_broken_or_missing_sources() {
        let tmp = std::env::temp_dir().join(format!("pengj-ws-skip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let (fm, ctx) = ws_test_fixture();

        // 无效 JSON：跳过且不 panic
        let broken = tmp.join("broken.code-workspace");
        std::fs::write(&broken, "{ not json").unwrap();
        assert!(!sync_workspace_file(&broken, &ctx, &fm, &proj).unwrap());

        // 顶层 settings 非对象：跳过
        let bad_settings = tmp.join("bad.code-workspace");
        std::fs::write(&bad_settings, r#"{ "folders": [], "settings": "nope" }"#).unwrap();
        assert!(!sync_workspace_file(&bad_settings, &ctx, &fm, &proj).unwrap());

        // 无模板来源（fm 无 .vscode/settings.json）且磁盘无 settings：跳过
        let empty_fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
        let plain = tmp.join("plain.code-workspace");
        std::fs::write(&plain, r#"{ "folders": [{ "path": "." }] }"#).unwrap();
        assert!(!sync_workspace_file(&plain, &ctx, &empty_fm, &proj).unwrap());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn sync_settings_file_merges_filenesting_and_preserves_user_keys() {
        let tmp = std::env::temp_dir().join(format!("pengj-st-sync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(proj.join(".vscode")).unwrap();
        let (fm, ctx) = ws_test_fixture();

        let settings = proj.join(".vscode").join("settings.json");
        std::fs::write(
            &settings,
            r#"{
  "editor.fontSize": 14,
  "explorer.fileNesting.enabled": false,
  "explorer.fileNesting.patterns": { "Cargo.toml": "user", "README.md": "keep-me" }
}"#,
        )
        .unwrap();

        // 首次同步发生变更，第二次幂等（不再写盘）
        assert_eq!(
            sync_settings_file(&settings, &ctx, &fm, &proj).unwrap(),
            SettingsSyncOutcome::Updated
        );
        assert_eq!(
            sync_settings_file(&settings, &ctx, &fm, &proj).unwrap(),
            SettingsSyncOutcome::Unchanged
        );

        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(
            parsed["explorer.fileNesting.enabled"],
            serde_json::json!(true)
        );
        assert_eq!(
            parsed["explorer.fileNesting.expand"],
            serde_json::json!(false)
        );
        assert_eq!(
            parsed["editor.fontSize"],
            serde_json::json!(14),
            "用户标量保留"
        );
        assert_eq!(
            parsed["explorer.fileNesting.patterns"]["Cargo.toml"],
            serde_json::json!("tpl"),
            "模板覆盖同名 key"
        );
        assert_eq!(
            parsed["explorer.fileNesting.patterns"]["README.md"],
            serde_json::json!("keep-me"),
            "用户其它 key 保留"
        );
        assert_eq!(
            parsed["rust-analyzer.cargo.checkOnSave.command"],
            serde_json::json!("clippy"),
            "中文编程 + rust 层写入 clippy"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn sync_settings_file_creates_missing_file() {
        let tmp = std::env::temp_dir().join(format!("pengj-st-create-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let (fm, ctx) = ws_test_fixture();

        let settings = proj.join(".vscode").join("settings.json");
        // 首次为项目补 vscode 层：创建文件；二次调用幂等
        assert_eq!(
            sync_settings_file(&settings, &ctx, &fm, &proj).unwrap(),
            SettingsSyncOutcome::Created
        );
        assert_eq!(
            sync_settings_file(&settings, &ctx, &fm, &proj).unwrap(),
            SettingsSyncOutcome::Unchanged
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(
            parsed["explorer.fileNesting.enabled"],
            serde_json::json!(true)
        );
        assert_eq!(
            parsed["rust-analyzer.cargo.checkOnSave.command"],
            serde_json::json!("clippy")
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn sync_settings_file_skips_without_template_or_disk_source() {
        let tmp = std::env::temp_dir().join(format!("pengj-st-skip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let empty_fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
        let ctx = RenderContext::new("proj", vec!["vscode".to_string()], BTreeMap::new());

        // 无模板来源且磁盘无 settings：不创建空文件
        let settings = proj.join(".vscode").join("settings.json");
        assert_eq!(
            sync_settings_file(&settings, &ctx, &empty_fm, &proj).unwrap(),
            SettingsSyncOutcome::Unchanged
        );
        assert!(!settings.exists());

        // 模板源非对象：保守跳过，不改动用户文件
        let mut arr_fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
        arr_fm
            .normal
            .insert(PathBuf::from(".vscode/settings.json"), b"[1, 2]".to_vec());
        std::fs::create_dir_all(proj.join(".vscode")).unwrap();
        std::fs::write(&settings, r#"{ "editor.fontSize": 14 }"#).unwrap();
        assert_eq!(
            sync_settings_file(&settings, &ctx, &arr_fm, &proj).unwrap(),
            SettingsSyncOutcome::Unchanged
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(
            parsed["editor.fontSize"],
            serde_json::json!(14),
            "用户文件未被改动"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn update_project_merges_vscode_settings_incrementally() {
        let tmp = std::env::temp_dir().join(format!("pengj-upd-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：common + vscode（settings.json 带 fileNesting）
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("vscode").join(".vscode")).unwrap();
        std::fs::write(
            tpl.join("vscode").join("layer.toml"),
            "name = \"VS Code\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("vscode").join(".vscode").join("settings.json"),
            r#"{
  "explorer.fileNesting.enabled": true,
  "explorer.fileNesting.patterns": { "Cargo.toml": "tpl" }
}"#,
        )
        .unwrap();

        // 项目：磁盘 settings 已有用户自定义，manifest 已记录 vscode 层
        let proj = tmp.join("proj");
        std::fs::create_dir_all(proj.join(".vscode")).unwrap();
        std::fs::write(
            proj.join(".vscode").join("settings.json"),
            r#"{
  "editor.fontSize": 14,
  "explorer.fileNesting.enabled": false,
  "explorer.fileNesting.patterns": { "Cargo.toml": "user", "README.md": "keep-me" }
}"#,
        )
        .unwrap();
        std::fs::write(
            proj.join(crate::manifest::MANIFEST_FILE),
            serde_json::json!({
                "tool": "pengj-templates",
                "version": "0.0.0",
                "project_name": "proj",
                "layers": ["common", "vscode"],
                "options": { "chinese_programming": true },
                "generated_at": "2026-01-01T00:00:00Z",
                "files": {},
            })
            .to_string(),
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        // 报告/manifest 中的路径使用平台分隔符，比较时统一经 Path 归一化
        let settings_rel = Path::new(".vscode/settings.json");
        let report = update_project(&templates, &proj).unwrap();
        assert!(
            report.updated.iter().any(|f| Path::new(f) == settings_rel),
            "模板变更应合并写入 settings.json"
        );

        let parsed: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(proj.join(".vscode").join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            parsed["explorer.fileNesting.enabled"],
            serde_json::json!(true)
        );
        assert_eq!(
            parsed["explorer.fileNesting.patterns"]["Cargo.toml"],
            serde_json::json!("tpl"),
            "模板覆盖同名 key"
        );
        assert_eq!(
            parsed["explorer.fileNesting.patterns"]["README.md"],
            serde_json::json!("keep-me"),
            "用户其它 key 保留"
        );
        assert_eq!(
            parsed["editor.fontSize"],
            serde_json::json!(14),
            "用户标量保留"
        );

        // 幂等：二次更新不再变更 settings.json
        let report2 = update_project(&templates, &proj).unwrap();
        assert!(!report2.updated.iter().any(|f| Path::new(f) == settings_rel));
        assert!(!report2.created.iter().any(|f| Path::new(f) == settings_rel));

        // manifest 记录的 sha 与磁盘内容一致，避免下次误判冲突
        let manifest = ProjectManifest::load(&proj).unwrap();
        let disk = std::fs::read(proj.join(settings_rel)).unwrap();
        let key = manifest
            .files
            .keys()
            .find(|k| Path::new(k) == settings_rel)
            .expect("manifest 应记录 settings.json");
        assert_eq!(manifest.files[key], sha256_hex(&disk));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn gitattributes_accumulates_across_layers() {
        let tmp = std::env::temp_dir().join(format!("pengj-gitattr-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let tpl = tmp.join("templates");

        // layer A
        let layer_a = tpl.join("layer_a");
        std::fs::create_dir_all(&layer_a).unwrap();
        std::fs::write(
            layer_a.join("layer.toml"),
            "name = \"Layer A\"\ndescription = \"A\"\ndepends = []\n",
        )
        .unwrap();
        std::fs::write(layer_a.join(".gitattributes"), "* text=auto eol=lf\n").unwrap();

        // layer B
        let layer_b = tpl.join("layer_b");
        std::fs::create_dir_all(&layer_b).unwrap();
        std::fs::write(
            layer_b.join("layer.toml"),
            "name = \"Layer B\"\ndescription = \"B\"\ndepends = [\"layer_a\"]\n",
        )
        .unwrap();
        std::fs::write(
            layer_b.join(".gitattributes"),
            "*.rs text eol=lf diff=rust\n",
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = generate(
            &templates,
            "demo",
            &["layer_b".to_string()],
            BTreeMap::new(),
            &tmp,
        )
        .unwrap();

        let gitattributes_path = PathBuf::from(&report.project_dir).join(".gitattributes");
        assert!(gitattributes_path.exists());
        let content = std::fs::read_to_string(&gitattributes_path).unwrap();
        assert!(content.contains("# --- layer_a 层 ---"));
        assert!(content.contains("* text=auto eol=lf"));
        assert!(content.contains("# --- layer_b 层 ---"));
        assert!(content.contains("*.rs text eol=lf diff=rust"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn replace_managed_block_injects_into_legacy_file() {
        // 存量文件没有受管块标记（旧版工具生成的 legacy 文件）：模板的受管块应被
        // 追加到末尾，块前的全部用户内容逐字节保留。
        let disk = "# Header\n\n[build]\nrustflags = [\"-C\", \"target-cpu=native\"]\n\n[alias]\nb = \"build\"\n";
        let tmpl = "<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged = crate::block::replace_managed_block(disk, tmpl);
        assert_eq!(
            merged,
            "# Header\n\n[build]\nrustflags = [\"-C\", \"target-cpu=native\"]\n\n[alias]\nb = \"build\"\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n"
        );

        // 磁盘已有同风格受管块：仅替换块区间，块外用户内容原样保留
        let disk_annotated = "# Header\n\n<!-- PENGJ_TEMPLATE_START -->\nold template\n<!-- PENGJ_TEMPLATE_END -->\n\n## Custom User Rules\n- Rule 1\n";
        let tmpl2 =
            "<!-- PENGJ_TEMPLATE_START -->\nnew updated template\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged2 = crate::block::replace_managed_block(disk_annotated, tmpl2);
        assert!(merged2.starts_with("# Header\n\n"));
        assert!(merged2.contains("new updated template"));
        assert!(merged2.ends_with("\n\n## Custom User Rules\n- Rule 1\n"));
        assert!(!merged2.contains("old template"));

        // Shell/TOML 风格 # 注释
        let disk_hash = "# PENGJ_TEMPLATE_START\nold cfg\n# PENGJ_TEMPLATE_END\ncustom = 1\n";
        let tmpl_hash = "# PENGJ_TEMPLATE_START\nnew cfg\n# PENGJ_TEMPLATE_END\n";
        let merged_hash = crate::block::replace_managed_block(disk_hash, tmpl_hash);
        assert_eq!(
            merged_hash,
            "# PENGJ_TEMPLATE_START\nnew cfg\n# PENGJ_TEMPLATE_END\ncustom = 1\n"
        );
    }

    #[test]
    fn render_file_map_returns_pure_rendered_normal_files() {
        // render_file_map 不再做受管块合并（合并决策在 adopt/update 循环里，
        // 以便区分替换/追加/TOML 结构化合并并上报 needs_review）：
        // 无论磁盘是否存在同名文件，普通文件一律返回纯渲染结果
        let tmp = std::env::temp_dir().join(format!("pengj-mblock-render-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();

        let mut fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
        fm.normal.insert(
            PathBuf::from("AGENTS.md"),
            b"<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n".to_vec(),
        );
        let ctx = RenderContext::new("proj", vec!["agent".to_string()], BTreeMap::new());

        // 磁盘已存在旧版本文件：渲染结果仍是纯模板，不受磁盘影响
        std::fs::write(
            proj.join("AGENTS.md"),
            "# Header\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v1\n<!-- PENGJ_TEMPLATE_END -->\n\n## Custom\n- user content\n",
        )
        .unwrap();

        let out = render_file_map(&fm, &ctx, &proj).unwrap();
        let rendered = String::from_utf8(out[&PathBuf::from("AGENTS.md")].clone()).unwrap();
        assert_eq!(
            rendered,
            "<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->"
        );

        // 磁盘不存在该文件：同样是纯渲染结果
        let empty_proj = tmp.join("empty");
        std::fs::create_dir_all(&empty_proj).unwrap();
        let out2 = render_file_map(&fm, &ctx, &empty_proj).unwrap();
        assert_eq!(
            out2[&PathBuf::from("AGENTS.md")].as_slice(),
            b"<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn update_project_preserves_user_content_outside_managed_block() {
        let tmp = std::env::temp_dir().join(format!("pengj-upd-mblock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：common + agent 层（AGENTS.md 含受管块，块内为模板托管内容）
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("agent")).unwrap();
        std::fs::write(
            tpl.join("agent").join("layer.toml"),
            "name = \"Agent\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("agent").join("AGENTS.md"),
            "# {{ project_name }} 编码规范\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // 项目：AGENTS.md 块外已有用户自定义内容，块内是旧版；manifest 记录的是
        // 原始生成内容（不含用户追加部分）的哈希
        let v1_template = "# proj 编码规范\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v1\n<!-- PENGJ_TEMPLATE_END -->\n";
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(
            proj.join("AGENTS.md"),
            "# proj 编码规范\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v1\n<!-- PENGJ_TEMPLATE_END -->\n\n## 项目专属\n- 用户规则\n",
        )
        .unwrap();
        std::fs::write(
            proj.join(crate::manifest::MANIFEST_FILE),
            serde_json::json!({
                "tool": "pengj-templates",
                "version": "0.0.0",
                "project_name": "proj",
                "layers": ["common", "agent"],
                "options": {},
                "generated_at": "2026-01-01T00:00:00Z",
                "files": { "AGENTS.md": sha256_hex(v1_template.as_bytes()) },
            })
            .to_string(),
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = update_project(&templates, &proj).unwrap();
        assert!(
            report.updated.iter().any(|f| f == "AGENTS.md"),
            "块内内容变更应更新 AGENTS.md"
        );
        assert!(report.conflicted.is_empty(), "含受管块的文件不应报冲突");

        let merged = std::fs::read_to_string(proj.join("AGENTS.md")).unwrap();
        assert!(merged.starts_with("# proj 编码规范\n\n"));
        assert!(merged.contains("managed v2"));
        assert!(merged.ends_with("\n\n## 项目专属\n- 用户规则\n"));
        assert!(!merged.contains("managed v1"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn update_project_injects_managed_block_into_legacy_file() {
        let tmp = std::env::temp_dir().join(format!("pengj-upd-legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：agent 层 AGENTS.md 含受管块（v2 内容）
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("agent")).unwrap();
        std::fs::write(
            tpl.join("agent").join("layer.toml"),
            "name = \"Agent\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("agent").join("AGENTS.md"),
            "# {{ project_name }} 编码规范\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // 存量项目：AGENTS.md 是旧版 adopt 生成的 legacy 文件，没有受管块标记；
        // manifest 记录的是旧版模板渲染哈希（不含用户追加内容），模拟旧工具留下的状态
        let v1_template = "# proj 编码规范\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v1\n<!-- PENGJ_TEMPLATE_END -->\n";
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(
            proj.join("AGENTS.md"),
            "# proj 编码规范\n\n## 项目专属\n- 用户规则\n",
        )
        .unwrap();
        std::fs::write(
            proj.join(crate::manifest::MANIFEST_FILE),
            serde_json::json!({
                "tool": "pengj-templates",
                "version": "0.0.0",
                "project_name": "proj",
                "layers": ["common", "agent"],
                "options": {},
                "generated_at": "2026-01-01T00:00:00Z",
                "files": { "AGENTS.md": sha256_hex(v1_template.as_bytes()) },
            })
            .to_string(),
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = update_project(&templates, &proj).unwrap();

        // 合并成功：注入受管块、不视为冲突
        assert!(
            report.updated.iter().any(|f| f == "AGENTS.md"),
            "legacy 文件应注入受管块并更新"
        );
        assert!(report.conflicted.is_empty(), "含受管块的文件不应报冲突");

        // 磁盘内容 = 原用户内容 + 追加的受管块，用户内容不丢失
        let merged = std::fs::read_to_string(proj.join("AGENTS.md")).unwrap();
        assert!(merged.starts_with("# proj 编码规范\n\n## 项目专属\n- 用户规则\n\n"));
        assert!(merged
            .contains("<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n"));
        assert!(!merged.contains("managed v1"));

        // manifest 记录合并后文件的哈希
        let manifest = ProjectManifest::load(&proj).unwrap();
        assert_eq!(
            manifest.files["AGENTS.md"],
            sha256_hex(merged.as_bytes()),
            "manifest 应记录合并后文件哈希"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn update_project_injects_block_when_manifest_records_merged_hash() {
        // 复现真实 bug：旧版 adopt 已把「磁盘 + 受管块」的合并哈希写进 manifest，却没把
        // 合并结果写盘。即使 manifest 哈希与当前渲染合并结果一致，update 也必须把受管块
        // 注入到无标记的 legacy 磁盘文件，而不是被「哈希一致 → 未变」分支跳过。
        let tmp =
            std::env::temp_dir().join(format!("pengj-upd-legacy-hash-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：agent 层 AGENTS.md 含受管块
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("agent")).unwrap();
        std::fs::write(
            tpl.join("agent").join("layer.toml"),
            "name = \"Agent\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("agent").join("AGENTS.md"),
            "# {{ project_name }} 编码规范\n\n<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // 磁盘 legacy 文件（无受管块标记）+ 用户自定义内容
        let disk = "# proj 编码规范\n\n## 项目专属\n- 用户规则\n";
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("AGENTS.md"), disk).unwrap();

        // 旧版 adopt 记录的「合并哈希」= sha(磁盘 + 追加的受管块)
        let templates = Templates::new(&tpl);
        let merged = crate::block::replace_managed_block(
            disk,
            "<!-- PENGJ_TEMPLATE_START -->\nmanaged v2\n<!-- PENGJ_TEMPLATE_END -->\n",
        );
        std::fs::write(
            proj.join(crate::manifest::MANIFEST_FILE),
            serde_json::json!({
                "tool": "pengj-templates",
                "version": "0.0.0",
                "project_name": "proj",
                "layers": ["common", "agent"],
                "options": {},
                "generated_at": "2026-01-01T00:00:00Z",
                "files": { "AGENTS.md": sha256_hex(merged.as_bytes()) },
            })
            .to_string(),
        )
        .unwrap();

        let report = update_project(&templates, &proj).unwrap();
        assert!(
            report.updated.iter().any(|f| f == "AGENTS.md"),
            "磁盘无标记时即使 manifest 哈希已匹配也应注入受管块"
        );
        assert!(report.conflicted.is_empty(), "含受管块的文件不应报冲突");

        // 磁盘内容 = 原用户内容 + 追加的受管块，用户内容不丢失
        let now = std::fs::read_to_string(proj.join("AGENTS.md")).unwrap();
        assert!(now.starts_with("# proj 编码规范\n\n## 项目专属\n- 用户规则\n\n"));
        assert!(now.contains("managed v2"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn update_project_keeps_concat_file_with_managed_block_untouched() {
        // concat 累加文件（.gitignore/.gitattributes）不走受管块合并分支：其渲染结果是
        // 逐层拼接、可含多个受管块，replace_managed_block 只处理单个块会丢内容。
        // 即使模板 .gitignore 含受管块，也绝不能覆盖用户已有的 .gitignore。
        let tmp = std::env::temp_dir().join(format!("pengj-upd-concat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("common").join(".gitignore"),
            "# PENGJ_TEMPLATE_START\n/node_modules\n# PENGJ_TEMPLATE_END\n",
        )
        .unwrap();

        // 项目：用户有自己的 .gitignore，与模板 concat 内容不同
        let user_gitignore = "/target/\n/debug/\n/my-custom-ignore/\n";
        let proj = tmp.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join(".gitignore"), user_gitignore).unwrap();

        // 渲染 concat，把模板哈希写进 manifest（模拟旧版 adopt 的基线）
        let templates = Templates::new(&tpl);
        let ordered = templates.resolve_layers(&["common".to_string()]).unwrap();
        let ctx = RenderContext::new("proj", ordered.clone(), BTreeMap::new());
        let fm = templates
            .build_file_map(&ordered, &BTreeMap::new())
            .unwrap();
        let bytes_map = render_file_map(&fm, &ctx, &proj).unwrap();
        let concat_sha = sha256_hex(&bytes_map[&PathBuf::from(".gitignore")]);
        std::fs::write(
            proj.join(crate::manifest::MANIFEST_FILE),
            serde_json::json!({
                "tool": "pengj-templates",
                "version": "0.0.0",
                "project_name": "proj",
                "layers": ["common"],
                "options": {},
                "generated_at": "2026-01-01T00:00:00Z",
                "files": { ".gitignore": concat_sha },
            })
            .to_string(),
        )
        .unwrap();

        let report = update_project(&templates, &proj).unwrap();

        // 用户 .gitignore 原样保留，不被 concat 受管块覆盖，也不报冲突
        assert_eq!(
            std::fs::read_to_string(proj.join(".gitignore")).unwrap(),
            user_gitignore
        );
        assert!(!report.updated.iter().any(|f| f == ".gitignore"));
        assert!(report.conflicted.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn adopt_project_injects_managed_block_into_legacy_file() {
        let tmp = std::env::temp_dir().join(format!("pengj-adopt-mblock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：agent 层 AGENTS.md 含受管块
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("agent")).unwrap();
        std::fs::write(
            tpl.join("agent").join("layer.toml"),
            "name = \"Agent\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("agent").join("AGENTS.md"),
            "<!-- PENGJ_TEMPLATE_START -->\nmanaged v1\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // 存量项目：AGENTS.md 是 legacy 文件，没有受管块标记
        let proj = tmp.join("legacy-app");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(
            proj.join("AGENTS.md"),
            "# 项目规范\n\n## 用户自定义规则\n- 规则 A\n- 规则 B\n",
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = adopt_project(
            &templates,
            &proj,
            &["common".to_string(), "agent".to_string()],
            BTreeMap::new(),
            false,
        )
        .expect("adopt should succeed");

        assert!(report.adopted.iter().any(|f| f == "AGENTS.md"));

        // 磁盘内容 = 原用户内容 + 追加的受管块，用户内容不丢失
        let disk = std::fs::read_to_string(proj.join("AGENTS.md")).unwrap();
        assert!(disk.starts_with("# 项目规范\n\n## 用户自定义规则\n- 规则 A\n- 规则 B\n\n"));
        assert!(disk
            .contains("<!-- PENGJ_TEMPLATE_START -->\nmanaged v1\n<!-- PENGJ_TEMPLATE_END -->\n"));

        // manifest 记录合并后文件的哈希
        let manifest = ProjectManifest::load(&proj).unwrap();
        assert_eq!(
            manifest.files["AGENTS.md"],
            sha256_hex(disk.as_bytes()),
            "manifest 应记录合并后文件哈希"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn adopt_project_initializes_manifest_for_existing_directory() {
        let tmp = std::env::temp_dir().join(format!("pengj-adopt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let tpl = tmp.join("templates");
        let common = tpl.join("common");
        std::fs::create_dir_all(&common).unwrap();
        std::fs::write(
            common.join("layer.toml"),
            "name = \"Common\"\ndescription = \"Common\"\n",
        )
        .unwrap();
        std::fs::write(common.join(".gitignore"), "target/\n").unwrap();

        let existing_project = tmp.join("my-existing-app");
        std::fs::create_dir_all(&existing_project).unwrap();
        std::fs::write(existing_project.join("README.md"), "# Existing").unwrap();
        std::fs::write(
            existing_project.join(".gitignore"),
            "target/\nlocal_custom/\n",
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = adopt_project(
            &templates,
            &existing_project,
            &["common".to_string()],
            BTreeMap::new(),
            false,
        )
        .expect("adopt should succeed");

        assert_eq!(report.project_name, "my-existing-app");
        assert!(existing_project.join(".pengj-templates.json").exists());
        let manifest = ProjectManifest::load(&existing_project).unwrap();
        assert_eq!(manifest.layers, vec!["common".to_string()]);
        assert!(manifest.files.contains_key(".gitignore"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 存量项目的技能主文档是「整文件全自定义」旧形态（无托管块）时：
    /// adopt 自动接管——框架插入 frontmatter 之后、原文下移到纳管过渡区
    /// （暂时双流程）；文件入托管清单，后续 update 走正常替换且无冲突。
    #[test]
    fn adopt_takes_over_legacy_custom_skill_with_transition_zone() {
        let tmp = std::env::temp_dir().join(format!("pengj-adopt-skill-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：agent 层带一个含受管块的 commit 技能
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("agent")).unwrap();
        std::fs::write(
            tpl.join("agent").join("layer.toml"),
            "name = \"Agent\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        let tpl_skill = tpl
            .join("agent")
            .join(".agents")
            .join("skills")
            .join("commit");
        std::fs::create_dir_all(&tpl_skill).unwrap();
        std::fs::write(
            tpl_skill.join("SKILL.md"),
            "---\nname: commit\ndescription: x\n---\n\n<!-- PENGJ_TEMPLATE_START -->\n模板流程 v1\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // 存量项目：全自定义技能（无托管块、非空）
        let proj = tmp.join("legacy-app");
        let user_skill = proj.join(".agents").join("skills").join("commit");
        std::fs::create_dir_all(&user_skill).unwrap();
        let user_content =
            "---\nname: commit\ndescription: 我自己的提交规范\n---\n\n# 我的流程\n- 领域三问\n";
        std::fs::write(user_skill.join("SKILL.md"), user_content).unwrap();

        let templates = Templates::new(&tpl);
        let report = adopt_project(
            &templates,
            &proj,
            &["common".to_string(), "agent".to_string()],
            BTreeMap::new(),
            false,
        )
        .expect("adopt should succeed");

        // 接管结果：模板整页为准（frontmatter 含 description 用模板的），
        // 用户 frontmatter 丢弃，原正文保留在过渡区下方
        let merged_text = std::fs::read_to_string(user_skill.join("SKILL.md")).unwrap();
        assert!(
            merged_text.starts_with("---\nname: commit\ndescription: x\n---"),
            "应以模板 frontmatter 开头，实际:\n{merged_text}"
        );
        assert!(
            !merged_text.contains("description: 我自己的提交规范"),
            "用户 frontmatter 应被模板覆盖丢弃"
        );
        assert_eq!(
            merged_text.matches("name: commit").count(),
            1,
            "全文只应有模板的 frontmatter"
        );
        let block_start = merged_text
            .find("PENGJ_TEMPLATE_START")
            .expect("应注入模板框架");
        assert!(merged_text.contains("模板流程 v1"));
        assert!(
            merged_text.contains("纳管过渡区") && merged_text.contains("# 我的流程"),
            "原全文应整体下移到过渡区下方"
        );
        assert!(
            block_start > merged_text.find("---\nname: commit").unwrap(),
            "框架应在 frontmatter 之后"
        );
        // 入托管清单 + 报双流程复核提示
        let manifest = ProjectManifest::load(&proj).unwrap();
        assert!(
            manifest
                .files
                .keys()
                .any(|k| k.replace('\\', "/").contains("commit/SKILL.md")),
            "接管后应入托管清单"
        );
        assert!(
            report
                .needs_review
                .iter()
                .any(|s| s.contains("SKILL.md") && s.contains("双流程")),
            "应报双流程复核提示，实际: {:?}",
            report.needs_review
        );

        // 后续 update：走正常块替换路径——无冲突、幂等
        let upd = update_project(&templates, &proj).expect("update should succeed");
        assert!(
            !upd.conflicted.iter().any(|c| c.path.contains("SKILL.md")),
            "接管后不应再报冲突，实际: {:?}",
            upd.conflicted
        );
        let again = update_project(&templates, &proj).unwrap();
        assert!(!again.updated.iter().any(|f| f.contains("SKILL.md")));
        assert_eq!(
            std::fs::read_to_string(user_skill.join("SKILL.md")).unwrap(),
            merged_text,
            "二次 update 不应改动文件"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 磁盘上的技能文档已含托管块（旧版模板生成）：守卫不得误触发，
    /// adopt 仍走正常的「原位替换块区间」路径并纳入托管清单。
    #[test]
    fn adopt_still_replaces_managed_block_in_existing_skill() {
        let tmp = std::env::temp_dir().join(format!("pengj-adopt-skill2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("agent")).unwrap();
        std::fs::write(
            tpl.join("agent").join("layer.toml"),
            "name = \"Agent\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        let tpl_skill = tpl
            .join("agent")
            .join(".agents")
            .join("skills")
            .join("caveman");
        std::fs::create_dir_all(&tpl_skill).unwrap();
        std::fs::write(
            tpl_skill.join("SKILL.md"),
            "---\nname: caveman\ndescription: x\n---\n\n<!-- PENGJ_TEMPLATE_START -->\n模板 v1\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // 项目：同结构旧版文件（含托管块 + 块外用户内容）
        let proj = tmp.join("legacy-app");
        let user_skill = proj.join(".agents").join("skills").join("caveman");
        std::fs::create_dir_all(&user_skill).unwrap();
        std::fs::write(
            user_skill.join("SKILL.md"),
            "---\nname: caveman\ndescription: x\n---\n\n<!-- PENGJ_TEMPLATE_START -->\n旧版 v0\n<!-- PENGJ_TEMPLATE_END -->\n\n## 用户备注\n- 保留\n",
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = adopt_project(
            &templates,
            &proj,
            &["common".to_string(), "agent".to_string()],
            BTreeMap::new(),
            false,
        )
        .expect("adopt should succeed");

        // 块区间被替换为模板 v1，块外用户内容保留；纳入托管清单
        let merged = std::fs::read_to_string(user_skill.join("SKILL.md")).unwrap();
        assert!(merged.contains("模板 v1"));
        assert!(!merged.contains("旧版 v0"));
        assert!(merged.contains("## 用户备注"));
        let manifest = ProjectManifest::load(&proj).unwrap();
        assert!(
            manifest
                .files
                .keys()
                .any(|k| k.replace('\\', "/").contains("caveman/SKILL.md")),
            "已托管的技能应入清单，实际: {:?}",
            manifest.files.keys().collect::<Vec<_>>()
        );
        assert!(
            !report.needs_review.iter().any(|s| s.contains("caveman")),
            "正常替换不应报未接管"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 复刻 ChahuRenderDebugger 纳管场景：存量项目自带 .cargo/config.toml（同名
    /// target 表）、带版本钉的 package.json、手写 commitlint.config.js 与自定义
    /// AGENTS.md。adopt 必须产出合法 TOML（无重复表头）、保留用户版本钉与脚本、
    /// 上报 needs_review 与 commitlint 接线指引；随后 update 幂等。
    #[test]
    fn adopt_legacy_project_full_scenario_then_update_is_idempotent() {
        let tmp = std::env::temp_dir().join(format!("pengj-adopt-chahu-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // ---- 模板三层：common / hooks（lefthook 类）/ rustx（cargo 配置）----
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();

        std::fs::create_dir_all(tpl.join("hooks")).unwrap();
        std::fs::write(
            tpl.join("hooks").join("layer.toml"),
            "name = \"Hooks\"\ndescription = \"x\"\ndepends = [\"common\"]\nupdate_ignore = [\"commitlint.config.js\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("hooks").join("commitlint.base.js"),
            "export default { extends: ['@commitlint/config-conventional'] };\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("hooks").join("commitlint.config.js"),
            "import base from './commitlint.base.js';\nexport default { ...base };\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("hooks").join("package.json"),
            "{\n  \"name\": \"{{ project_slug }}\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"engines\": { \"node\": \">=20\" },\n  \"scripts\": {\n    \"prepare\": \"template-prepare\",\n    \"commitlint\": \"commitlint\"\n  },\n  \"devDependencies\": {\n    \"@commitlint/cli\": \"latest\",\n    \"@commitlint/config-conventional\": \"latest\",\n    \"lefthook\": \"latest\"\n  }\n}\n",
        )
        .unwrap();

        std::fs::create_dir_all(tpl.join("rustx").join(".cargo")).unwrap();
        std::fs::write(
            tpl.join("rustx").join("layer.toml"),
            "name = \"RustX\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("rustx").join(".cargo").join("config.toml"),
            "# PENGJ_TEMPLATE_START\n[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\n\n[target.x86_64-unknown-linux-gnu]\nlinker = \"lld\"\n# PENGJ_TEMPLATE_END\n",
        )
        .unwrap();

        std::fs::create_dir_all(tpl.join("agentdoc")).unwrap();
        std::fs::write(
            tpl.join("agentdoc").join("layer.toml"),
            "name = \"AgentDoc\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("agentdoc").join("AGENTS.md"),
            "<!-- PENGJ_TEMPLATE_START -->\n# 模板规范 v1\n<!-- PENGJ_TEMPLATE_END -->\n",
        )
        .unwrap();

        // ---- 存量项目 ----
        let proj = tmp.join("legacy-app");
        std::fs::create_dir_all(proj.join(".cargo")).unwrap();
        std::fs::write(
            proj.join(".cargo").join("config.toml"),
            "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\nrustflags = [\"-C\", \"target-cpu=x86-64-v3\"]\n",
        )
        .unwrap();
        let user_pkg = "{\n  \"name\": \"legacy-app\",\n  \"version\": \"0.2.0\",\n  \"private\": true,\n  \"description\": \"my app\",\n  \"scripts\": {\n    \"prepare\": \"lefthook install\",\n    \"postinstall\": \"echo done\"\n  },\n  \"devDependencies\": {\n    \"@commitlint/cli\": \"^20.5.3\",\n    \"lefthook\": \"^2.1.10\",\n    \"user-only-lib\": \"^1.0.0\"\n  }\n}\n";
        std::fs::write(proj.join("package.json"), user_pkg).unwrap();
        std::fs::write(
            proj.join("commitlint.config.js"),
            "export default { extends: ['@commitlint/config-conventional'], rules: { 'scope-enum': [2, 'always', ['core']] } };\n",
        )
        .unwrap();
        std::fs::write(proj.join("AGENTS.md"), "# 项目自有规范\n- 规则 A\n").unwrap();

        let templates = Templates::new(&tpl);
        let layers = [
            "common".to_string(),
            "hooks".to_string(),
            "rustx".to_string(),
            "agentdoc".to_string(),
        ];
        let report = adopt_project(&templates, &proj, &layers, BTreeMap::new(), false)
            .expect("adopt should succeed");

        // -- commitlint 自动接线：config 被改写为继承 base，无需手动步骤 --
        assert!(report.created.iter().any(|f| f == "commitlint.base.js"));
        assert!(
            report.manual_steps.is_empty(),
            "ESM 对象字面量应自动接线，不应有手动步骤，实际: {:?}",
            report.manual_steps
        );
        let wired_cfg = std::fs::read_to_string(proj.join("commitlint.config.js")).unwrap();
        assert!(wired_cfg.contains("import base from './commitlint.base.js';"));
        assert!(wired_cfg.contains("const pengjUserConfig ="));
        assert!(wired_cfg.contains("...base.rules"));
        assert!(
            report
                .needs_review
                .iter()
                .any(|s| s.starts_with("commitlint.config.js")),
            "自动接线应标记复核，实际: {:?}",
            report.needs_review
        );
        // 用户原有规则保留在 pengjUserConfig 中
        assert!(wired_cfg.contains("'scope-enum'"));

        // -- TOML 结构化合并：合法、去重、保用户键 --
        let cfg_text = std::fs::read_to_string(proj.join(".cargo").join("config.toml")).unwrap();
        let cfg: toml::Value = toml::from_str(&cfg_text).expect("合并结果必须是合法 TOML");
        assert_eq!(
            cfg["target"]["x86_64-pc-windows-msvc"]["linker"].as_str(),
            Some("rust-lld")
        );
        assert_eq!(
            cfg["target"]["x86_64-pc-windows-msvc"]["rustflags"]
                .as_array()
                .map(|a| a.len()),
            Some(2),
            "用户 rustflags 必须原样保留"
        );
        assert_eq!(
            cfg["target"]["x86_64-unknown-linux-gnu"]["linker"].as_str(),
            Some("lld")
        );
        assert_eq!(
            cfg_text.matches("[target.x86_64-pc-windows-msvc]").count(),
            1,
            "windows 表不得重复定义"
        );
        assert!(
            report
                .needs_review
                .iter()
                .any(|f| f.replace('\\', "/").starts_with(".cargo/config.toml")),
            "TOML 首次接入应标记复核，实际: {:?}",
            report.needs_review
        );

        // -- package.json 模板优先并集 + 键序保持 --
        let pkg_text = std::fs::read_to_string(proj.join("package.json")).unwrap();
        assert!(
            pkg_text.starts_with("{\n  \"name\": \"legacy-app\""),
            "首键必须仍是用户的 name（键序保持），实际: {pkg_text}"
        );
        assert!(
            pkg_text.contains("\"@commitlint/cli\": \"latest\""),
            "同名依赖版本一律以模板为准（模板 latest 覆盖用户 ^20.5.3），实际: {pkg_text}"
        );
        assert!(pkg_text.contains("\"lefthook\": \"latest\""));
        assert!(
            pkg_text.contains("\"user-only-lib\": \"^1.0.0\""),
            "用户独有依赖保留"
        );
        assert!(
            pkg_text.contains("\"prepare\": \"lefthook install\""),
            "用户脚本不被模板覆盖"
        );
        assert!(
            pkg_text.contains("\"commitlint\": \"commitlint\""),
            "模板新脚本并入，实际:\n{pkg_text}"
        );
        assert!(pkg_text.contains("\"@commitlint/config-conventional\""));
        assert!(pkg_text.contains("\"engines\""));

        // -- 文本受管块追加进 legacy AGENTS.md 并标记复核 --
        let agents = std::fs::read_to_string(proj.join("AGENTS.md")).unwrap();
        assert!(agents.starts_with("# 项目自有规范"));
        assert!(agents.contains("PENGJ_TEMPLATE_START"));
        assert!(
            report
                .needs_review
                .iter()
                .any(|f| f.starts_with("AGENTS.md")),
            "追加进 legacy 文件应标记复核，实际: {:?}",
            report.needs_review
        );

        // -- update 幂等：不再有更新/新增/冲突 --
        let upd = update_project(&templates, &proj).expect("update should succeed");
        assert!(upd.updated.is_empty(), "不应再有更新: {:?}", upd.updated);
        assert!(upd.created.is_empty());
        assert!(
            upd.conflicted.is_empty(),
            "不应有冲突: {:?}",
            upd.conflicted
        );
        assert!(upd.needs_review.is_empty());
        assert_eq!(
            std::fs::read_to_string(proj.join(".cargo").join("config.toml")).unwrap(),
            cfg_text
        );
        assert_eq!(
            std::fs::read_to_string(proj.join("package.json")).unwrap(),
            pkg_text
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn merge_json_update_dep_versions_follow_template_pins() {
        // 更新/纳管语义：依赖类字段同名包版本一律以模板为准（含模板未定版的
        // latest）；脚本仍用户优先
        let mut base: serde_json::Value = serde_json::from_str(
            r#"{"scripts":{"prepare":"user-prepare"},"devDependencies":{"lefthook":"^2.1.10","@commitlint/cli":"^20.5.3"}}"#,
        )
        .unwrap();
        let incoming: serde_json::Value = serde_json::from_str(
            r#"{"scripts":{"prepare":"tpl-prepare"},"devDependencies":{"lefthook":"latest","@commitlint/cli":"^21.2.2"}}"#,
        )
        .unwrap();

        merge_json(&mut base, &incoming, false);
        assert_eq!(
            base["devDependencies"]["@commitlint/cli"], "^21.2.2",
            "模板固定版本覆盖用户旧版本钉"
        );
        assert_eq!(
            base["devDependencies"]["lefthook"], "latest",
            "模板 latest 同样覆盖用户版本钉（解析版本由 lockfile 决定）"
        );
        assert_eq!(base["scripts"]["prepare"], "user-prepare", "脚本保持用户值");
    }

    #[test]
    fn try_wire_commitlint_base_dedupes_rules_and_members_matching_base() {
        let tmp = std::env::temp_dir().join(format!("pengj-wire-dedupe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("commitlint.base.js"),
            "export default {\n  extends: ['@commitlint/config-conventional'],\n  rules: {\n    'body-max-line-length': [0],\n    'type-enum': [2, 'always', ['feat', 'fix']],\n  },\n};\n",
        )
        .unwrap();
        // 用户配置：body-max-line-length / type-enum 与 base 完全等价（仅格式不同），
        // scope-enum 是项目差异；extends 也与 base 相同
        std::fs::write(
            tmp.join("commitlint.config.js"),
            "export default {\n  extends: [\"@commitlint/config-conventional\"],\n  rules: {\n    'body-max-line-length': [0],\n    'scope-enum': [2, 'always', ['core']],\n    \"type-enum\": [\n      2,\n      'always',\n      ['feat', 'fix'],\n    ],\n  },\n};\n",
        )
        .unwrap();

        assert_eq!(
            try_wire_commitlint_base(&tmp).unwrap(),
            CommitlintWireOutcome::Wired
        );

        let wired = std::fs::read_to_string(tmp.join("commitlint.config.js")).unwrap();
        assert!(wired.contains("import base from './commitlint.base.js';"));
        assert!(wired.contains("const pengjUserConfig ="));
        // 项目差异保留
        assert!(wired.contains("'scope-enum'"));
        // 等价条目已从用户配置中删除（全文只应出现在 base 引用与展开处，不再有字面量）
        assert!(
            !wired.contains("'body-max-line-length'"),
            "等价 rules 条目应被去重，实际:\n{wired}"
        );
        assert!(
            !wired.contains("@commitlint/config-conventional"),
            "等价 extends 应被去重，实际:\n{wired}"
        );
        // 展开结构完整，仍是合法导出
        assert!(wired.contains("...base.rules"));
        assert!(wired.trim_end().ends_with("};"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn try_wire_commitlint_base_transforms_esm_object_config() {
        let tmp = std::env::temp_dir().join(format!("pengj-wire-esm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("commitlint.base.js"), "export default {};\n").unwrap();
        std::fs::write(
            tmp.join("commitlint.config.js"),
            "export default {\n  extends: ['@commitlint/config-conventional'],\n  rules: { 'scope-enum': [2, 'always', ['core']] },\n};\n",
        )
        .unwrap();

        assert_eq!(
            try_wire_commitlint_base(&tmp).unwrap(),
            CommitlintWireOutcome::Wired
        );
        // 二次调用幂等：已引用 base
        assert_eq!(
            try_wire_commitlint_base(&tmp).unwrap(),
            CommitlintWireOutcome::AlreadyWired
        );

        let wired = std::fs::read_to_string(tmp.join("commitlint.config.js")).unwrap();
        assert!(wired.starts_with("import base from './commitlint.base.js';"));
        assert!(wired.contains("const pengjUserConfig ="));
        assert!(wired.contains("'scope-enum'"), "用户规则保留");
        assert!(wired.contains("...base.rules"));
        assert!(wired.trim_end().ends_with("};"));
        // 改写结果仍是合法 JS 导出结构：base 铺底在前、用户配置展开在后
        let export_pos = wired.rfind("export default").unwrap();
        let tail = &wired[export_pos..];
        assert!(tail.find("...base,").unwrap() < tail.find("...pengjUserConfig,").unwrap());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn try_wire_commitlint_base_requires_manual_for_cjs_or_non_literal() {
        let tmp = std::env::temp_dir().join(format!("pengj-wire-cjs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("commitlint.base.js"), "export default {};\n").unwrap();

        // CJS module.exports：不动文件，报 ManualRequired
        std::fs::write(
            tmp.join("commitlint.config.js"),
            "module.exports = { rules: {} };\n",
        )
        .unwrap();
        assert_eq!(
            try_wire_commitlint_base(&tmp).unwrap(),
            CommitlintWireOutcome::ManualRequired
        );
        assert_eq!(
            std::fs::read_to_string(tmp.join("commitlint.config.js")).unwrap(),
            "module.exports = { rules: {} };\n",
            "无法改写时不得动用户文件"
        );

        // export default 后不是对象字面量（工厂函数）：同样保守跳过
        std::fs::write(
            tmp.join("commitlint.config.js"),
            "export default makeConfig();\n",
        )
        .unwrap();
        assert_eq!(
            try_wire_commitlint_base(&tmp).unwrap(),
            CommitlintWireOutcome::ManualRequired
        );

        // 无 config 文件：NoConfig
        std::fs::remove_file(tmp.join("commitlint.config.js")).unwrap();
        assert_eq!(
            try_wire_commitlint_base(&tmp).unwrap(),
            CommitlintWireOutcome::NoConfig
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn adopt_merges_settings_into_workspace_file_instead_of_creating_settings() {
        let tmp = std::env::temp_dir().join(format!("pengj-adopt-ws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 模板：common + vscode 层
        let tpl = tmp.join("templates");
        std::fs::create_dir_all(tpl.join("common")).unwrap();
        std::fs::write(
            tpl.join("common").join("layer.toml"),
            "name = \"Common\"\ndescription = \"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tpl.join("vscode").join(".vscode")).unwrap();
        std::fs::write(
            tpl.join("vscode").join("layer.toml"),
            "name = \"VS Code\"\ndescription = \"x\"\ndepends = [\"common\"]\n",
        )
        .unwrap();
        std::fs::write(
            tpl.join("vscode").join(".vscode").join("settings.json"),
            r#"{
  "explorer.fileNesting.enabled": true,
  "explorer.fileNesting.patterns": { "Cargo.toml": "tpl" }
}"#,
        )
        .unwrap();

        // 存量项目：已有 *.code-workspace（含用户自定义 settings），没有 .vscode/settings.json
        let proj = tmp.join("legacy-ws-app");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(
            proj.join("my.code-workspace"),
            r#"{
  "folders": [{ "path": "." }],
  "settings": {
    "editor.fontSize": 14,
    "explorer.fileNesting.patterns": { "README.md": "keep-me" }
  }
}"#,
        )
        .unwrap();

        let templates = Templates::new(&tpl);
        let report = adopt_project(
            &templates,
            &proj,
            &["common".to_string(), "vscode".to_string()],
            BTreeMap::new(),
            false,
        )
        .expect("adopt should succeed");

        // 不新建独立 settings 文件；配置并入 workspace 文件
        assert!(
            !proj.join(".vscode").join("settings.json").exists(),
            "有工作区文件时不得新建 .vscode/settings.json"
        );
        assert!(
            report
                .needs_review
                .iter()
                .any(|s| s.contains("code-workspace")),
            "并入工作区文件应标记复核，实际: {:?}",
            report.needs_review
        );
        let ws: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(proj.join("my.code-workspace")).unwrap())
                .unwrap();
        assert_eq!(
            ws["settings"]["editor.fontSize"],
            serde_json::json!(14),
            "用户标量保留"
        );
        assert_eq!(
            ws["settings"]["explorer.fileNesting.enabled"],
            serde_json::json!(true)
        );
        assert_eq!(
            ws["settings"]["explorer.fileNesting.patterns"]["Cargo.toml"],
            serde_json::json!("tpl")
        );
        assert_eq!(
            ws["settings"]["explorer.fileNesting.patterns"]["README.md"],
            serde_json::json!("keep-me"),
            "用户 patterns 其它 key 保留"
        );

        // 后续 update：不创建独立 settings 文件、workspace 合并幂等
        let upd = update_project(&templates, &proj).expect("update should succeed");
        assert!(!upd.created.iter().any(|f| f.contains("settings.json")));
        assert!(!proj.join(".vscode").join("settings.json").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn merge_json_update_keeps_user_values_and_fills_missing_only() {
        let mut base: serde_json::Value = serde_json::from_str(
            "{\"aaa\":\"keep\",\"scripts\":{\"build\":\"user-build\",\"prepare\":\"user-prepare\"},\"devDependencies\":{\"lefthook\":\"^2.1.10\"}}",
        )
        .unwrap();
        let incoming: serde_json::Value = serde_json::from_str(
            "{\"bbb\":\"tpl\",\"scripts\":{\"prepare\":\"tpl-prepare\",\"test\":\"tpl-test\"},\"devDependencies\":{\"lefthook\":\"latest\",\"@commitlint/cli\":\"latest\"}}",
        )
        .unwrap();

        // 更新语义（overwrite_other=false）：脚本同名键用户优先、缺失才补；
        // 依赖类字段同名包版本一律以模板为准
        merge_json(&mut base, &incoming, false);
        assert_eq!(base["aaa"], "keep");
        assert_eq!(base["bbb"], "tpl");
        assert_eq!(base["scripts"]["prepare"], "user-prepare");
        assert_eq!(base["scripts"]["build"], "user-build");
        assert_eq!(base["scripts"]["test"], "tpl-test");
        assert_eq!(
            base["devDependencies"]["lefthook"], "latest",
            "依赖版本跟随模板"
        );
        assert_eq!(
            base["devDependencies"]["@commitlint/cli"], "latest",
            "用户缺失的依赖补齐"
        );

        // 生成语义（overwrite_other=true，层间合并）：后来层覆盖
        let mut fresh = serde_json::json!({});
        merge_json(&mut fresh, &incoming, true);
        assert_eq!(fresh["scripts"]["prepare"], "tpl-prepare");
    }
}
