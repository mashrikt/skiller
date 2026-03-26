use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use walkdir::WalkDir;
use yaml_rust2::YamlLoader;

use crate::models::*;

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    YamlParse(String),
    #[error("Skill not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("{0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, SkillError>;

const VAULT_METADATA_FILE: &str = ".skiller-meta.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillStorageMetadata {
    original_path: String,
    scope: SkillScope,
}

fn stable_skill_id(path: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, path.as_bytes()).to_string()
}

fn metadata_path(skill_dir: &Path) -> PathBuf {
    skill_dir.join(VAULT_METADATA_FILE)
}

fn read_storage_metadata(skill_dir: &Path) -> Option<SkillStorageMetadata> {
    let content = fs::read_to_string(metadata_path(skill_dir)).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_storage_metadata(skill_dir: &Path, metadata: &SkillStorageMetadata) -> Result<()> {
    let content = serde_json::to_string(metadata)
        .map_err(|e| SkillError::General(format!("Failed to serialize skill metadata: {}", e)))?;
    fs::write(metadata_path(skill_dir), content)?;
    Ok(())
}

fn remove_storage_metadata(skill_dir: &Path) -> Result<()> {
    let path = metadata_path(skill_dir);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn vault_dir_name(skill: &Skill) -> String {
    let base_name = Path::new(&skill.original_path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|name| !name.is_empty())
        .or_else(|| {
            Path::new(&skill.path)
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|name| !name.is_empty())
        })
        .unwrap_or(&skill.name);
    let short_id = skill.id.get(..8).unwrap_or(&skill.id);
    format!("{}--{}", short_id, base_name)
}

/// Parse a SKILL.md file into frontmatter + body content + raw content
pub fn parse_skill_md(path: &Path) -> Result<(SkillFrontmatter, String, String)> {
    let content = fs::read_to_string(path)?;
    let (fm, body) = parse_skill_content(&content);
    Ok((fm, body, content))
}

/// Parse SKILL.md content string into frontmatter + body
fn parse_skill_content(content: &str) -> (SkillFrontmatter, String) {
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        return (SkillFrontmatter::default(), content.to_string());
    }

    // Find second "---" delimiter
    let after_first = &trimmed[3..];
    if let Some(end_idx) = after_first.find("\n---") {
        let yaml_str = after_first[..end_idx].trim();
        let body = after_first[end_idx + 4..].trim().to_string();

        let frontmatter = parse_yaml_frontmatter(yaml_str);
        (frontmatter, body)
    } else {
        (SkillFrontmatter::default(), content.to_string())
    }
}

fn parse_yaml_frontmatter(yaml_str: &str) -> SkillFrontmatter {
    let docs = match YamlLoader::load_from_str(yaml_str) {
        Ok(docs) => docs,
        Err(_) => return SkillFrontmatter::default(),
    };

    if docs.is_empty() {
        return SkillFrontmatter::default();
    }

    let doc = &docs[0];

    let get_str = |key: &str| -> Option<String> {
        doc[key].as_str().map(|s| s.to_string())
    };

    let get_bool = |key: &str| -> Option<bool> {
        doc[key].as_bool()
    };

    let get_str_vec = |key: &str| -> Option<Vec<String>> {
        doc[key].as_vec().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
    };

    SkillFrontmatter {
        name: get_str("name"),
        description: get_str("description"),
        disable_model_invocation: get_bool("disable_model_invocation"),
        user_invocable: get_bool("user_invocable"),
        allowed_tools: get_str_vec("allowed_tools"),
        model: get_str("model"),
        effort: get_str("effort"),
        context: get_str("context"),
        agent: get_str("agent"),
    }
}

/// Re-parse frontmatter from raw content (used by db.rs when loading from disk)
pub fn parse_frontmatter_from_content(content: &str) -> SkillFrontmatter {
    if content.is_empty() {
        return SkillFrontmatter::default();
    }
    let (fm, _) = parse_skill_content(content);
    fm
}

