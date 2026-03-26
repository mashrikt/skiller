import { useMemo } from "react";
import type { Skill, AppState } from "../types";

interface DashboardProps {
  appState: AppState;
  skills: Skill[];
  syncing: boolean;
  onSync: () => void;
  onAddProject: () => void;
  onToggleSkill: (skill: Skill) => void;
  onSelectSkill: (skill: Skill) => void;
}

const statCards = (s: AppState) => [
  {
    label: "Total Skills",
    value: s.total_skills,
    color: "text-accent",
    bg: "bg-accent/10",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
  },
  {
    label: "Enabled",
    value: s.enabled_skills,
    color: "text-success",
    bg: "bg-success-muted",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
      </svg>
    ),
  },
  {
    label: "Disabled",
    value: s.disabled_skills,
    color: "text-text-muted",
    bg: "bg-surface-3/40",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636" />
      </svg>
    ),
  },
  {
    label: "Projects",
    value: s.projects,
    color: "text-warning",
    bg: "bg-warning-muted",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
    ),
  },
];

export default function Dashboard({
  appState,
  skills,
  syncing,
  onSync,
  onAddProject,
  onToggleSkill,
  onSelectSkill,
}: DashboardProps) {
  const cards = statCards(appState);
  const recentSkills = useMemo(
    () => [...skills].sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()).slice(0, 6),
    [skills],
  );

  return (
    <div className="fade-in space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-text-primary tracking-tight">Dashboard</h1>
        <p className="text-sm text-text-secondary mt-1">Overview of your Claude Code skills</p>
      </div>

      {/* Stat cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {cards.map((c) => (
          <div
            key={c.label}
            className="bg-surface-2 border border-surface-3/60 rounded-xl p-4"
          >
            <div className={`w-9 h-9 rounded-lg ${c.bg} flex items-center justify-center ${c.color} mb-3`}>
              {c.icon}
            </div>
            <p className={`text-2xl font-bold ${c.color}`}>{c.value}</p>
            <p className="text-xs text-text-muted mt-0.5">{c.label}</p>
          </div>
        ))}
      </div>

      {/* Quick actions */}
      <div>
        <h2 className="text-sm font-semibold text-text-primary mb-3">Quick Actions</h2>
        <div className="flex gap-3">
          <button
            onClick={onSync}
            disabled={syncing}
            className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-accent/10 text-accent text-sm font-medium
                       hover:bg-accent/20 transition-colors disabled:opacity-50"
          >
            <svg
              className={`w-4 h-4 ${syncing ? "animate-spin" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            {syncing ? "Syncing..." : "Sync Skills"}
          </button>
          <button
            onClick={onAddProject}
            className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-surface-3/60 text-text-secondary text-sm font-medium
                       hover:bg-surface-3 hover:text-text-primary transition-colors"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            Add Project
          </button>
        </div>
      </div>

      {/* Recent Skills */}
      <div>
        <h2 className="text-sm font-semibold text-text-primary mb-3">Recent Skills</h2>
        {recentSkills.length === 0 ? (
          <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-8 text-center">
            <p className="text-text-muted text-sm">No skills found. Try syncing first.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {recentSkills.map((skill) => (
              <div
                key={skill.id}
                onClick={() => onSelectSkill(skill)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onSelectSkill(skill); }}
                className="bg-surface-2 border border-surface-3/60 rounded-lg p-3.5 flex items-center justify-between gap-3
                           cursor-pointer card-hover"
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium text-text-primary truncate">{skill.name}</p>
                  <p className="text-xs text-text-muted truncate mt-0.5">
                    {skill.description || "No description"}
                  </p>
                </div>
                <label
                  className="toggle-switch flex-shrink-0"
                  onClick={(e) => e.stopPropagation()}
                >
                  <input
                    type="checkbox"
                    checked={skill.enabled}
                    aria-label={skill.enabled ? "Disable skill" : "Enable skill"}
                    onChange={() => onToggleSkill(skill)}
                  />
                  <span className="toggle-slider" />
                </label>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
