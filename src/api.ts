import { invoke } from "@tauri-apps/api/core";
import type { Skill, Project, AppState, CommunityRepo, CommunitySkill, CommunitySyncResult, CustomRepo } from "./types";

// ─── Skills ─────────────────────────────────────────────────────

export async function getAllSkills(): Promise<Skill[]> {
  try {
    return await invoke<Skill[]>("cmd_get_all_skills");
  } catch (e) {
    console.error("Failed to get skills:", e);
    return [];
  }
}

export async function getSkillDetails(id: string): Promise<Skill | null> {
  try {
    return await invoke<Skill | null>("cmd_get_skill_details", { id });
  } catch (e) {
    console.error("Failed to get skill details:", e);
    return null;
  }
}

export async function enableSkill(id: string): Promise<Skill> {
  return invoke<Skill>("cmd_enable_skill", { id });
}

export async function disableSkill(id: string): Promise<Skill> {
  return invoke<Skill>("cmd_disable_skill", { id });
}

export async function deleteSkill(id: string): Promise<void> {
  return invoke<void>("cmd_delete_skill", { id });
}

export async function syncSkills(): Promise<Skill[]> {
  try {
    return await invoke<Skill[]>("cmd_sync_skills");
  } catch (e) {
    console.error("Failed to sync skills:", e);
    return [];
  }
}

export async function searchSkills(query: string): Promise<Skill[]> {
  try {
    return await invoke<Skill[]>("cmd_search_skills", { query });
  } catch (e) {
    console.error("Failed to search skills:", e);
    return [];
  }
}

// ─── Tags ───────────────────────────────────────────────────────

export async function addTag(skillId: string, tag: string): Promise<void> {
  return invoke<void>("cmd_add_tag", { skillId, tag });
}

export async function removeTag(skillId: string, tag: string): Promise<void> {
  return invoke<void>("cmd_remove_tag", { skillId, tag });
}

// ─── Projects ───────────────────────────────────────────────────

export async function getProjects(): Promise<Project[]> {
  try {
    return await invoke<Project[]>("cmd_get_projects");
  } catch (e) {
    console.error("Failed to get projects:", e);
    return [];
  }
}

export async function addProject(path: string): Promise<Project> {
  return invoke<Project>("cmd_add_project", { path });
}

export async function removeProject(id: string): Promise<void> {
  return invoke<void>("cmd_remove_project", { id });
}

// ─── App State ──────────────────────────────────────────────────

export async function getAppState(): Promise<AppState> {
  try {
    return await invoke<AppState>("cmd_get_app_state");
  } catch (e) {
    console.error("Failed to get app state:", e);
    return { total_skills: 0, enabled_skills: 0, disabled_skills: 0, projects: 0 };
  }
}

// ─── Community ─────────────────────────────────────────────────

export async function getCommunityRepos(): Promise<CommunityRepo[]> {
  try {
    return await invoke<CommunityRepo[]>("cmd_get_community_repos");
  } catch (e) {
    console.error("Failed to get community repos:", e);
    return [];
  }
}

export async function syncCommunity(): Promise<CommunitySyncResult> {
  try {
    return await invoke<CommunitySyncResult>("cmd_sync_community");
  } catch (e) {
    console.error("Failed to sync community:", e);
    return { total_repos: 0, total_skills: 0, skills: [], errors: [String(e)] };
  }
}

export async function fetchSingleRepo(repoUrl: string, repoId: string, repoName: string, author: string, skillsPath: string): Promise<CommunitySkill[]> {
  return invoke<CommunitySkill[]>("cmd_fetch_single_repo", { repoUrl, repoId, repoName, author, skillsPath });
}

export async function fetchSkillContent(url: string): Promise<string> {
  return invoke<string>("cmd_fetch_skill_content", { url });
}

export async function installCommunitySkill(skill: CommunitySkill): Promise<string> {
  return invoke<string>("cmd_install_community_skill", { skill });
}

export async function addCustomRepo(owner: string, repo: string, skillsPath: string): Promise<CustomRepo> {
  return invoke<CustomRepo>("cmd_add_custom_repo", { owner, repo, skillsPath });
}

export async function getCustomRepos(): Promise<CustomRepo[]> {
  try {
    return await invoke<CustomRepo[]>("cmd_get_custom_repos");
  } catch (e) {
    console.error("Failed to get custom repos:", e);
    return [];
  }
}

export async function removeCustomRepo(id: string): Promise<void> {
  return invoke<void>("cmd_remove_custom_repo", { id });
}

// ─── GitHub Token ──────────────────────────────────────────────

export async function getGithubToken(): Promise<string | null> {
  try {
    return await invoke<string | null>("cmd_get_github_token");
  } catch {
    return null;
  }
}

export async function setGithubToken(token: string): Promise<void> {
  return invoke<void>("cmd_set_github_token", { token });
}

export async function deleteGithubToken(): Promise<void> {
  return invoke<void>("cmd_delete_github_token");
}