/// Discover all SKILL.md files within a directory tree and build Skill structs
pub fn discover_skills_in_dir(dir: &Path, scope: SkillScope) -> Vec<Skill> {
    let mut skills = Vec::new();

    if !dir.exists() {
        return skills;
    }

    for entry in WalkDir::new(dir).min_depth(1).max_depth(3) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_name() != "SKILL.md" {
            continue;
        }

        let skill_md_path = entry.path();
        let skill_dir = match skill_md_path.parent() {
            Some(p) => p,
            None => continue,
        };

        let (frontmatter, _body, content) = match parse_skill_md(skill_md_path) {
            Ok(result) => result,
            Err(_) => continue,
        };

        let dir_name = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let name = frontmatter
            .name
            .clone()
            .unwrap_or_else(|| dir_name.clone());

        let description = frontmatter.description.clone().unwrap_or_default();

        let now = Utc::now().to_rfc3339();
        let skill_dir_str = skill_dir.to_string_lossy().to_string();

        // Generate a deterministic ID from the path so re-scanning finds the same skill
        let id = stable_skill_id(&skill_dir_str);

        let vault_dir = dirs::home_dir()
            .map(|h| h.join(".skiller").join("vault"));
        let enabled = match vault_dir {
            Some(ref vd) => !skill_dir.starts_with(vd),
            None => true,
        };

        skills.push(Skill {
            id,
            name,
            description,
            path: skill_dir_str.clone(),
            original_path: skill_dir_str,
            scope: scope.clone(),
            enabled,
            tags: Vec::new(),
            category: String::new(),
            author: String::new(),
            source_url: String::new(),
            content,
            frontmatter,
            created_at: now.clone(),
            updated_at: now,
        });
    }

    skills
}

/// Discover global skills in ~/.claude/skills/
pub fn discover_global_skills() -> Vec<Skill> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let global_dir = home.join(".claude").join("skills");
    discover_skills_in_dir(&global_dir, SkillScope::Global)
}

/// Discover disabled skills in ~/.skiller/vault/
pub fn discover_vault_skills() -> Vec<Skill> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let vault_dir = home.join(".skiller").join("vault");
    let mut skills = discover_skills_in_dir(&vault_dir, SkillScope::Global);

    // All vault skills are disabled
    for skill in &mut skills {
        if let Some(metadata) = read_storage_metadata(Path::new(&skill.path)) {
            skill.id = stable_skill_id(&metadata.original_path);
            skill.original_path = metadata.original_path;
            skill.scope = metadata.scope;
        }
        skill.enabled = false;
    }

    skills
}

/// Discover project-level skills in <project_path>/.claude/skills/
pub fn discover_project_skills(project_path: &str) -> Vec<Skill> {
    let project = Path::new(project_path);
    let skills_dir = project.join(".claude").join("skills");
    discover_skills_in_dir(&skills_dir, SkillScope::Project(project_path.to_string()))
}

/// Enable a skill: move from vault back to its original location
pub fn enable_skill(skill: &Skill) -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| SkillError::General("No home directory".into()))?;

    let target_dir = if !skill.original_path.is_empty() {
        PathBuf::from(&skill.original_path)
    } else {
        let skill_dir_name = Path::new(&skill.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&skill.name);

        match &skill.scope {
            SkillScope::Project(project_path) => Path::new(project_path)
                .join(".claude")
                .join("skills")
                .join(skill_dir_name),
            _ => home.join(".claude").join("skills").join(skill_dir_name),
        }
    };

    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let source = Path::new(&skill.path);
    if source.exists() {
        move_dir(source, &target_dir)?;
        remove_storage_metadata(&target_dir)?;
    }

    Ok(target_dir.to_string_lossy().to_string())
}

