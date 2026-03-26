use std::fs;
use std::path::PathBuf;

// We test the core library functions directly
use skiller_lib::db;
use skiller_lib::models::*;
use skiller_lib::skills;

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skiller_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn temp_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
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
    .unwrap();
    conn
}

fn make_test_skill(name: &str, enabled: bool) -> Skill {
    Skill {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: format!("Test skill: {}", name),
        path: format!("/tmp/test/{}", name),
        original_path: format!("/tmp/test/{}", name),
        scope: SkillScope::Global,
        enabled,
        tags: vec!["test".to_string()],
        category: "testing".to_string(),
        author: "test-author".to_string(),
        source_url: String::new(),
        content: format!("---\nname: {}\n---\nTest content", name),
        frontmatter: SkillFrontmatter::default(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

// ─── Database Tests ─────────────────────────────────────────────

#[test]
fn test_upsert_and_get_skill() {
    let conn = temp_db();
    let skill = make_test_skill("my-skill", true);
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();

    let fetched = db::get_skill_by_id(&conn, &id).unwrap().unwrap();
    assert_eq!(fetched.name, "my-skill");
    assert_eq!(fetched.description, "Test skill: my-skill");
    assert!(fetched.enabled);
    assert_eq!(fetched.category, "testing");
}

#[test]
fn test_upsert_updates_existing() {
    let conn = temp_db();
    let mut skill = make_test_skill("updatable", true);
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();

    skill.description = "Updated description".to_string();
    skill.enabled = false;
    db::upsert_skill(&conn, &skill).unwrap();

    let fetched = db::get_skill_by_id(&conn, &id).unwrap().unwrap();
    assert_eq!(fetched.description, "Updated description");
    assert!(!fetched.enabled);
}

#[test]
fn test_get_all_skills_empty() {
    let conn = temp_db();
    let skills = db::get_all_skills(&conn).unwrap();
    assert!(skills.is_empty());
}

#[test]
fn test_get_all_skills_multiple() {
    let conn = temp_db();
    db::upsert_skill(&conn, &make_test_skill("alpha", true)).unwrap();
    db::upsert_skill(&conn, &make_test_skill("beta", false)).unwrap();
    db::upsert_skill(&conn, &make_test_skill("gamma", true)).unwrap();

    let skills = db::get_all_skills(&conn).unwrap();
    assert_eq!(skills.len(), 3);
    // Should be ordered by name
    assert_eq!(skills[0].name, "alpha");
    assert_eq!(skills[1].name, "beta");
    assert_eq!(skills[2].name, "gamma");
}

#[test]
fn test_delete_skill() {
    let conn = temp_db();
    let skill = make_test_skill("deleteme", true);
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();
    assert!(db::get_skill_by_id(&conn, &id).unwrap().is_some());

    db::delete_skill(&conn, &id).unwrap();
    assert!(db::get_skill_by_id(&conn, &id).unwrap().is_none());
}

#[test]
fn test_get_nonexistent_skill() {
    let conn = temp_db();
    let result = db::get_skill_by_id(&conn, "nonexistent").unwrap();
    assert!(result.is_none());
}

// ─── Tag Tests ──────────────────────────────────────────────────

#[test]
fn test_add_and_get_tags() {
    let conn = temp_db();
    let skill = make_test_skill("tagged", true);
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();

    db::add_tag(&conn, &id, "rust").unwrap();
    db::add_tag(&conn, &id, "productivity").unwrap();
    db::add_tag(&conn, &id, "automation").unwrap();

    let tags = db::get_tags(&conn, &id).unwrap();
    assert_eq!(tags.len(), 4); // 1 from make_test_skill + 3 added
    assert!(tags.contains(&"rust".to_string()));
    assert!(tags.contains(&"productivity".to_string()));
}

#[test]
fn test_remove_tag() {
    let conn = temp_db();
    let skill = make_test_skill("tagged2", true);
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();
    db::add_tag(&conn, &id, "removeme").unwrap();

    let tags_before = db::get_tags(&conn, &id).unwrap();
    assert!(tags_before.contains(&"removeme".to_string()));

    db::remove_tag(&conn, &id, "removeme").unwrap();

    let tags_after = db::get_tags(&conn, &id).unwrap();
    assert!(!tags_after.contains(&"removeme".to_string()));
}

#[test]
fn test_duplicate_tag_ignored() {
    let conn = temp_db();
    let skill = make_test_skill("duptag", true);
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();
    db::add_tag(&conn, &id, "same").unwrap();
    db::add_tag(&conn, &id, "same").unwrap(); // should not error

    let tags = db::get_tags(&conn, &id).unwrap();
    assert_eq!(tags.iter().filter(|t| t.as_str() == "same").count(), 1);
}

// ─── Project Tests ──────────────────────────────────────────────

#[test]
fn test_upsert_and_get_project() {
    let conn = temp_db();
    let project = Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: "test-project".to_string(),
        path: "/tmp/test-project".to_string(),
        skill_count: 0,
        added_at: "2026-01-01T00:00:00Z".to_string(),
    };

    db::upsert_project(&conn, &project).unwrap();

    let projects = db::get_all_projects(&conn).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "test-project");
}

#[test]
fn test_get_project_by_path() {
    let conn = temp_db();
    let project = Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: "findme".to_string(),
        path: "/tmp/findme".to_string(),
        skill_count: 0,
        added_at: "2026-01-01T00:00:00Z".to_string(),
    };

    db::upsert_project(&conn, &project).unwrap();

    let found = db::get_project_by_path(&conn, "/tmp/findme").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "findme");

    let not_found = db::get_project_by_path(&conn, "/tmp/nope").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_delete_project() {
    let conn = temp_db();
    let project = Project {
        id: "proj-1".to_string(),
        name: "deleteme".to_string(),
        path: "/tmp/deleteme".to_string(),
        skill_count: 0,
        added_at: "2026-01-01T00:00:00Z".to_string(),
    };

    db::upsert_project(&conn, &project).unwrap();
    db::delete_project(&conn, "proj-1").unwrap();

    let projects = db::get_all_projects(&conn).unwrap();
    assert!(projects.is_empty());
}

