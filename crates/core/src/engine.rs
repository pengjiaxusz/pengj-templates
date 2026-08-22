use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::context::RenderContext;
use crate::error::{CoreError, Result};
use crate::layer::{LayerInfo, LayerMeta, LAYER_META_FILE};
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

/// 需要按层累加拼接而非后层覆盖的文件（如 `.gitignore`）。
/// 每个层贡献一段，按依赖顺序拼接，每段前带来源层注释。
const ACCUMULATE_FILES: &[&str] = &[".gitignore"];

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

/// 模板文件分三类收集：普通（后层覆盖）、拼接（.gitignore）、JSON 并集（package.json）
struct FileMap {
    /// 普通文件：路径 -> 最终原始字节（未渲染，后层覆盖前层）
    normal: BTreeMap<PathBuf, Vec<u8>>,
    /// 拼接累加文件：路径 -> [(层名, 该层原始字节)]，保持依赖顺序
    concat: BTreeMap<PathBuf, Vec<(String, Vec<u8>)>>,
    /// JSON 并集文件：路径 -> [(层名, 该层原始字节)]，保持依赖顺序
    json: BTreeMap<PathBuf, Vec<(String, Vec<u8>)>>,
}

/// 按合并顺序收集模板文件，按类型分发到对应桶
impl Templates {
    fn build_file_map(&self, ordered: &[String]) -> Result<FileMap> {
        let mut fm = FileMap {
            normal: BTreeMap::new(),
            concat: BTreeMap::new(),
            json: BTreeMap::new(),
        };
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
    let fm = templates.build_file_map(&ordered)?;
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
    let fm = templates.build_file_map(&ordered)?;
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
                    Some(_) => {
                        // 本地被用户改过，保留记录，跳过
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
                let target = project_dir.join(rel);
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
