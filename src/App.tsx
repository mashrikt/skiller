import { useEffect, useState, useCallback } from "react";
import type { ActiveView, Skill } from "./types";
import { useSkills } from "./hooks/useSkills";
import Sidebar from "./components/Sidebar";
import Dashboard from "./components/Dashboard";
import SkillList from "./components/SkillList";
import SkillDetail from "./components/SkillDetail";
import ProjectManager from "./components/ProjectManager";
import CommunityBrowser from "./components/CommunityBrowser";
import Settings from "./components/Settings";

export default function App() {
  const store = useSkills();
  const { syncSkills, loadAll, clearError, error } = store;
  const [activeView, setActiveView] = useState<ActiveView>("dashboard");
  const [selectedSkillId, setSelectedSkillId] = useState<string | null>(null);

  // Initial load: sync then load everything
  useEffect(() => {
    (async () => {
      await syncSkills();
      await loadAll();
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [syncSkills, loadAll]);

  const navigate = useCallback((view: ActiveView) => {
    setActiveView(view);
    setSelectedSkillId(null);
  }, []);

  const openSkillDetail = useCallback((skill: Skill) => {
    setSelectedSkillId(skill.id);
    setActiveView("skill-detail");
  }, []);

  const goBackFromDetail = useCallback(() => {
    setActiveView("skills");
    setSelectedSkillId(null);
  }, []);

  const goToProjects = useCallback(() => {
    setActiveView("projects");
  }, []);

  const handleCommunityInstalled = useCallback(() => {
    syncSkills().then(() => loadAll());
  }, [syncSkills, loadAll]);

  // Auto-dismiss error toast after 5 seconds
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => clearError(), 5000);
      return () => clearTimeout(timer);
    }
  }, [error, clearError]);

  // Render main content
  const renderContent = () => {
    switch (activeView) {
      case "dashboard":
        return (
          <Dashboard
            appState={store.appState}
            skills={store.skills}
            syncing={store.syncing}
            onSync={store.syncSkills}
            onAddProject={goToProjects}
            onToggleSkill={store.toggleSkill}
            onSelectSkill={openSkillDetail}
          />
        );

      case "skills":
        return (
          <SkillList
            skills={store.skills}
            onToggle={store.toggleSkill}
            onClick={openSkillDetail}
            onSearch={store.searchSkills}
          />
        );

      case "skill-detail":
        return selectedSkillId ? (
          <SkillDetail
            skillId={selectedSkillId}
            onBack={goBackFromDetail}
            onToggle={store.toggleSkill}
            onDelete={store.deleteSkill}
            onAddTag={store.addTag}
            onRemoveTag={store.removeTag}
            getSkillDetails={store.getSkillDetails}
          />
        ) : (
          <SkillList
            skills={store.skills}
            onToggle={store.toggleSkill}
            onClick={openSkillDetail}
            onSearch={store.searchSkills}
          />
        );

      case "projects":
        return (
          <ProjectManager
            projects={store.projects}
            onAddProject={store.addProject}
            onRemoveProject={store.removeProject}
            onSelectProject={() => navigate("skills")}
          />
        );

      case "community":
        return (
          <CommunityBrowser
            onInstalled={handleCommunityInstalled}
          />
        );

      case "settings":
        return <Settings />;

      default:
        return null;
    }
  };

  return (
    <div className="h-screen flex bg-surface-0 text-text-primary font-sans overflow-hidden">
      <Sidebar
        activeView={activeView}
        onNavigate={navigate}
        appState={store.appState}
      />

      <main className="flex-1 overflow-y-auto">
        <div className="max-w-5xl mx-auto px-8 py-8">
          {/* Loading overlay */}
          {store.loading ? (
            <div className="flex flex-col items-center justify-center h-[60vh] gap-4">
              <div className="w-8 h-8 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
              <p className="text-sm text-text-muted">Loading skills...</p>
            </div>
          ) : (
            renderContent()
          )}

          {/* Error toast */}
          {store.error && (
            <div className="fixed bottom-6 right-6 bg-danger-muted border border-danger/30 text-danger text-sm rounded-lg px-4 py-3 shadow-lg max-w-sm fade-in">
              {store.error}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