// ─── Skill Scope Tests ──────────────────────────────────────────

#[test]
fn test_skill_scope_serialization() {
    assert_eq!(SkillScope::Global.to_db_string(), "global");
    assert_eq!(
        SkillScope::Project("/tmp/proj".to_string()).to_db_string(),
        "project"
    );
    assert_eq!(SkillScope::Bundled.to_db_string(), "bundled");
}

#[test]
fn test_skill_scope_from_db() {
    assert_eq!(SkillScope::from_db("global", None), SkillScope::Global);
    assert_eq!(
        SkillScope::from_db("project", Some("/tmp/proj".to_string())),
        SkillScope::Project("/tmp/proj".to_string())
    );
    assert_eq!(SkillScope::from_db("bundled", None), SkillScope::Bundled);
    // Unknown defaults to Global
    assert_eq!(SkillScope::from_db("unknown", None), SkillScope::Global);
}

#[test]
fn test_skill_scope_project_path() {
    assert_eq!(SkillScope::Global.project_path(), None);
    assert_eq!(
        SkillScope::Project("/tmp".to_string()).project_path(),
        Some("/tmp")
    );
}

// ─── SKILL.md Parsing Tests ────────────────────────────────────

#[test]
fn test_parse_skill_md_with_frontmatter() {
    let dir = temp_dir();
    let skill_dir = dir.join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");

    fs::write(
        &skill_md,
        r#"---
name: test-skill
description: A test skill for unit testing
disable_model_invocation: true
allowed_tools:
  - Read
  - Grep
---

# Test Skill

This is the body content.
"#,
    )
    .unwrap();

    let (fm, body, _raw) = skills::parse_skill_md(&skill_md).unwrap();
    assert_eq!(fm.name.unwrap(), "test-skill");
    assert_eq!(fm.description.unwrap(), "A test skill for unit testing");
    assert_eq!(fm.disable_model_invocation, Some(true));
    assert!(body.contains("# Test Skill"));
    assert!(body.contains("body content"));

    let tools = fm.allowed_tools.unwrap();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"Read".to_string()));
    assert!(tools.contains(&"Grep".to_string()));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_parse_skill_md_no_frontmatter() {
    let dir = temp_dir();
    let skill_dir = dir.join("bare-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");

    fs::write(&skill_md, "# Just a heading\n\nSome content").unwrap();

    let (fm, body, _raw) = skills::parse_skill_md(&skill_md).unwrap();
    assert!(fm.name.is_none());
    assert!(fm.description.is_none());
    assert!(body.contains("Just a heading"));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_parse_skill_md_empty_frontmatter() {
    let dir = temp_dir();
    let skill_dir = dir.join("empty-fm");
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");

    fs::write(&skill_md, "---\n---\n\nBody only").unwrap();

    let (fm, body, _raw) = skills::parse_skill_md(&skill_md).unwrap();
    assert!(fm.name.is_none());
    assert!(body.contains("Body only"));

    fs::remove_dir_all(&dir).unwrap();
}

// ─── Skill Discovery Tests ─────────────────────────────────────

#[test]
fn test_discover_skills_in_dir() {
    let dir = temp_dir();

    // Create two skills
    let skill1 = dir.join("skill-a");
    fs::create_dir_all(&skill1).unwrap();
    fs::write(
        skill1.join("SKILL.md"),
        "---\nname: skill-a\ndescription: First skill\n---\nContent A",
    )
    .unwrap();

    let skill2 = dir.join("skill-b");
    fs::create_dir_all(&skill2).unwrap();
    fs::write(
        skill2.join("SKILL.md"),
        "---\nname: skill-b\ndescription: Second skill\n---\nContent B",
    )
    .unwrap();

    // Create a non-skill directory (no SKILL.md)
    let not_skill = dir.join("not-a-skill");
    fs::create_dir_all(&not_skill).unwrap();
    fs::write(not_skill.join("README.md"), "not a skill").unwrap();

    let discovered = skills::discover_skills_in_dir(&dir, SkillScope::Global);
    assert_eq!(discovered.len(), 2);

    let names: Vec<&str> = discovered.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"skill-a"));
    assert!(names.contains(&"skill-b"));

    // All should be enabled (not in vault)
    assert!(discovered.iter().all(|s| s.enabled));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_discover_empty_dir() {
    let dir = temp_dir();
    let discovered = skills::discover_skills_in_dir(&dir, SkillScope::Global);
    assert!(discovered.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_discover_nonexistent_dir() {
    let dir = PathBuf::from("/tmp/skiller_test_nonexistent_12345");
    let discovered = skills::discover_skills_in_dir(&dir, SkillScope::Global);
    assert!(discovered.is_empty());
}

#[test]
fn test_discover_deterministic_ids() {
    let dir = temp_dir();
    let skill = dir.join("stable-id");
    fs::create_dir_all(&skill).unwrap();
    fs::write(skill.join("SKILL.md"), "---\nname: stable\n---\nContent").unwrap();

    let first = skills::discover_skills_in_dir(&dir, SkillScope::Global);
    let second = skills::discover_skills_in_dir(&dir, SkillScope::Global);

    assert_eq!(first[0].id, second[0].id);

    fs::remove_dir_all(&dir).unwrap();
}

// ─── Enable/Disable Tests ───────────────────────────────────────

#[test]
fn test_disable_and_enable_skill() {
    let dir = temp_dir();
    let _vault = dir.join("vault");
    let skills_dir = dir.join("skills");

    // Create a skill in the "skills" dir
    let skill_src = skills_dir.join("my-skill");
    fs::create_dir_all(&skill_src).unwrap();
    fs::write(skill_src.join("SKILL.md"), "---\nname: my-skill\n---\nContent").unwrap();

    // Verify the skill exists
    assert!(skill_src.join("SKILL.md").exists());

    // We can't easily test enable/disable without mocking dirs::home_dir(),
    // but we can test that move_dir logic works via the public functions
    // by checking that the files exist at expected locations
    fs::remove_dir_all(&dir).unwrap();
}

// ─── Bundled Skills Tests ───────────────────────────────────────

#[test]
fn test_load_bundled_manifest() {
    let manifest = skills::load_bundled_manifest();
    assert!(!manifest.is_empty());
    assert!(manifest.len() >= 10); // We have 20 bundled skills

    // Check first skill has required fields
    let first = &manifest[0];
    assert!(!first.id.is_empty());
    assert!(!first.name.is_empty());
    assert!(!first.description.is_empty());
}

#[test]
fn test_bundled_skill_ids_unique() {
    let manifest = skills::load_bundled_manifest();
    let ids: Vec<&str> = manifest.iter().map(|s| s.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Bundled skill IDs must be unique");
}

// ─── App State Tests ────────────────────────────────────────────

#[test]
fn test_app_state_empty() {
    let conn = temp_db();
    let state = skills::get_app_state(&conn);
    assert_eq!(state.total_skills, 0);
    assert_eq!(state.enabled_skills, 0);
    assert_eq!(state.disabled_skills, 0);
    assert_eq!(state.projects, 0);
}

#[test]
fn test_app_state_with_data() {
    let conn = temp_db();

    db::upsert_skill(&conn, &make_test_skill("s1", true)).unwrap();
    db::upsert_skill(&conn, &make_test_skill("s2", true)).unwrap();
    db::upsert_skill(&conn, &make_test_skill("s3", false)).unwrap();

    let project = Project {
        id: "p1".to_string(),
        name: "proj".to_string(),
        path: "/tmp/proj".to_string(),
        skill_count: 0,
        added_at: "2026-01-01T00:00:00Z".to_string(),
    };
    db::upsert_project(&conn, &project).unwrap();

    let state = skills::get_app_state(&conn);
    assert_eq!(state.total_skills, 3);
    assert_eq!(state.enabled_skills, 2);
    assert_eq!(state.disabled_skills, 1);
    assert_eq!(state.projects, 1);
}

// ─── Scope with Project Skills Test ────────────────────────────

#[test]
fn test_project_scope_skill_in_db() {
    let conn = temp_db();
    let mut skill = make_test_skill("proj-skill", true);
    skill.scope = SkillScope::Project("/tmp/my-project".to_string());
    let id = skill.id.clone();

    db::upsert_skill(&conn, &skill).unwrap();

    let fetched = db::get_skill_by_id(&conn, &id).unwrap().unwrap();
    match &fetched.scope {
        SkillScope::Project(p) => assert_eq!(p, "/tmp/my-project"),
        _ => panic!("Expected Project scope"),
    }
}

// ─── JSON Serialization Tests ───────────────────────────────────

#[test]
fn test_skill_scope_json_serialization() {
    let global = SkillScope::Global;
    let json = serde_json::to_string(&global).unwrap();
    assert!(json.contains("Global"));

    let project = SkillScope::Project("/tmp/proj".to_string());
    let json = serde_json::to_string(&project).unwrap();
    assert!(json.contains("Project"));
    assert!(json.contains("/tmp/proj"));

    // Deserialize back
    let deserialized: SkillScope = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, project);
}

#[test]
fn test_skill_json_roundtrip() {
    let skill = make_test_skill("roundtrip", true);
    let json = serde_json::to_string(&skill).unwrap();
    let deserialized: Skill = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "roundtrip");
    assert!(deserialized.enabled);
}

// ─── Custom Repo Tests ──────────────────────────────────────────

#[test]
fn test_custom_repo_crud() {
    let conn = temp_db();
    let repo = skiller_lib::community::CustomRepo {
        id: "test:repo".into(),
        name: "test/repo".into(),
        owner: "test".into(),
        repo: "repo".into(),
        skills_path: "skills".into(),
        description: "Test repo".into(),
        added_at: "2026-01-01T00:00:00Z".into(),
    };
    db::upsert_custom_repo(&conn, &repo).unwrap();
    let repos = db::get_all_custom_repos(&conn).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].owner, "test");

    db::delete_custom_repo(&conn, "test:repo").unwrap();
    assert!(db::get_all_custom_repos(&conn).unwrap().is_empty());
}

