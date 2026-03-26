# Problems Users Face with Claude Code Skills

Research compiled from GitHub Issues, Reddit, Hacker News, blogs, and security research.

---

## 1. Skills Not Triggering / Unreliable Activation
- Claude decides skill invocation probabilistically based on description matching — no guarantee it fires even when perfectly written
- Claude sometimes reads SKILL.md files manually instead of invoking the skill properly
- Activation described as "extremely unstable" even with explicit references
- Generic descriptions cause low relevance scores, so skills never get routed to

## 2. Skills Not Discovered / Not Loading
- Skills in `.claude/skills/` sometimes don't appear in the Skill tool's list
- Global skills in `~/.claude/skills/` not loaded even after restarts
- Flat `.md` files not surfaced — only folder-based `SKILL.md` discovered
- No recursive skill discovery — subdirectory organization breaks discovery
- ENOENT error on managed skills directory aborts ALL skill loading
- Symlinked directories not detected by `/skills` command
- Zero skills discovered on Windows due to path handling bugs
- Wrong skill search paths on Windows (Cowork)

## 3. `/skills` Command Shows Nothing
- Debug logs confirm skills are loaded, but `/skills` displays "No skills found"

## 4. Stale Cache / Skills Not Updating
- Cached skills show old content instead of reading current SKILL.md files
- Plugin cache not invalidated when source files change
- Plugin uninstall leaves stale cache — reinstalls read from stale cache
- Marketplace updates not reflected — Claude uses stale cached version
- No `--force` flag for plugin install to clean cache
- `/clear` doesn't reset cached skills
- Server-side KV cache serves stale context

## 5. YAML Frontmatter Fragility
- Prettier reformats YAML frontmatter and breaks skill discovery
- Bare number values in frontmatter crash the system
- Name field in VS Code must match parent directory name exactly — mismatch causes silent failure

## 6. Cross-Platform / VS Code Silent Failures
- SKILL.md works in CLI but silently fails in VS Code with zero error messages
- VS Code extension crashes when Claude pushes extension host past ~2-3GB memory
- Hardcoded Linux paths make extension non-functional on Windows
- Claude generates Linux commands on Windows instead of platform-appropriate syntax

## 7. Skills vs Commands vs Plugins Confusion
- Users don't understand the distinction between skills, slash commands, subagents, and plugins
- Skills sometimes interpreted as SlashCommands, requesting wrong permission type
- No public deprecation timeline for `.claude/commands/` (source code labels it "commands_DEPRECATED")
- Custom slash commands from `.claude/commands/` no longer appear in autocomplete
- Documentation inconsistency about the commands-to-skills migration

## 8. No Easy Enable/Disable Per-Project
- No mechanism to enable/disable skills per-project without editing frontmatter
- Skills kept "just in case" bloat context — disabling is clunky
- No UI or dashboard to manage skill state

## 9. Context Bloat from Too Many Skills
- Most users install too many skills — testing showed 40 of 47 skills made output worse
- Skills consume token budget from context window (default ~15K char limit for descriptions)
- Adding MCP servers can lose 50K+ tokens to tool schemas before session begins
- No visibility into how much context budget skills are consuming

## 10. Skills Forgotten After Context Compaction
- Skills get compacted out of context during long sessions
- Specific details from earlier in session compressed away during summarization
- Requires manual re-invocation after compaction

## 11. Skills Ignored in Multi-Step Tasks
- User-created skills reportedly non-functional in complex multi-step workflows
- No visibility into which skills subagents (e.g., Plan agent) have access to

## 12. Security Vulnerabilities
- 36% of Agent Skills ecosystem has at least one security flaw (Snyk study)
- 13.4% contain critical issues: malware, prompt injection, exposed secrets
- 30+ malicious skills distributed via ClawHub targeting Claude Code users
- Prompt injection via hidden instructions embedded in skills
- Code injection vulnerability (CVE-2025-59536) allowing arbitrary shell commands

## 13. Zero Debugging / Observability
- No feedback when skills fail — completely silent failures
- No built-in skill validation or linting (community built third-party tools to fill gap)
- No visual differentiation for skills/agents/commands output

## 14. Confusing Documentation
- Relationship between skills, commands, subagents, and plugins poorly explained
- Trial-and-error learning required — docs present multiple approaches without clear defaults
- Best practices scattered across blogs, not in official docs

## 15. Cowork-Specific Bugs
- Skills pipeline regression making custom skills non-functional in Cowork
- Drag-and-drop skill upload gives "Internal server error" on Mac
- Symlinked skills fail validation in Cowork but execute correctly

## 16. Marketplace / Plugin Discovery Issues
- `/reload-plugins` does not load skills from newly installed marketplace plugins
- Skill tool only recognizes installed plugins, not local `~/.claude/skills/`
- Multiple official plugins still use deprecated commands format instead of skills

## 17. Vendor Lock-In / No Portability
- Claude Code doesn't follow its own Agent Skills open standard (`.agents/skills/` vs `.claude/skills/`)
- Claude-specific frontmatter fields that other agents ignore or choke on
- No AGENTS.md support despite 3,020+ upvotes requesting it

## 18. Monolithic Skill Architecture
- Everything crammed into one SKILL.md file leads to failures at scale
- No standard for separating concerns (process steps, context, rules, examples)
- No support for standard `.github/skills/` directory

---

## Problems Skillr (Our App) Can Address

| Problem | How Skillr Helps |
|---------|-----------------|
| No UI to manage skills | Dashboard with enable/disable toggles |
| No easy enable/disable | Move skills between active (`.claude/`) and disabled (`skillr/`) directories |
| Context bloat | Let users activate only what they need per-project |
| Skills not discovered | Validate skill structure before enabling |
| Stale cache | Force refresh when toggling skills |
| No debugging | Show skill metadata, validate YAML frontmatter |
| Security concerns | Scan/flag suspicious skills before enabling |
| Skills vs commands confusion | Unified view of all skill types |
| Per-project management | Support both global and project-level skill toggling |
| No observability | Show context budget impact of enabled skills |
