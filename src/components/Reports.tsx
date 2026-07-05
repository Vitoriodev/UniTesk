import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
  PieChart, Pie, Cell,
} from "recharts";

interface MonthlyCount {
  month: string;
  count: number;
}

interface MonthlyAmount {
  month: string;
  amount: number;
}

interface ProjectHours {
  project_name: string;
  hours: number;
}

interface ReportStats {
  totalProjects: number;
  totalArticles: number;
  totalClients: number;
  totalTeams: number;
  totalUsers: number;
  assignmentsByMonth: MonthlyCount[];
  assignmentsPending: number;
  assignmentsDone: number;
  assignmentsOverdue: number;
  revenueByMonth: MonthlyAmount[];
  totalRevenue: number;
  pendingAmount: number;
  hoursByProject: ProjectHours[];
  totalHours: number;
  invoicesDraft: number;
  invoicesSent: number;
  invoicesPaid: number;
  invoicesOverdue: number;
  invoicesCancelled: number;
}

const PIE_COLORS = ["#4f46e5", "#22c55e", "#ef4444", "#f59e0b", "#0891b2"];
const MONTH_NAMES: Record<string, string> = {
  "01": "Jan", "02": "Fev", "03": "Mar", "04": "Abr",
  "05": "Mai", "06": "Jun", "07": "Jul", "08": "Ago",
  "09": "Set", "10": "Out", "11": "Nov", "12": "Dez",
};

function formatMonth(ym: string): string {
  const parts = ym.split("-");
  if (parts.length === 2) {
    return `${MONTH_NAMES[parts[1]] || parts[1]}/${parts[0].slice(2)}`;
  }
  return ym;
}

function formatCurrency(value: number): string {
  return `R$ ${value.toFixed(2).replace(".", ",")}`;
}