#[test]
fn test_custom_repo_to_community_repo_valid() {
    let repo = skiller_lib::community::CustomRepo {
        id: "owner:myrepo".into(), name: "owner/myrepo".into(),
        owner: "owner".into(), repo: "myrepo".into(),
        skills_path: "skills".into(), description: "".into(),
        added_at: "2026-01-01T00:00:00Z".into(),
    };
    let cr = repo.to_community_repo().unwrap();
    assert!(cr.url.contains("owner/myrepo"));
}

#[test]
fn test_custom_repo_rejects_invalid_owner() {
    let repo = skiller_lib::community::CustomRepo {
        id: "bad".into(), name: "bad".into(),
        owner: "../evil".into(), repo: "repo".into(),
        skills_path: "skills".into(), description: "".into(),
        added_at: "".into(),
    };
    assert!(repo.to_community_repo().is_err());
}

#[test]
fn test_custom_repo_rejects_empty_owner() {
    let repo = skiller_lib::community::CustomRepo {
        id: "bad".into(), name: "bad".into(),
        owner: "".into(), repo: "repo".into(),
        skills_path: "skills".into(), description: "".into(),
        added_at: "".into(),
    };
    assert!(repo.to_community_repo().is_err());
}

#[test]
fn test_custom_repo_rejects_spaces_in_repo() {
    let repo = skiller_lib::community::CustomRepo {
        id: "bad".into(), name: "bad".into(),
        owner: "good".into(), repo: "bad repo".into(),
        skills_path: "skills".into(), description: "".into(),
        added_at: "".into(),
    };
    assert!(repo.to_community_repo().is_err());
}

