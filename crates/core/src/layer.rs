use serde::Deserialize;

/// layer.toml 的元数据结构
#[derive(Debug, Clone, Deserialize)]
pub struct LayerMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub depends: Vec<String>,
    /// 更新黑名单：这些文件仅首次生成时写入，之后归用户所有，
    /// `update` 时完全跳过（不覆盖、不报冲突、不删除上报）。
    /// 相对本层根目录，如 `src/main.rs`。
    #[serde(default)]
    pub update_ignore: Vec<String>,
}

/// 暴露给上层（CLI / GUI）的层信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct LayerInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub depends: Vec<String>,
    pub file_count: usize,
}

/// 暴露给上层（CLI / GUI）的技能信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
}

/// 层元数据文件名，写入时跳过
pub const LAYER_META_FILE: &str = "layer.toml";
