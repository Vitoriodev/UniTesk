import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Project {
  id: number;
  name: string;
}

interface Client {
  id: number;
  name: string;
}

interface Invoice {
  id: number;
  project_id: number | null;
  project_name: string | null;
  client_id: number | null;
  client_name: string | null;
  number: string;
  description: string | null;
  amount: number;
  tax: number;
  total: number;
  status: string;
  issue_date: string;
  due_date: string | null;
  paid_date: string | null;
  notes: string | null;
  created_at: string;
}

type StatusFilter = "all" | "draft" | "sent" | "paid" | "cancelled" | "overdue";

function Finance() {
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
  const [showModal, setShowModal] = useState(false);
  const [editingInvoice, setEditingInvoice] = useState<Invoice | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);
  const [form, setForm] = useState({
    project_id: null as number | null,
    client_id: null as number | null,
    number: "",
    description: "",
    amount: 0,
    tax: 0,
    status: "draft",
    issue_date: new Date().toISOString().split("T")[0],
    due_date: "",
    paid_date: "",
    notes: "",
  });

  useEffect(() => {
    loadData();
  }, [statusFilter]);

  async function loadData() {
    try {
      const status = statusFilter === "all" ? null : statusFilter;
      const data = await invoke<Invoice[]>("get_invoices", { statusFilter: status });
      setInvoices(data);
    } catch {
      setInvoices([]);
    }
    try {
      const p = await invoke<Project[]>("get_projects");
      setProjects(p);
    } catch {}
    try {
      const c = await invoke<Client[]>("get_clients");
      setClients(c);
    } catch {}
  }

  function openCreateModal() {
    setEditingInvoice(null);
    const now = new Date().toISOString().split("T")[0];
    const nextMonth = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString().split("T")[0];
    setForm({
      project_id: null,
      client_id: null,
      number: `INV-${Date.now()}`,
      description: "",
      amount: 0,
      tax: 0,
      status: "draft",
      issue_date: now,
      due_date: nextMonth,
      paid_date: "",
      notes: "",
    });
    setShowModal(true);
  }

  function openEditModal(invoice: Invoice) {
    setEditingInvoice(invoice);
    setForm({
      project_id: invoice.project_id,
      client_id: invoice.client_id,
      number: invoice.number,
      description: invoice.description || "",
      amount: invoice.amount,
      tax: invoice.tax,
      status: invoice.status,
      issue_date: invoice.issue_date,
      due_date: invoice.due_date || "",
      paid_date: invoice.paid_date || "",
      notes: invoice.notes || "",
    });
    setShowModal(true);
  }

  const totalCalc = form.amount + form.tax;

  async function saveInvoice() {
    if (!form.number) return;
    try {
      if (editingInvoice) {
        await invoke("update_invoice", {
          id: editingInvoice.id,
          projectId: form.project_id,
          clientId: form.client_id,
          number: form.number,
          description: form.description || null,
          amount: form.amount,
          tax: form.tax,
          total: totalCalc,
          status: form.status,
          issueDate: form.issue_date,
          dueDate: form.due_date || null,
          paidDate: form.paid_date || null,
          notes: form.notes || null,
        });
      } else {
        await invoke("create_invoice", {
          projectId: form.project_id,
          clientId: form.client_id,
          number: form.number,
          description: form.description || null,
          amount: form.amount,
          tax: form.tax,
          total: totalCalc,
          status: form.status,
          issueDate: form.issue_date,
          dueDate: form.due_date || null,
          notes: form.notes || null,
        });
      }
      setShowModal(false);
      loadData();
    } catch (err) {
      alert("Erro ao salvar fatura: " + String(err));
    }
  }

  async function confirmDelete(id: number) {
    try {
      await invoke("delete_invoice", { id });
      setDeleteConfirm(null);
      loadData();
    } catch {
      setInvoices(invoices.filter((i) => i.id !== id));
      setDeleteConfirm(null);
    }
  }

  function getStatusBadge(status: string) {
    switch (status) {
      case "paid":
        return <span className="badge badge-done">✅ Paga</span>;
      case "sent":
        return <span className="badge badge-progress">📨 Enviada</span>;
      case "draft":
        return <span className="badge badge-pending">📝 Rascunho</span>;
      case "cancelled":
        return <span className="badge badge-overdue">❌ Cancelada</span>;
      case "overdue":
        return <span className="badge badge-overdue">🔴 Vencida</span>;
      default:
        return <span className="badge badge-pending">{status}</span>;
    }
  }

  function formatCurrency(value: number): string {
    return new Intl.NumberFormat("pt-BR", {
      style: "currency",
      currency: "BRL",
    }).format(value);
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + "T12:00:00");
    return d.toLocaleDateString("pt-BR");
  }

  // Stats
  const totalRevenue = invoices
    .filter((i) => i.status === "paid")
    .reduce((sum, i) => sum + i.total, 0);
  const pendingAmount = invoices
    .filter((i) => i.status === "draft" || i.status === "sent")
    .reduce((sum, i) => sum + i.total, 0);
  const overdueCount = invoices.filter((i) => i.status === "overdue").length;

  const filterTabs: { id: StatusFilter; label: string }[] = [
    { id: "all", label: "Todas" },
    { id: "draft", label: "📝 Rascunho" },
    { id: "sent", label: "📨 Enviadas" },
    { id: "paid", label: "✅ Pagas" },
    { id: "overdue", label: "🔴 Vencidas" },
    { id: "cancelled", label: "❌ Canceladas" },
  ];

  return (
    <div>
      <div className="flex-between" style={{ marginBottom: 20 }}>
        <h2 className="section-title">💰 Financeiro</h2>
        <button className="btn btn-primary" onClick={openCreateModal}>
          ➕ Nova Fatura
        </button>
      </div>

      {/* Stats */}
      <div className="grid-3" style={{ marginBottom: 20 }}>
        <div className="card stat-card stat-card--clients" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 500 }}>
            💰 Receita Total
          </p>
          <p className="stat-value" style={{ color: "var(--stat-clients)", fontSize: "1.5rem" }}>
            {formatCurrency(totalRevenue)}
          </p>
        </div>
        <div className="card stat-card stat-card--pending" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 500 }}>
            ⏳ A Receber
          </p>
          <p className="stat-value" style={{ color: "var(--stat-pending)", fontSize: "1.5rem" }}>
            {formatCurrency(pendingAmount)}
          </p>
        </div>
        <div className="card stat-card stat-card--overdue" style={{ padding: "16px 20px" }}>
          <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 500 }}>
            🔴 Vencidas
          </p>
          <p className="stat-value" style={{ color: "var(--stat-overdue)", fontSize: "1.5rem" }}>
            {overdueCount}
          </p>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="filter-tabs" style={{ marginBottom: 16 }}>
        {filterTabs.map((tab) => (
          <button
            key={tab.id}
            className={`filter-tab ${statusFilter === tab.id ? "filter-tab--active" : ""}`}
            onClick={() => setStatusFilter(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {invoices.length === 0 ? (
        <div className="card text-center" style={{ padding: 48 }}>
          <p style={{ fontSize: "3rem", marginBottom: 12 }}>💰</p>
          <p className="text-secondary" style={{ fontSize: "1.1rem" }}>
            {statusFilter === "all"
              ? "Nenhuma fatura cadastrada."
              : "Nenhuma fatura com este status."}
          </p>
          <p className="text-secondary" style={{ marginBottom: 16 }}>
            Crie faturas para gerenciar o financeiro dos seus projetos!
          </p>
          <button className="btn btn-primary" onClick={openCreateModal}>
            ➕ Nova Fatura
          </button>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {invoices.map((inv) => (
            <div className="card" key={inv.id} style={{ padding: 16 }}>
              <div className="flex-between" style={{ marginBottom: 8 }}>
                <div>
                  <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 4 }}>
                    <p style={{ fontWeight: 700, fontSize: "1rem" }}>
                      {inv.number}
                    </p>
                    {getStatusBadge(inv.status)}
                  </div>
                  {inv.description && (
                    <p className="text-secondary text-sm">{inv.description}</p>
                  )}
                </div>
                <div style={{ textAlign: "right" }}>
                  <p style={{ fontSize: "1.3rem", fontWeight: 700, color: "var(--primary)" }}>
                    {formatCurrency(inv.total)}
                  </p>
                </div>
              </div>

              <div className="meta-info" style={{ flexWrap: "wrap", gap: 8 }}>
                {inv.project_name && <span>📁 {inv.project_name}</span>}
                {inv.client_name && <span>🤝 {inv.client_name}</span>}
                <span>📅 Emissão: {formatDate(inv.issue_date)}</span>
                {inv.due_date && <span>⏰ Vencimento: {formatDate(inv.due_date)}</span>}
                {inv.paid_date && <span>✅ Pago em: {formatDate(inv.paid_date)}</span>}
              </div>

              <div className="meta-info" style={{ marginTop: 4 }}>
                <span>Valor: {formatCurrency(inv.amount)}</span>
                {inv.tax > 0 && <span>• Imposto: {formatCurrency(inv.tax)}</span>}
              </div>

              <div className="flex gap-6" style={{ marginTop: 8 }}>
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={() => openEditModal(inv)}
                >
                  ✏️ Editar
                </button>
                {deleteConfirm === inv.id ? (
                  <div className="flex gap-6">
                    <button
                      className="btn btn-danger btn-sm"
                      onClick={() => confirmDelete(inv.id)}
                    >
                      Confirmar
                    </button>
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => setDeleteConfirm(null)}
                    >
                      Cancelar
                    </button>
                  </div>
                ) : (
                  <button
                    className="btn btn-danger btn-sm"
                    onClick={() => setDeleteConfirm(inv.id)}
                  >
                    🗑️ Excluir
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 550 }}>
            <h2>{editingInvoice ? "✏️ Editar Fatura" : "💰 Nova Fatura"}</h2>

            <div className="form-group">
              <label>Número da Fatura *</label>
              <input
                className="form-input"
                value={form.number}
                onChange={(e) => setForm({ ...form, number: e.target.value })}
              />
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <div className="form-group">
                <label>Projeto</label>
                <select
                  className="form-input"
                  value={form.project_id ?? ""}
                  onChange={(e) =>
                    setForm({ ...form, project_id: e.target.value ? Number(e.target.value) : null })
                  }
                >
                  <option value="">Nenhum</option>
                  {projects.map((p) => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </select>
              </div>
              <div className="form-group">
                <label>Cliente</label>
                <select
                  className="form-input"
                  value={form.client_id ?? ""}
                  onChange={(e) =>
                    setForm({ ...form, client_id: e.target.value ? Number(e.target.value) : null })
                  }
                >
                  <option value="">Nenhum</option>
                  {clients.map((c) => (
                    <option key={c.id} value={c.id}>{c.name}</option>
                  ))}
                </select>
              </div>
            </div>

            <div className="form-group">
              <label>Descrição</label>
              <input
                className="form-input"
                placeholder="Descrição dos serviços..."
                value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })}
              />
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <div className="form-group">
                <label>Valor (R$) *</label>
                <input
                  className="form-input"
                  type="number"
                  min={0}
                  step={0.01}
                  value={form.amount || ""}
                  onChange={(e) =>
                    setForm({ ...form, amount: parseFloat(e.target.value) || 0 })
                  }
                />
              </div>
              <div className="form-group">
                <label>Imposto (R$)</label>
                <input
                  className="form-input"
                  type="number"
                  min={0}
                  step={0.01}
                  value={form.tax || ""}
                  onChange={(e) =>
                    setForm({ ...form, tax: parseFloat(e.target.value) || 0 })
                  }
                />
              </div>
            </div>

            {/* Total calculator */}
            <div
              className="card"
              style={{
                padding: "12px 16px",
                background: "var(--primary-light)",
                marginBottom: 16,
              }}
            >
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                  <p style={{ fontSize: "0.85rem", color: "var(--text-secondary)" }}>Total</p>
                  <p style={{ fontSize: "1.8rem", fontWeight: 700, color: "var(--primary)" }}>
                    {formatCurrency(totalCalc)}
                  </p>
                </div>
                <div style={{ fontSize: "0.85rem", color: "var(--text-secondary)" }}>
                  <p>Valor: {formatCurrency(form.amount)}</p>
                  {form.tax > 0 && <p>+ Imposto: {formatCurrency(form.tax)}</p>}
                </div>
              </div>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 12 }}>
              <div className="form-group">
                <label>Data de Emissão *</label>
                <input
                  className="form-input"
                  type="date"
                  value={form.issue_date}
                  onChange={(e) => setForm({ ...form, issue_date: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label>Vencimento</label>
                <input
                  className="form-input"
                  type="date"
                  value={form.due_date}
                  onChange={(e) => setForm({ ...form, due_date: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label>Status</label>
                <select
                  className="form-input"
                  value={form.status}
                  onChange={(e) => setForm({ ...form, status: e.target.value })}
                >
                  <option value="draft">📝 Rascunho</option>
                  <option value="sent">📨 Enviada</option>
                  <option value="paid">✅ Paga</option>
                  <option value="cancelled">❌ Cancelada</option>
                  <option value="overdue">🔴 Vencida</option>
                </select>
              </div>
            </div>

            {form.status === "paid" && (
              <div className="form-group">
                <label>Data de Pagamento</label>
                <input
                  className="form-input"
                  type="date"
                  value={form.paid_date}
                  onChange={(e) => setForm({ ...form, paid_date: e.target.value })}
                />
              </div>
            )}

            <div className="form-group">
              <label>Observações</label>
              <textarea
                className="form-textarea"
                placeholder="Informações adicionais..."
                value={form.notes}
                onChange={(e) => setForm({ ...form, notes: e.target.value })}
              />
            </div>

            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowModal(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={saveInvoice}>
                {editingInvoice ? "Salvar" : "Criar Fatura"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Finance;
