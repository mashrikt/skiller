import { useState, useEffect, useRef } from "react";
import type { Skill } from "../types";

interface SkillDetailProps {
  skillId: string;
  onBack: () => void;
  onToggle: (skill: Skill) => void;
  onDelete: (id: string) => Promise<void>;
  onAddTag: (skillId: string, tag: string) => void;
  onRemoveTag: (skillId: string, tag: string) => void;
  getSkillDetails: (id: string) => Promise<Skill | null>;
}

function scopeText(scope: Skill["scope"]): string {
  if (scope.type === "Global") return "Global";
  if (scope.type === "Bundled") return "Bundled";
  if (scope.type === "Project") return `Project: ${scope.project_path.split("/").pop()}`;
  return "Unknown";
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

export default function SkillDetail({
  skillId,
  onBack,
  onToggle,
  onDelete,
  onAddTag,
  onRemoveTag,
  getSkillDetails,
}: SkillDetailProps) {
  const [skill, setSkill] = useState<Skill | null>(null);
  const [loading, setLoading] = useState(true);
  const [tagInput, setTagInput] = useState("");
  const [confirmDelete, setConfirmDelete] = useState(false);
  const deleteTimerRef = useRef<number>();

  useEffect(() => () => clearTimeout(deleteTimerRef.current), []);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    getSkillDetails(skillId).then((s) => {
      if (!cancelled) {
        setSkill(s);
        setLoading(false);
      }
    });
    return () => { cancelled = true; };
  }, [skillId, getSkillDetails]);

  const handleAddTag = () => {
    const tag = tagInput.trim();
    if (tag && skill) {
      onAddTag(skill.id, tag);
      setSkill({ ...skill, tags: [...skill.tags, tag] });
      setTagInput("");
    }
  };

  const handleRemoveTag = (tag: string) => {
    if (skill) {
      onRemoveTag(skill.id, tag);
      setSkill({ ...skill, tags: skill.tags.filter((t) => t !== tag) });
    }
  };

  const handleDelete = async () => {
    if (confirmDelete && skill) {
      await onDelete(skill.id);
      onBack();
    } else {
      setConfirmDelete(true);
      deleteTimerRef.current = window.setTimeout(() => setConfirmDelete(false), 3000);
    }
  };

  if (loading) {
    return (
      <div className="fade-in flex items-center justify-center h-64">
        <div className="w-6 h-6 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
      </div>
    );
  }

  if (!skill) {
    return (
      <div className="fade-in text-center py-16">
        <p className="text-text-muted">Skill not found</p>
        <button onClick={onBack} className="mt-4 text-sm text-accent hover:text-accent-hover transition-colors">
          Go back
        </button>
      </div>
    );
  }

  const frontmatterEntries = Object.entries(skill.frontmatter).filter(
    ([, v]) => v !== null && v !== undefined && v !== "" && !(Array.isArray(v) && v.length === 0),
  );

  return (
    <div className="slide-in-right space-y-6 max-w-3xl">
      {/* Back */}
      <button
        onClick={onBack}
        className="flex items-center gap-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
      >
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
        </svg>
        Back to skills
      </button>

      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-text-primary tracking-tight">{skill.name}</h1>
          <p className="text-sm text-text-secondary mt-1.5 leading-relaxed">{skill.description || "No description"}</p>
        </div>
        <label className="toggle-switch flex-shrink-0 mt-1" title={skill.enabled ? "Disable" : "Enable"}>
          <input
            type="checkbox"
            checked={skill.enabled}
            aria-label={skill.enabled ? "Disable skill" : "Enable skill"}
            onChange={() => {
              onToggle(skill);
              setSkill({ ...skill, enabled: !skill.enabled });
            }}
          />
          <span className="toggle-slider" />
        </label>
      </div>

      {/* Metadata */}
      <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-4 grid grid-cols-2 gap-y-3 gap-x-6 text-sm">
        <div>
          <span className="text-text-muted text-xs">Scope</span>
          <p className="text-text-primary font-medium mt-0.5">{scopeText(skill.scope)}</p>
        </div>
        <div>
          <span className="text-text-muted text-xs">Category</span>
          <p className="text-text-primary font-medium mt-0.5">{skill.category || "Uncategorized"}</p>
        </div>
        <div>
          <span className="text-text-muted text-xs">Author</span>
          <p className="text-text-primary font-medium mt-0.5">{skill.author || "Unknown"}</p>
        </div>
        <div>
          <span className="text-text-muted text-xs">Source</span>
          {skill.source_url ? (
            <p className="text-accent text-xs mt-0.5 truncate">{skill.source_url}</p>
          ) : (
            <p className="text-text-muted mt-0.5">-</p>
          )}
        </div>
        <div>
          <span className="text-text-muted text-xs">Created</span>
          <p className="text-text-secondary mt-0.5">{formatDate(skill.created_at)}</p>
        </div>
        <div>
          <span className="text-text-muted text-xs">Updated</span>
          <p className="text-text-secondary mt-0.5">{formatDate(skill.updated_at)}</p>
        </div>
      </div>

      {/* Tags */}
      <div>
        <h3 className="text-sm font-semibold text-text-primary mb-2">Tags</h3>
        <div className="flex items-center flex-wrap gap-2">
          {skill.tags.map((tag) => (
            <span
              key={tag}
              className="inline-flex items-center gap-1 pill bg-surface-3/80 text-text-secondary"
            >
              {tag}
              <button
                onClick={() => handleRemoveTag(tag)}
                aria-label={`Remove tag ${tag}`}
                className="ml-0.5 text-text-muted hover:text-danger transition-colors"
              >
                <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </span>
          ))}
          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleAddTag();
            }}
            className="flex items-center gap-1.5"
          >
            <input
              type="text"
              value={tagInput}
              onChange={(e) => setTagInput(e.target.value)}
              placeholder="Add tag..."
              className="w-24 bg-surface-3/50 border border-surface-4 rounded px-2 py-1 text-xs text-text-primary
                         placeholder:text-text-muted focus:outline-none focus:border-accent/50 transition-colors"
            />
            <button
              type="submit"
              disabled={!tagInput.trim()}
              className="text-xs text-accent hover:text-accent-hover disabled:text-text-muted transition-colors"
            >
              Add
            </button>
          </form>
        </div>
      </div>

      {/* Frontmatter */}
      {frontmatterEntries.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-text-primary mb-2">Frontmatter</h3>
          <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-4 space-y-2">
            {frontmatterEntries.map(([k, v]) => (
              <div key={k} className="flex items-baseline gap-3 text-xs">
                <span className="text-text-muted font-mono w-28 flex-shrink-0">{k}</span>
                <span className="text-text-secondary break-all">
                  {Array.isArray(v) ? v.join(", ") : typeof v === "boolean" ? (v ? "true" : "false") : String(v)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Content */}
      <div>
        <h3 className="text-sm font-semibold text-text-primary mb-2">SKILL.md Content</h3>
        <pre className="code-block max-h-96 overflow-y-auto">{skill.content || "No content"}</pre>
      </div>

      {/* Path */}
      <div>
        <h3 className="text-sm font-semibold text-text-primary mb-2">File Path</h3>
        <p className="text-xs text-text-muted font-mono bg-surface-2 border border-surface-3/60 rounded-lg px-3 py-2 break-all">
          {skill.path}
        </p>
      </div>

      {/* Danger zone */}
      <div className="pt-2 border-t border-surface-3/40">
        <button
          onClick={handleDelete}
          aria-label="Delete skill"
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors
            ${
              confirmDelete
                ? "bg-danger text-white"
                : "bg-danger-muted text-danger hover:bg-danger/20"
            }`}
        >
          {confirmDelete ? "Click again to confirm" : "Delete Skill"}
        </button>
      </div>
    </div>
  );
}
