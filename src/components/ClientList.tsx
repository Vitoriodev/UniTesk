import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Client {
  id: number;
  name: string;
  email: string | null;
  phone: string | null;
  company: string | null;
  notes: string | null;
  created_at: string;
}

function ClientList() {
  const [clients, setClients] = useState<Client[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [editingClient, setEditingClient] = useState<Client | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);
  const [form, setForm] = useState({
    name: "",
    email: "",
    phone: "",
    company: "",
    notes: "",
  });

  useEffect(() => {
    loadClients();
  }, []);

  async function loadClients() {
    try {
      const data = await invoke<Client[]>("get_clients");
      setClients(data);
    } catch {
      setClients([]);
    }
  }

  function openCreateModal() {
    setEditingClient(null);
    setForm({ name: "", email: "", phone: "", company: "", notes: "" });
    setShowModal(true);
  }

  function openEditModal(client: Client) {
    setEditingClient(client);
    setForm({
      name: client.name,
      email: client.email || "",
      phone: client.phone || "",
      company: client.company || "",
      notes: client.notes || "",
    });
    setShowModal(true);
  }

  async function saveClient() {
    if (!form.name.trim()) return;
    try {
      if (editingClient) {
        await invoke("update_client", {
          id: editingClient.id,
          name: form.name,
          email: form.email || null,
          phone: form.phone || null,
          company: form.company || null,
          notes: form.notes || null,
        });
      } else {
        await invoke("create_client", {
          name: form.name,
          email: form.email || null,
          phone: form.phone || null,
          company: form.company || null,
          notes: form.notes || null,
        });
      }
      setShowModal(false);
      loadClients();
    } catch (err) {
      alert("Erro ao salvar cliente: " + String(err));
    }
  }

  async function confirmDelete(id: number) {
    try {
      await invoke("delete_client", { id });
      setDeleteConfirm(null);
      loadClients();
    } catch {
      setClients(clients.filter((c) => c.id !== id));
      setDeleteConfirm(null);
    }
  }

  const filtered = clients.filter(
    (c) =>
      c.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (c.email || "").toLowerCase().includes(searchTerm.toLowerCase()) ||
      (c.company || "").toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div>
      <div className="flex-between" style={{ marginBottom: 20 }}>
        <h2 className="section-title">🤝 Clientes</h2>
        <button className="btn btn-primary" onClick={openCreateModal}>
          ➕ Novo Cliente
        </button>
      </div>

      <div className="search-row">
        <input
          className="form-input"
          placeholder="🔍 Buscar clientes..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        {searchTerm && (
          <button className="btn btn-secondary btn-sm" onClick={() => setSearchTerm("")}>
            ✕ Limpar
          </button>
        )}
      </div>

      {filtered.length === 0 ? (
        <div className="card text-center" style={{ padding: 48 }}>
          <p style={{ fontSize: "3rem", marginBottom: 12 }}>🤝</p>
          <p className="text-secondary" style={{ fontSize: "1.1rem" }}>
            {searchTerm ? "Nenhum cliente encontrado." : "Nenhum cliente cadastrado."}
          </p>
          <p className="text-secondary" style={{ marginBottom: 16 }}>
            Cadastre seus clientes para vincular a projetos!
          </p>
          <button className="btn btn-primary" onClick={openCreateModal}>
            ➕ Novo Cliente
          </button>
        </div>
      ) : (
        <div className="grid-2">
          {filtered.map((client) => (
            <div className="card" key={client.id}>
              <div className="flex-between" style={{ marginBottom: 12 }}>
                <h3 className="project-name">{client.name}</h3>
                <div className="flex gap-6">
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => openEditModal(client)}
                    title="Editar"
                  >
                    ✏️
                  </button>
                  {deleteConfirm === client.id ? (
                    <div className="flex gap-6">
                      <button
                        className="btn btn-danger btn-sm"
                        onClick={() => confirmDelete(client.id)}
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
                      onClick={() => setDeleteConfirm(client.id)}
                      title="Excluir"
                    >
                      🗑️
                    </button>
                  )}
                </div>
              </div>

              <div className="meta-info" style={{ flexWrap: "wrap", gap: 8 }}>
                {client.company && (
                  <span className="badge badge-progress">🏢 {client.company}</span>
                )}
                {client.email && <span>✉️ {client.email}</span>}
                {client.phone && <span>📞 {client.phone}</span>}
              </div>

              {client.notes && (
                <p className="text-secondary text-sm" style={{ marginTop: 8 }}>
                  📝 {client.notes}
                </p>
              )}

              <p className="text-xs text-secondary" style={{ marginTop: 12 }}>
                Criado em {new Date(client.created_at).toLocaleDateString("pt-BR")}
              </p>
            </div>
          ))}
        </div>
      )}

      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>{editingClient ? "✏️ Editar Cliente" : "🤝 Novo Cliente"}</h2>

            <div className="form-group">
              <label>Nome *</label>
              <input
                className="form-input"
                placeholder="Nome do cliente"
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
              />
            </div>

            <div className="form-group">
              <label>Empresa</label>
              <input
                className="form-input"
                placeholder="Nome da empresa"
                value={form.company}
                onChange={(e) => setForm({ ...form, company: e.target.value })}
              />
            </div>

            <div className="form-group">
              <label>E-mail</label>
              <input
                className="form-input"
                type="email"
                placeholder="cliente@email.com"
                value={form.email}
                onChange={(e) => setForm({ ...form, email: e.target.value })}
              />
            </div>

            <div className="form-group">
              <label>Telefone</label>
              <input
                className="form-input"
                placeholder="(11) 99999-9999"
                value={form.phone}
                onChange={(e) => setForm({ ...form, phone: e.target.value })}
              />
            </div>

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
              <button className="btn btn-primary" onClick={saveClient}>
                {editingClient ? "Salvar" : "Criar"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ClientList;
