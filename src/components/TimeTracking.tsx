import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Project {
  id: number;
  name: string;
}

interface TimeEntry {
  id: number;
  project_id: number;
  project_name: string | null;
  user_id: number | null;
  user_name: string | null;
  description: string | null;
  start_time: string;
  end_time: string | null;
  duration_minutes: number | null;
  billable: boolean;
  hourly_rate: number | null;
  created_at: string;
}

function TimeTracking() {
  const [entries, setEntries] = useState<TimeEntry[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [activeEntry, setActiveEntry] = useState<TimeEntry | null>(null);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const [filterProject, setFilterProject] = useState<number | null>(null);
  const [filterDateFrom, setFilterDateFrom] = useState("");
  const [filterDateTo, setFilterDateTo] = useState("");
  const [showManualModal, setShowManualModal] = useState(false);
  const [hoursSummary, setHoursSummary] = useState({ today: 0, week: 0 });
  const [manualForm, setManualForm] = useState({
    project_id: null as number | null,
    description: "",
    duration_minutes: 60,
    entry_date: new Date().toISOString().split("T")[0],
    billable: true,
    hourly_rate: 0,
  });
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);
  const [startDesc, setStartDesc] = useState("");
  const [startProject, setStartProject] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    loadData();
    loadSummary();
    checkActiveEntry();
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, []);

  useEffect(() => {
    if (activeEntry) {
      const startTime = new Date(activeEntry.start_time).getTime();
      const updateElapsed = () => {
        setElapsedSeconds(Math.floor((Date.now() - startTime) / 1000));
      };
      updateElapsed();
      timerRef.current = setInterval(updateElapsed, 1000);
      return () => {
        if (timerRef.current) clearInterval(timerRef.current);
      };
    } else {
      setElapsedSeconds(0);
      if (timerRef.current) clearInterval(timerRef.current);
    }
  }, [activeEntry]);

  async function loadData() {
    try {
      const data = await invoke<TimeEntry[]>("get_time_entries", {
        projectId: filterProject,
        dateFrom: filterDateFrom || null,
        dateTo: filterDateTo || null,
      });
      setEntries(data);
    } catch {
      setEntries([]);
    }
    try {
      const data = await invoke<Project[]>("get_projects");
      setProjects(data);
    } catch {
      setProjects([]);
    }
  }

  async function loadSummary() {
    try {
      const [today, week] = await invoke<[number, number]>("get_hours_summary");
      setHoursSummary({ today, week });
    } catch {}
  }

  async function checkActiveEntry() {
    try {
      const active = await invoke<TimeEntry | null>("get_active_time_entry");
      setActiveEntry(active);
    } catch {}
  }

  async function handleStartTimer() {
    if (!startProject) return;
    try {
      const entry = await invoke<TimeEntry>("start_time_entry", {
        projectId: startProject,
        userId: null,
        description: startDesc || null,
      });
      setActiveEntry(entry);
      setStartProject(null);
      setStartDesc("");
    } catch {
      // Fallback local
      setActiveEntry({
        id: Date.now(),
        project_id: startProject,
        project_name: projects.find((p) => p.id === startProject)?.name || "",
        user_id: null,
        user_name: null,
        description: startDesc || null,
        start_time: new Date().toISOString(),
        end_time: null,
        duration_minutes: null,
        billable: true,
        hourly_rate: null,
        created_at: new Date().toISOString(),
      });
      setStartProject(null);
      setStartDesc("");
    }
  }

  async function handleStopTimer() {
    if (!activeEntry) return;
    try {
      await invoke<TimeEntry>("stop_time_entry", { id: activeEntry.id });
      setActiveEntry(null);
      loadData();
      loadSummary();
    } catch {
      setActiveEntry(null);
      loadData();
      loadSummary();
    }
  }

  async function handleManualSubmit() {
    if (!manualForm.project_id || manualForm.duration_minutes <= 0) return;
    try {
      await invoke("add_manual_time_entry", {
        projectId: manualForm.project_id,
        userId: null,
        description: manualForm.description || null,
        durationMinutes: manualForm.duration_minutes,
        entryDate: manualForm.entry_date,
        billable: manualForm.billable,
        hourlyRate: manualForm.hourly_rate > 0 ? manualForm.hourly_rate : null,
      });
      setShowManualModal(false);
      loadData();
      loadSummary();
    } catch (err) {
      alert("Erro ao adicionar horas: " + String(err));
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("delete_time_entry", { id });
      setDeleteConfirm(null);
      loadData();
      loadSummary();
    } catch {
      setEntries(entries.filter((e) => e.id !== id));
      setDeleteConfirm(null);
    }
  }

  function loadFiltered() {
    loadData();
    loadSummary();
  }

  function formatDuration(minutes: number): string {
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return `${h}h ${m}m`;
  }

  function formatElapsed(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }

  function formatTime(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" });
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString("pt-BR");
  }

  function getBillableValue(entry: TimeEntry): number {
    if (!entry.hourly_rate || !entry.duration_minutes) return 0;
    return (entry.hourly_rate * entry.duration_minutes) / 60;
  }

  return (
    <div>
      <div className="flex-between" style={{ marginBottom: 20 }}>
        <h2 className="section-title">⏱️ Controle de Horas</h2>
        <button className="btn btn-primary" onClick={() => setShowManualModal(true)}>
          ➕ Registrar Horas
        </button>
      </div>

      {/* Timer Ativo */}
      {activeEntry ? (
        <div
          className="card"
          style={{
            background: "linear-gradient(135deg, var(--primary), #7c3aed)",
            color: "white",
            textAlign: "center",
            padding: 32,
          }}
        >
          <p style={{ fontSize: "0.9rem", opacity: 0.85, marginBottom: 4 }}>
            ⏱️ Timer em andamento
          </p>
          <p style={{ fontSize: "3rem", fontWeight: 700, fontFamily: "monospace", letterSpacing: 4 }}>
            {formatElapsed(elapsedSeconds)}
          </p>
          <p style={{ fontSize: "1.1rem", marginTop: 8 }}>
            📁 {activeEntry.project_name || "Projeto"}
          </p>
          {activeEntry.description && (
            <p style={{ fontSize: "0.85rem", opacity: 0.8 }}>
              {activeEntry.description}
            </p>
          )}
          <button
            className="btn"
            style={{
              marginTop: 20,
              background: "white",
              color: "var(--danger)",
              fontWeight: 700,
              padding: "12px 32px",
              fontSize: "1rem",
            }}
            onClick={handleStopTimer}
          >
            ⏹️ Parar Timer
          </button>
        </div>
      ) : (
        <div className="card" style={{ marginBottom: 20 }}>
          <div className="card-header">
            <h3 className="card-title">▶️ Iniciar Timer</h3>
          </div>
          <div style={{ display: "flex", gap: 12, alignItems: "end", flexWrap: "wrap" }}>
            <div className="form-group" style={{ flex: 1, minWidth: 200, marginBottom: 0 }}>
              <label>Projeto *</label>
              <select
                className="form-input"
                value={startProject ?? ""}
                onChange={(e) => setStartProject(e.target.value ? Number(e.target.value) : null)}
              >
                <option value="">Selecione um projeto...</option>
                {projects.map((p) => (
                  <option key={p.id} value={p.id}>{p.name}</option>
                ))}
              </select>
            </div>
            <div className="form-group" style={{ flex: 2, minWidth: 200, marginBottom: 0 }}>
              <label>Descrição (opcional)</label>
              <input
                className="form-input"
                placeholder="O que você está fazendo?"
                value={startDesc}
                onChange={(e) => setStartDesc(e.target.value)}
              />
            </div>
            <button
              className="btn btn-primary"
              onClick={handleStartTimer}
              disabled={!startProject}
              style={{ padding: "10px 24px", marginBottom: 1 }}
            >
              ▶️ Iniciar
            </button>
          </div>
        </div>
      )}

      {/* Resumo de Horas */}
      <div className="grid-3" style={{ marginBottom: 20 }}>
        <div className="card stat-card stat-card--projects" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 500 }}>
            ⏱️ Hoje
          </p>
          <p className="stat-value" style={{ color: "var(--stat-projects)" }}>
            {formatDuration(hoursSummary.today)}
          </p>
        </div>
        <div className="card stat-card stat-card--clients" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 500 }}>
            📅 Esta Semana
          </p>
          <p className="stat-value" style={{ color: "var(--stat-clients)" }}>
            {formatDuration(Math.round(hoursSummary.week))}
          </p>
        </div>
        <div className="card stat-card stat-card--teams" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 500 }}>
            📊 Total Registrado
          </p>
          <p className="stat-value" style={{ color: "var(--stat-teams)" }}>
            {entries.length} entrada(s)
          </p>
        </div>
      </div>

      {/* Filtros */}
      <div className="card" style={{ marginBottom: 20 }}>
        <div style={{ display: "flex", gap: 12, alignItems: "end", flexWrap: "wrap" }}>
          <div className="form-group" style={{ flex: 1, minWidth: 150, marginBottom: 0 }}>
            <label>Projeto</label>
            <select
              className="form-input"
              value={filterProject ?? ""}
              onChange={(e) => setFilterProject(e.target.value ? Number(e.target.value) : null)}
            >
              <option value="">Todos</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>{p.name}</option>
              ))}
            </select>
          </div>
          <div className="form-group" style={{ marginBottom: 0 }}>
            <label>De</label>
            <input
              className="form-input"
              type="date"
              value={filterDateFrom}
              onChange={(e) => setFilterDateFrom(e.target.value)}
            />
          </div>
          <div className="form-group" style={{ marginBottom: 0 }}>
            <label>Até</label>
            <input
              className="form-input"
              type="date"
              value={filterDateTo}
              onChange={(e) => setFilterDateTo(e.target.value)}
            />
          </div>
          <button className="btn btn-primary" onClick={loadFiltered} style={{ marginBottom: 1 }}>
            🔍 Filtrar
          </button>
          {(filterProject || filterDateFrom || filterDateTo) && (
            <button
              className="btn btn-secondary"
              onClick={() => {
                setFilterProject(null);
                setFilterDateFrom("");
                setFilterDateTo("");
                setTimeout(loadFiltered, 0);
              }}
              style={{ marginBottom: 1 }}
            >
              ✕ Limpar
            </button>
          )}
        </div>
      </div>

      {/* Lista de Entradas */}
      {entries.length === 0 ? (
        <div className="card text-center" style={{ padding: 48 }}>
          <p style={{ fontSize: "3rem", marginBottom: 12 }}>⏱️</p>
          <p className="text-secondary" style={{ fontSize: "1.1rem" }}>
            Nenhum registro de horas encontrado.
          </p>
          <p className="text-secondary" style={{ marginBottom: 16 }}>
            Inicie o timer ou registre horas manualmente!
          </p>
          <button className="btn btn-primary" onClick={() => setShowManualModal(true)}>
            ➕ Registrar Horas
          </button>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {entries.map((entry) => (
            <div className="card" key={entry.id} style={{ padding: 16 }}>
              <div className="flex-between" style={{ marginBottom: 8 }}>
                <div>
                  <p style={{ fontWeight: 600 }}>
                    📁 {entry.project_name || "Projeto"}
                  </p>
                  {entry.description && (
                    <p className="text-secondary text-sm">{entry.description}</p>
                  )}
                </div>
                <div className="flex gap-6">
                  {entry.duration_minutes !== null && (
                    <span
                      className="badge badge-progress"
                      style={{ fontSize: "0.85rem", padding: "4px 12px" }}
                    >
                      ⏱️ {formatDuration(entry.duration_minutes)}
                    </span>
                  )}
                  {entry.billable && entry.hourly_rate && entry.duration_minutes && (
                    <span className="badge badge-published" style={{ fontSize: "0.75rem" }}>
                      💰 R$ {getBillableValue(entry).toFixed(2)}
                    </span>
                  )}
                  {!entry.billable && (
                    <span className="badge badge-pending" style={{ fontSize: "0.7rem" }}>
                      Não faturável
                    </span>
                  )}
                </div>
              </div>
              <div className="meta-info">
                <span>📅 {formatDate(entry.start_time)}</span>
                <span>•</span>
                <span>▶️ {formatTime(entry.start_time)}</span>
                {entry.end_time && (
                  <>
                    <span>•</span>
                    <span>⏹️ {formatTime(entry.end_time)}</span>
                  </>
                )}
                {entry.hourly_rate && (
                  <>
                    <span>•</span>
                    <span>💰 R$ {entry.hourly_rate.toFixed(2)}/h</span>
                  </>
                )}
              </div>
              <div className="flex gap-6" style={{ marginTop: 8 }}>
                {deleteConfirm === entry.id ? (
                  <div className="flex gap-6">
                    <button
                      className="btn btn-danger btn-xs"
                      onClick={() => handleDelete(entry.id)}
                    >
                      Confirmar
                    </button>
                    <button
                      className="btn btn-secondary btn-xs"
                      onClick={() => setDeleteConfirm(null)}
                    >
                      Cancelar
                    </button>
                  </div>
                ) : (
                  <button
                    className="btn btn-danger btn-xs"
                    onClick={() => setDeleteConfirm(entry.id)}
                    title="Excluir"
                  >
                    🗑️ Excluir
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal: Registro Manual */}
      {showManualModal && (
        <div className="modal-overlay" onClick={() => setShowManualModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>➕ Registrar Horas</h2>

            <div className="form-group">
              <label>Projeto *</label>
              <select
                className="form-input"
                value={manualForm.project_id ?? ""}
                onChange={(e) =>
                  setManualForm({ ...manualForm, project_id: e.target.value ? Number(e.target.value) : null })
                }
              >
                <option value="">Selecione...</option>
                {projects.map((p) => (
                  <option key={p.id} value={p.id}>{p.name}</option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label>Descrição</label>
              <input
                className="form-input"
                placeholder="O que foi feito?"
                value={manualForm.description}
                onChange={(e) => setManualForm({ ...manualForm, description: e.target.value })}
              />
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <div className="form-group">
                <label>Data</label>
                <input
                  className="form-input"
                  type="date"
                  value={manualForm.entry_date}
                  onChange={(e) => setManualForm({ ...manualForm, entry_date: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label>Duração (minutos) *</label>
                <input
                  className="form-input"
                  type="number"
                  min={1}
                  value={manualForm.duration_minutes}
                  onChange={(e) =>
                    setManualForm({ ...manualForm, duration_minutes: Math.max(1, parseInt(e.target.value) || 0) })
                  }
                />
              </div>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <div className="form-group">
                <label>Valor Hora (R$)</label>
                <input
                  className="form-input"
                  type="number"
                  min={0}
                  step={0.5}
                  value={manualForm.hourly_rate || ""}
                  onChange={(e) =>
                    setManualForm({ ...manualForm, hourly_rate: e.target.value ? parseFloat(e.target.value) : 0 })
                  }
                  placeholder="0,00"
                />
              </div>
              <div className="form-group">
                <label>Faturável</label>
                <select
                  className="form-input"
                  value={manualForm.billable ? "true" : "false"}
                  onChange={(e) => setManualForm({ ...manualForm, billable: e.target.value === "true" })}
                >
                  <option value="true">Sim</option>
                  <option value="false">Não</option>
                </select>
              </div>
            </div>

            {manualForm.hourly_rate > 0 && manualForm.duration_minutes > 0 && (
              <div
                className="card"
                style={{
                  padding: "12px 16px",
                  background: "var(--primary-light)",
                  textAlign: "center",
                }}
              >
                <p style={{ fontSize: "0.85rem", color: "var(--text-secondary)" }}>
                  Valor calculado
                </p>
                <p style={{ fontSize: "1.5rem", fontWeight: 700, color: "var(--primary)" }}>
                  R$ {((manualForm.hourly_rate * manualForm.duration_minutes) / 60).toFixed(2)}
                </p>
              </div>
            )}

            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowManualModal(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={handleManualSubmit}>
                Salvar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default TimeTracking;
