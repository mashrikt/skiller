import { useState, useEffect, useRef } from "react";
import type { Project } from "../types";

interface ProjectManagerProps {
  projects: Project[];
  onAddProject: (path: string) => void;
  onRemoveProject: (id: string) => void;
  onSelectProject: (project: Project) => void;
}

export default function ProjectManager({
  projects,
  onAddProject,
  onRemoveProject,
  onSelectProject,
}: ProjectManagerProps) {
  const [pathInput, setPathInput] = useState("");
  const [confirmRemoveId, setConfirmRemoveId] = useState<string | null>(null);
  const removeTimerRef = useRef<number>();

  useEffect(() => () => clearTimeout(removeTimerRef.current), []);

  const handleAdd = () => {
    const path = pathInput.trim();
    if (path) {
      onAddProject(path);
      setPathInput("");
    }
  };

  const handleRemove = (id: string) => {
    if (confirmRemoveId === id) {
      onRemoveProject(id);
      setConfirmRemoveId(null);
    } else {
      setConfirmRemoveId(id);
      removeTimerRef.current = window.setTimeout(() => setConfirmRemoveId(null), 3000);
    }
  };

  return (
    <div className="fade-in space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-text-primary tracking-tight">Projects</h1>
        <p className="text-sm text-text-secondary mt-1">Manage your registered project directories</p>
      </div>

      {/* Add project */}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          handleAdd();
        }}
        className="flex gap-3"
      >
        <input
          type="text"
          value={pathInput}
          onChange={(e) => setPathInput(e.target.value)}
          placeholder="/path/to/your/project"
          aria-label="Project path"
          className="flex-1 bg-surface-2 border border-surface-3 rounded-lg px-4 py-2.5 text-sm font-mono
                     text-text-primary placeholder:text-text-muted
                     focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/30 transition-all duration-200"
        />
        <button
          type="submit"
          disabled={!pathInput.trim()}
          className="px-5 py-2.5 rounded-lg bg-accent text-white text-sm font-medium
                     hover:bg-accent-hover transition-colors disabled:opacity-40 disabled:cursor-not-allowed flex-shrink-0"
        >
          Add Project
        </button>
      </form>

      {/* Project list */}
      {projects.length === 0 ? (
        <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-12 text-center">
          <svg
            className="w-10 h-10 text-text-muted mx-auto mb-3"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          <p className="text-sm text-text-muted">No projects registered</p>
          <p className="text-xs text-text-muted mt-1">Add a project path to get started</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {projects.map((project) => (
            <div
              key={project.id}
              className="bg-surface-2 border border-surface-3/60 rounded-xl p-4 card-hover cursor-pointer"
              onClick={() => onSelectProject(project)}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onSelectProject(project); }}
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <svg className="w-4 h-4 text-warning flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    <h3 className="text-sm font-semibold text-text-primary truncate">{project.name}</h3>
                  </div>
                  <p className="text-xs text-text-muted font-mono truncate">{project.path}</p>
                  <p className="text-xs text-text-secondary mt-1.5">
                    {project.skill_count} skill{project.skill_count !== 1 ? "s" : ""}
                  </p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleRemove(project.id);
                  }}
                  className={`flex-shrink-0 p-1.5 rounded-md transition-colors text-xs font-medium
                    ${
                      confirmRemoveId === project.id
                        ? "bg-danger text-white"
                        : "text-text-muted hover:text-danger hover:bg-danger-muted"
                    }`}
                  title={confirmRemoveId === project.id ? "Click to confirm" : "Remove project"}
                >
                  {confirmRemoveId === project.id ? (
                    <span className="px-1">Confirm?</span>
                  ) : (
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  )}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
