use std::collections::HashMap;
use std::path::Path;

use pengj_core::{adopt_project, generate, list_workspace_files, update_project, Templates};
use tauri::Manager;

/// 定位模板目录：环境 PENGJ_TEMPLATES > app 资源目录 templates/ > 默认位置
fn resolve_templates(app: &tauri::AppHandle) -> Result<Templates, String> {
    if let Ok(p) = app.path().resource_dir() {
        let candidate = p.join("templates");
        if candidate.is_dir() {
            return Ok(Templates::new(candidate));
        }
    }
    let dir = pengj_core::default_templates_dir().map_err(|e| e.to_string())?;
    Ok(Templates::new(dir))
}

/// 列出所有可用层
#[tauri::command]
fn cmd_list_layers(app: tauri::AppHandle) -> Result<Vec<pengj_core::LayerInfo>, String> {
    let templates = resolve_templates(&app)?;
    templates.list_layers().map_err(|e| e.to_string())
}

/// 列出所有可用技能（供 UI 勾选）
#[tauri::command]
fn cmd_list_skills(app: tauri::AppHandle) -> Result<Vec<pengj_core::SkillInfo>, String> {
    let templates = resolve_templates(&app)?;
    templates.list_skills().map_err(|e| e.to_string())
}

/// 在 parent_dir 下生成 <name>/ 项目
#[tauri::command]
fn cmd_create_project(
    app: tauri::AppHandle,
    name: String,
    layers: Vec<String>,
    parent_dir: String,
    options: HashMap<String, serde_json::Value>,
) -> Result<pengj_core::GenerateReport, String> {
    let templates = resolve_templates(&app)?;
    generate(
        &templates,
        &name,
        &layers,
        options.into_iter().collect(),
        Path::new(&parent_dir),
    )
    .map_err(|e| e.to_string())
}

/// 同步模板更新到已生成的项目
#[tauri::command]
fn cmd_update_project(
    app: tauri::AppHandle,
    project_dir: String,
) -> Result<pengj_core::UpdateReport, String> {
    let templates = resolve_templates(&app)?;
    update_project(&templates, Path::new(&project_dir)).map_err(|e| e.to_string())
}

/// 纳管已有存量项目
#[tauri::command]
fn cmd_adopt_project(
    app: tauri::AppHandle,
    project_dir: String,
    layers: Vec<String>,
    options: HashMap<String, serde_json::Value>,
    force: Option<bool>,
) -> Result<pengj_core::AdoptReport, String> {
    let templates = resolve_templates(&app)?;
    adopt_project(
        &templates,
        Path::new(&project_dir),
        &layers,
        options.into_iter().collect(),
        force.unwrap_or(false),
    )
    .map_err(|e| e.to_string())
}

/// 列出项目根目录下的工作区文件（`*.code-workspace`，非递归，按文件名排序）。
///
/// 供前端在更新前展示可选的 workspace。core 的 `update_project` 会自动同步全部
/// workspace（fileNesting 等），此处选择仅作展示/提示。目录不存在或无匹配文件时
/// 返回空数组而非报错。
#[tauri::command]
fn cmd_list_workspaces(project_dir: String) -> Result<Vec<String>, String> {
    Ok(list_workspace_files(Path::new(&project_dir))
        .into_iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            cmd_list_layers,
            cmd_list_skills,
            cmd_create_project,
            cmd_update_project,
            cmd_adopt_project,
            cmd_list_workspaces
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
