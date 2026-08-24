use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 生成项目时写入根目录的 manifest，`update` 依赖它判断哪些文件是模板托管的
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub tool: String,
    pub version: String,
    pub project_name: String,
    /// 实际启用的层（按合并顺序）
    pub layers: Vec<String>,
    /// 生成时用户选定的选项（edition、use_sccache 等），update 时据此重渲染
    #[serde(default)]
    pub options: BTreeMap<String, serde_json::Value>,
    pub generated_at: String,
    /// 相对路径 -> 生成内容的 sha256（十六进制）
    pub files: BTreeMap<String, String>,
}

pub const MANIFEST_FILE: &str = ".pengj-templates.json";

impl ProjectManifest {
    pub fn load(dir: &std::path::Path) -> Result<Self, crate::error::CoreError> {
        let path = dir.join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&path)
            .map_err(|_| crate::error::CoreError::MissingManifest(dir.display().to_string()))?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self, dir: &std::path::Path) -> Result<(), crate::error::CoreError> {
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join(MANIFEST_FILE), text)?;
        Ok(())
    }
}
