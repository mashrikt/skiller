import { useState, useEffect } from "react";
import type { CommunitySkill, CustomRepo } from "../types";
import * as api from "../api";

interface RepoInfo {
  id: string;
  name: string;
  author: string;
  desc: string;
  count: string;
  url: string;
  skillsPath: string;
}

const REPOS: RepoInfo[] = [
  { id: "anthropics-skills", name: "Anthropic Official", author: "anthropics", desc: "Official skills from Anthropic", count: "17+", url: "https://api.github.com/repos/anthropics/skills/contents/skills", skillsPath: "skills" },
  { id: "affaan-everything", name: "Everything Claude Code", author: "affaan-m", desc: "119 skills, 28 agents", count: "119+", url: "https://api.github.com/repos/affaan-m/everything-claude-code/contents/skills", skillsPath: "skills" },
  { id: "obra-superpowers", name: "Obra Superpowers", author: "obra", desc: "TDD, debugging, planning", count: "14+", url: "https://api.github.com/repos/obra/superpowers/contents/skills", skillsPath: "skills" },
  { id: "vercel-agent-skills", name: "Vercel Agent Skills", author: "vercel", desc: "React/Next.js best practices", count: "6+", url: "https://api.github.com/repos/vercel-labs/agent-skills/contents/skills", skillsPath: "skills" },
  { id: "kdense-scientific", name: "K-Dense Scientific", author: "K-Dense-AI", desc: "Genomics, chemistry, physics", count: "178+", url: "https://api.github.com/repos/K-Dense-AI/claude-scientific-skills/contents/scientific-skills", skillsPath: "scientific-skills" },
  { id: "qdhenry-command-suite", name: "Claude Command Suite", author: "qdhenry", desc: "Security, architecture, dev skills", count: "12+", url: "https://api.github.com/repos/qdhenry/Claude-Command-Suite/contents/.claude/skills", skillsPath: ".claude/skills" },
  { id: "levnikolaevich-skills", name: "Lev Nikolaevich", author: "levnikolaevich", desc: "129 skills: agile, auditor, MCP", count: "129+", url: "https://api.github.com/repos/levnikolaevich/claude-code-skills/contents/skills-catalog", skillsPath: "skills-catalog" },
  { id: "mrgoonie-claudekit", name: "ClaudeKit Skills", author: "mrgoonie", desc: "Shopify, payments, AI, backend", count: "32+", url: "https://api.github.com/repos/mrgoonie/claudekit-skills/contents/.claude/skills", skillsPath: ".claude/skills" },
  { id: "glebis-skills", name: "Glebis Skills", author: "glebis", desc: "Deep research, health, humanizer", count: "37+", url: "https://api.github.com/repos/glebis/claude-skills/contents", skillsPath: "" },
  { id: "gentleman-skills", name: "Gentleman Programming", author: "Gentleman-Programming", desc: "Angular, React 19, Django, AI SDK", count: "15+", url: "https://api.github.com/repos/Gentleman-Programming/Gentleman-Skills/contents/curated", skillsPath: "curated" },
  { id: "ahmedasmar-devops", name: "DevOps Skills", author: "ahmedasmar", desc: "K8s, Terraform, CI/CD, AWS", count: "7+", url: "https://api.github.com/repos/ahmedasmar/devops-claude-skills/contents", skillsPath: "" },
  { id: "vincenthopf-skills", name: "Vincent Hopf Skills", author: "vincenthopf", desc: "Browser agent, deep research", count: "8+", url: "https://api.github.com/repos/vincenthopf/My-Claude-Code/contents/skills", skillsPath: "skills" },
  { id: "conorluddy-ios", name: "iOS Simulator", author: "conorluddy", desc: "iOS app testing automation", count: "1+", url: "https://api.github.com/repos/conorluddy/ios-simulator-skill/contents", skillsPath: "" },
  { id: "lackeyjb-playwright", name: "Playwright Skill", author: "lackeyjb", desc: "Browser automation & testing", count: "1+", url: "https://api.github.com/repos/lackeyjb/playwright-skill/contents/skills", skillsPath: "skills" },
];

interface CommunityBrowserProps {
  onInstalled: () => void;
}

