//! `.cargo/config.toml` 等结构化 TOML 文件的受管块合并
//!
//! 文本级托管块追加（[`crate::block::replace_managed_block`]）对 TOML 不安全：
//! 模板受管块里的表（如 `[target.x86_64-pc-windows-msvc]`）可能与用户既有配置
//! 中的同名表重复，产生非法的重复表定义，直接导致 cargo 无法解析整个文件。
//!
//! 本模块按「表级并集 + 键级去重」做结构化合并：
//! - 模板受管块中用户完全没有的表 → 整表保留进新的受管块；
//! - 模板与用户同键同值 → 去重（从受管块剔除；用户区已有等价定义）；
//! - 模板要往用户已有的表里补新键 / 同键不同值 → 冲突：整个文件不写盘，
//!   逐条上报冲突明细，由用户手工对齐后重跑 `update`
//!   （TOML 禁止重复表头，无法把一张表拆到文件两处，故不能静默代劳）。
//!
//! 用户区（受管块之外）的内容永不改动：用户多出来的键原样保留。

use std::sync::OnceLock;

use toml::Value;

use crate::block::{extract_managed_block, BlockStyle};

/// 走 TOML 结构化受管合并的文件（相对项目根路径，统一以 `/` 分隔）
pub(crate) const MERGE_TOML_MANAGED_FILES: &[&str] = &[".cargo/config.toml"];

/// 判断相对路径是否需要 TOML 结构化合并（跨平台：把 `\` 归一成 `/` 再比较）
pub(crate) fn is_toml_managed(rel: &std::path::Path) -> bool {
    let rel_str = rel.to_str().unwrap_or("").replace('\\', "/");
    MERGE_TOML_MANAGED_FILES.contains(&rel_str.as_str())
}

/// TOML 受管合并结果
pub(crate) enum TomlMergeOutcome {
    /// 合并完成；携带最终完整文件内容（调用方自行与磁盘比对决定是否写盘）
    Merged(String),
    /// 键值冲突，禁止写盘；携带人类可读的冲突明细（逐条一行）
    Conflict(String),
}

/// 对磁盘现有内容 `disk_text` 与模板渲染结果 `incoming_text` 做结构化合并。
///
/// 规则：
/// - 渲染结果不含受管块 → 原样返回渲染结果（该文件类型正常都含块，防御式兜底）
/// - 磁盘既有受管块 → 先整体移除，剩余部分视为用户区
/// - 合并后的新受管块统一追加到用户区末尾
pub(crate) fn merge_toml_managed(disk_text: &str, incoming_text: &str) -> TomlMergeOutcome {
    let Some(incoming_block) = extract_managed_block(incoming_text) else {
        return TomlMergeOutcome::Merged(incoming_text.to_string());
    };

    // 拆出用户区（移除磁盘上既有的受管块，若有）
    let user_part = match extract_managed_block(disk_text) {
        Some(old) => format!("{}{}", &disk_text[..old.start], &disk_text[old.end..]),
        None => disk_text.to_string(),
    };

    let Ok(incoming_val) = toml::from_str::<Value>(incoming_block.body.as_str()) else {
        // 受管块正文不是合法 TOML（理论上不会发生）：回退为文本追加语义，不炸更新
        let rebuilt = rebuild_block(incoming_block.style, incoming_block.body.as_str());
        return TomlMergeOutcome::Merged(join_user_and_block(&user_part, &rebuilt));
    };
    let empty_table = Value::Table(toml::map::Map::new());
    let user_val = toml::from_str::<Value>(user_part.trim()).unwrap_or(empty_table);

    let mut conflicts: Vec<String> = Vec::new();
    let retained = subtract_covered(
        as_table(&incoming_val),
        as_table(&user_val),
        String::new(),
        &mut conflicts,
    );

    // 保留树里将要产出的表头若与用户已有表头重合 → 会产生非法的重复表定义，
    // 逐节点报冲突（TOML 禁止重复表头，无法静默代劳，需人工并表）
    let mut user_table_paths = std::collections::BTreeSet::new();
    collect_table_paths(as_table(&user_val), "", &mut user_table_paths);
    push_header_conflicts(&retained, "", &user_table_paths, &mut conflicts);

    if !conflicts.is_empty() {
        let mut msg = String::from("TOML 受管合并存在同名键冲突（用户与模板值不同），本次未写入：");
        for c in &conflicts {
            msg.push_str("\n  - ");
            msg.push_str(c);
        }
        return TomlMergeOutcome::Conflict(msg);
    }

    if retained.is_empty() {
        // 模板要写的键全部被用户区等价覆盖：受管块整体省去，只留用户区
        return TomlMergeOutcome::Merged(normalize_tail(&user_part));
    }

    let Ok(serialized) = toml::to_string_pretty(&retained) else {
        // 序列化失败（理论不会发生）：同样回退为文本追加语义
        let rebuilt = rebuild_block(incoming_block.style, incoming_block.body.as_str());
        return TomlMergeOutcome::Merged(join_user_and_block(&user_part, &rebuilt));
    };
    let rebuilt = rebuild_block(incoming_block.style, &serialized);
    TomlMergeOutcome::Merged(join_user_and_block(&user_part, &rebuilt))
}

