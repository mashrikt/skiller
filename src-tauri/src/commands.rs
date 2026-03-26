use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

use crate::community;
use crate::db;
use crate::models::*;
use crate::skills;

pub struct AppData {
    pub db: Mutex<Connection>,
}

#[tauri::command]
pub fn cmd_get_all_skills(state: tauri::State<'_, AppData>) -> Result<Vec<Skill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_skills(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_get_skill_details(
    state: tauri::State<'_, AppData>,
    id: String,
) -> Result<Option<Skill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_skill_by_id(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_enable_skill(state: tauri::State<'_, AppData>, id: String) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let skill = db::get_skill_by_id(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Skill not found: {}", id))?;

    if skill.enabled {
        return Ok(skill);
    }

    let new_path = skills::enable_skill(&skill).map_err(|e| e.to_string())?;

    let mut updated = skill.clone();
    updated.enabled = true;
    updated.path = new_path;
    updated.updated_at = Utc::now().to_rfc3339();

    db::upsert_skill(&conn, &updated).map_err(|e| e.to_string())?;

    // Re-fetch to get fresh content
    db::get_skill_by_id(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Skill disappeared after enable".to_string())
}

#[tauri::command]
pub fn cmd_disable_skill(state: tauri::State<'_, AppData>, id: String) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let skill = db::get_skill_by_id(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Skill not found: {}", id))?;

    if !skill.enabled {
        return Ok(skill);
    }

    let new_path = skills::disable_skill(&skill).map_err(|e| e.to_string())?;

    let mut updated = skill.clone();
    updated.enabled = false;
    updated.path = new_path;
    updated.updated_at = Utc::now().to_rfc3339();

    db::upsert_skill(&conn, &updated).map_err(|e| e.to_string())?;

    db::get_skill_by_id(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Skill disappeared after disable".to_string())
}

#[tauri::command]
pub fn cmd_delete_skill(state: tauri::State<'_, AppData>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let skill = db::get_skill_by_id(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Skill not found: {}", id))?;

    skills::delete_skill_files(&skill.path).map_err(|e| e.to_string())?;
    db::delete_skill(&conn, &id).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_sync_skills(state: tauri::State<'_, AppData>) -> Result<Vec<Skill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills::sync_skills(&conn).map_err(|e| e.to_string())?;
    db::get_all_skills(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_get_projects(state: tauri::State<'_, AppData>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_projects(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_add_project(state: tauri::State<'_, AppData>, path: String) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let canonical_path = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|e| format!("Path does not exist or cannot be resolved: {} ({})", path, e))?;

    if !canonical_path.is_dir() {
        return Err(format!("Path is not a directory: {}", canonical_path.display()));
    }

    let canonical_path = canonical_path.to_string_lossy().to_string();

    // Check if already exists
    if let Ok(Some(existing)) = db::get_project_by_path(&conn, &canonical_path) {
        return Ok(existing);
    }

    let name = std::path::Path::new(&canonical_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        path: canonical_path.clone(),
        skill_count: 0,
        added_at: Utc::now().to_rfc3339(),
    };

    db::upsert_project(&conn, &project).map_err(|e| e.to_string())?;

    // Sync to pick up any skills in this project
    skills::sync_skills(&conn).map_err(|e| format!("Skill sync failed: {}", e))?;

    // Re-fetch to get updated skill count
    db::get_project_by_path(&conn, &canonical_path)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project disappeared after add".to_string())
}

#[tauri::command]
pub fn cmd_remove_project(state: tauri::State<'_, AppData>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_project(&conn, &id).map_err(|e| e.to_string())?;
    skills::sync_skills(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_get_bundled_skills(
    state: tauri::State<'_, AppData>,
) -> Result<Vec<BundledSkill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(skills::get_bundled_skills(&conn))
}

#[tauri::command]
pub fn cmd_install_bundled_skill(
    state: tauri::State<'_, AppData>,
    id: String,
) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let bundled_skills = skills::get_bundled_skills(&conn);
    let bundled = bundled_skills
        .iter()
        .find(|b| b.id == id)
        .ok_or_else(|| format!("Bundled skill not found: {}", id))?;

    if bundled.installed {
        return Err(format!("Skill '{}' is already installed", bundled.name));
    }

    let skill_path = skills::install_bundled_skill(bundled).map_err(|e| e.to_string())?;

    // Sync to pick up the new skill
    skills::sync_skills(&conn).map_err(|e| e.to_string())?;

    // Find the newly installed skill in the DB by path
    let all = db::get_all_skills(&conn).map_err(|e| e.to_string())?;
    let installed = all
        .into_iter()
        .find(|s| s.path == skill_path)
        .ok_or_else(|| "Installed skill not found in database after sync".to_string())?;

    // Update with bundled metadata
    let mut updated = installed;
    updated.category = bundled.category.clone();
    updated.author = bundled.author.clone();
    updated.source_url = bundled.source.clone();
    updated.tags = bundled.tags.clone();
    updated.updated_at = Utc::now().to_rfc3339();

    db::upsert_skill(&conn, &updated).map_err(|e| e.to_string())?;

    db::get_skill_by_id(&conn, &updated.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Skill disappeared after metadata update".to_string())
}

#[tauri::command]
pub fn cmd_add_tag(
    state: tauri::State<'_, AppData>,
    skill_id: String,
    tag: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::add_tag(&conn, &skill_id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_remove_tag(
    state: tauri::State<'_, AppData>,
    skill_id: String,
    tag: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::remove_tag(&conn, &skill_id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_get_app_state(state: tauri::State<'_, AppData>) -> Result<AppState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(skills::get_app_state(&conn))
}

#[tauri::command]
pub fn cmd_search_skills(
    state: tauri::State<'_, AppData>,
    query: String,
) -> Result<Vec<Skill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let all_skills = db::get_all_skills(&conn).map_err(|e| e.to_string())?;

    let query_lower = query.to_lowercase();
    let terms: Vec<&str> = query_lower.split_whitespace().collect();

    if terms.is_empty() {
        return Ok(all_skills);
    }

    let filtered = all_skills
        .into_iter()
        .filter(|skill| {
            let searchable = format!(
                "{} {} {} {} {} {}",
                skill.name,
                skill.description,
                skill.category,
                skill.author,
                skill.tags.join(" "),
                skill.content,
            )
            .to_lowercase();

            terms.iter().all(|term| searchable.contains(term))
        })
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub fn cmd_get_community_repos(
    state: tauri::State<'_, AppData>,
) -> Result<Vec<community::CommunityRepo>, String> {
    let mut repos = community::get_community_repos();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let custom = db::get_all_custom_repos(&conn).map_err(|e| e.to_string())?;
    for cr in &custom {
        repos.push(cr.to_community_repo()?);
    }
    Ok(repos)
}

#[tauri::command]
pub async fn cmd_sync_community(
    state: tauri::State<'_, AppData>,
) -> Result<community::CommunitySyncResult, String> {
    let custom = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::get_all_custom_repos(&conn).map_err(|e| e.to_string())?
    };

    let result = community::sync_community_repos(custom).await;
    Ok(result)
}

#[tauri::command]
pub async fn cmd_install_community_skill(
    state: tauri::State<'_, AppData>,
    skill: community::CommunitySkill,
) -> Result<String, String> {
    let path = community::install_community_skill(&skill).await?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills::sync_skills(&conn).map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
pub async fn cmd_fetch_single_repo(
    repo_url: String,
    repo_id: String,
    repo_name: String,
    author: String,
    skills_path: String,
) -> Result<Vec<community::CommunitySkill>, String> {
    community::validate_repo_contents_url(&repo_url)?;
    let repo = community::CommunityRepo {
        id: repo_id,
        name: repo_name,
        url: repo_url,
        description: String::new(),
        skills_path,
        author,
    };
    community::fetch_repo_skills(&repo).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_fetch_skill_content(url: String) -> Result<String, String> {
    community::validate_skill_content_url(&url)?;
    community::fetch_skill_md_content(&url).await
}

#[tauri::command]
pub fn cmd_add_custom_repo(
    state: tauri::State<'_, AppData>,
    owner: String,
    repo: String,
    skills_path: String,
) -> Result<community::CustomRepo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if !owner.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') || owner.is_empty() {
        return Err(format!("Invalid GitHub owner name: {}", owner));
    }
    if !repo.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') || repo.is_empty() {
        return Err(format!("Invalid GitHub repo name: {}", repo));
    }
    let normalized_skills_path = community::validate_skills_path(&skills_path)?;

    let id = format!("{}:{}", owner, repo);
    let name = format!("{}/{}", owner, repo);

    let custom = community::CustomRepo {
        id,
        name,
        owner,
        repo,
        skills_path: if normalized_skills_path.is_empty() {
            "skills".into()
        } else {
            normalized_skills_path
        },
        description: "User-added repository".into(),
        added_at: Utc::now().to_rfc3339(),
    };

    db::upsert_custom_repo(&conn, &custom).map_err(|e| e.to_string())?;
    Ok(custom)
}

#[tauri::command]
pub fn cmd_get_custom_repos(
    state: tauri::State<'_, AppData>,
) -> Result<Vec<community::CustomRepo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_custom_repos(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_remove_custom_repo(
    state: tauri::State<'_, AppData>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_custom_repo(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_get_github_token() -> Result<Option<String>, String> {
    Ok(community::load_github_token())
}

#[tauri::command]
pub fn cmd_set_github_token(token: String) -> Result<(), String> {
    community::save_github_token(&token)
}

#[tauri::command]
pub fn cmd_delete_github_token() -> Result<(), String> {
    community::delete_github_token()
}