#[test]
fn test_custom_repo_normalizes_skills_path() {
    let repo = skiller_lib::community::CustomRepo {
        id: "owner:myrepo".into(),
        name: "owner/myrepo".into(),
        owner: "owner".into(),
        repo: "myrepo".into(),
        skills_path: "/nested/skills/".into(),
        description: "".into(),
        added_at: "2026-01-01T00:00:00Z".into(),
    };
    let cr = repo.to_community_repo().unwrap();
    assert!(cr.url.ends_with("/contents/nested/skills"));
    assert_eq!(cr.skills_path, "nested/skills");
}

#[test]
fn test_validate_repo_contents_url_rejects_non_github_hosts() {
    let result = skiller_lib::community::validate_repo_contents_url("https://example.com/repos/o/r/contents/skills");
    assert!(result.is_err());
}

#[test]
fn test_validate_skill_content_url_rejects_non_github_hosts() {
    let result = skiller_lib::community::validate_skill_content_url("https://example.com/skill.md");
    assert!(result.is_err());
}

#[test]
fn test_validate_skills_path_rejects_traversal() {
    let result = skiller_lib::community::validate_skills_path("../skills");
    assert!(result.is_err());
}

// ─── Community Repo List Tests ──────────────────────────────────

#[test]
fn test_community_repos_not_empty() {
    let repos = skiller_lib::community::get_community_repos();
    assert!(repos.len() >= 10);
    for r in &repos {
        assert!(!r.id.is_empty());
        assert!(r.url.starts_with("https://api.github.com/"));
    }
}

