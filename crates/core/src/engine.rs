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
/// - 生成时：各层按依赖顺序并集合并（模板版本覆盖同名依赖，用户自加项不冲突）
/// - 更新时：以用户现有文件为底，模板的依赖/脚本并入（同名依赖模板优先），
///   用户自己加的库与其余字段（name/version 等标量）保留不动
const MERGE_JSON_FILES: &[&str] = &["package.json"];

/// package.json 里「模板可覆盖/并集」的字段（依赖与脚本），其余标量以用户为准
const MERGE_JSON_UNION_KEYS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "scripts",
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
/// - 依赖/脚本字段：并集，同名项 `incoming` 优先
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
                        for (ik, ivv) in iv {
                            bm.insert(ik.clone(), ivv.clone());
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
/// `project_dir` 用于判断合并型文件是否已存在：
/// - 不存在（生成空项目）-> 以各层为底并集
/// - 已存在（更新）-> 以现有用户文件为底并集，保留用户字段
fn render_file_map(
    fm: &FileMap,
    ctx: &RenderContext,
    project_dir: &Path,
) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut out: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();

    for (rel, bytes) in &fm.normal {
        out.insert(rel.clone(), render_bytes(bytes, ctx)?);
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct AdoptReport {
    pub project_name: String,
    pub layers: Vec<String>,
    pub created: Vec<String>,
    pub adopted: Vec<String>,
}

/// 纳管存量项目：为已有目录初始化 `.pengj-templates.json` manifest
///
/// 逻辑：
/// 1. 若项目根目录已存在 manifest，且没有指定 force，则返回错误。
/// 2. 解析所选层与渲染选项，在内存中渲染出完整的模板 FileMap。
/// 3. 对模板渲染出的每个文件：
///    - 若本地文件已存在：
///      - 尝试通过锚点合并（`try_merge_slots`）同步受保护的模板区域；
///      - 记录当前磁盘文件的哈希（或合并后哈希）作为初始基线，计入 `adopted`。
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

        if target.exists() {
            let cur = std::fs::read(&target).unwrap_or_default();
            if let Some(merged) = try_merge_slots(&cur, bytes) {
                if merged != cur {
                    write_file(&target, &merged)?;
                }
                manifest_files.insert(rel_str.clone(), sha256_hex(&merged));
            } else {
                // 存量文件保留用户当前内容，基线记录当前磁盘文件的哈希
                manifest_files.insert(rel_str.clone(), sha256_hex(&cur));
            }
            adopted.push(rel_str);
        } else {
            write_file(&target, bytes)?;
            manifest_files.insert(rel_str.clone(), sha256_hex(bytes));
            created.push(rel_str);
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
    })
}

// ---------- 锚点区域保护合并 (Slot / Anchor Merge) ----------

const SLOT_MARKERS: &[(&str, &str)] = &[
    (
        "<!-- PENGJ_TEMPLATE_START -->",
        "<!-- PENGJ_TEMPLATE_END -->",
    ),
    ("// PENGJ_TEMPLATE_START", "// PENGJ_TEMPLATE_END"),
    ("# PENGJ_TEMPLATE_START", "# PENGJ_TEMPLATE_END"),
    ("/* PENGJ_TEMPLATE_START */", "/* PENGJ_TEMPLATE_END */"),
];

/// 尝试按锚点标记将模板中的受保护区间合并入磁盘现有文件。
///
/// 逻辑：
/// 若模板内容与磁盘文件均包含同一种合法配对的锚点标记 `(start_marker, end_marker)`，
/// 则将磁盘文件中从 `start_marker` 到 `end_marker`（含标记本身）的区间替换为
/// 模板中对应区间的完整内容，保留磁盘文件在标记外部的所有用户定制代码/文字。
pub fn try_merge_slots(disk_bytes: &[u8], template_bytes: &[u8]) -> Option<Vec<u8>> {
    let disk_str = std::str::from_utf8(disk_bytes).ok()?;
    let template_str = std::str::from_utf8(template_bytes).ok()?;

    for &(start_m, end_m) in SLOT_MARKERS {
        if let (Some(t_start), Some(d_start)) = (template_str.find(start_m), disk_str.find(start_m))
        {
            let t_end_rel = template_str[t_start..].find(end_m)?;
            let t_end = t_start + t_end_rel + end_m.len();

            let d_end_rel = disk_str[d_start..].find(end_m)?;
            let d_end = d_start + d_end_rel + end_m.len();

            let template_section = &template_str[t_start..t_end];

            let mut merged = String::with_capacity(disk_str.len() + template_section.len());
            merged.push_str(&disk_str[..d_start]);
            merged.push_str(template_section);
            merged.push_str(&disk_str[d_end..]);

            return Some(merged.into_bytes());
        }
    }

    None
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
    pub unchanged: usize,
}