/// 用给定风格重建受管块文本（含头注释说明归属，保证尾随换行）
fn rebuild_block(style: BlockStyle, body: &str) -> String {
    format!(
        "{}\n# Managed by pengj-templates: update replaces only this block, content outside is user-owned\n{}\n{}\n",
        style.start_marker(),
        body.trim(),
        style.end_marker()
    )
}

/// 用户区 + 受管块拼接：保证中间恰好一个空行、结尾单个换行
fn join_user_and_block(user_part: &str, block: &str) -> String {
    let mut out = normalize_tail(user_part);
    out.push('\n');
    out.push_str(block);
    out
}

/// 去掉尾部空白并保证单个换行结尾
fn normalize_tail(text: &str) -> String {
    text.trim_end().to_string() + "\n"
}

fn as_table(v: &Value) -> &toml::map::Map<String, Value> {
    static EMPTY: OnceLock<toml::map::Map<String, Value>> = OnceLock::new();
    v.as_table()
        .unwrap_or_else(|| EMPTY.get_or_init(toml::map::Map::new))
}

/// 从 `tmpl` 中剔除「已被 `user` 等价覆盖」的键，返回剩余应写入受管块的表。
///
/// - 键只在模板侧出现（整棵子树都是新表）→ 保留
/// - 两侧都是表 → 递归下钻，子树全空则整体剔除
/// - 其余情况同键不同值 → 记入 `conflicts`（含点分路径与两侧值的调试表示）；同键同值 → 剔除
fn subtract_covered(
    tmpl: &toml::map::Map<String, Value>,
    user: &toml::map::Map<String, Value>,
    prefix: String,
    conflicts: &mut Vec<String>,
) -> toml::map::Map<String, Value> {
    let mut retained = toml::map::Map::new();
    for (k, tv) in tmpl {
        let path = path_of(k, &prefix);
        match user.get(k) {
            None => {
                retained.insert(k.clone(), tv.clone());
            }
            Some(uv) => match (tv.as_table(), uv.as_table()) {
                (Some(tt), Some(ut)) => {
                    // 用户已有同名节点：只保留模板侧多出来的部分；
                    // 若剩余子树会与用户已有表头重合，由 push_header_conflicts 统一判定
                    let sub = subtract_covered(tt, ut, path, conflicts);
                    if !sub.is_empty() {
                        retained.insert(k.clone(), Value::Table(sub));
                    }
                }
                _ => {
                    if tv != uv {
                        conflicts.push(format!(
                            "{path}: 用户 {:?} / 模板 {:?}，请手工对齐后重跑 update",
                            uv, tv
                        ));
                    }
                    // 同键同值：已覆盖，剔除
                }
            },
        }
    }
    retained
}

