import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Assignment {
  id: number;
  title: string;
  description: string;
  due_date: string;
  due_time: string | null;
  notification_time: string | null;
  project_name: string;
  status: string;
}

interface AssignmentFile {
  id: number;
  assignment_id: number;
  original_name: string;
  stored_name: string;
  file_size: number;
  mime_type: string;
  created_at: string;
}

function CalendarView() {
  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [selectedDate, setSelectedDate] = useState<string>("");
  const [currentMonth, setCurrentMonth] = useState(new Date().getMonth());
  const [currentYear, setCurrentYear] = useState(new Date().getFullYear());
  const [newAssignment, setNewAssignment] = useState({
    title: "",
    description: "",
    project_name: "",
  });
  const [newTime, setNewTime] = useState<string>("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [selectedAssignment, setSelectedAssignment] = useState<Assignment | null>(null);
  const [showFilesModal, setShowFilesModal] = useState(false);
  const [assignmentFiles, setAssignmentFiles] = useState<AssignmentFile[]>([]);
  const [uploadingFile, setUploadingFile] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const NOTIFICATION_INTERVAL = 60000;

  // Funções getter para evitar valores stale (desatualizados)
  function getTodayStr(): string {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  }

  function getNowTime(): string {
    const d = new Date();
    return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  }

  useEffect(() => {
    if ("Notification" in window && Notification.permission === "default") {
      Notification.requestPermission();
    }
  }, []);

  useEffect(() => {
    loadAssignments();
    const interval = setInterval(checkNotifications, NOTIFICATION_INTERVAL);
    return () => clearInterval(interval);
  }, []);

  async function checkNotifications() {
    try {
      const todayAssignments = await invoke<Assignment[]>("check_today_assignments");
      if (todayAssignments.length > 0) {
        loadAssignments();
      }
    } catch {
      const saved = localStorage.getItem("unitesk_assignments");
      if (saved) {
        try {
          const all: Assignment[] = JSON.parse(saved);
          const today = new Date();
          const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`;
          const nowMinutes = today.getHours() * 60 + today.getMinutes();

          for (const a of all) {
            const notifTime = a.notification_time || a.due_time;
            if (a.due_date === todayStr && a.status === "pending" && notifTime) {
              const [h, m] = notifTime.split(":").map(Number);
              const activityMinutes = h * 60 + m;
              if (nowMinutes >= activityMinutes - 5 && nowMinutes <= activityMinutes + 5) {
                if ("Notification" in window && Notification.permission === "granted") {
                  new Notification("📚 Prazo Hoje!", {
                    body: `A atividade '${a.title}' vence hoje às ${notifTime}!`,
                  });
                }
              }
            }
          }
        } catch {
          // ignore
        }
      }
    }
  }

  async function loadAssignments() {
    try {
      const data = await invoke<Assignment[]>("get_assignments");
      const enriched = data.map((a) => enrichAssignmentStatus(a));
      setAssignments(enriched);
      localStorage.setItem("unitesk_assignments", JSON.stringify(enriched));
    } catch {
      const saved = localStorage.getItem("unitesk_assignments");
      if (saved) {
        try {
          const parsed: Assignment[] = JSON.parse(saved);
          setAssignments(parsed.map((a) => enrichAssignmentStatus(a)));
        } catch {
          setAssignments([]);
        }
      } else {
        setAssignments([]);
      }
    }
  }

  function enrichAssignmentStatus(a: Assignment): Assignment {
    const todayStr = getTodayStr();
    if (a.status === "pending" && a.due_date < todayStr) {
      return { ...a, status: "overdue" };
    }
    return a;
  }

  async function createAssignment() {
    if (!newAssignment.title || !selectedDate) return;
    setErrorMessage(null);

    try {
      await invoke("create_assignment", {
        title: newAssignment.title,
        description: newAssignment.description,
        dueDate: selectedDate,
        dueTime: newTime || null,
        projectName: newAssignment.project_name,
      });
      resetForm();
      await loadAssignments();
    } catch {
      const updatedAssignments = [
        ...assignments,
        {
          id: Date.now(),
          title: newAssignment.title,
          description: newAssignment.description,
          due_date: selectedDate,
          due_time: newTime || null,
          notification_time: newTime || null,
          project_name: newAssignment.project_name,
          status: "pending" as const,
        },
      ];
      setAssignments(updatedAssignments);
      localStorage.setItem("unitesk_assignments", JSON.stringify(updatedAssignments));
      resetForm();
    }
  }

  function resetForm() {
    setNewAssignment({ title: "", description: "", project_name: "" });
    setNewTime("");
    setShowModal(false);
    setErrorMessage(null);
  }

  async function deleteAssignment(id: number) {
    if (!confirm("Tem certeza que deseja excluir esta atividade? Todos os arquivos anexados também serão removidos.")) return;
    setErrorMessage(null);
    try {
      await invoke("delete_assignment", { id });
      await loadAssignments();
    } catch {
      const updatedAssignments = assignments.filter((a) => a.id !== id);
      setAssignments(updatedAssignments);
      localStorage.setItem("unitesk_assignments", JSON.stringify(updatedAssignments));
    }
  }

  async function markAsDone(id: number) {
    setErrorMessage(null);
    try {
      await invoke("mark_assignment_done", { id });
      await loadAssignments();
    } catch {
      const updatedAssignments = assignments.map((a) =>
        a.id === id ? { ...a, status: "done" as const } : a
      );
      setAssignments(updatedAssignments);
      localStorage.setItem("unitesk_assignments", JSON.stringify(updatedAssignments));
    }
  }

  // ========== Gerenciamento de Arquivos ==========

  async function openFilesModal(assignment: Assignment) {
    setSelectedAssignment(assignment);
    setAssignmentFiles([]);
    setShowFilesModal(true);

    try {
      const files = await invoke<AssignmentFile[]>("get_assignment_files", {
        assignmentId: assignment.id,
      });
      setAssignmentFiles(files);
    } catch {
      setAssignmentFiles([]);
    }
  }

  async function handleFileUpload(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file || !selectedAssignment) return;

    if (file.size > 10 * 1024 * 1024) {
      setErrorMessage("Arquivo muito grande! Máximo permitido: 10 MB.");
      return;
    }

    setUploadingFile(true);
    setErrorMessage(null);

    try {
      const buffer = await file.arrayBuffer();
      const fileData = Array.from(new Uint8Array(buffer));

      await invoke("add_assignment_file", {
        assignmentId: selectedAssignment.id,
        originalName: file.name,
        fileData: fileData,
        mimeType: file.type || "application/octet-stream",
      });

      const files = await invoke<AssignmentFile[]>("get_assignment_files", {
        assignmentId: selectedAssignment.id,
      });
      setAssignmentFiles(files);
    } catch {
      setErrorMessage("Erro ao fazer upload do arquivo. Tente novamente.");
    } finally {
      setUploadingFile(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  }

  async function downloadFile(file: AssignmentFile) {
    try {
      const result = await invoke<[string, string, number[]]>("get_assignment_file_data", {
        fileId: file.id,
      });
      const [originalName, mimeType, data] = result;
      const blob = new Blob([new Uint8Array(data)], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = originalName;
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      setErrorMessage("Erro ao baixar o arquivo.");
    }
  }

  async function deleteAssignmentFile(fileId: number) {
    try {
      await invoke("delete_assignment_file", { id: fileId });
      setAssignmentFiles(assignmentFiles.filter((f) => f.id !== fileId));
    } catch {
      setErrorMessage("Erro ao excluir o arquivo.");
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  function getFileIcon(mimeType: string): string {
    if (mimeType.startsWith("image/")) return "🖼️";
    if (mimeType.includes("pdf")) return "📕";
    if (mimeType.includes("zip") || mimeType.includes("rar") || mimeType.includes("7z")) return "📦";
    if (mimeType.includes("text") || mimeType.includes("document")) return "📄";
    return "📎";
  }

  // ========== Navegação do Calendário ==========

  const monthNames = [
    "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
    "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
  ];

  const dayNames = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"];

  const years = useMemo(() => {
    const y = [];
    for (let i = currentYear - 10; i <= currentYear + 5; i++) {
      y.push(i);
    }
    return y;
  }, [currentYear]);

  function goToMonth(month: number) {
    setCurrentMonth(month);
  }

  function goToYear(year: number) {
    setCurrentYear(year);
  }

  function goToToday() {
    const now = new Date();
    setCurrentMonth(now.getMonth());
    setCurrentYear(now.getFullYear());
  }

  function prevMonth() {
    if (currentMonth === 0) {
      setCurrentMonth(11);
      setCurrentYear((y) => y - 1);
    } else {
      setCurrentMonth((m) => m - 1);
    }
  }

  function nextMonth() {
    if (currentMonth === 11) {
      setCurrentMonth(0);
      setCurrentYear((y) => y + 1);
    } else {
      setCurrentMonth((m) => m + 1);
    }
  }

  const daysInMonth = new Date(currentYear, currentMonth + 1, 0).getDate();
  const firstDayOfMonth = new Date(currentYear, currentMonth, 1).getDay();

  const getAssignmentsForDay = useCallback(
    (day: number) => {
      const dateStr = `${currentYear}-${String(currentMonth + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
      return assignments.filter((a) => a.due_date.startsWith(dateStr));
    },
    [assignments, currentMonth, currentYear]
  );

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "done":
        return <span className="badge badge-done">✅ Concluído</span>;
      case "overdue":
        return <span className="badge badge-overdue">🔴 Atrasado</span>;
      default:
        return <span className="badge badge-pending">⏳ Pendente</span>;
    }
  };

  const sortedAssignments = useMemo(
    () =>
      [...assignments].sort(
        (a, b) => new Date(a.due_date).getTime() - new Date(b.due_date).getTime()
      ),
    [assignments]
  );

  function handleDayClick(day: number) {
    const dateStr = `${currentYear}-${String(currentMonth + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    setSelectedDate(dateStr);
    setNewTime(getNowTime());
    setShowModal(true);
    setErrorMessage(null);
  }

  function handleNewAssignmentClick() {
    setSelectedDate(getTodayStr());
    setNewTime(getNowTime());
    setShowModal(true);
    setErrorMessage(null);
  }

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 20,
          flexWrap: "wrap",
          gap: 8,
        }}
      >
        <h2 style={{ fontSize: "1.4rem" }}>📅 Calendário de Atividades</h2>
        <button className="btn btn-primary" onClick={handleNewAssignmentClick}>
          ➕ Nova Atividade
        </button>
      </div>

      {/* Mensagem de erro */}
      {errorMessage && (
        <div
          style={{
            background: "rgba(239, 68, 68, 0.1)",
            color: "var(--danger)",
            padding: "12px 16px",
            borderRadius: 8,
            marginBottom: 16,
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <span>❌ {errorMessage}</span>
          <button
            className="btn btn-sm"
            style={{ background: "transparent", color: "var(--danger)", fontWeight: 700 }}
            onClick={() => setErrorMessage(null)}
          >
            ✕
          </button>
        </div>
      )}

      <div className="grid-2">
        {/* Calendar */}
        <div className="card">
          <div className="calendar-header">
            <div className="calendar-nav">
              <button className="btn btn-secondary btn-sm" onClick={prevMonth}>
                ←
              </button>
              <select
                className="calendar-nav-select"
                value={currentMonth}
                onChange={(e) => goToMonth(Number(e.target.value))}
                style={{ minWidth: 120 }}
              >
                {monthNames.map((name, idx) => (
                  <option key={name} value={idx}>{name}</option>
                ))}
              </select>
              <select
                className="calendar-nav-select"
                value={currentYear}
                onChange={(e) => goToYear(Number(e.target.value))}
                style={{ minWidth: 85 }}
              >
                {years.map((y) => (
                  <option key={y} value={y}>{y}</option>
                ))}
              </select>
              <button className="btn btn-secondary btn-sm" onClick={nextMonth}>
                →
              </button>
            </div>
            <button className="btn btn-outline-primary btn-sm" onClick={goToToday}>
              📅 Hoje
            </button>
          </div>

          <div className="calendar-grid">
            {dayNames.map((name) => (
              <div key={name} className="calendar-day-header">
                {name}
              </div>
            ))}

            {Array.from({ length: firstDayOfMonth }).map((_, i) => (
              <div key={`empty-${i}`} />
            ))}

            {Array.from({ length: daysInMonth }).map((_, i) => {
              const day = i + 1;
              const dateStr = `${currentYear}-${String(currentMonth + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
              const dayAssignments = getAssignmentsForDay(day);
              const todayStr = getTodayStr();
              const isToday = dateStr === todayStr;
              const hasOverdue = dayAssignments.some((a) => a.status === "overdue");
              const hasPending = dayAssignments.some((a) => a.status === "pending" && dateStr <= todayStr);
              const hasScheduled = dayAssignments.some((a) => a.status === "pending" && dateStr > todayStr);
              const isPast = dateStr < todayStr;

              let dotClass = "day-dot--done";
              if (hasOverdue) dotClass = "day-dot--overdue";
              else if (hasPending) dotClass = "day-dot--pending";
              else if (hasScheduled) dotClass = "day-dot--scheduled";

              return (
                <button
                  key={day}
                  onClick={() => handleDayClick(day)}
                  className={`calendar-day ${isToday ? "calendar-day--today" : ""} ${
                    dayAssignments.length > 0 ? "calendar-day--has-events" : ""
                  }`}
                  style={isPast && !isToday ? { opacity: 0.6 } : {}}
                >
                  {day}
                  {dayAssignments.length > 0 && (
                    <div className={`day-dot ${dotClass}`} />
                  )}
                </button>
              );
            })}
          </div>
        </div>

        {/* Assignments List */}
        <div>
          <h3 style={{ marginBottom: 12, fontWeight: 600 }}>
            Atividades Próximas
          </h3>
          {assignments.length === 0 ? (
            <div className="card" style={{ textAlign: "center", padding: 32 }}>
              <p style={{ fontSize: "3rem", marginBottom: 12 }}>📅</p>
              <p style={{ color: "var(--text-secondary)" }}>
                Nenhuma atividade cadastrada.
              </p>
              <p style={{ color: "var(--text-secondary)", fontSize: "0.85rem" }}>
                Clique em um dia no calendário para adicionar!
              </p>
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 8, maxHeight: "60vh", overflowY: "auto", paddingRight: 4 }}>
              {sortedAssignments.map((assignment) => {
                  const hasTime = assignment.due_time || assignment.notification_time;
                  return (
                    <div className="card" key={assignment.id} style={{ padding: 16 }}>
                      <div
                        style={{
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "flex-start",
                        }}
                      >
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <p style={{ fontWeight: 600, marginBottom: 4 }}>
                            {assignment.title}
                          </p>
                          <p style={{ color: "var(--text-secondary)", fontSize: "0.8rem" }}>
                            {assignment.project_name && `📁 ${assignment.project_name} • `}
                            📅{" "}
                            {new Date(assignment.due_date).toLocaleDateString("pt-BR")}
                            {hasTime && (
                              <> • ⏰ {assignment.notification_time || assignment.due_time}</>
                            )}
                          </p>
                          {assignment.description && (
                            <p style={{ color: "var(--text-secondary)", fontSize: "0.8rem", marginTop: 4 }}>
                              {assignment.description}
                            </p>
                          )}
                          <button
                            className="btn btn-sm"
                            style={{
                              marginTop: 8,
                              background: "var(--bg)",
                              border: "1px solid var(--border)",
                              borderRadius: 6,
                              padding: "4px 10px",
                              fontSize: "0.75rem",
                              cursor: "pointer",
                            }}
                            onClick={() => openFilesModal(assignment)}
                          >
                            📎 Arquivos
                          </button>
                        </div>
                        <div style={{ display: "flex", gap: 8, alignItems: "center", flexShrink: 0 }}>
                          <div style={{ display: "flex", gap: 4, alignItems: "center" }}>
                            {getStatusBadge(assignment.status)}
                            {assignment.status !== "done" && (
                              <button
                                className="btn btn-primary btn-xs"
                                onClick={() => markAsDone(assignment.id)}
                                title="Marcar como concluído"
                              >
                                ✅
                              </button>
                            )}
                            <button
                              className="btn btn-danger btn-xs"
                              onClick={() => deleteAssignment(assignment.id)}
                              title="Excluir atividade"
                            >
                              🗑️
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })}
            </div>
          )}
        </div>
      </div>

      {/* Modal: New Assignment */}
      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>📅 Nova Atividade</h2>
            <div className="form-group">
              <label>Data</label>
              <input
                className="form-input"
                type="date"
                value={selectedDate}
                onChange={(e) => setSelectedDate(e.target.value)}
              />
            </div>
            <div className="form-group">
              <label>Horário de Notificação ⏰</label>
              <input
                className="form-input"
                type="time"
                value={newTime}
                onChange={(e) => setNewTime(e.target.value)}
              />
              <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)", marginTop: 4 }}>
                Você receberá uma notificação neste horário no dia da atividade.
              </p>
            </div>
            <div className="form-group">
              <label>Título da Atividade *</label>
              <input
                className="form-input"
                placeholder="Ex: Entrega do artigo de Redes"
                value={newAssignment.title}
                onChange={(e) =>
                  setNewAssignment({ ...newAssignment, title: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Disciplina / Projeto</label>
              <input
                className="form-input"
                placeholder="Ex: Redes de Computadores"
                value={newAssignment.project_name}
                onChange={(e) =>
                  setNewAssignment({
                    ...newAssignment,
                    project_name: e.target.value,
                  })
                }
              />
            </div>
            <div className="form-group">
              <label>Descrição</label>
              <textarea
                className="form-textarea"
                placeholder="Descreva a atividade..."
                value={newAssignment.description}
                onChange={(e) =>
                  setNewAssignment({
                    ...newAssignment,
                    description: e.target.value,
                  })
                }
              />
            </div>
            <div className="modal-actions">
              <button
                className="btn btn-secondary"
                onClick={() => setShowModal(false)}
              >
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={createAssignment}>
                Salvar
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal: Gerenciar Arquivos da Atividade */}
      {showFilesModal && selectedAssignment && (
        <div className="modal-overlay" onClick={() => setShowFilesModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>📎 Arquivos — {selectedAssignment.title}</h2>

            {assignmentFiles.length === 0 ? (
              <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: 24 }}>
                Nenhum arquivo anexado a esta atividade.
              </p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {assignmentFiles.map((file) => (
                  <div
                    key={file.id}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "10px 12px",
                      background: "var(--bg)",
                      borderRadius: 8,
                      border: "1px solid var(--border)",
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                      <span>{getFileIcon(file.mime_type)}</span>
                      <div>
                        <p style={{ fontSize: "0.85rem", fontWeight: 500 }}>{file.original_name}</p>
                        <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)" }}>
                          {formatFileSize(file.file_size)}
                        </p>
                      </div>
                    </div>
                    <div style={{ display: "flex", gap: 4 }}>
                      <button
                        className="btn btn-sm btn-primary"
                        onClick={() => downloadFile(file)}
                        title="Baixar"
                      >
                        📥
                      </button>
                      <button
                        className="btn btn-sm btn-danger"
                        onClick={() => deleteAssignmentFile(file.id)}
                        title="Excluir"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            <div style={{ marginTop: 20, borderTop: "1px solid var(--border)", paddingTop: 20 }}>
              <label className="btn btn-primary" style={{ cursor: "pointer", display: "inline-flex" }}>
                {uploadingFile ? "⏳ Enviando..." : "📤 Anexar Arquivo"}
                <input
                  ref={fileInputRef}
                  type="file"
                  style={{ display: "none" }}
                  onChange={handleFileUpload}
                  disabled={uploadingFile}
                />
              </label>
              <p style={{ fontSize: "0.75rem", color: "var(--text-secondary)", marginTop: 8 }}>
                Máximo: 10 MB. Formatos aceitos: PDF, imagens, documentos, etc.
              </p>
            </div>

            <div className="modal-actions">
              <button
                className="btn btn-secondary"
                onClick={() => setShowFilesModal(false)}
              >
                Fechar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default CalendarView;
