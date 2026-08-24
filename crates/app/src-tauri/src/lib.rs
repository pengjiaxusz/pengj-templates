use std::collections::HashMap;
use std::path::Path;

use pengj_core::{generate, update_project, Templates};
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            cmd_list_layers,
            cmd_list_skills,
            cmd_create_project,
            cmd_update_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