function Reports() {
  const [stats, setStats] = useState<ReportStats | null>(null);
  const [activeChart, setActiveChart] = useState("all");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const data = await invoke<ReportStats>("get_report_stats");
        setStats(data);
      } catch {
        // fallback
      }
      setLoading(false);
    }
    load();
  }, []);

  if (loading) {
    return (
      <div style={{ textAlign: "center", padding: "60px 0", color: "var(--text-secondary)" }}>
        <p style={{ fontSize: "2rem", marginBottom: 8 }}>⏳</p>
        <p>Carregando relatórios...</p>
      </div>
    );
  }

  if (!stats) {
    return (
      <div style={{ textAlign: "center", padding: "60px 0", color: "var(--text-secondary)" }}>
        <p style={{ fontSize: "2rem", marginBottom: 8 }}>📊</p>
        <p>Não foi possível carregar os relatórios.</p>
      </div>
    );
  }

  const assignmentStatusData = [
    { name: "Pendentes", value: stats.assignmentsPending },
    { name: "Concluídas", value: stats.assignmentsDone },
    { name: "Atrasadas", value: stats.assignmentsOverdue },
  ].filter((d) => d.value > 0);

  const invoiceStatusData = [
    { name: "Rascunho", value: stats.invoicesDraft },
    { name: "Enviadas", value: stats.invoicesSent },
    { name: "Pagas", value: stats.invoicesPaid },
    { name: "Vencidas", value: stats.invoicesOverdue },
    { name: "Canceladas", value: stats.invoicesCancelled },
  ].filter((d) => d.value > 0);

  const hoursData = stats.hoursByProject.map((h) => ({
    name: h.project_name.length > 20 ? h.project_name.slice(0, 20) + "…" : h.project_name,
    hours: h.hours,
  }));

  const revenueChartData = stats.revenueByMonth.map((r) => ({
    month: formatMonth(r.month),
    receita: r.amount,
  }));

  const assignmentsChartData = stats.assignmentsByMonth.map((a) => ({
    month: formatMonth(a.month),
    atividades: a.count,
  }));

  const charts = [
    { id: "all", label: "Todos" },
    { id: "activities", label: "📅 Atividades" },
    { id: "finance", label: "💰 Financeiro" },
    { id: "hours", label: "⏱️ Horas" },
    { id: "invoices", label: "📋 Faturas" },
  ];

  return (
    <div>
      <h2 style={{ marginBottom: 20, fontSize: "1.4rem" }}>📊 Relatórios</h2>

      {/* Stats Cards */}
      <div className="grid-4" style={{ marginBottom: 24 }}>
        <div className="card report-stat-card">
          <div className="report-stat-icon">📁</div>
          <div>
            <p className="report-stat-label">Projetos</p>
            <p className="report-stat-value">{stats.totalProjects}</p>
          </div>
        </div>
        <div className="card report-stat-card">
          <div className="report-stat-icon">📄</div>
          <div>
            <p className="report-stat-label">Documentos</p>
            <p className="report-stat-value">{stats.totalArticles}</p>
          </div>
        </div>
        <div className="card report-stat-card">
          <div className="report-stat-icon">💰</div>
          <div>
            <p className="report-stat-label">Receita Total</p>
            <p className="report-stat-value">{formatCurrency(stats.totalRevenue)}</p>
          </div>
        </div>
        <div className="card report-stat-card">
          <div className="report-stat-icon">⏱️</div>
          <div>
            <p className="report-stat-label">Horas Registradas</p>
            <p className="report-stat-value">{stats.totalHours.toFixed(1)}h</p>
          </div>
        </div>
      </div>

      {/* Chart Filter Tabs */}
      <div className="filter-tabs" style={{ marginBottom: 20 }}>
        {charts.map((c) => (
          <button
            key={c.id}
            className={`filter-tab ${activeChart === c.id ? "filter-tab--active" : ""}`}
            onClick={() => setActiveChart(c.id)}
          >
            {c.label}
          </button>
        ))}
      </div>

      <div className="reports-grid">
        {/* Atividades por Mês */}
        {(activeChart === "all" || activeChart === "activities") && (
          <div className="card chart-card">
            <h3 className="card-title" style={{ marginBottom: 16 }}>
              📅 Atividades por Mês
            </h3>
            {assignmentsChartData.length > 0 ? (
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={assignmentsChartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis dataKey="month" fontSize={12} tick={{ fill: "var(--text-secondary)" }} />
                  <YAxis fontSize={12} tick={{ fill: "var(--text-secondary)" }} allowDecimals={false} />
                  <Tooltip
                    contentStyle={{
                      background: "var(--bg-card)",
                      border: "1px solid var(--border)",
                      borderRadius: "8px",
                      fontSize: "0.85rem",
                    }}
                  />
                  <Bar dataKey="atividades" fill="var(--primary)" radius={[6, 6, 0, 0]} name="Atividades" />
                </BarChart>
              </ResponsiveContainer>
            ) : (
              <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: 40 }}>
                Nenhuma atividade registrada nos últimos meses.
              </p>
            )}
          </div>
        )}

        {/* Status das Atividades */}
        {(activeChart === "all" || activeChart === "activities") && (
          <div className="card chart-card">
            <h3 className="card-title" style={{ marginBottom: 16 }}>
              🎯 Status das Atividades
            </h3>
            {assignmentStatusData.length > 0 ? (
              <ResponsiveContainer width="100%" height={280}>
                <PieChart>
                  <Pie
                    data={assignmentStatusData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={100}
                    paddingAngle={4}
                    dataKey="value"
                    label={({ name, value }) => `${name}: ${value}`}
                    labelLine={false}
                  >
                    {assignmentStatusData.map((_, idx) => (
                      <Cell key={idx} fill={PIE_COLORS[idx % PIE_COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      background: "var(--bg-card)",
                      border: "1px solid var(--border)",
                      borderRadius: "8px",
                      fontSize: "0.85rem",
                    }}
                  />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: 40 }}>
                Nenhuma atividade registrada.
              </p>
            )}
          </div>
        )}

        {/* Receita por Mês */}
        {(activeChart === "all" || activeChart === "finance") && (
          <div className="card chart-card">
            <h3 className="card-title" style={{ marginBottom: 16 }}>
              💰 Receita por Mês
            </h3>
            {revenueChartData.length > 0 ? (
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={revenueChartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis dataKey="month" fontSize={12} tick={{ fill: "var(--text-secondary)" }} />
                  <YAxis
                    fontSize={12}
                    tick={{ fill: "var(--text-secondary)" }}
                    tickFormatter={(v: number) => `R$${v}`}
                  />
                  <Tooltip
                    formatter={(value: any) => [formatCurrency(Number(value ?? 0)), "Receita"]}
                    contentStyle={{
                      background: "var(--bg-card)",
                      border: "1px solid var(--border)",
                      borderRadius: "8px",
                      fontSize: "0.85rem",
                    }}
                  />
                  <Bar dataKey="receita" fill="#22c55e" radius={[6, 6, 0, 0]} name="Receita" />
                </BarChart>
              </ResponsiveContainer>
            ) : (
              <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: 40 }}>
                Nenhuma receita registrada nos últimos meses.
              </p>
            )}
          </div>
        )}

        {/* Horas por Projeto */}
        {(activeChart === "all" || activeChart === "hours") && (
          <div className="card chart-card">
            <h3 className="card-title" style={{ marginBottom: 16 }}>
              ⏱️ Horas por Projeto
            </h3>
            {hoursData.length > 0 ? (
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={hoursData} layout="vertical">
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis type="number" fontSize={12} tick={{ fill: "var(--text-secondary)" }} />
                  <YAxis
                    dataKey="name"
                    type="category"
                    fontSize={11}
                    tick={{ fill: "var(--text-secondary)" }}
                    width={120}
                  />
                  <Tooltip
                    formatter={(value: any) => [`${Number(value ?? 0).toFixed(1)}h`, "Horas"]}
                    contentStyle={{
                      background: "var(--bg-card)",
                      border: "1px solid var(--border)",
                      borderRadius: "8px",
                      fontSize: "0.85rem",
                    }}
                  />
                  <Bar dataKey="hours" fill="#0891b2" radius={[0, 6, 6, 0]} name="Horas" />
                </BarChart>
              </ResponsiveContainer>
            ) : (
              <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: 40 }}>
                Nenhum registro de horas encontrado.
              </p>
            )}
          </div>
        )}

        {/* Status das Faturas */}
        {(activeChart === "all" || activeChart === "invoices") && (
          <div className="card chart-card">
            <h3 className="card-title" style={{ marginBottom: 16 }}>
              📋 Status das Faturas
            </h3>
            {invoiceStatusData.length > 0 ? (
              <ResponsiveContainer width="100%" height={280}>
                <PieChart>
                  <Pie
                    data={invoiceStatusData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={100}
                    paddingAngle={4}
                    dataKey="value"
                    label={({ name, value }) => `${name}: ${value}`}
                    labelLine={false}
                  >
                    {invoiceStatusData.map((_, idx) => (
                      <Cell key={idx} fill={PIE_COLORS[idx % PIE_COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      background: "var(--bg-card)",
                      border: "1px solid var(--border)",
                      borderRadius: "8px",
                      fontSize: "0.85rem",
                    }}
                  />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: 40 }}>
                Nenhuma fatura registrada.
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default Reports;
