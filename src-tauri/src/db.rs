use std::path::PathBuf;

use rusqlite::{params, Connection, Result as SqlResult};

use crate::models::{Project, Skill, SkillScope};

pub fn get_db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(".skiller").join("skiller.db"))
}

pub fn init_db() -> Result<Connection, String> {
    let db_path = get_db_path()?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create ~/.skiller/ directory: {}", e))?;
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            path TEXT NOT NULL,
            original_path TEXT NOT NULL DEFAULT '',
            scope TEXT NOT NULL DEFAULT 'global',
            scope_project TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            category TEXT NOT NULL DEFAULT '',
            author TEXT NOT NULL DEFAULT '',
            source_url TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS skill_tags (
            skill_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            PRIMARY KEY (skill_id, tag),
            FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            added_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS custom_repos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            owner TEXT NOT NULL,
            repo TEXT NOT NULL,
            skills_path TEXT NOT NULL DEFAULT 'skills',
            description TEXT NOT NULL DEFAULT '',
            added_at TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| format!("Failed to create tables: {}", e))?;

    Ok(conn)
}

pub fn upsert_skill(conn: &Connection, skill: &Skill) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO skills (id, name, description, path, original_path, scope, scope_project, enabled, category, author, source_url, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            path = excluded.path,
            original_path = excluded.original_path,
            scope = excluded.scope,
            scope_project = excluded.scope_project,
            enabled = excluded.enabled,
            category = excluded.category,
            author = excluded.author,
            source_url = excluded.source_url,
            updated_at = excluded.updated_at",
        params![
            skill.id,
            skill.name,
            skill.description,
            skill.path,
            skill.original_path,
            skill.scope.to_db_string(),
            skill.scope.project_path(),
            skill.enabled as i32,
            skill.category,
            skill.author,
            skill.source_url,
            skill.created_at,
            skill.updated_at,
        ],
    )?;

    // Sync tags: delete old, insert new
    conn.execute("DELETE FROM skill_tags WHERE skill_id = ?1", params![skill.id])?;
    for tag in &skill.tags {
        conn.execute(
            "INSERT OR IGNORE INTO skill_tags (skill_id, tag) VALUES (?1, ?2)",
            params![skill.id, tag],
        )?;
    }

    Ok(())
}

