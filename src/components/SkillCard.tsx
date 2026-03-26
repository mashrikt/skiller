import type { Skill } from "../types";

interface SkillCardProps {
  skill: Skill;
  onToggle: (skill: Skill) => void;
  onClick: (skill: Skill) => void;
}

function scopeLabel(scope: Skill["scope"]): { text: string; classes: string } {
  if (scope.type === "Global") return { text: "Global", classes: "bg-accent/15 text-accent" };
  if (scope.type === "Bundled") return { text: "Bundled", classes: "bg-info-muted text-info" };
  if (scope.type === "Project") {
    const name = scope.project_path.split("/").pop() || "Project";
    return { text: name, classes: "bg-success-muted text-success" };
  }
  return { text: "Unknown", classes: "bg-surface-3 text-text-muted" };
}

export default function SkillCard({ skill, onToggle, onClick }: SkillCardProps) {
  const badge = scopeLabel(skill.scope);

  return (
    <div
      onClick={() => onClick(skill)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onClick(skill); }}
      className="group bg-surface-2 border border-surface-3/60 rounded-xl p-4 cursor-pointer card-hover relative"
    >
      {/* Header row */}
      <div className="flex items-start justify-between gap-3 mb-2">
        <div className="min-w-0 flex-1">
          <h3 className="text-sm font-semibold text-text-primary truncate leading-snug">
            {skill.name}
          </h3>
        </div>
        {/* Toggle */}
        <label
          className="toggle-switch"
          onClick={(e) => e.stopPropagation()}
        >
          <input
            type="checkbox"
            checked={skill.enabled}
            aria-label={skill.enabled ? "Disable skill" : "Enable skill"}
            onChange={() => onToggle(skill)}
          />
          <span className="toggle-slider" />
        </label>
      </div>

      {/* Description */}
      <p className="text-xs text-text-secondary line-clamp-2 mb-3 leading-relaxed">
        {skill.description || "No description"}
      </p>

      {/* Footer */}
      <div className="flex items-center gap-2 flex-wrap">
        <span className={`pill ${badge.classes}`}>{badge.text}</span>
        {skill.tags.slice(0, 3).map((tag) => (
          <span
            key={tag}
            className="pill bg-surface-3/80 text-text-muted"
          >
            {tag}
          </span>
        ))}
        {skill.tags.length > 3 && (
          <span className="pill bg-surface-3/80 text-text-muted">
            +{skill.tags.length - 3}
          </span>
        )}
      </div>
    </div>
  );
}
