use serde::{Deserialize, Serialize};

/// A community skill repo we can fetch skills from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRepo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub skills_path: String,
    pub author: String,
}

/// Validate that a GitHub owner or repo name is safe (alphanumeric, dot, hyphen, underscore)
fn is_valid_github_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

fn normalize_skills_path(skills_path: &str) -> String {
    skills_path.trim_matches('/').to_string()
}

pub fn validate_skills_path(skills_path: &str) -> Result<String, String> {
    let normalized = normalize_skills_path(skills_path);
    if normalized.contains("..") || normalized.contains('\\') {
        return Err(format!("Invalid skills path: {}", skills_path));
    }
    Ok(normalized)
}

fn parse_allowed_url(url: &str) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("Invalid URL '{}': {}", url, e))?;
    if parsed.scheme() != "https" {
        return Err(format!("Only https URLs are allowed: {}", url));
    }
    Ok(parsed)
}

fn is_allowed_repo_contents_url(url: &reqwest::Url) -> bool {
    matches!(url.host_str(), Some("api.github.com"))
        && url.path().starts_with("/repos/")
        && url.path().contains("/contents")
        && !url.path().contains("..")
}

fn is_allowed_skill_content_url(url: &reqwest::Url) -> bool {
    match url.host_str() {
        Some("api.github.com") => {
            url.path().starts_with("/repos/")
                && url.path().contains("/contents/")
                && !url.path().contains("..")
        }
        Some("raw.githubusercontent.com") => !url.path().contains(".."),
        _ => false,
    }
}

pub fn validate_repo_contents_url(url: &str) -> Result<(), String> {
    let parsed = parse_allowed_url(url)?;
    if !is_allowed_repo_contents_url(&parsed) {
        return Err(format!("Unsupported repository URL: {}", url));
    }
    Ok(())
}

pub fn validate_skill_content_url(url: &str) -> Result<(), String> {
    let parsed = parse_allowed_url(url)?;
    if !is_allowed_skill_content_url(&parsed) {
        return Err(format!("Unsupported skill content URL: {}", url));
    }
    Ok(())
}

/// A user-added custom repo stored in SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRepo {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub repo: String,
    pub skills_path: String,
    pub description: String,
    pub added_at: String,
}

impl CustomRepo {
    pub fn to_community_repo(&self) -> Result<CommunityRepo, String> {
        if !is_valid_github_name(&self.owner) {
            return Err(format!("Invalid GitHub owner name: {}", self.owner));
        }
        if !is_valid_github_name(&self.repo) {
            return Err(format!("Invalid GitHub repo name: {}", self.repo));
        }
        let skills_path = validate_skills_path(&self.skills_path)?;
        Ok(CommunityRepo {
            id: format!("custom-{}", self.id),
            name: self.name.clone(),
            url: format!(
                "https://api.github.com/repos/{}/{}/contents/{}",
                self.owner, self.repo, skills_path
            ),
            description: self.description.clone(),
            skills_path,
            author: self.owner.clone(),
        })
    }
}

/// A skill discovered from a community repo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitySkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub repo_id: String,
    pub repo_name: String,
    pub author: String,
    pub source_url: String,
    pub download_url: String,
    pub dir_name: String,
    pub tags: Vec<String>,
    pub installed: bool,
}

/// Result from syncing community repos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitySyncResult {
    pub total_repos: usize,
    pub total_skills: usize,
    pub skills: Vec<CommunitySkill>,
    pub errors: Vec<String>,
}