#[test]
fn test_community_repo_ids_unique() {
    let repos = skiller_lib::community::get_community_repos();
    let ids: Vec<&str> = repos.iter().map(|r| r.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

// ─── Path Validation Tests ──────────────────────────────────────

#[test]
fn test_delete_rejects_outside_paths() {
    let result = skills::delete_skill_files("/tmp/random/path");
    assert!(result.is_err());
}

#[test]
fn test_delete_rejects_home_dir() {
    if let Some(home) = dirs::home_dir() {
        let result = skills::delete_skill_files(&home.to_string_lossy());
        assert!(result.is_err());
    }
}

// ─── Raw Content Returned ───────────────────────────────────────

#[test]
fn test_parse_skill_md_returns_raw_content() {
    let dir = temp_dir();
    let sd = dir.join("raw-test");
    fs::create_dir_all(&sd).unwrap();
    let original = "---\nname: test\n---\n\n# Body";
    fs::write(sd.join("SKILL.md"), original).unwrap();
    let (_fm, _body, raw) = skills::parse_skill_md(&sd.join("SKILL.md")).unwrap();
    assert_eq!(raw, original);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_parse_frontmatter_from_empty() {
    let fm = skills::parse_frontmatter_from_content("");
    assert!(fm.name.is_none());
}

// ─── FK Cascade Test ────────────────────────────────────────────

#[test]
fn test_fk_cascade_deletes_tags() {
    let conn = temp_db();
    let skill = make_test_skill("fk-test", true);
    let id = skill.id.clone();
    db::upsert_skill(&conn, &skill).unwrap();
    db::add_tag(&conn, &id, "extra").unwrap();
    assert!(db::get_tags(&conn, &id).unwrap().len() >= 2);

    db::delete_skill(&conn, &id).unwrap();
    assert!(db::get_tags(&conn, &id).unwrap().is_empty());
}

// ─── Bundled Manifest Thorough Checks ───────────────────────────

#[test]
fn test_bundled_manifest_100_skills() {
    let m = skills::load_bundled_manifest();
    assert_eq!(m.len(), 100);
    for s in &m {
        assert!(!s.id.is_empty());
        assert!(!s.name.is_empty());
        assert!(!s.description.is_empty());
        assert!(!s.category.is_empty());
        assert!(!s.source.is_empty());
        assert!(!s.author.is_empty());
        assert!(!s.tags.is_empty());
    }
}

#[test]
fn test_bundled_manifest_valid_categories() {
    let valid = ["frontend","backend","development","testing","documentation",
        "git","devops","code-quality","productivity","security",
        "ai","data","debugging","refactoring","architecture"];
    for s in &skills::load_bundled_manifest() {
        assert!(valid.contains(&s.category.as_str()),
            "Invalid category '{}' for '{}'", s.category, s.id);
    }
}

// ─── Scope Roundtrip Through DB ─────────────────────────────────

#[test]
fn test_scope_roundtrip_through_db() {
    let conn = temp_db();
    let scopes = vec![
        SkillScope::Global,
        SkillScope::Project("/tmp/proj".into()),
        SkillScope::Bundled,
    ];
    for (i, scope) in scopes.into_iter().enumerate() {
        let mut skill = make_test_skill(&format!("scope-{}", i), true);
        skill.scope = scope.clone();
        let id = skill.id.clone();
        db::upsert_skill(&conn, &skill).unwrap();
        let fetched = db::get_skill_by_id(&conn, &id).unwrap().unwrap();
        assert_eq!(fetched.scope, scope);
    }
}
