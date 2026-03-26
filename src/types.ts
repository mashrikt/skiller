// ─── Skill Types ────────────────────────────────────────────────

export type SkillScope =
  | { type: "Global" }
  | { type: "Project"; project_path: string }
  | { type: "Bundled" };

export interface SkillFrontmatter {
  name: string | null;
  description: string | null;
  disable_model_invocation: boolean | null;
  user_invocable: boolean | null;
  allowed_tools: string[] | null;
  model: string | null;
  effort: string | null;
  context: string | null;
  agent: string | null;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  path: string;
  original_path: string;
  scope: SkillScope;
  enabled: boolean;
  tags: string[];
  category: string;
  author: string;
  source_url: string;
  content: string;
  frontmatter: SkillFrontmatter;
  created_at: string;
  updated_at: string;
}

// ─── Project Types ──────────────────────────────────────────────

export interface Project {
  id: string;
  name: string;
  path: string;
  skill_count: number;
  added_at: string;
}

// ─── App State ──────────────────────────────────────────────────

export interface AppState {
  total_skills: number;
  enabled_skills: number;
  disabled_skills: number;
  projects: number;
}

// ─── View Types ─────────────────────────────────────────────────

export type ActiveView =
  | "dashboard"
  | "skills"
  | "projects"
  | "community"
  | "settings"
  | "skill-detail";

export type SkillFilter = "all" | "enabled" | "disabled" | "global" | "project";

// ─── Community Types ────────────────────────────────────────────

export interface CommunityRepo {
  id: string;
  name: string;
  url: string;
  description: string;
  skills_path: string;
  author: string;
}

export interface CommunitySkill {
  id: string;
  name: string;
  description: string;
  category: string;
  repo_id: string;
  repo_name: string;
  author: string;
  source_url: string;
  download_url: string;
  dir_name: string;
  tags: string[];
  installed: boolean;
}

export interface CommunitySyncResult {
  total_repos: number;
  total_skills: number;
  skills: CommunitySkill[];
  errors: string[];
}

export interface CustomRepo {
  id: string;
  name: string;
  owner: string;
  repo: string;
  skills_path: string;
  description: string;
  added_at: string;
}
