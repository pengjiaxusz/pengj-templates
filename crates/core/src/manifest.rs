use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 统一将 manifest 相对路径键名中的反斜杠归一化为 POSIX 正斜杠，
/// 避免 Windows 生成的清单在 Linux/macOS 下被误判为未托管文件。
pub(crate) fn normalize_rel_path(path: &str) -> String {
    path.replace('\\', "/")
}

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
    /// 相对路径 -> 生成内容的 sha256（十六进制，统一以 `/` 分隔）
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
        let mut manifest: Self = serde_json::from_str(&text)?;
        manifest.normalize_file_keys();
        Ok(manifest)
    }

    pub fn save(&self, dir: &std::path::Path) -> Result<(), crate::error::CoreError> {
        let path = dir.join(MANIFEST_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut normalized = self.clone();
        normalized.normalize_file_keys();
        let text = serde_json::to_string_pretty(&normalized)?;
        std::fs::write(&path, text)?;
        Ok(())
    }

    /// 将 `files` 中的路径键名统一归一化为 `/` 分隔
    pub fn normalize_file_keys(&mut self) {
        let normalized = std::mem::take(&mut self.files)
            .into_iter()
            .map(|(k, v)| (normalize_rel_path(&k), v))
            .collect();
        self.files = normalized;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_load_and_save_normalize_windows_backslashes() {
        let tmp = std::env::temp_dir().join(format!("pengj-manifest-norm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(MANIFEST_DIR)).unwrap();

        let raw = r#"{
  "tool": "pengj-templates",
  "version": "0.80.1",
  "project_name": "demo",
  "layers": ["common"],
  "options": {},
  "generated_at": "2026-01-01T00:00:00Z",
  "files": {
    ".pengj-templates\\scripts\\doctor.sh": "abc123",
    ".agents\\skills\\demo\\SKILL.md": "def456"
  }
}"#;
        std::fs::write(tmp.join(MANIFEST_FILE), raw).unwrap();

        let loaded = ProjectManifest::load(&tmp).unwrap();
        assert_eq!(
            loaded
                .files
                .get(".pengj-templates/scripts/doctor.sh")
                .map(String::as_str),
            Some("abc123")
        );
        assert_eq!(
            loaded
                .files
                .get(".agents/skills/demo/SKILL.md")
                .map(String::as_str),
            Some("def456")
        );

        loaded.save(&tmp).unwrap();
        let saved_text = std::fs::read_to_string(tmp.join(MANIFEST_FILE)).unwrap();
        assert!(!saved_text.contains("\\\\"));
        assert!(saved_text.contains(".pengj-templates/scripts/doctor.sh"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
