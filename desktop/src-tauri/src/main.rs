#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use serde::Serialize;
use skills_hub_rs::models::{
    CustomTool, DetectedSkill, InstallOptions, OnlineSkillResult, Scope, SkillRecord, SyncMode,
    TargetRecord,
};
use skills_hub_rs::{online, SkillsHub};
use tauri_plugin_dialog::DialogExt;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn hub() -> Result<SkillsHub, String> {
    SkillsHub::open(None).map_err(err)
}

async fn blocking<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|e| format!("blocking task failed: {e}"))?
}

#[derive(Debug, Serialize)]
struct SkillView {
    skill: SkillRecord,
    tags: Vec<String>,
    targets: Vec<TargetRecord>,
}

#[derive(Debug, Serialize)]
struct ToolView {
    key: String,
    label: String,
    global_dir: String,
    project_dir: Option<String>,
    detected: bool,
    mode: String,
    force_copy: bool,
    builtin: bool,
}

#[derive(Debug, Serialize)]
struct Dashboard {
    root: String,
    db: String,
    skills: Vec<SkillView>,
    tools: Vec<ToolView>,
}

#[derive(Debug, Serialize)]
struct RecycleView {
    id: String,
    name: String,
    deleted_at: i64,
    tags: Vec<String>,
    target_count: usize,
}

fn skill_view(hub: &SkillsHub, skill: SkillRecord) -> Result<SkillView, String> {
    let tags = hub.store.tags_for_skill(&skill.id).map_err(err)?;
    let targets = hub.store.list_targets(&skill.id).map_err(err)?;
    Ok(SkillView {
        skill,
        tags,
        targets,
    })
}

#[tauri::command]
fn dashboard() -> Result<Dashboard, String> {
    let hub = hub()?;
    let skills = hub
        .store
        .list_skills()
        .map_err(err)?
        .into_iter()
        .map(|skill| skill_view(&hub, skill))
        .collect::<Result<Vec<_>, _>>()?;

    let tools = hub
        .list_tools()
        .map_err(err)?
        .into_iter()
        .map(|tool| {
            let detected = tool
                .detect_dir
                .as_ref()
                .map(|p| p.exists())
                .unwrap_or_else(|| tool.global_dir.exists());
            ToolView {
                key: tool.key,
                label: tool.label,
                global_dir: tool.global_dir.to_string_lossy().to_string(),
                project_dir: tool.project_dir.map(|p| p.to_string_lossy().to_string()),
                detected,
                mode: tool.sync_mode.to_string(),
                force_copy: tool.force_copy,
                builtin: tool.builtin,
            }
        })
        .collect();

    Ok(Dashboard {
        root: hub.root.to_string_lossy().to_string(),
        db: hub.store.db_path().to_string_lossy().to_string(),
        skills,
        tools,
    })
}

#[tauri::command]
async fn install_local(path: String, tags: Vec<String>) -> Result<Vec<SkillRecord>, String> {
    blocking(move || {
        let hub = hub()?;
        let options = InstallOptions {
            tags,
            ..Default::default()
        };
        hub.install_local(Path::new(&path), &options).map_err(err)
    })
    .await
}

#[tauri::command]
async fn install_git(
    url: String,
    revision: Option<String>,
    subdir: Option<String>,
    tags: Vec<String>,
) -> Result<Vec<SkillRecord>, String> {
    blocking(move || {
        let hub = hub()?;
        let options = InstallOptions {
            tags,
            ..Default::default()
        };
        let subdir = subdir.filter(|s| !s.trim().is_empty()).map(PathBuf::from);
        hub.install_git(&url, revision.as_deref(), subdir.as_deref(), &options)
            .map_err(err)
    })
    .await
}

#[tauri::command]
async fn sync_skill(
    skill: String,
    tool: String,
    scope: String,
    project: Option<String>,
    mode: String,
    overwrite: bool,
) -> Result<TargetRecord, String> {
    blocking(move || {
        let hub = hub()?;
        let scope: Scope = scope.parse().map_err(err)?;
        let mode: SyncMode = mode.parse().map_err(err)?;
        let project = project.filter(|s| !s.trim().is_empty()).map(PathBuf::from);
        hub.sync_skill(&skill, &tool, scope, project.as_deref(), mode, overwrite)
            .map_err(err)
    })
    .await
}