/// List of curated community repos to sync from.
/// Every repo here is verified to have subdirectories containing SKILL.md files.
pub fn get_community_repos() -> Vec<CommunityRepo> {
    vec![
        CommunityRepo {
            id: "anthropics-skills".into(),
            name: "Anthropic Official Skills".into(),
            url: "https://api.github.com/repos/anthropics/skills/contents/skills".into(),
            description: "Official skills from Anthropic".into(),
            skills_path: "skills".into(),
            author: "anthropics".into(),
        },
        CommunityRepo {
            id: "affaan-everything".into(),
            name: "Everything Claude Code".into(),
            url: "https://api.github.com/repos/affaan-m/everything-claude-code/contents/skills".into(),
            description: "28 agents, 119 skills, 60 commands, AgentShield security".into(),
            skills_path: "skills".into(),
            author: "affaan-m".into(),
        },
        CommunityRepo {
            id: "obra-superpowers".into(),
            name: "Obra Superpowers".into(),
            url: "https://api.github.com/repos/obra/superpowers/contents/skills".into(),
            description: "TDD, debugging, and planning skills by Jesse Vincent".into(),
            skills_path: "skills".into(),
            author: "obra".into(),
        },
        CommunityRepo {
            id: "vercel-agent-skills".into(),
            name: "Vercel Agent Skills".into(),
            url: "https://api.github.com/repos/vercel-labs/agent-skills/contents/skills".into(),
            description: "React/Next.js best practices from Vercel Engineering".into(),
            skills_path: "skills".into(),
            author: "vercel".into(),
        },
        CommunityRepo {
            id: "kdense-scientific".into(),
            name: "K-Dense Scientific Skills".into(),
            url: "https://api.github.com/repos/K-Dense-AI/claude-scientific-skills/contents/scientific-skills".into(),
            description: "170+ scientific skills: genomics, chemistry, physics, bioinformatics".into(),
            skills_path: "scientific-skills".into(),
            author: "K-Dense-AI".into(),
        },
        CommunityRepo {
            id: "qdhenry-command-suite".into(),
            name: "Claude Command Suite".into(),
            url: "https://api.github.com/repos/qdhenry/Claude-Command-Suite/contents/.claude/skills".into(),
            description: "Security, architecture, and development skills".into(),
            skills_path: ".claude/skills".into(),
            author: "qdhenry".into(),
        },
        CommunityRepo {
            id: "levnikolaevich-skills".into(),
            name: "Lev Nikolaevich Skills".into(),
            url: "https://api.github.com/repos/levnikolaevich/claude-code-skills/contents/skills-catalog".into(),
            description: "129 skills: agile pipeline, codebase auditor, MCP tools".into(),
            skills_path: "skills-catalog".into(),
            author: "levnikolaevich".into(),
        },
        CommunityRepo {
            id: "mrgoonie-claudekit".into(),
            name: "ClaudeKit Skills".into(),
            url: "https://api.github.com/repos/mrgoonie/claudekit-skills/contents/.claude/skills".into(),
            description: "Shopify, payments, sequential thinking, 32 skills".into(),
            skills_path: ".claude/skills".into(),
            author: "mrgoonie".into(),
        },
        CommunityRepo {
            id: "glebis-skills".into(),
            name: "Glebis Skills".into(),
            url: "https://api.github.com/repos/glebis/claude-skills/contents".into(),
            description: "Deep research, health research, TDD, text humanizer".into(),
            skills_path: "".into(),
            author: "glebis".into(),
        },
        CommunityRepo {
            id: "gentleman-skills".into(),
            name: "Gentleman Programming Skills".into(),
            url: "https://api.github.com/repos/Gentleman-Programming/Gentleman-Skills/contents/curated".into(),
            description: "Angular, React 19, Django, AI SDK, and more".into(),
            skills_path: "curated".into(),
            author: "Gentleman-Programming".into(),
        },
        CommunityRepo {
            id: "ahmedasmar-devops".into(),
            name: "DevOps Skills".into(),
            url: "https://api.github.com/repos/ahmedasmar/devops-claude-skills/contents".into(),
            description: "Kubernetes, Terraform, CI/CD, AWS cost optimization".into(),
            skills_path: "".into(),
            author: "ahmedasmar".into(),
        },
        CommunityRepo {
            id: "vincenthopf-skills".into(),
            name: "Vincent Hopf Skills".into(),
            url: "https://api.github.com/repos/vincenthopf/My-Claude-Code/contents/skills".into(),
            description: "Browser automation, deep research, pi-agent".into(),
            skills_path: "skills".into(),
            author: "vincenthopf".into(),
        },
        CommunityRepo {
            id: "conorluddy-ios".into(),
            name: "iOS Simulator Skill".into(),
            url: "https://api.github.com/repos/conorluddy/ios-simulator-skill/contents".into(),
            description: "iOS app testing with 21 automation scripts".into(),
            skills_path: "".into(),
            author: "conorluddy".into(),
        },
        CommunityRepo {
            id: "lackeyjb-playwright".into(),
            name: "Playwright Skill".into(),
            url: "https://api.github.com/repos/lackeyjb/playwright-skill/contents/skills".into(),
            description: "Browser automation with Playwright for testing and workflows".into(),
            skills_path: "skills".into(),
            author: "lackeyjb".into(),
        },
    ]
}