export default function CommunityBrowser({ onInstalled }: CommunityBrowserProps) {
  // Repo detail view
  const [selectedRepo, setSelectedRepo] = useState<RepoInfo | null>(null);
  const [repoSkills, setRepoSkills] = useState<CommunitySkill[]>([]);
  const [loadingRepo, setLoadingRepo] = useState(false);
  const [repoError, setRepoError] = useState<string | null>(null);

  // Skill preview
  const [previewSkill, setPreviewSkill] = useState<CommunitySkill | null>(null);
  const [previewContent, setPreviewContent] = useState<string | null>(null);
  const [loadingPreview, setLoadingPreview] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  // Install
  const [installingId, setInstallingId] = useState<string | null>(null);

  // Search across loaded skills
  const [searchQuery, setSearchQuery] = useState("");

  // Add repo form
  const [showAddRepo, setShowAddRepo] = useState(false);
  const [repoOwner, setRepoOwner] = useState("");
  const [repoName, setRepoName] = useState("");
  const [repoPath, setRepoPath] = useState("skills");
  const [customRepos, setCustomRepos] = useState<CustomRepo[]>([]);
  const [addingRepo, setAddingRepo] = useState(false);

  useEffect(() => {
    api.getCustomRepos().then(setCustomRepos);
  }, []);

  const handleClickRepo = async (repo: RepoInfo) => {
    setSelectedRepo(repo);
    setRepoSkills([]);
    setRepoError(null);
    setLoadingRepo(true);
    setSearchQuery("");
    try {
      const skills = await api.fetchSingleRepo(repo.url, repo.id, repo.name, repo.author, repo.skillsPath);
      setRepoSkills(skills);
    } catch (e) {
      setRepoError(String(e));
    } finally {
      setLoadingRepo(false);
    }
  };

  const handleClickSkill = async (skill: CommunitySkill) => {
    setPreviewSkill(skill);
    setPreviewContent(null);
    setPreviewError(null);
    setLoadingPreview(true);
    try {
      const content = await api.fetchSkillContent(skill.download_url);
      setPreviewContent(content);
    } catch (e) {
      setPreviewError(String(e));
    } finally {
      setLoadingPreview(false);
    }
  };

  const handleInstall = async (skill: CommunitySkill) => {
    setInstallingId(skill.id);
    try {
      await api.installCommunitySkill(skill);
      setRepoSkills((prev) => prev.map((s) => s.id === skill.id ? { ...s, installed: true } : s));
      if (previewSkill?.id === skill.id) {
        setPreviewSkill({ ...skill, installed: true });
      }
      onInstalled();
    } catch (e) {
      console.error("Install failed:", e);
    } finally {
      setInstallingId(null);
    }
  };

  const handleAddRepo = async () => {
    if (!repoOwner.trim() || !repoName.trim()) return;
    setAddingRepo(true);
    try {
      const repo = await api.addCustomRepo(repoOwner.trim(), repoName.trim(), repoPath.trim());
      setCustomRepos((prev) => [...prev, repo]);
      setRepoOwner("");
      setRepoName("");
      setRepoPath("skills");
      setShowAddRepo(false);
    } catch (e) {
      console.error("Failed to add repo:", e);
    } finally {
      setAddingRepo(false);
    }
  };

  const handleRemoveRepo = async (id: string) => {
    try {
      await api.removeCustomRepo(id);
      setCustomRepos((prev) => prev.filter((r) => r.id !== id));
    } catch (e) {
      console.error("Failed to remove repo:", e);
    }
  };

  const filteredSkills = searchQuery
    ? repoSkills.filter((s) =>
        s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        s.category.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : repoSkills;

  // ─── SKILL PREVIEW VIEW ─────────────────────────────────────
  if (previewSkill) {
    return (
      <div className="fade-in space-y-5 max-w-3xl">
        {/* Back */}
        <button
          onClick={() => { setPreviewSkill(null); setPreviewContent(null); setPreviewError(null); }}
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
            <h1 className="text-2xl font-bold text-text-primary tracking-tight">{previewSkill.name}</h1>
            <p className="text-sm text-text-secondary mt-1">
              by <span className="text-accent">{previewSkill.author}</span> &middot; {previewSkill.repo_name}
            </p>
          </div>
          {previewSkill.installed ? (
            <span className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-success-muted text-success text-sm font-medium">
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
              Installed
            </span>
          ) : (
            <button
              onClick={() => handleInstall(previewSkill)}
              disabled={installingId === previewSkill.id}
              className="flex items-center gap-2 px-5 py-2 rounded-lg bg-accent text-white text-sm font-medium
                         hover:bg-accent-hover transition-colors disabled:opacity-50 flex-shrink-0"
            >
              {installingId === previewSkill.id ? (
                <>
                  <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  Installing...
                </>
              ) : (
                <>
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                  </svg>
                  Install Skill
                </>
              )}
            </button>
          )}
        </div>

        {/* Metadata */}
        <div className="flex items-center gap-2 flex-wrap">
          <span className="pill bg-accent/15 text-accent">{previewSkill.category}</span>
          {previewSkill.tags.map((tag) => (
            <span key={tag} className="pill bg-surface-3/80 text-text-muted">{tag}</span>
          ))}
        </div>

        {/* Source link */}
        {previewSkill.source_url && (
          <p className="text-xs text-text-muted">
            Source: <span className="text-accent font-mono">{previewSkill.source_url}</span>
          </p>
        )}

        {/* Loading */}
        {loadingPreview && (
          <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-12 text-center">
            <div className="w-6 h-6 border-2 border-accent/30 border-t-accent rounded-full animate-spin mx-auto mb-3" />
            <p className="text-sm text-text-secondary">Fetching SKILL.md...</p>
          </div>
        )}

        {/* Error */}
        {previewError && (
          <div className="bg-danger-muted border border-danger/30 text-danger text-sm rounded-xl px-4 py-3">
            {previewError}
          </div>
        )}

        {/* Content */}
        {previewContent && (
          <div>
            <h3 className="text-sm font-semibold text-text-primary mb-2">SKILL.md</h3>
            <pre className="bg-surface-2 border border-surface-3/60 rounded-xl p-4 text-xs text-text-secondary font-mono
                            overflow-x-auto max-h-[60vh] overflow-y-auto whitespace-pre-wrap leading-relaxed">
              {previewContent}
            </pre>
          </div>
        )}
      </div>
    );
  }

  // ─── REPO SKILL LIST VIEW ─────────────────────────────────────
  if (selectedRepo) {
    return (
      <div className="fade-in space-y-5">
        {/* Back + header */}
        <button
          onClick={() => { setSelectedRepo(null); setRepoSkills([]); setRepoError(null); }}
          className="flex items-center gap-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
          </svg>
          Back to repositories
        </button>

        <div>
          <h1 className="text-2xl font-bold text-text-primary tracking-tight">{selectedRepo.name}</h1>
          <p className="text-sm text-text-secondary mt-1">
            by <span className="text-accent">{selectedRepo.author}</span> · {selectedRepo.desc}
          </p>
        </div>

        {/* Loading */}
        {loadingRepo && (
          <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-12 text-center">
            <div className="w-8 h-8 border-2 border-accent/30 border-t-accent rounded-full animate-spin mx-auto mb-3" />
            <p className="text-sm text-text-secondary">Fetching skills from {selectedRepo.author}/{selectedRepo.name}...</p>
          </div>
        )}

        {/* Error */}
        {repoError && (
          <div className="bg-danger-muted border border-danger/30 text-danger text-sm rounded-xl px-4 py-3">
            Failed to fetch: {repoError}
          </div>
        )}

        {/* Skills */}
        {!loadingRepo && !repoError && (
          <>
            <div className="flex items-center gap-4">
              <span className="text-sm text-text-secondary">
                <span className="text-accent font-semibold">{repoSkills.length}</span> skills found
              </span>
              {repoSkills.length > 5 && (
                <div className="relative flex-1 max-w-xs">
                  <svg className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Filter skills..."
                    aria-label="Filter skills"
                    className="w-full bg-surface-2 border border-surface-3/60 rounded-lg pl-10 pr-3 py-1.5 text-sm text-text-primary
                               placeholder:text-text-muted focus:outline-none focus:border-accent/50 transition-colors"
                  />
                </div>
              )}
            </div>

            {filteredSkills.length === 0 && repoSkills.length === 0 ? (
              <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-8 text-center">
                <p className="text-sm text-text-muted">No skill directories found in this repo</p>
              </div>
            ) : filteredSkills.length === 0 && repoSkills.length > 0 ? (
              <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-8 text-center">
                <p className="text-sm text-text-muted">No skills match your filter</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
                {filteredSkills.map((skill) => (
                  <div
                    key={skill.id}
                    className="bg-surface-2 border border-surface-3/60 rounded-xl p-4 flex flex-col card-hover cursor-pointer group"
                    role="button"
                    tabIndex={0}
                    onClick={() => handleClickSkill(skill)}
                    onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") handleClickSkill(skill); }}
                  >
                    <div className="mb-2">
                      <h3 className="text-sm font-semibold text-text-primary group-hover:text-accent transition-colors">{skill.name}</h3>
                      <span className="text-xs text-text-muted">{skill.category}</span>
                    </div>
                    <p className="text-xs text-text-secondary line-clamp-2 mb-3 flex-1">
                      {skill.description}
                    </p>
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-text-muted opacity-0 group-hover:opacity-100 transition-opacity">
                        Click to preview
                      </span>
                      {skill.installed ? (
                        <span className="flex items-center gap-1 text-xs text-success font-medium">
                          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                          </svg>
                          Installed
                        </span>
                      ) : (
                        <button
                          onClick={(e) => { e.stopPropagation(); handleInstall(skill); }}
                          disabled={installingId === skill.id}
                          className="px-3 py-1 rounded-lg bg-accent/15 text-accent text-xs font-medium
                                     hover:bg-accent/25 transition-colors disabled:opacity-50"
                        >
                          {installingId === skill.id ? "Installing..." : "Install"}
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </div>
    );
  }

  // ─── REPO LIST VIEW ─────────────────────────────────────────
  return (
    <div className="fade-in space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-text-primary tracking-tight">Community Skills</h1>
        <p className="text-sm text-text-secondary mt-1">
          Click a repository to browse and install its skills
        </p>
      </div>

      {/* Add Repo */}
      <div className="space-y-3">
        <button
          onClick={() => setShowAddRepo(!showAddRepo)}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-surface-2 border border-surface-3/60 text-text-secondary text-xs font-medium
                     hover:text-text-primary hover:border-accent/40 transition-colors"
        >
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          Add Repository
        </button>

        {showAddRepo && (
          <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-4 space-y-3 slide-in-right">
            <p className="text-xs text-text-muted">Add a GitHub repo (e.g. github.com/<b>owner</b>/<b>repo</b> with skills in <b>/skills</b>)</p>
            <div className="flex items-center gap-2">
              <input type="text" value={repoOwner} onChange={(e) => setRepoOwner(e.target.value)} placeholder="owner" aria-label="Repository owner"
                className="flex-1 bg-surface-3/50 border border-surface-4 rounded-lg px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent/50 transition-colors" />
              <span className="text-text-muted">/</span>
              <input type="text" value={repoName} onChange={(e) => setRepoName(e.target.value)} placeholder="repo" aria-label="Repository name"
                className="flex-1 bg-surface-3/50 border border-surface-4 rounded-lg px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent/50 transition-colors" />
              <span className="text-text-muted">/</span>
              <input type="text" value={repoPath} onChange={(e) => setRepoPath(e.target.value)} placeholder="skills" aria-label="Skills path"
                className="w-24 bg-surface-3/50 border border-surface-4 rounded-lg px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent/50 transition-colors" />
              <button onClick={handleAddRepo} disabled={!repoOwner.trim() || !repoName.trim() || addingRepo}
                className="px-4 py-2 rounded-lg bg-accent text-white text-sm font-medium hover:bg-accent-hover transition-colors disabled:opacity-50">
                {addingRepo ? "Adding..." : "Add"}
              </button>
            </div>
          </div>
        )}

        {customRepos.length > 0 && (
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-xs text-text-muted">Your repos:</span>
            {customRepos.map((cr) => (
              <span key={cr.id} className="inline-flex items-center gap-1.5 pill bg-accent-subtle text-accent">
                {cr.owner}/{cr.repo}
                <button onClick={() => handleRemoveRepo(cr.id)} aria-label={`Remove repository ${cr.owner}/${cr.repo}`} className="text-accent/60 hover:text-danger transition-colors">
                  <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Repo grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {REPOS.map((repo) => (
          <button
            key={repo.id}
            onClick={() => handleClickRepo(repo)}
            className="bg-surface-2 border border-surface-3/60 rounded-xl p-3 flex items-center gap-3 text-left
                       hover:border-accent/40 hover:bg-surface-3/30 cursor-pointer transition-all group"
          >
            <div className="w-8 h-8 rounded-lg bg-accent-subtle flex items-center justify-center flex-shrink-0 group-hover:bg-accent/20 transition-colors">
              <svg className="w-4 h-4 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
            </div>
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium text-text-primary truncate group-hover:text-accent transition-colors">{repo.name}</p>
              <p className="text-xs text-text-muted truncate">{repo.desc}</p>
            </div>
            <span className="text-xs text-accent bg-accent-subtle px-2 py-0.5 rounded-full flex-shrink-0 font-medium">{repo.count}</span>
            <svg className="w-4 h-4 text-text-muted group-hover:text-accent transition-colors flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        ))}

        {/* Custom repos as clickable cards too */}
        {customRepos.map((cr) => (
          <button
            key={cr.id}
            onClick={() => handleClickRepo({
              id: `custom-${cr.id}`,
              name: `${cr.owner}/${cr.repo}`,
              author: cr.owner,
              desc: cr.description,
              count: "?",
              url: `https://api.github.com/repos/${cr.owner}/${cr.repo}/contents/${cr.skills_path}`,
              skillsPath: cr.skills_path,
            })}
            className="bg-surface-2 border border-accent/30 rounded-xl p-3 flex items-center gap-3 text-left
                       hover:border-accent/50 hover:bg-surface-3/30 cursor-pointer transition-all group"
          >
            <div className="w-8 h-8 rounded-lg bg-accent/20 flex items-center justify-center flex-shrink-0">
              <svg className="w-4 h-4 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
              </svg>
            </div>
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium text-text-primary truncate group-hover:text-accent transition-colors">{cr.owner}/{cr.repo}</p>
              <p className="text-xs text-text-muted truncate">Custom repository · /{cr.skills_path}</p>
            </div>
            <svg className="w-4 h-4 text-text-muted group-hover:text-accent transition-colors flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        ))}
      </div>
    </div>
  );
}