/// 点分路径拼接
fn path_of(key: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

/// 收集 `table` 中所有表节点的点分路径（含中间层；根路径 "" 不记录）
fn collect_table_paths(
    table: &toml::map::Map<String, Value>,
    prefix: &str,
    out: &mut std::collections::BTreeSet<String>,
) {
    for (k, v) in table {
        if let Some(t) = v.as_table() {
            let path = path_of(k, prefix);
            out.insert(path.clone());
            collect_table_paths(t, &path, out);
        }
    }
}

/// 检查保留树中将要产出的表头是否与用户已有表头重合：重合即冲突。
///
/// 只有「带直接叶子值」的表节点才会被序列化成 `[表头]`；纯中间层节点
/// （只含子表）不产生表头。`[target.a]` 与 `[target.b]` 是不同表头、可以共存；
/// 只有完全相同的表头路径才是非法 TOML。
fn push_header_conflicts(
    retained: &toml::map::Map<String, Value>,
    prefix: &str,
    user_table_paths: &std::collections::BTreeSet<String>,
    conflicts: &mut Vec<String>,
) {
    for (k, v) in retained {
        let Some(t) = v.as_table() else { continue };
        let path = path_of(k, prefix);
        let direct_leaves: Vec<String> = t
            .iter()
            .filter(|(_, lv)| !lv.is_table())
            .map(|(lk, _)| path_of(lk, &path))
            .collect();
        if !direct_leaves.is_empty() && user_table_paths.contains(&path) {
            conflicts.push(format!(
                "{}: 模板新键 {} 需并入用户已有的同名表（TOML 禁止重复表头），请手工添加后重跑 update",
                path,
                direct_leaves.join(", ")
            ));
            // 该表头已冲突，其子表不再单独下钻：手工并表时一并处理
            continue;
        }
        push_header_conflicts(t, &path, user_table_paths, conflicts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: &str = "# PENGJ_TEMPLATE_START";
    const END: &str = "# PENGJ_TEMPLATE_END";

    fn wrap(body: &str) -> String {
        format!("{START}\n{body}{END}\n")
    }

    #[test]
    fn dedupes_equal_table_and_keeps_user_extras() {
        // 复现 ChahuRenderDebugger 场景：用户已有 windows target 表（linker 同值 +
        // 额外 rustflags），模板再写同名表 → 不得产生重复表定义
        let disk = "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\nrustflags = [\"-C\", \"target-cpu=x86-64-v3\"]\n";
        let incoming = wrap(
            "# 注释行\n[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\n\n[target.x86_64-unknown-linux-gnu]\nlinker = \"lld\"\n",
        );

        let merged = match merge_toml_managed(disk, &incoming) {
            TomlMergeOutcome::Merged(t) => t,
            TomlMergeOutcome::Conflict(m) => panic!("不应冲突: {m}"),
        };

        // 解析必须成功（无重复表定义）
        let parsed: Value = toml::from_str(&merged).expect("合并结果必须是合法 TOML");
        // 用户区 rustflags 原样保留（数组 2 个元素）
        assert_eq!(
            parsed["target"]["x86_64-pc-windows-msvc"]["rustflags"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        // 模板新增的 linux 表进入受管块
        assert!(merged.contains("[target.x86_64-unknown-linux-gnu]"));
        // windows 表只剩用户区那一份（受管块内不再重复 linker 行）
        assert_eq!(merged.matches("[target.x86_64-pc-windows-msvc]").count(), 1);
        assert_eq!(merged.matches("linker = \"rust-lld\"").count(), 1);
        assert_eq!(merged.matches("PENGJ_TEMPLATE_START").count(), 1);
    }

    #[test]
    fn reports_conflict_on_differing_values() {
        let disk = "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld.exe\"\n";
        let incoming = wrap("[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\n");

        match merge_toml_managed(disk, &incoming) {
            TomlMergeOutcome::Conflict(msg) => {
                assert!(msg.contains("target.x86_64-pc-windows-msvc.linker"));
            }
            TomlMergeOutcome::Merged(_) => panic!("同键不同值必须报冲突"),
        }
    }

    #[test]
    fn reports_conflict_when_template_needs_keys_merged_into_user_table() {
        // 用户已有 [profile.dev]，模板还要往里加 incremental：
        // TOML 禁止重复表头 → 必须报冲突而不是产出非法文件
        let disk = "[profile.dev]\nopt-level = 0\n";
        let incoming = wrap("[profile.dev]\nopt-level = 0\nincremental = true\n");

        match merge_toml_managed(disk, &incoming) {
            TomlMergeOutcome::Conflict(msg) => {
                assert!(msg.contains("profile.dev"));
                assert!(msg.contains("incremental"));
            }
            TomlMergeOutcome::Merged(_) => panic!("部分重叠必须报冲突"),
        }
    }

    #[test]
    fn sibling_subtables_under_existing_parent_are_not_conflicts() {
        // 用户已有 [target.x86_64-pc-windows-msvc]，模板新增
        // [target.x86_64-unknown-linux-gnu]：不同表头，可以共存，不算冲突
        let disk = "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\n";
        let incoming = wrap("[target.x86_64-unknown-linux-gnu]\nlinker = \"lld\"\n");

        let merged = match merge_toml_managed(disk, &incoming) {
            TomlMergeOutcome::Merged(t) => t,
            TomlMergeOutcome::Conflict(m) => panic!("兄弟子表不应冲突: {m}"),
        };
        assert_eq!(merged.matches("[target.").count(), 2);
        assert!(merged.contains(START));
    }

    #[test]
    fn replaces_existing_managed_block_and_is_idempotent() {
        // 第二次 update：磁盘旧受管块被剥离后，其表不再属于用户区，
        // 模板内容会以新受管块的形式重新落位（末尾），且结果幂等
        let disk = format!(
            "[alias]\nfoo = \"bar\"\n\n{}\n[target.x86_64-unknown-linux-gnu]\nlinker = \"lld\"\n{}\n",
            START, END
        );
        let incoming = wrap("[target.x86_64-unknown-linux-gnu]\nlinker = \"lld\"\n");

        let first = match merge_toml_managed(&disk, &incoming) {
            TomlMergeOutcome::Merged(t) => t,
            TomlMergeOutcome::Conflict(m) => panic!("不应冲突: {m}"),
        };
        // 用户区 [alias] 保留在文件前部，linux 表恰好一份
        assert!(first.starts_with("[alias]"));
        assert_eq!(
            first.matches("[target.x86_64-unknown-linux-gnu]").count(),
            1
        );
        assert!(first.contains(START));

        // 幂等：对合并结果再跑一次不再变化
        let second = match merge_toml_managed(&first, &incoming) {
            TomlMergeOutcome::Merged(t) => t,
            TomlMergeOutcome::Conflict(m) => panic!("不应冲突: {m}"),
        };
        assert_eq!(first, second);
    }

    #[test]
    fn fresh_append_equal_content_drops_block() {
        // legacy 文件与模板完全等价 → 受管块省去，用户内容一字不动
        let disk = "[build]\nrustc-wrapper = \"sccache\"\n";
        let incoming = wrap("[build]\nrustc-wrapper = \"sccache\"\n");

        match merge_toml_managed(disk, &incoming) {
            TomlMergeOutcome::Merged(t) => {
                assert_eq!(t, disk);
            }
            TomlMergeOutcome::Conflict(m) => panic!("不应冲突: {m}"),
        }
    }

    #[test]
    fn fresh_append_new_tables_into_legacy_file_keeps_user_content() {
        // 用户有自己的表，模板带来全新表 → 新表进受管块追加，用户区不动
        let disk = "[alias]\nfoo = \"bar\"\n";
        let incoming = wrap("[target.x86_64-unknown-linux-gnu]\nlinker = \"lld\"\n");

        let merged = match merge_toml_managed(disk, &incoming) {
            TomlMergeOutcome::Merged(t) => t,
            TomlMergeOutcome::Conflict(m) => panic!("不应冲突: {m}"),
        };
        let parsed: Value = toml::from_str(&merged).expect("合法 TOML");
        assert_eq!(parsed["alias"]["foo"].as_str(), Some("bar"));
        assert_eq!(
            parsed["target"]["x86_64-unknown-linux-gnu"]["linker"].as_str(),
            Some("lld")
        );
        assert!(merged.starts_with("[alias]"), "用户区应保持在文件前部");
        assert!(merged.contains(START));
    }

    #[test]
    fn incoming_without_managed_block_passes_through() {
        let disk = "a = 1\n";
        let incoming = "b = 2\n";
        match merge_toml_managed(disk, incoming) {
            TomlMergeOutcome::Merged(t) => assert_eq!(t, incoming),
            TomlMergeOutcome::Conflict(_) => panic!("不应冲突"),
        }
    }

    #[test]
    fn is_toml_managed_matches_normalized_path() {
        use std::path::Path;
        assert!(is_toml_managed(Path::new(".cargo/config.toml")));
        assert!(is_toml_managed(Path::new(".cargo\\config.toml")));
        assert!(!is_toml_managed(Path::new("Cargo.toml")));
        assert!(!is_toml_managed(Path::new("AGENTS.md")));
    }
}
