import { useState, useMemo, useCallback } from "react";
import type { Skill, SkillFilter } from "../types";
import SearchBar from "./SearchBar";
import SkillCard from "./SkillCard";

interface SkillListProps {
  skills: Skill[];
  onToggle: (skill: Skill) => void;
  onClick: (skill: Skill) => void;
  onSearch: (query: string) => Promise<Skill[]>;
}

const filters: { id: SkillFilter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "enabled", label: "Enabled" },
  { id: "disabled", label: "Disabled" },
  { id: "global", label: "Global" },
  { id: "project", label: "Project" },
];

export default function SkillList({ skills, onToggle, onClick, onSearch }: SkillListProps) {
  const [activeFilter, setActiveFilter] = useState<SkillFilter>("all");
  const [selectedProject, setSelectedProject] = useState<string>("all");
  const [searchResults, setSearchResults] = useState<Skill[] | null>(null);

  const handleSearch = useCallback(
    async (query: string) => {
      if (!query) {
        setSearchResults(null);
        return;
      }
      const results = await onSearch(query);
      setSearchResults(results);
    },
    [onSearch],
  );

  // Extract unique project paths from skills
  const projects = useMemo(() => {
    const paths = new Set<string>();
    for (const s of skills) {
      if (s.scope.type === "Project") {
        paths.add(s.scope.project_path);
      }
    }
    return Array.from(paths).sort();
  }, [skills]);

  const baseSkills = searchResults ?? skills;

  const filtered = useMemo(() => {
    let result = baseSkills;

    // Apply scope filter
    switch (activeFilter) {
      case "enabled":
        result = result.filter((s) => s.enabled);
        break;
      case "disabled":
        result = result.filter((s) => !s.enabled);
        break;
      case "global":
        result = result.filter((s) => s.scope.type === "Global");
        break;
      case "project":
        result = result.filter((s) => s.scope.type === "Project");
        break;
    }

    // Apply project dropdown filter
    if (selectedProject !== "all") {
      result = result.filter(
        (s) => s.scope.type === "Project" && s.scope.project_path === selectedProject
      );
    }

    return result;
  }, [baseSkills, activeFilter, selectedProject]);

  const handleFilterClick = (id: SkillFilter) => {
    setActiveFilter(id);
    // Reset project dropdown when switching away from project filter
    if (id !== "project") {
      setSelectedProject("all");
    }
  };

  return (
    <div className="fade-in space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-text-primary tracking-tight">All Skills</h1>
        <p className="text-sm text-text-secondary mt-1">Manage your Claude Code skills</p>
      </div>

      {/* Search + Filters */}
      <div className="space-y-3">
        <SearchBar onSearch={handleSearch} />
        <div className="flex items-center gap-2 flex-wrap">
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => handleFilterClick(f.id)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors
                ${
                  activeFilter === f.id
                    ? "bg-accent/15 text-accent"
                    : "bg-surface-2 text-text-muted hover:text-text-secondary hover:bg-surface-3/60"
                }`}
            >
              {f.label}
            </button>
          ))}

          {/* Project dropdown — shows when there are project skills */}
          {projects.length > 0 && (
            <select
              value={selectedProject}
              onChange={(e) => {
                setSelectedProject(e.target.value);
                if (e.target.value !== "all") {
                  setActiveFilter("project");
                }
              }}
              aria-label="Filter by project"
              className="px-2 py-1.5 rounded-lg text-xs font-medium bg-surface-2 border border-surface-3/60
                         text-text-secondary hover:text-text-primary focus:outline-none focus:border-accent/50
                         transition-colors cursor-pointer appearance-none
                         bg-[url('data:image/svg+xml;charset=UTF-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%2212%22%20height%3D%2212%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%20stroke%3D%22%236b7280%22%20stroke-width%3D%222%22%3E%3Cpath%20d%3D%22M6%209l6%206%206-6%22%2F%3E%3C%2Fsvg%3E')]
                         bg-no-repeat bg-[right_6px_center] pr-6"
            >
              <option value="all">All Projects</option>
              {projects.map((p) => (
                <option key={p} value={p}>
                  {p.split("/").pop()}
                </option>
              ))}
            </select>
          )}

          <span className="ml-auto text-xs text-text-muted">
            {filtered.length} skill{filtered.length !== 1 ? "s" : ""}
          </span>
        </div>
      </div>

      {/* Grid */}
      {filtered.length === 0 ? (
        <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-12 text-center">
          <svg
            className="w-10 h-10 text-text-muted mx-auto mb-3"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
            />
          </svg>
          <p className="text-sm text-text-muted">No skills match your criteria</p>
          <p className="text-xs text-text-muted mt-1">Try a different search or filter</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {filtered.map((skill) => (
            <SkillCard
              key={skill.id}
              skill={skill}
              onToggle={onToggle}
              onClick={onClick}
            />
          ))}
        </div>
      )}
    </div>
  );
}
