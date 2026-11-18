use serde::Deserialize;

/// layer.toml 的元数据结构
#[derive(Debug, Clone, Deserialize)]
pub struct LayerMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub depends: Vec<String>,
    /// 本层配套的技能名：勾选本层时，这些技能必须同时进入选中集合。
    ///
    /// 「层负责产出机制、技能负责讲清用法」，两者缺一即为「半接入」。
    /// 联动由层自己声明，引擎不再为某一层写死专用函数。
    #[serde(default)]
    pub skills: Vec<String>,
    /// 更新黑名单：这些文件仅首次生成时写入，之后归用户所有，
    /// `update` 时完全跳过（不覆盖、不报冲突、不删除上报）。
    /// 相对本层根目录，如 `src/main.rs`。
    #[serde(default)]
    pub update_ignore: Vec<String>,
}

/// 暴露给上层（CLI / MCP）的层信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct LayerInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub depends: Vec<String>,
    /// 本层配套的技能（勾选本层会自动带上）
    pub skills: Vec<String>,
    pub file_count: usize,
}

/// 暴露给上层（CLI / MCP）的技能信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
}

/// 层元数据文件名，写入时跳过
pub const LAYER_META_FILE: &str = "layer.toml";
