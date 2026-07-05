import { useState, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DashboardStats {
  totalProjects: number;
  totalArticles: number;
  totalClients: number;
  totalTeams: number;
  totalUsers: number;
  pendingAssignments: number;
  overdueAssignments: number;
  nextDeadline: string | null;
  nextDeadlineName: string | null;
  hoursToday: number;
  hoursWeek: number;
  totalRevenue: number;
  pendingInvoices: number;
  pendingAmount: number;
}

interface Assignment {
  id: number;
  title: string;
  description: string;
  due_date: string;
  due_time: string | null;
  notification_time: string | null;
  project_name: string;
  status: string;
  created_at: string;
}

type Tab = "dashboard" | "projects" | "calendar" | "articles" | "clients" | "teams" | "hours" | "finance";

// Contador animado — anima de 0 até o valor final em ~400ms
function AnimatedCounter({ value, duration = 400 }: { value: number; duration?: number }) {
  const [displayValue, setDisplayValue] = useState(0);

  useEffect(() => {
    // Proteção contra valores inválidos (NaN, undefined, negativo)
    const safeValue = typeof value === 'number' && isFinite(value) && value > 0 ? value : 0;
    if (safeValue === 0) {
      setDisplayValue(0);
      return;
    }
    const steps = Math.min(15, Math.max(1, safeValue));
    const increment = safeValue / steps;
    let current = 0;
    let step = 0;

    const interval = setInterval(() => {
      step++;
      current = Math.round(increment * step);
      setDisplayValue(Math.min(current, safeValue));
      if (step >= steps) {
        setDisplayValue(safeValue);
        clearInterval(interval);
      }
    }, duration / steps);

    return () => clearInterval(interval);
  }, [value, duration]);

  return <span className="counter-value">{displayValue}</span>;
}

function Dashboard({ onNavigate }: { onNavigate?: (tab: Tab) => void }) {
  const [stats, setStats] = useState<DashboardStats>({
    totalProjects: 0,
    totalArticles: 0,
    totalClients: 0,
    totalTeams: 0,
    totalUsers: 0,
    pendingAssignments: 0,
    overdueAssignments: 0,
    nextDeadline: null,
    nextDeadlineName: null,
    hoursToday: 0,
    hoursWeek: 0,
    totalRevenue: 0,
    pendingInvoices: 0,
    pendingAmount: 0,
  });
  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [importMessage, setImportMessage] = useState<string | null>(null);
  const [statsLoaded, setStatsLoaded] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    async function loadData() {
      try {
        const data = await invoke<DashboardStats>("get_dashboard_stats");
        setStats(data);
        setStatsLoaded(true);
      } catch {
        setStats({
          totalProjects: 0,
          totalArticles: 0,
          totalClients: 0,
          totalTeams: 0,
          totalUsers: 0,
          pendingAssignments: 0,
          overdueAssignments: 0,
          nextDeadline: null,
          nextDeadlineName: null,
          hoursToday: 0,
          hoursWeek: 0,
          totalRevenue: 0,
          pendingInvoices: 0,
          pendingAmount: 0,
        });
        setStatsLoaded(true);
      }

      // Load recent assignments for the timeline
      try {
        const data = await invoke<Assignment[]>("get_assignments");
        setAssignments(
          data
            .sort(
              (a: Assignment, b: Assignment) =>
                new Date(b.created_at || b.due_date).getTime() -
                new Date(a.created_at || a.due_date).getTime()
            )
            .slice(0, 5)
        );
      } catch {
        const saved = localStorage.getItem("unitesk_assignments");
        if (saved) {
          try {
            const parsed: Assignment[] = JSON.parse(saved);
            setAssignments(
              parsed
                .sort(
                  (a, b) =>
                    new Date(b.created_at || b.due_date).getTime() -
                    new Date(a.created_at || a.due_date).getTime()
                )
                .slice(0, 5)
            );
          } catch {
            // ignore
          }
        }
      }
    }
    loadData();
  }, []);

  async function handleExport() {
    setExporting(true);
    setImportMessage(null);
    try {
      const data = await invoke<any>("export_all_data");
      const jsonStr = JSON.stringify(data, null, 2);
      const blob = new Blob([jsonStr], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `unitesk_backup_${new Date().toISOString().split("T")[0]}.unitesk`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setImportMessage("✅ Dados exportados com sucesso!");
    } catch (err) {
      setImportMessage("❌ Erro ao exportar dados: " + String(err));
    }
    setExporting(false);
  }

  async function handleImport(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setImporting(true);
    setImportMessage(null);
    try {
      const text = await file.text();
      const data = JSON.parse(text);
      const result = await invoke<string>("import_all_data", { data });
      setImportMessage("✅ " + result);
      const statsData = await invoke<DashboardStats>("get_dashboard_stats");
      setStats(statsData);
    } catch (err) {
      setImportMessage("❌ Erro ao importar dados: " + String(err));
    }
    setImporting(false);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  }

  const completionRate = useMemo(() => {
    const total = stats.totalProjects + stats.overdueAssignments;
    if (total === 0) return 0;
    const doneProjects = stats.totalProjects - stats.overdueAssignments;
    return Math.round(Math.max(0, (doneProjects / total) * 100));
  }, [stats]);

  const statCards = [
    {
      title: "Projetos",
      value: stats.totalProjects,
      icon: "📁",
      cssClass: "stat-card--projects",
      color: "var(--stat-projects)",
    },
    {
      title: "Documentos",
      value: stats.totalArticles,
      icon: "📄",
      cssClass: "stat-card--articles",
      color: "var(--stat-articles)",
    },
    {
      title: "Clientes",
      value: stats.totalClients,
      icon: "🤝",
      cssClass: "stat-card--clients",
      color: "var(--stat-clients)",
    },
    {
      title: "Equipes",
      value: stats.totalTeams,
      icon: "👥",
      cssClass: "stat-card--teams",
      color: "var(--stat-teams)",
    },
    {
      title: "Usuários",
      value: stats.totalUsers,
      icon: "👤",
      cssClass: "stat-card--users",
      color: "var(--stat-users)",
    },
    {
      title: "Pendentes",
      value: stats.pendingAssignments,
      icon: "⏳",
      cssClass: "stat-card--pending",
      color: "var(--stat-pending)",
    },
    {
      title: "Atrasados",
      value: stats.overdueAssignments,
      icon: "🔴",
      cssClass: "stat-card--overdue",
      color: "var(--stat-overdue)",
    },
  ];

  const getAssignmentIcon = (status: string) => {
    switch (status) {
      case "done": return "✅";
      case "overdue": return "🔴";
      case "pending": return "⏳";
      default: return "📅";
    }
  };

  const getStatusClass = (status: string) => {
    switch (status) {
      case "done": return "timeline-item--done";
      case "overdue": return "timeline-item--overdue";
      case "pending": return "timeline-item--pending";
      default: return "";
    }
  };

  return (
    <div>
      <h2 style={{ marginBottom: 20, fontSize: "1.4rem" }}>📊 Dashboard</h2>

      {/* Welcome Card */}
      <div className="welcome-card" style={{ animation: "slideUp 0.3s ease" }}>
        <h2>Olá! 👋</h2>
        <p>
          {statsLoaded
            ? `Bem-vindo ao Unitesk. Você tem ${stats.pendingAssignments + stats.overdueAssignments} atividades pendentes e ${stats.totalProjects} projetos em andamento.`
            : "Carregando suas informações..."}
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid-4" style={{ marginBottom: 8 }}>
        {statCards.map((card, idx) => (
          <div
            className={`card stat-card ${card.cssClass}`}
            key={card.title}
            style={{
              animation: `slideUp 0.3s ease ${idx * 0.1}s both`,
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 16,
              }}
            >
              <div className="stat-icon-wrapper">
                <span style={{ fontSize: "1.5rem" }}>{card.icon}</span>
              </div>
              <div>
                <p style={{ color: "var(--text-secondary)", fontSize: "0.8rem", fontWeight: 500 }}>
                  {card.title}
                </p>
                <p className="stat-value">
                  {statsLoaded ? (
                    <AnimatedCounter value={card.value} duration={600 + idx * 100} />
                  ) : (
                    "—"
                  )}
                </p>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="grid-2">
        {/* Progress & Next Deadline */}
        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
          <div className="card">
            <div className="card-header">
              <h3 className="card-title">📊 Progresso Geral</h3>
            </div>
            <div style={{ marginBottom: 16 }}>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
                <span style={{ fontSize: "0.85rem", color: "var(--text-secondary)" }}>
                  Atividades Concluídas
                </span>
                <span style={{ fontSize: "0.85rem", fontWeight: 600 }}>
                  {stats.totalProjects > 0 || stats.pendingAssignments > 0
                    ? `${completionRate}%`
                    : "—"}
                </span>
              </div>
              <div className="progress-bar-container">
                <div
                  className={`progress-bar-fill ${
                    completionRate >= 70
                      ? "progress-bar-fill--success"
                      : completionRate >= 40
                      ? "progress-bar-fill--primary"
                      : completionRate > 0
                      ? "progress-bar-fill--warning"
                      : ""
                  }`}
                  style={{ width: stats.totalProjects > 0 ? `${completionRate}%` : "0%" }}
                />
              </div>
            </div>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 12,
              }}
            >
              <div
                style={{
                  background: "var(--bg)",
                  borderRadius: "var(--radius-sm)",
                  padding: "12px 16px",
                  textAlign: "center",
                }}
              >
                <p style={{ fontSize: "1.3rem", fontWeight: 700, color: "var(--success)" }}>
                  {statsLoaded ? stats.totalProjects - stats.overdueAssignments : "—"}
                </p>
                <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)" }}>
                  Em dia
                </p>
              </div>
              <div
                style={{
                  background: "var(--bg)",
                  borderRadius: "var(--radius-sm)",
                  padding: "12px 16px",
                  textAlign: "center",
                }}
              >
                <p style={{ fontSize: "1.3rem", fontWeight: 700, color: "var(--danger)" }}>
                  {statsLoaded ? stats.overdueAssignments : "—"}
                </p>
                <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)" }}>
                  Atrasados
                </p>
              </div>
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <h3 className="card-title">📅 Próximo Prazo</h3>
            </div>
            {stats.nextDeadline ? (
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 16,
                  padding: "8px 0",
                }}
              >
                <div
                  style={{
                    width: 56,
                    height: 56,
                    borderRadius: 12,
                    background: "var(--primary-light)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: "1.8rem",
                  }}
                >
                  ⏰
                </div>
                <div>
                  <p style={{ fontWeight: 600, fontSize: "1rem" }}>{stats.nextDeadlineName}</p>
                  <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem" }}>
                    {new Date(stats.nextDeadline).toLocaleDateString("pt-BR", {
                      weekday: "long",
                      year: "numeric",
                      month: "long",
                      day: "numeric",
                    })}
                  </p>
                </div>
              </div>
            ) : (
              <div style={{ padding: "12px 0" }}>
                <p style={{ color: "var(--text-secondary)", textAlign: "center" }}>
                  Nenhum prazo próximo 🎉
                </p>
              </div>
            )}
          </div>
        </div>          {/* Horas do Dia */}
          <div className="card">
            <div className="card-header">
              <h3 className="card-title">⏱️ Horas Registradas</h3>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <div className="stat-card" style={{ padding: 16, textAlign: "center" }}>
                <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)", fontWeight: 500 }}>Hoje</p>
                <p style={{ fontSize: "1.8rem", fontWeight: 700, color: "var(--stat-projects)" }}>
                  {Math.floor(stats.hoursToday / 60)}h {stats.hoursToday % 60}m
                </p>
              </div>
              <div className="stat-card" style={{ padding: 16, textAlign: "center" }}>
                <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)", fontWeight: 500 }}>Esta Semana</p>
                <p style={{ fontSize: "1.8rem", fontWeight: 700, color: "var(--stat-clients)" }}>
                  {Math.floor(stats.hoursWeek / 60)}h {Math.round(stats.hoursWeek % 60)}m
                </p>
              </div>
            </div>
            <button
              className="btn btn-secondary btn-sm"
              style={{ marginTop: 12, width: "100%", justifyContent: "center" }}
              onClick={() => onNavigate?.("hours")}
            >
              ⏱️ Ver Controle de Horas →
            </button>
          </div>

          {/* Financeiro */}
          <div className="card">
            <div className="card-header">
              <h3 className="card-title">💰 Financeiro</h3>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 12 }}>
              <div style={{ padding: "12px 16px", background: "var(--bg)", borderRadius: "var(--radius-sm)", textAlign: "center" }}>
                <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)", fontWeight: 500 }}>Receita Total</p>
                <p style={{ fontSize: "1.5rem", fontWeight: 700, color: "var(--stat-clients)" }}>
                  R$ {stats.totalRevenue.toFixed(2).replace('.', ',')}
                </p>
              </div>
              <div style={{ padding: "12px 16px", background: "var(--bg)", borderRadius: "var(--radius-sm)", textAlign: "center" }}>
                <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)", fontWeight: 500 }}>A Receber</p>
                <p style={{ fontSize: "1.5rem", fontWeight: 700, color: "var(--stat-pending)" }}>
                  R$ {stats.pendingAmount.toFixed(2).replace('.', ',')}
                </p>
              </div>
            </div>
            <button
              className="btn btn-secondary btn-sm"
              style={{ width: "100%", justifyContent: "center" }}
              onClick={() => onNavigate?.("finance")}
            >
              💰 Ver Financeiro →
            </button>
          </div>

          {/* Quick Actions & Timeline */}
        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
          <div className="card">
            <div className="card-header">
              <h3 className="card-title">⚡ Ações Rápidas</h3>
            </div>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 8,
              }}
            >
              <button
                className="btn btn-primary"
                onClick={() => onNavigate?.("projects")}
                style={{ justifyContent: "center", padding: "12px 16px" }}
              >
                ➕ Novo Projeto
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => onNavigate?.("articles")}
                style={{ justifyContent: "center", padding: "12px 16px" }}
              >
                📄 Novo Artigo
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => onNavigate?.("calendar")}
                style={{ justifyContent: "center", padding: "12px 16px" }}
              >
                📅 Nova Atividade
              </button>
              <button
                className="btn btn-outline-primary"
                onClick={handleExport}
                disabled={exporting}
                style={{ justifyContent: "center", padding: "12px 16px" }}
              >
                {exporting ? "⏳" : "📤 Exportar"}
              </button>
            </div>

            <hr style={{ margin: "16px 0", border: "none", borderTop: "1px solid var(--border)" }} />

            <button
              className="btn btn-secondary"
              onClick={() => fileInputRef.current?.click()}
              disabled={importing}
              style={{ width: "100%", justifyContent: "center", padding: "12px 16px" }}
            >
              {importing ? "⏳ Importando..." : "📥 Importar Dados"}
            </button>

            <input
              ref={fileInputRef}
              type="file"
              accept=".unitesk"
              className="hidden-input"
              onChange={handleImport}
            />

            {importMessage && (
              <p
                style={{
                  fontSize: "0.8rem",
                  color: importMessage.includes("❌") ? "var(--danger)" : "var(--success)",
                  marginTop: 8,
                  textAlign: "center",
                }}
              >
                {importMessage}
              </p>
            )}
          </div>

          {/* Recent Activity Timeline */}
          {assignments.length > 0 && (
            <div className="card">
              <div className="card-header">
                <h3 className="card-title">🕐 Atividades Recentes</h3>
              </div>
              <div className="timeline">
                {assignments.map((a) => (
                  <div key={a.id} className={`timeline-item ${getStatusClass(a.status)}`}>
                    <p className="timeline-item-title">
                      {getAssignmentIcon(a.status)} {a.title}
                    </p>
                    <p className="timeline-item-meta">
                      {a.project_name && `📁 ${a.project_name} • `}
                      {new Date(a.due_date).toLocaleDateString("pt-BR")}
                      {a.due_time && ` às ${a.due_time}`}
                    </p>
                  </div>
                ))}
              </div>
              <button
                className="btn btn-secondary btn-sm"
                style={{ marginTop: 12, width: "100%", justifyContent: "center" }}
                onClick={() => onNavigate?.("calendar")}
              >
                Ver todas as atividades →
              </button>
            </div>
          )}
        </div>
      </div>


    </div>
  );
}

export default Dashboard;
