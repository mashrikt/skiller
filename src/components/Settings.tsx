import { useState, useEffect } from "react";
import * as api from "../api";

export default function Settings() {
  const [token, setToken] = useState("");
  const [hasToken, setHasToken] = useState(false);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    api.getGithubToken().then((t) => {
      if (t) {
        setHasToken(true);
        setToken("ghp_" + "•".repeat(32));
      }
    });
  }, []);

  const handleSave = async () => {
    if (!token.trim() || token.includes("•")) return;
    setSaving(true);
    setMessage(null);
    try {
      await api.setGithubToken(token.trim());
      setHasToken(true);
      setToken("ghp_" + "•".repeat(32));
      setMessage({ type: "success", text: "Token saved! You now have 5,000 requests/hour." });
    } catch (e) {
      setMessage({ type: "error", text: String(e) });
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    try {
      await api.deleteGithubToken();
      setHasToken(false);
      setToken("");
      setMessage({ type: "success", text: "Token removed. Back to 60 requests/hour." });
    } catch (e) {
      setMessage({ type: "error", text: String(e) });
    }
  };

  return (
    <div className="fade-in space-y-8 max-w-2xl">
      <div>
        <h1 className="text-2xl font-bold text-text-primary tracking-tight">Settings</h1>
        <p className="text-sm text-text-secondary mt-1">Configure Skiller</p>
      </div>

      {/* GitHub Token */}
      <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-5 space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-surface-3 flex items-center justify-center">
            <svg className="w-4 h-4 text-text-secondary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
            </svg>
          </div>
          <div>
            <h3 className="text-sm font-semibold text-text-primary">GitHub Token</h3>
            <p className="text-xs text-text-muted">
              {hasToken
                ? "Token configured — 5,000 requests/hour"
                : "No token — limited to 60 requests/hour"}
            </p>
          </div>
          {hasToken && (
            <span className="ml-auto pill bg-success-muted text-success">Active</span>
          )}
        </div>

        <div className="space-y-2">
          <p className="text-xs text-text-muted">
            A GitHub personal access token removes API rate limits when browsing community skills.
            No scopes needed — a classic token with zero permissions works.
          </p>
          <div className="flex items-center gap-2">
            <input
              type={hasToken ? "password" : "text"}
              value={token}
              onChange={(e) => { setToken(e.target.value); setMessage(null); }}
              onFocus={() => { if (hasToken) { setToken(""); } }}
              placeholder="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
              aria-label="GitHub personal access token"
              className="flex-1 bg-surface-3/50 border border-surface-4 rounded-lg px-3 py-2 text-sm text-text-primary font-mono
                         placeholder:text-text-muted focus:outline-none focus:border-accent/50 transition-colors"
            />
            <button
              onClick={handleSave}
              disabled={saving || !token.trim() || token.includes("•")}
              className="px-4 py-2 rounded-lg bg-accent text-white text-sm font-medium
                         hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              {saving ? "Saving..." : "Save"}
            </button>
            {hasToken && (
              <button
                onClick={handleDelete}
                className="px-3 py-2 rounded-lg bg-danger-muted text-danger text-sm font-medium
                           hover:bg-danger/20 transition-colors"
              >
                Remove
              </button>
            )}
          </div>
        </div>

        {message && (
          <p className={`text-xs font-medium ${message.type === "success" ? "text-success" : "text-danger"}`}>
            {message.text}
          </p>
        )}

        <div className="text-xs text-text-muted space-y-1 pt-1 border-t border-surface-3/40">
          <p className="font-medium text-text-secondary">How to create a token:</p>
          <p>1. Go to github.com → Settings → Developer settings → Personal access tokens → Tokens (classic)</p>
          <p>2. Click "Generate new token (classic)"</p>
          <p>3. Give it a name like "Skiller", set expiration, select <strong>no scopes</strong></p>
          <p>4. Copy the token and paste it above</p>
        </div>
      </div>

      {/* Storage info */}
      <div className="bg-surface-2 border border-surface-3/60 rounded-xl p-5 space-y-3">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-surface-3 flex items-center justify-center">
            <svg className="w-4 h-4 text-text-secondary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
            </svg>
          </div>
          <div>
            <h3 className="text-sm font-semibold text-text-primary">Storage</h3>
            <p className="text-xs text-text-muted">Where Skiller keeps its data</p>
          </div>
        </div>
        <div className="space-y-1.5 text-xs">
          <div className="flex items-center justify-between">
            <span className="text-text-muted">Database</span>
            <span className="text-text-secondary font-mono">~/.skiller/skiller.db</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-text-muted">Disabled skills vault</span>
            <span className="text-text-secondary font-mono">~/.skiller/vault/</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-text-muted">GitHub token</span>
            <span className="text-text-secondary font-mono">~/.skiller/github_token</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-text-muted">Active skills</span>
            <span className="text-text-secondary font-mono">~/.claude/skills/</span>
          </div>
        </div>
      </div>
    </div>
  );
}