/// GitHub API content entry
#[derive(Debug, Deserialize)]
struct GithubContent {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
    html_url: Option<String>,
    url: Option<String>,
}

/// Load GitHub token from ~/.skiller/github_token if it exists
pub fn load_github_token() -> Option<String> {
    let home = dirs::home_dir()?;
    let token_path = home.join(".skiller").join("github_token");
    std::fs::read_to_string(token_path).ok().map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
}

/// Save GitHub token to ~/.skiller/github_token
pub fn save_github_token(token: &str) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("No home directory")?;
    let dir = home.join(".skiller");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;
    let token_path = dir.join("github_token");
    std::fs::write(&token_path, token.trim())
        .map_err(|e| format!("Failed to save token: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set token file permissions: {}", e))?;
    }

    Ok(())
}

/// Delete GitHub token
pub fn delete_github_token() -> Result<(), String> {
    let home = dirs::home_dir().ok_or("No home directory")?;
    let path = home.join(".skiller").join("github_token");
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete token: {}", e))?;
    }
    Ok(())
}

/// Build an HTTP client with optional GitHub auth
fn build_github_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("Skiller/1.0")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))
}

/// Add auth header if token exists
fn github_request(client: &reqwest::Client, url: &str) -> reqwest::RequestBuilder {
    let parsed = reqwest::Url::parse(url).expect("validated GitHub URL");
    let mut req = client
        .get(url)
        .header("Accept", "application/vnd.github.v3+json");
    if parsed.host_str() == Some("api.github.com") {
        if let Some(token) = load_github_token() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
    }
    req
}

/// Fetch skill directories from a GitHub repo contents API URL
pub async fn fetch_repo_skills(repo: &CommunityRepo) -> Result<Vec<CommunitySkill>, String> {
    validate_repo_contents_url(&repo.url)?;
    let client = build_github_client()?;

    let response = github_request(&client, &repo.url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {}: {}", repo.name, e))?;

    if response.status().as_u16() == 403 {
        return Err(format!(
            "GitHub rate limit hit for {}. Add a GitHub token in Settings to get 5,000 req/hr.",
            repo.name
        ));
    }

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned {} for {}",
            response.status(),
            repo.name
        ));
    }

    let entries: Vec<GithubContent> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response from {}: {}", repo.name, e))?;

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let skills_dir = home.join(".claude").join("skills");

    let mut skills = Vec::new();

    for entry in entries {
        if entry.entry_type != "dir" {
            continue;
        }

        let dir_name = entry.name.clone();
        let skill_id = format!("{}-{}", repo.id, dir_name);
        let installed = skills_dir.join(&dir_name).join("SKILL.md").exists();
        let source_url = entry.html_url.unwrap_or_default();

        // Use the GitHub API URL for the directory, append /SKILL.md
        // entry.url is like: https://api.github.com/repos/owner/repo/contents/skills/name?ref=main
        // We use the repo.url (parent) + /dir_name/SKILL.md for the API fetch
        let download_url = entry
            .url
            .as_ref()
            .map(|url| format!("{}/SKILL.md", url.trim_end_matches('/')))
            .ok_or_else(|| format!("Missing GitHub API URL for {}", dir_name))?;
        validate_skill_content_url(&download_url)?;

        // Clean up the name
        let name = dir_name
            .replace('-', " ")
            .split_whitespace()
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        skills.push(CommunitySkill {
            id: skill_id,
            name,
            description: format!("Skill from {}", repo.name),
            category: categorize_from_repo(&repo.id),
            repo_id: repo.id.clone(),
            repo_name: repo.name.clone(),
            author: repo.author.clone(),
            source_url,
            download_url,
            dir_name,
            tags: vec![repo.author.clone()],
            installed,
        });
    }

    Ok(skills)
}