/// 按 manifest 记录把模板的最新内容同步到已生成的项目
///
/// 规则：
/// - 模板文件内容未变 -> 跳过
/// - 模板变了、本地文件未动过 -> 覆盖
/// - 模板变了、本地文件被改过 -> 冲突，跳过并上报
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

        // JSON 并集文件：合并结果按构造保留了用户字段，直接写回，不再冲突跳过
        if json_merge_keys.contains(rel.as_path()) {
            let target = project_dir.join(rel);
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
        if rel == Path::new(VSCODE_SETTINGS_REL) {
            let target = project_dir.join(rel);
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

        match manifest.files.get(&rel_str) {
            Some(old_sha) if *old_sha == new_sha => {
                unchanged += 1;
                new_files.insert(rel_str, new_sha);
            }
            Some(old_sha) => {
                let target = project_dir.join(rel);
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
                    Some(cur) => {
                        // 本地被用户改过：优先尝试锚点插槽合并（保留锚点外用户代码）
                        if let Some(merged_bytes) = try_merge_slots(&cur, bytes) {
                            if merged_bytes == cur {
                                unchanged += 1;
                                new_files.insert(rel_str, sha256_hex(&cur));
                            } else {
                                write_file(&target, &merged_bytes)?;
                                let merged_sha = sha256_hex(&merged_bytes);
                                new_files.insert(rel_str.clone(), merged_sha);
                                updated.push(rel_str);
                            }
                        } else {
                            // 无锚点或锚点不匹配，保持冲突跳过
                            new_files.insert(rel_str.clone(), old_sha.clone());
                            conflicted.push(ConflictInfo {
                                path: rel_str,
                                reason: "文件已被本地修改，跳过更新".to_string(),
                            });
                        }
                    }
                }
            }
            None => {
                // 模板新增的文件
                let target = project_dir.join(rel);
                if target.exists() {
                    let cur = std::fs::read(&target).unwrap_or_default();
                    if let Some(merged_bytes) = try_merge_slots(&cur, bytes) {
                        if merged_bytes == cur {
                            unchanged += 1;
                            new_files.insert(rel_str, sha256_hex(&cur));
                        } else {
                            write_file(&target, &merged_bytes)?;
                            let merged_sha = sha256_hex(&merged_bytes);
                            new_files.insert(rel_str.clone(), merged_sha);
                            updated.push(rel_str);
                        }
                    } else {
                        conflicted.push(ConflictInfo {
                            path: rel_str.clone(),
                            reason: "文件已存在但未被模板托管，跳过".to_string(),
                        });
                    }
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

    manifest.files = new_files;
    manifest.save(project_dir)?;

    Ok(UpdateReport {
        project_name: manifest.project_name.clone(),
        layers: ordered,
        updated,
        created,
        conflicted,
        removed,
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

    // 读取 workspace：失败即跳过（不 panic）
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(false);
    };
    let Ok(mut ws) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Ok(false);
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

    let mut out = serde_json::to_string_pretty(&ws)?;
    out.push('\n');
    std::fs::write(path, out)?;
    Ok(true)
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
    fn try_merge_slots_preserves_surrounding_custom_content() {
        let disk = b"# Header\n\n<!-- PENGJ_TEMPLATE_START -->\nold template\n<!-- PENGJ_TEMPLATE_END -->\n\n## Custom User Rules\n- Rule 1\n";
        let tmpl =
            b"<!-- PENGJ_TEMPLATE_START -->\nnew updated template\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged = try_merge_slots(disk, tmpl).expect("should merge slot");
        let merged_str = String::from_utf8(merged).unwrap();
        assert!(merged_str.starts_with("# Header\n\n"));
        assert!(merged_str.contains("new updated template"));
        assert!(merged_str.ends_with("\n\n## Custom User Rules\n- Rule 1\n"));

        // Shell/TOML 风格 # 注释
        let disk_hash = b"# PENGJ_TEMPLATE_START\nold cfg\n# PENGJ_TEMPLATE_END\ncustom = 1\n";
        let tmpl_hash = b"# PENGJ_TEMPLATE_START\nnew cfg\n# PENGJ_TEMPLATE_END\n";
        let merged_hash = try_merge_slots(disk_hash, tmpl_hash).expect("should merge # slot");
        assert_eq!(
            String::from_utf8(merged_hash).unwrap(),
            "# PENGJ_TEMPLATE_START\nnew cfg\n# PENGJ_TEMPLATE_END\ncustom = 1\n"
        );
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
}
