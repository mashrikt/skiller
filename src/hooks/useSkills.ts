import { useState, useCallback } from "react";
import type { Skill, Project, AppState } from "../types";
import * as api from "../api";

interface UseSkillsReturn {
  skills: Skill[];
  projects: Project[];
  appState: AppState;
  loading: boolean;
  syncing: boolean;
  error: string | null;

  loadAll: () => Promise<void>;
  syncSkills: () => Promise<void>;
  enableSkill: (id: string) => Promise<void>;
  disableSkill: (id: string) => Promise<void>;
  toggleSkill: (skill: Skill) => Promise<void>;
  deleteSkill: (id: string) => Promise<void>;
  addProject: (path: string) => Promise<void>;
  removeProject: (id: string) => Promise<void>;
  addTag: (skillId: string, tag: string) => Promise<void>;
  removeTag: (skillId: string, tag: string) => Promise<void>;
  searchSkills: (query: string) => Promise<Skill[]>;
  getSkillDetails: (id: string) => Promise<Skill | null>;
  clearError: () => void;
}

const defaultAppState: AppState = {
  total_skills: 0,
  enabled_skills: 0,
  disabled_skills: 0,
  projects: 0,
};

export function useSkills(): UseSkillsReturn {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [appState, setAppState] = useState<AppState>(defaultAppState);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [s, p, st] = await Promise.all([
      api.getAllSkills(),
      api.getProjects(),
      api.getAppState(),
    ]);
    setSkills(s);
    setProjects(p);
    setAppState(st);
  }, []);

  const loadAll = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [refresh]);

  const syncSkills = useCallback(async () => {
    setSyncing(true);
    setError(null);
    try {
      await api.syncSkills();
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setSyncing(false);
    }
  }, [refresh]);

  const enableSkill = useCallback(
    async (id: string) => {
      try {
        await api.enableSkill(id);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const disableSkill = useCallback(
    async (id: string) => {
      try {
        await api.disableSkill(id);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const toggleSkill = useCallback(
    async (skill: Skill) => {
      if (skill.enabled) {
        await disableSkill(skill.id);
      } else {
        await enableSkill(skill.id);
      }
    },
    [enableSkill, disableSkill],
  );

  const deleteSkillFn = useCallback(
    async (id: string) => {
      try {
        await api.deleteSkill(id);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const addProject = useCallback(
    async (path: string) => {
      try {
        await api.addProject(path);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const removeProject = useCallback(
    async (id: string) => {
      try {
        await api.removeProject(id);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const addTag = useCallback(
    async (skillId: string, tag: string) => {
      try {
        await api.addTag(skillId, tag);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const removeTag = useCallback(
    async (skillId: string, tag: string) => {
      try {
        await api.removeTag(skillId, tag);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    [refresh],
  );

  const searchSkillsFn = useCallback(async (query: string) => {
    try {
      return await api.searchSkills(query);
    } catch (e) {
      setError(String(e));
      return [];
    }
  }, []);

  const getSkillDetails = useCallback(async (id: string) => {
    try {
      return await api.getSkillDetails(id);
    } catch (e) {
      setError(String(e));
      return null;
    }
  }, []);

  const clearError = useCallback(() => setError(null), []);

  return {
    skills,
    projects,
    appState,
    loading,
    syncing,
    error,
    loadAll,
    syncSkills,
    enableSkill,
    disableSkill,
    toggleSkill,
    deleteSkill: deleteSkillFn,
    addProject,
    removeProject,
    addTag,
    removeTag,
    searchSkills: searchSkillsFn,
    getSkillDetails,
    clearError,
  };
}
