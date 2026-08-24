//! pengj-templates 引擎
//!
//! 分层模板：模板按「层」组织（templates/<layer>/），生成时按依赖顺序合并渲染，
//! 并在目标项目写入 `.pengj.json` manifest，供后续 `update` 同步模板变更。

pub mod context;
pub mod engine;
pub mod error;
pub mod layer;
pub mod manifest;
pub mod render;

pub use engine::{default_templates_dir, generate, update_project, Templates};
pub use engine::{ConflictInfo, GenerateReport, UpdateReport};
pub use error::{CoreError, Result};
pub use layer::{LayerInfo, SkillInfo};
pub use manifest::{ProjectManifest, MANIFEST_FILE};