/// Disable a skill: move to ~/.skiller/vault/
pub fn disable_skill(skill: &Skill) -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| SkillError::General("No home directory".into()))?;

    let vault_dir = home
        .join(".skiller")
        .join("vault")
        .join(vault_dir_name(skill));

    if let Some(parent) = vault_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let source = Path::new(&skill.path);
    if source.exists() {
        move_dir(source, &vault_dir)?;
        let metadata = SkillStorageMetadata {
            original_path: if skill.original_path.is_empty() {
                skill.path.clone()
            } else {
                skill.original_path.clone()
            },
            scope: skill.scope.clone(),
        };
        write_storage_metadata(&vault_dir, &metadata)?;
    }

    Ok(vault_dir.to_string_lossy().to_string())
}

/// Delete a skill's files from disk entirely
pub fn delete_skill_files(path: &str) -> Result<()> {
    let dir = Path::new(path);

    // Validate the path is within an allowed location
    let home = dirs::home_dir().ok_or_else(|| SkillError::General("No home directory".into()))?;
    let allowed_prefixes = [
        home.join(".claude").join("skills"),
        home.join(".skiller").join("vault"),
    ];

    let canonical = dir
        .canonicalize()
        .map_err(|e| SkillError::General(format!("Cannot resolve path: {}", e)))?;

    let is_allowed = allowed_prefixes.iter().any(|prefix| {
        prefix
            .canonicalize()
            .map(|cp| canonical.starts_with(&cp))
            .unwrap_or(false)
    });

    // Also allow project skill directories (<project>/.claude/skills/*)
    let in_project_skills = canonical
        .ancestors()
        .any(|ancestor| ancestor.file_name().and_then(|n| n.to_str()) == Some("skills"))
        && canonical
            .to_string_lossy()
            .contains(&format!("{}{}{}", std::path::MAIN_SEPARATOR, ".claude", std::path::MAIN_SEPARATOR));

    if !is_allowed && !in_project_skills {
        return Err(SkillError::General(format!(
            "Refusing to delete path outside of known skill directories: {}",
            path
        )));
    }

    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// Install a bundled skill by creating a placeholder SKILL.md
pub fn install_bundled_skill(bundled: &BundledSkill) -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| SkillError::General("No home directory".into()))?;

    let skill_dir = home.join(".claude").join("skills").join(&bundled.id);
    fs::create_dir_all(&skill_dir)?;

    let skill_md = skill_dir.join("SKILL.md");
    let escaped_name = bundled.name.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ");
    let escaped_desc = bundled.description.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ");
    let content = format!(
        r#"---
name: "{}"
description: "{}"
---

# {}

{}

> This skill was installed from the Skiller bundled skill catalog.
> Source: {}
> Author: {}
> Category: {}
> Tags: {}
"#,
        escaped_name,
        escaped_desc,
        bundled.name,
        bundled.description,
        bundled.source,
        bundled.author,
        bundled.category,
        bundled.tags.join(", "),
    );

    fs::write(&skill_md, content)?;

    Ok(skill_dir.to_string_lossy().to_string())
}

/// Get overall application state from the database
pub fn get_app_state(conn: &rusqlite::Connection) -> AppState {
    let total_skills: usize = conn
        .query_row("SELECT COUNT(*) FROM skills", [], |row| row.get(0))
        .unwrap_or(0);

    let enabled_skills: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM skills WHERE enabled = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let disabled_skills = total_skills.saturating_sub(enabled_skills);

    let projects: usize = conn
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
        .unwrap_or(0);

    AppState {
        total_skills,
        enabled_skills,
        disabled_skills,
        projects,
    }
}