/// GitHub file content response
#[derive(Debug, Deserialize)]
struct GithubFileContent {
    content: Option<String>,
    encoding: Option<String>,
    download_url: Option<String>,
}

/// Fetch SKILL.md content via GitHub API (handles both API JSON and raw responses)
pub async fn fetch_skill_md_content(url: &str) -> Result<String, String> {
    validate_skill_content_url(url)?;
    let client = build_github_client()?;

    // First try: request raw content directly via Accept header
    let response = github_request(&client, url)
        .header("Accept", "application/vnd.github.v3.raw")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch SKILL.md: {}", e))?;

    if response.status().as_u16() == 403 {
        return Err("GitHub rate limit hit. Add a GitHub token in Settings.".into());
    }

    if !response.status().is_success() {
        return Err(format!("Failed to download SKILL.md: {}", response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // If the response is JSON (API didn't honor raw accept), parse it
    if text.starts_with('{') {
        if let Ok(file) = serde_json::from_str::<GithubFileContent>(&text) {
            // Try download_url first (direct raw link)
            if let Some(dl_url) = &file.download_url {
                let raw_resp = github_request(&client, dl_url)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to fetch raw content: {}", e))?;
                if raw_resp.status().is_success() {
                    return raw_resp.text().await.map_err(|e| e.to_string());
                }
            }
            // Fallback: decode base64 content
            if let (Some(content), Some(encoding)) = (&file.content, &file.encoding) {
                if encoding == "base64" {
                    let cleaned = content.replace('\n', "").replace('\r', "");
                    use base64::Engine;
                    let decoded = base64::engine::general_purpose::STANDARD
                        .decode(&cleaned)
                        .map_err(|e| format!("Base64 decode error: {}", e))?;
                    return String::from_utf8(decoded)
                        .map_err(|e| format!("UTF-8 decode error: {}", e));
                }
            }
        }
    }

    // Response was already raw text
    Ok(text)
}

/// Install a community skill by downloading its SKILL.md
pub async fn install_community_skill(skill: &CommunitySkill) -> Result<String, String> {
    // Validate dir_name to prevent path traversal
    if !is_valid_github_name(&skill.dir_name) {
        return Err(format!("Invalid skill directory name: {}", skill.dir_name));
    }
    validate_skill_content_url(&skill.download_url)?;

    let content = fetch_skill_md_content(&skill.download_url).await?;

    let home = dirs::home_dir().ok_or("No home directory")?;

    let skill_dir = home.join(".claude").join("skills").join(&skill.dir_name);
    if skill_dir.exists() {
        return Err(format!(
            "A skill directory already exists at {}",
            skill_dir.display()
        ));
    }
    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create skill directory: {}", e))?;

    let skill_md = skill_dir.join("SKILL.md");
    std::fs::write(&skill_md, content)
        .map_err(|e| format!("Failed to write SKILL.md: {}", e))?;

    Ok(skill_dir.to_string_lossy().to_string())
}

/// Sync all community repos (built-in + custom) and return combined results
pub async fn sync_community_repos(custom_repos: Vec<CustomRepo>) -> CommunitySyncResult {
    let mut repos = get_community_repos();
    let mut all_skills = Vec::new();
    let mut errors = Vec::new();

    // Add user's custom repos
    for cr in &custom_repos {
        match cr.to_community_repo() {
            Ok(repo) => repos.push(repo),
            Err(e) => errors.push(e),
        }
    }

    for repo in &repos {
        match fetch_repo_skills(repo).await {
            Ok(skills) => all_skills.extend(skills),
            Err(e) => errors.push(e),
        }
    }

    let total_repos = repos.len();
    let total_skills = all_skills.len();

    CommunitySyncResult {
        total_repos,
        total_skills,
        skills: all_skills,
        errors,
    }
}

fn categorize_from_repo(repo_id: &str) -> String {
    match repo_id {
        "obra-superpowers" => "productivity".into(),
        "vercel-agent-skills" => "code-quality".into(),
        "ahmedasmar-devops" => "devops".into(),
        _ => "development".into(),
    }
}
