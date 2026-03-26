use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub original_path: String,
    pub scope: SkillScope,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub category: String,
    pub author: String,
    pub source_url: String,
    pub content: String,
    pub frontmatter: SkillFrontmatter,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "project_path")]
pub enum SkillScope {
    Global,
    Project(String),
    Bundled,
}

impl SkillScope {
    pub fn to_db_string(&self) -> &str {
        match self {
            SkillScope::Global => "global",
            SkillScope::Project(_) => "project",
            SkillScope::Bundled => "bundled",
        }
    }

    pub fn project_path(&self) -> Option<&str> {
        match self {
            SkillScope::Project(p) => Some(p.as_str()),
            _ => None,
        }
    }

    pub fn from_db(scope: &str, project: Option<String>) -> Self {
        match scope {
            "project" => SkillScope::Project(project.unwrap_or_default()),
            "bundled" => SkillScope::Bundled,
            _ => SkillScope::Global,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub disable_model_invocation: Option<bool>,
    pub user_invocable: Option<bool>,
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub context: Option<String>,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub skill_count: i32,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundledSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub source: String,
    pub author: String,
    pub tags: Vec<String>,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub total_skills: usize,
    pub enabled_skills: usize,
    pub disabled_skills: usize,
    pub projects: usize,
}

#[derive(Debug, Deserialize)]
pub struct BundledManifest {
    pub skills: Vec<BundledManifestEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BundledManifestEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub source: String,
    pub author: String,
    pub tags: Vec<String>,
}
