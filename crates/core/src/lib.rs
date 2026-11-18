//! pengj-templates 引擎
//!
//! 分层模板：模板按「层」组织（templates/<layer>/），生成时按依赖顺序合并渲染，
//! 并在目标项目写入 `.pengj-templates/manifest.json` manifest，供后续 `update` 同步模板变更。

pub mod block;
pub mod context;
pub mod engine;
pub mod error;
pub mod i18n_audit;
pub mod layer;
pub mod manifest;
pub mod migrations;
pub mod render;
pub mod repo;
pub mod subrepo;
pub mod subrepo_check;
pub(crate) mod toml_merge;
pub mod worktree;
pub mod worktree_create;

pub use engine::{
    add_layers_to_project, adopt_project, audit_project, default_templates_dir, generate,
    list_workspace_files, platform_data_dir, sync_workspace_file, update_project,
    update_project_with_options, Templates,
};
pub use engine::{
    AddLayersReport, AdoptReport, AuditFileStatus, AuditItem, AuditReport, ConflictInfo,
    GenerateReport, UpdateOptions, UpdateReport,
};
pub use error::{CoreError, Result};
pub use i18n_audit::{
    audit_generated, audit_project_lang, contains_han, format_findings, Finding, FindingKind, Lang,
    LangAudit,
};
pub use layer::{LayerInfo, SkillInfo};
pub use manifest::{ProjectManifest, MANIFEST_FILE};
pub use migrations::{notes_between, MigrationNote, MIGRATIONS};
pub use repo::{default_links, repo_init, repo_status, LinkSpec, RepoItem, RepoReport, RepoStatus};
pub use subrepo::{DependencyMode, SubrepoEntry, SubrepoRegistry, SUBREPO_REGISTRY_REL};
pub use subrepo_check::{check_project, CheckItem, CheckLevel, CheckReport};
pub use worktree::{
    check_shared_target, conflicting_override, ensure_shared_target, project_key,
    shared_target_dir, SharedTarget, SharedTargetState, SHARED_TARGET_DIR_NAME,
};
pub use worktree_create::{create_worktree, worktree_dir_name, WorktreeReport};
