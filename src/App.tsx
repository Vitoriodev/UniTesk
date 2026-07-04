import { useState, useEffect } from "react";
import Dashboard from "./components/Dashboard";
import ProjectList from "./components/ProjectList";
import CalendarView from "./components/CalendarView";
import ArticleManager from "./components/ArticleManager";

type Tab = "dashboard" | "projects" | "calendar" | "articles";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("dashboard");
  const [theme, setTheme] = useState(() => localStorage.getItem("unitesk_theme") || "light");

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("unitesk_theme", theme);
  }, [theme]);

  function toggleTheme() {
    setTheme((t) => (t === "dracula" ? "light" : "dracula"));
  }

  const tabs: { id: Tab; label: string; icon: string }[] = [
    { id: "dashboard", label: "Dashboard", icon: "📊" },
    { id: "projects", label: "Projetos", icon: "📁" },
    { id: "calendar", label: "Calendário", icon: "📅" },
    { id: "articles", label: "Artigos", icon: "📄" },
  ];

  const renderContent = () => {
    switch (activeTab) {
      case "dashboard":
        return <Dashboard onNavigate={setActiveTab} />;
      case "projects":
        return <ProjectList />;
      case "calendar":
        return <CalendarView />;
      case "articles":
        return <ArticleManager />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="app-container">
      <header className="app-header">
        <div style={{ position: "absolute", right: 24, top: 24 }}>
          <button
            className="btn btn-sm"
            onClick={toggleTheme}
            title={theme === "dracula" ? "Tema Claro" : "Tema Dracula"}
            style={{
              background: "rgba(255,255,255,0.15)",
              border: "1px solid rgba(255,255,255,0.3)",
              borderRadius: "var(--radius-sm)",
              padding: "6px 10px",
              cursor: "pointer",
              fontSize: "1rem",
              color: "white",
            }}
          >
            {theme === "dracula" ? "☀️" : "🌙"}
          </button>
        </div>
        <h1>
          <span className="logo-icon">🎓</span> Unitesk
        </h1>
        <p className="app-subtitle">Gerenciador de Projetos Acadêmicos</p>
      </header>

      <nav className="app-nav">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`nav-btn ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            <span className="nav-icon">{tab.icon}</span>
            <span className="nav-label">{tab.label}</span>
          </button>
        ))}
      </nav>

      <main className="app-main">{renderContent()}</main>

      <footer className="app-footer">
        <p>Unitesk v1.1 — Mantenha seus projetos acadêmicos organizados</p>
      </footer>
    </div>
  );
}

export default App;