pub fn get_all_skills(conn: &Connection) -> SqlResult<Vec<Skill>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, path, original_path, scope, scope_project, enabled, category, author, source_url, created_at, updated_at FROM skills ORDER BY name",
    )?;

    let skill_rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let scope_str: String = row.get(5)?;
        let scope_project: Option<String> = row.get(6)?;
        let enabled_int: i32 = row.get(7)?;

        Ok(SkillRow {
            id,
            name: row.get(1)?,
            description: row.get(2)?,
            path: row.get(3)?,
            original_path: row.get(4)?,
            scope: SkillScope::from_db(&scope_str, scope_project),
            enabled: enabled_int != 0,
            category: row.get(8)?,
            author: row.get(9)?,
            source_url: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut skills = Vec::new();
    for row_result in skill_rows {
        let row = row_result?;
        let tags = get_tags(conn, &row.id)?;
        let content = load_skill_content(&row.path);
        let frontmatter = crate::skills::parse_frontmatter_from_content(&content);

        skills.push(Skill {
            id: row.id,
            name: row.name,
            description: row.description,
            path: row.path,
            original_path: row.original_path,
            scope: row.scope,
            enabled: row.enabled,
            tags,
            category: row.category,
            author: row.author,
            source_url: row.source_url,
            content,
            frontmatter,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
    }

    Ok(skills)
}

pub fn get_skill_by_id(conn: &Connection, id: &str) -> SqlResult<Option<Skill>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, path, original_path, scope, scope_project, enabled, category, author, source_url, created_at, updated_at FROM skills WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        let id: String = row.get(0)?;
        let scope_str: String = row.get(5)?;
        let scope_project: Option<String> = row.get(6)?;
        let enabled_int: i32 = row.get(7)?;

        Ok(SkillRow {
            id,
            name: row.get(1)?,
            description: row.get(2)?,
            path: row.get(3)?,
            original_path: row.get(4)?,
            scope: SkillScope::from_db(&scope_str, scope_project),
            enabled: enabled_int != 0,
            category: row.get(8)?,
            author: row.get(9)?,
            source_url: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    if let Some(row_result) = rows.next() {
        let row = row_result?;
        let tags = get_tags(conn, &row.id)?;
        let content = load_skill_content(&row.path);
        let frontmatter = crate::skills::parse_frontmatter_from_content(&content);

        Ok(Some(Skill {
            id: row.id,
            name: row.name,
            description: row.description,
            path: row.path,
            original_path: row.original_path,
            scope: row.scope,
            enabled: row.enabled,
            tags,
            category: row.category,
            author: row.author,
            source_url: row.source_url,
            content,
            frontmatter,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_skill(conn: &Connection, id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM skill_tags WHERE skill_id = ?1", params![id])?;
    conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn add_tag(conn: &Connection, skill_id: &str, tag: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO skill_tags (skill_id, tag) VALUES (?1, ?2)",
        params![skill_id, tag],
    )?;
    Ok(())
}

pub fn remove_tag(conn: &Connection, skill_id: &str, tag: &str) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM skill_tags WHERE skill_id = ?1 AND tag = ?2",
        params![skill_id, tag],
    )?;
    Ok(())
}

pub fn get_tags(conn: &Connection, skill_id: &str) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag FROM skill_tags WHERE skill_id = ?1 ORDER BY tag")?;
    let tags = stmt
        .query_map(params![skill_id], |row| row.get(0))?
        .collect::<SqlResult<Vec<String>>>()?;
    Ok(tags)
}

pub fn upsert_project(conn: &Connection, project: &Project) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO projects (id, name, path, added_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            path = excluded.path",
        params![project.id, project.name, project.path, project.added_at],
    )?;
    Ok(())
}

pub fn get_all_projects(conn: &Connection) -> SqlResult<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT id, name, path, added_at FROM projects ORDER BY name")?;
    let projects = stmt
        .query_map([], |row| {
            let path: String = row.get(2)?;
            let skill_count = count_project_skills(&path);
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path,
                skill_count,
                added_at: row.get(3)?,
            })
        })?
        .collect::<SqlResult<Vec<Project>>>()?;
    Ok(projects)
}

pub fn delete_project(conn: &Connection, id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_project_by_path(conn: &Connection, path: &str) -> SqlResult<Option<Project>> {
    let mut stmt =
        conn.prepare("SELECT id, name, path, added_at FROM projects WHERE path = ?1")?;
    let mut rows = stmt.query_map(params![path], |row| {
        let p: String = row.get(2)?;
        let skill_count = count_project_skills(&p);
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: p,
            skill_count,
            added_at: row.get(3)?,
        })
    })?;

    match rows.next() {
        Some(Ok(project)) => Ok(Some(project)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

// ─── Custom Repos ───────────────────────────────────────────────

pub fn upsert_custom_repo(conn: &Connection, repo: &crate::community::CustomRepo) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO custom_repos (id, name, owner, repo, skills_path, description, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            owner = excluded.owner,
            repo = excluded.repo,
            skills_path = excluded.skills_path,
            description = excluded.description",
        params![repo.id, repo.name, repo.owner, repo.repo, repo.skills_path, repo.description, repo.added_at],
    )?;
    Ok(())
}

pub fn get_all_custom_repos(conn: &Connection) -> SqlResult<Vec<crate::community::CustomRepo>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, owner, repo, skills_path, description, added_at FROM custom_repos ORDER BY name"
    )?;
    let repos = stmt
        .query_map([], |row| {
            Ok(crate::community::CustomRepo {
                id: row.get(0)?,
                name: row.get(1)?,
                owner: row.get(2)?,
                repo: row.get(3)?,
                skills_path: row.get(4)?,
                description: row.get(5)?,
                added_at: row.get(6)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(repos)
}

pub fn delete_custom_repo(conn: &Connection, id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM custom_repos WHERE id = ?1", params![id])?;
    Ok(())
}

// Internal helper structs and functions

struct SkillRow {
    id: String,
    name: String,
    description: String,
    path: String,
    original_path: String,
    scope: SkillScope,
    enabled: bool,
    category: String,
    author: String,
    source_url: String,
    created_at: String,
    updated_at: String,
}

fn load_skill_content(path: &str) -> String {
    let skill_md = std::path::Path::new(path).join("SKILL.md");
    std::fs::read_to_string(&skill_md).unwrap_or_default()
}

fn count_project_skills(project_path: &str) -> i32 {
    let skills_dir = std::path::Path::new(project_path)
        .join(".claude")
        .join("skills");
    if !skills_dir.exists() {
        return 0;
    }
    std::fs::read_dir(&skills_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join("SKILL.md").exists())
                .count() as i32
        })
        .unwrap_or(0)
}
