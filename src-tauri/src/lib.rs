mod commands;
pub mod community;
pub mod db;
pub mod models;
pub mod skills;

use commands::AppData;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = db::init_db().expect("Failed to initialize database (unrecoverable)");

    // Initial sync of skills from disk into the database
    if let Err(e) = skills::sync_skills(&conn) {
        eprintln!("Warning: initial skill sync failed: {}", e);
    }

    let app_data = AppData {
        db: Mutex::new(conn),
    };

    tauri::Builder::default()
        .manage(app_data)
        .invoke_handler(tauri::generate_handler![
            commands::cmd_get_all_skills,
            commands::cmd_get_skill_details,
            commands::cmd_enable_skill,
            commands::cmd_disable_skill,
            commands::cmd_delete_skill,
            commands::cmd_sync_skills,
            commands::cmd_get_projects,
            commands::cmd_add_project,
            commands::cmd_remove_project,
            commands::cmd_get_bundled_skills,
            commands::cmd_install_bundled_skill,
            commands::cmd_add_tag,
            commands::cmd_remove_tag,
            commands::cmd_get_app_state,
            commands::cmd_search_skills,
            commands::cmd_fetch_single_repo,
            commands::cmd_fetch_skill_content,
            commands::cmd_get_community_repos,
            commands::cmd_sync_community,
            commands::cmd_install_community_skill,
            commands::cmd_add_custom_repo,
            commands::cmd_get_custom_repos,
            commands::cmd_remove_custom_repo,
            commands::cmd_get_github_token,
            commands::cmd_set_github_token,
            commands::cmd_delete_github_token,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Skiller");
}
