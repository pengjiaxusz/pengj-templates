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

/// 生成项目时写入的统一工具目录，模板产物都收在这里，根下只留必须待在根上的文件
pub const MANIFEST_DIR: &str = ".pengj-templates";

/// 引擎写入的清单路径（相对项目根）：`update` 依赖它判断哪些文件是模板托管的
pub const MANIFEST_FILE: &str = ".pengj-templates/manifest.json";

impl ProjectManifest {
    pub fn load(dir: &std::path::Path) -> Result<Self, crate::error::CoreError> {
        let path = dir.join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&path)
            .map_err(|_| crate::error::CoreError::MissingManifest(dir.display().to_string()))?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self, dir: &std::path::Path) -> Result<(), crate::error::CoreError> {
        let path = dir.join(MANIFEST_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, text)?;
        Ok(())
    }
}