/// Sync all skills on disk into the database.
/// Discovers global, vault, and project skills. Upserts found skills, removes stale DB entries.
pub fn sync_skills(conn: &rusqlite::Connection) -> Result<()> {
    let mut all_skills: Vec<Skill> = Vec::new();

    // Global skills
    all_skills.extend(discover_global_skills());

    // Vault (disabled) skills
    all_skills.extend(discover_vault_skills());

    // Project skills
    let projects = crate::db::get_all_projects(conn)?;
    for project in &projects {
        all_skills.extend(discover_project_skills(&project.path));
    }

    // Pre-fetch all existing skills into a HashMap to avoid N+1 queries
    let existing_skills = crate::db::get_all_skills(conn)?;
    let existing_map: std::collections::HashMap<String, Skill> = existing_skills
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();
    let existing_by_path: std::collections::HashMap<String, Skill> = existing_map
        .values()
        .cloned()
        .map(|s| (s.path.clone(), s))
        .collect();
    let existing_by_original_path: std::collections::HashMap<String, Skill> = existing_map
        .values()
        .cloned()
        .map(|s| (s.original_path.clone(), s))
        .collect();

    // Collect all discovered IDs
    let mut discovered_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for skill in &all_skills {
        discovered_ids.insert(skill.id.clone());

        let mut skill_to_save = skill.clone();

        // Preserve existing tags and metadata from the pre-fetched map
        if let Some(existing) = existing_map
            .get(&skill.id)
            .or_else(|| existing_by_path.get(&skill.path))
            .or_else(|| existing_by_original_path.get(&skill.original_path))
        {
            skill_to_save.id = existing.id.clone();
            if skill_to_save.original_path.is_empty() || skill_to_save.original_path == skill_to_save.path {
                skill_to_save.original_path = existing.original_path.clone();
            }
            if matches!(skill.scope, SkillScope::Global) && matches!(existing.scope, SkillScope::Project(_)) {
                skill_to_save.scope = existing.scope.clone();
            }
            if !existing.tags.is_empty() && skill_to_save.tags.is_empty() {
                skill_to_save.tags = existing.tags.clone();
            }
            if skill_to_save.category.is_empty() && !existing.category.is_empty() {
                skill_to_save.category = existing.category.clone();
            }
            if skill_to_save.author.is_empty() && !existing.author.is_empty() {
                skill_to_save.author = existing.author.clone();
            }
            if skill_to_save.source_url.is_empty() && !existing.source_url.is_empty() {
                skill_to_save.source_url = existing.source_url.clone();
            }
            // Keep original created_at
            skill_to_save.created_at = existing.created_at.clone();
        }

        crate::db::upsert_skill(conn, &skill_to_save)?;
    }

    // Remove DB entries whose files no longer exist on disk
    for (id, _) in &existing_map {
        if !discovered_ids.contains(id) {
            crate::db::delete_skill(conn, id)?;
        }
    }

    Ok(())
}

/// Load the bundled skills manifest
pub fn load_bundled_manifest() -> Vec<BundledManifestEntry> {
    let manifest_json = include_str!("../../bundled-skills/manifest.json");
    match serde_json::from_str::<BundledManifest>(manifest_json) {
        Ok(manifest) => manifest.skills,
        Err(_) => Vec::new(),
    }
}

/// Get bundled skills with their installed status
pub fn get_bundled_skills(_conn: &rusqlite::Connection) -> Vec<BundledSkill> {
    let entries = load_bundled_manifest();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    entries
        .into_iter()
        .map(|entry| {
            let skill_dir = home.join(".claude").join("skills").join(&entry.id);
            let installed = skill_dir.join("SKILL.md").exists();

            BundledSkill {
                id: entry.id,
                name: entry.name,
                description: entry.description,
                category: entry.category,
                source: entry.source,
                author: entry.author,
                tags: entry.tags,
                installed,
            }
        })
        .collect()
}

/// Move a directory by copying then removing (works across filesystems)
fn move_dir(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        return Err(SkillError::General(format!(
            "Destination already exists, refusing to merge skill directories: {}",
            dst.display()
        )));
    }

    // Try rename first (fast, same filesystem)
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    // Fallback: copy tree then remove source
    copy_dir_recursive(src, dst)?;
    fs::remove_dir_all(src)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
