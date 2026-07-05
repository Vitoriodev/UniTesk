import { useState, useEffect } from "react";
import Dashboard from "./components/Dashboard";
import ProjectList from "./components/ProjectList";
import CalendarView from "./components/CalendarView";
import ArticleManager from "./components/ArticleManager";
import ClientList from "./components/ClientList";
import TeamList from "./components/TeamList";
import TimeTracking from "./components/TimeTracking";
import Finance from "./components/Finance";
import NotificationPanel from "./components/NotificationPanel";
import Reports from "./components/Reports";

type Tab = "dashboard" | "projects" | "calendar" | "articles" | "clients" | "teams" | "hours" | "finance" | "reports";

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
    { id: "clients", label: "Clientes", icon: "🤝" },
    { id: "finance", label: "Financeiro", icon: "💰" },
    { id: "hours", label: "Horas", icon: "⏱️" },
    { id: "calendar", label: "Atividades", icon: "📅" },
    { id: "articles", label: "Documentos", icon: "📄" },
    { id: "teams", label: "Equipes", icon: "👥" },
    { id: "reports", label: "Relatórios", icon: "📊" },
  ];

  const renderContent = () => {
    switch (activeTab) {
      case "dashboard":
        return <Dashboard onNavigate={setActiveTab} />;
      case "projects":
        return <ProjectList />;
      case "clients":
        return <ClientList />;
      case "finance":
        return <Finance />;
      case "hours":
        return <TimeTracking />;
      case "calendar":
        return <CalendarView />;
      case "articles":
        return <ArticleManager />;
      case "teams":
        return <TeamList />;
      case "reports":
        return <Reports />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="app-container">
      <header className="app-header">
        <div className="app-header-actions">
          <NotificationPanel />
          <button
            className="theme-toggle-btn"
            onClick={toggleTheme}
            title={theme === "dracula" ? "Tema Claro" : "Tema Dracula"}
          >
            {theme === "dracula" ? "☀️" : "🌙"}
          </button>
        </div>
        <h1>
          <span className="logo-icon">🚀</span> Unitesk
        </h1>
        <p className="app-subtitle">
          Gerencie seus projetos com eficiência
        </p>
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
        <p>Unitesk v2.0 — Mantenha tudo organizado 🚀</p>
      </footer>
    </div>
  );
}

export default App;