#[tauri::command]
fn unsync_skill(
    skill: String,
    tool: String,
    scope: String,
    project: Option<String>,
) -> Result<(), String> {
    let hub = hub()?;
    let scope: Scope = scope.parse().map_err(err)?;
    let project = project.filter(|s| !s.trim().is_empty()).map(PathBuf::from);
    hub.unsync_skill(&skill, &tool, scope, project.as_deref())
        .map_err(err)
}

#[tauri::command]
fn set_enabled(skill: String, enabled: bool) -> Result<(), String> {
    let hub = hub()?;
    if enabled {
        hub.enable_skill(&skill)
    } else {
        hub.disable_skill(&skill)
    }
    .map_err(err)
}

#[tauri::command]
async fn update_skill(skill: String) -> Result<String, String> {
    blocking(move || hub()?.update_skill(&skill).map_err(err)).await
}

#[tauri::command]
async fn update_all() -> Result<Vec<(String, String)>, String> {
    blocking(move || hub()?.update_all().map_err(err)).await
}

#[tauri::command]
fn remove_skill(skill: String, permanent: bool) -> Result<Option<String>, String> {
    hub()?.remove_skill(&skill, permanent).map_err(err)
}

#[tauri::command]
fn add_tag(skill: String, tag: String) -> Result<(), String> {
    let hub = hub()?;
    let record = hub
        .store
        .get_skill(&skill)
        .map_err(err)?
        .ok_or_else(|| format!("skill {skill:?} not found"))?;
    hub.store.add_tag(&record.id, &tag).map_err(err)
}

#[tauri::command]
fn remove_tag(skill: String, tag: String) -> Result<(), String> {
    let hub = hub()?;
    let record = hub
        .store
        .get_skill(&skill)
        .map_err(err)?
        .ok_or_else(|| format!("skill {skill:?} not found"))?;
    hub.store.remove_tag(&record.id, &tag).map_err(err)
}

#[tauri::command]
fn recycle_list() -> Result<Vec<RecycleView>, String> {
    let hub = hub()?;
    hub.list_recycle().map_err(err).map(|entries| {
        entries
            .into_iter()
            .map(|(id, snapshot)| RecycleView {
                id,
                name: snapshot.skill.name,
                deleted_at: snapshot.deleted_at,
                tags: snapshot.tags,
                target_count: snapshot.targets.len(),
            })
            .collect()
    })
}

#[tauri::command]
fn recycle_restore(id: String) -> Result<SkillRecord, String> {
    hub()?.restore_recycle(&id).map_err(err)
}

#[tauri::command]
fn recycle_purge(all: bool) -> Result<usize, String> {
    hub()?.purge_recycle(all).map_err(err)
}

#[tauri::command]
async fn search_online(query: String, limit: usize) -> Result<Vec<OnlineSkillResult>, String> {
    blocking(move || online::search(&query, limit).map_err(err)).await
}

#[tauri::command]
async fn scan_installed(tool: Option<String>) -> Result<Vec<DetectedSkill>, String> {
    blocking(move || hub()?.scan_tool(tool.as_deref()).map_err(err)).await
}

#[tauri::command]
fn save_custom_tool(
    key: String,
    label: String,
    global_dir: String,
    project_dir: Option<String>,
    detect_dir: Option<String>,
    mode: String,
) -> Result<(), String> {
    let hub = hub()?;
    let sync_mode: SyncMode = mode.parse().map_err(err)?;
    hub.add_custom_tool(CustomTool {
        key,
        label,
        global_dir,
        project_dir: project_dir.filter(|v| !v.trim().is_empty()),
        detect_dir: detect_dir.filter(|v| !v.trim().is_empty()),
        sync_mode,
        enabled: true,
    })
    .map_err(err)
}

#[tauri::command]
fn remove_custom_tool(key: String) -> Result<(), String> {
    hub()?.remove_custom_tool(&key).map_err(err)
}

#[tauri::command]
async fn pick_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let selected = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|path| path.into_path().ok())
        .map(|path| path.to_string_lossy().into_owned());

    Ok(selected)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            dashboard,
            install_local,
            install_git,
            sync_skill,
            unsync_skill,
            set_enabled,
            update_skill,
            update_all,
            remove_skill,
            add_tag,
            remove_tag,
            recycle_list,
            recycle_restore,
            recycle_purge,
            search_online,
            scan_installed,
            save_custom_tool,
            remove_custom_tool,
            pick_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Skills Hub Rust desktop");
}
