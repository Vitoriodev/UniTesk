import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Article {
  id: number;
  title: string;
  content: string;
  project_name: string;
  created_at: string;
}

type ArticleStatus = "draft" | "published" | "scheduled";
type FilterTab = "all" | "draft" | "published" | "scheduled";

interface ArticleExtended extends Article {
  status: ArticleStatus;
  scheduled_date?: string;
}

function ArticleManager() {
  const [articles, setArticles] = useState<ArticleExtended[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [viewingArticle, setViewingArticle] = useState<ArticleExtended | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [activeFilter, setActiveFilter] = useState<FilterTab>("all");
  const [newArticle, setNewArticle] = useState({
    title: "",
    content: "",
    project_name: "",
    scheduled_date: "",
  });

  useEffect(() => {
    loadArticles();
  }, []);

  async function loadArticles() {
    try {
      const data = await invoke<Article[]>("get_articles");
      const extended = data.map((a) => enrichArticle(a));
      setArticles(extended);
      localStorage.setItem("unitesk_articles", JSON.stringify(extended));
    } catch {
      const saved = localStorage.getItem("unitesk_articles");
      if (saved) {
        try {
          setArticles(JSON.parse(saved));
        } catch {
          setArticles([]);
        }
      } else {
        setArticles([]);
      }
    }
  }

  // Migração: ler status da chave antiga (v1) se a nova (v2) não existir
  function migrateOldStatuses(): Record<number, { status: ArticleStatus; scheduled_date?: string }> {
    const oldData = localStorage.getItem("unitesk_article_statuses");
    if (!oldData) return {};
    try {
      const oldMap: Record<number, string> = JSON.parse(oldData);
      const newMap: Record<number, { status: ArticleStatus; scheduled_date?: string }> = {};
      for (const [id, status] of Object.entries(oldMap)) {
        const s = status === "published" ? "published" : "draft";
        newMap[Number(id)] = { status: s };
      }
      localStorage.setItem("unitesk_article_statuses_v2", JSON.stringify(newMap));
      localStorage.removeItem("unitesk_article_statuses");
      return newMap;
    } catch {
      return {};
    }
  }

  // Salva/recupera todos os status em uma única chave do localStorage
  function getStatusMap(): Record<number, { status: ArticleStatus; scheduled_date?: string }> {
    try {
      const data = localStorage.getItem("unitesk_article_statuses_v2");
      if (!data) {
        // Tentar migrar da chave antiga
        return migrateOldStatuses();
      }
      return JSON.parse(data);
    } catch {
      return {};
    }
  }

  function saveStatusMap(map: Record<number, { status: ArticleStatus; scheduled_date?: string }>) {
    localStorage.setItem("unitesk_article_statuses_v2", JSON.stringify(map));
  }

  function enrichArticle(a: Article): ArticleExtended {
    const statusMap = getStatusMap();
    const entry = statusMap[a.id];
    if (entry) {
      return { ...a, status: entry.status, scheduled_date: entry.scheduled_date };
    }
    return { ...a, status: "draft" };
  }

  async function toggleArticleStatus(id: number) {
    const statusMap = getStatusMap();
    const entry = statusMap[id] || { status: "draft" as ArticleStatus };
    if (entry.status === "draft") {
      entry.status = "published";
    } else if (entry.status === "published") {
      entry.status = "draft";
    } else if (entry.status === "scheduled") {
      entry.status = "published";
      delete entry.scheduled_date;
    }
    statusMap[id] = entry;
    saveStatusMap(statusMap);

    const updatedArticles = articles.map((a) =>
      a.id === id ? { ...a, status: entry.status, scheduled_date: entry.scheduled_date } : a
    );
    setArticles(updatedArticles);
    localStorage.setItem("unitesk_articles", JSON.stringify(updatedArticles));
  }

  async function createArticle() {
    if (!newArticle.title) return;
    
    const hasSchedule = !!newArticle.scheduled_date;
    const initialStatus: ArticleStatus = hasSchedule ? "scheduled" : "draft";
    
    try {
      await invoke("create_article", {
        title: newArticle.title,
        content: newArticle.content,
        projectName: newArticle.project_name,
        scheduledDate: newArticle.scheduled_date || null,
      });
      resetForm();
      loadArticles();
    } catch {
      const newId = Date.now();
      const newArt: ArticleExtended = {
        id: newId,
        title: newArticle.title,
        content: newArticle.content,
        project_name: newArticle.project_name,
        created_at: new Date().toISOString(),
        status: initialStatus,
        scheduled_date: hasSchedule ? newArticle.scheduled_date : undefined,
      };
      // Salvar status no mapa
      const statusMap = getStatusMap();
      statusMap[newId] = { status: initialStatus, scheduled_date: hasSchedule ? newArticle.scheduled_date : undefined };
      saveStatusMap(statusMap);

      const updated = [...articles, newArt];
      setArticles(updated);
      localStorage.setItem("unitesk_articles", JSON.stringify(updated));
      resetForm();
    }
  }

  function resetForm() {
    setNewArticle({ title: "", content: "", project_name: "", scheduled_date: "" });
    setShowModal(false);
  }

  async function deleteArticle(id: number) {
    if (!confirm("Tem certeza que deseja excluir este artigo?")) return;
    try {
      await invoke("delete_article", { id });
      // Limpar status do artigo deletado
      const statusMap = getStatusMap();
      delete statusMap[id];
      saveStatusMap(statusMap);
      loadArticles();
    } catch {
      const updated = articles.filter((a) => a.id !== id);
      setArticles(updated);
      localStorage.setItem("unitesk_articles", JSON.stringify(updated));
      // Limpar status do artigo deletado
      const statusMap = getStatusMap();
      delete statusMap[id];
      saveStatusMap(statusMap);
    }
  }

  const filteredArticles = useMemo(() => {
    let filtered = articles;
    if (activeFilter === "draft") {
      filtered = filtered.filter((a) => a.status === "draft");
    } else if (activeFilter === "published") {
      filtered = filtered.filter((a) => a.status === "published");
    } else if (activeFilter === "scheduled") {
      filtered = filtered.filter((a) => a.status === "scheduled");
    }
    if (searchTerm) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter(
        (a) =>
          a.title.toLowerCase().includes(term) ||
          a.content.toLowerCase().includes(term) ||
          a.project_name.toLowerCase().includes(term)
      );
    }
    return filtered;
  }, [articles, searchTerm, activeFilter]);

  const stats = useMemo(() => {
    return {
      total: articles.length,
      drafts: articles.filter((a) => a.status === "draft").length,
      published: articles.filter((a) => a.status === "published").length,
      scheduled: articles.filter((a) => a.status === "scheduled").length,
    };
  }, [articles]);

  const getFilterTab = (tab: FilterTab, label: string, count: number) => (
    <button
      key={tab}
      className={`filter-tab ${activeFilter === tab ? "filter-tab--active" : ""}`}
      onClick={() => setActiveFilter(tab)}
    >
      {label} <span className="count-badge">{count}</span>
    </button>
  );

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 20,
        }}
      >
        <h2 style={{ fontSize: "1.4rem" }}>📄 Documentos Acadêmicos</h2>
        <button
          className="btn btn-primary"
          onClick={() => setShowModal(true)}
        >
          ➕ Novo Documento
        </button>
      </div>

      {/* Filtros de status */}
      <div className="filter-tabs">
      {getFilterTab("all", "Todos", stats.total)}
      {getFilterTab("draft", "📝 Rascunhos", stats.drafts)}
      {getFilterTab("scheduled", "📅 Agendados", stats.scheduled)}
      {getFilterTab("published", "✅ Prontos", stats.published)}
      </div>

      {/* Busca */}
      <div className="search-row">
        <input
          className="form-input"
          placeholder="🔍 Pesquisar documentos..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        {searchTerm && (
          <button
            className="btn btn-secondary btn-sm"
            onClick={() => setSearchTerm("")}
          >
            ✕ Limpar
          </button>
        )}
      </div>

      {filteredArticles.length === 0 ? (
        <div className="card" style={{ textAlign: "center", padding: 48 }}>
          <p style={{ fontSize: "3rem", marginBottom: 12 }}>📚</p>
          <p style={{ color: "var(--text-secondary)", fontSize: "1.1rem" }}>
            {searchTerm
              ? "Nenhum documento encontrado para esta busca."
              : activeFilter === "draft"
              ? "Nenhum rascunho ainda."
              : activeFilter === "scheduled"
              ? "Nenhum documento agendado."
              : activeFilter === "published"
              ? "Nenhum documento pronto ainda."
              : "Nenhum documento cadastrado."}
          </p>
          <p style={{ color: "var(--text-secondary)", marginBottom: 16 }}>
            {!searchTerm && "Adicione documentos para seus projetos acadêmicos!"}
          </p>
          {!searchTerm && (
            <button
              className="btn btn-primary"
              onClick={() => setShowModal(true)}
            >
              ➕ Adicionar Documento
            </button>
          )}
        </div>
      ) : (
        <div className="grid-2">
          {filteredArticles.map((article) => (
            <div className="card" key={article.id}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "flex-start",
                  marginBottom: 8,
                }}
              >
                <h3 style={{ fontWeight: 600, fontSize: "1rem", flex: 1, minWidth: 0 }}>
                  {article.title}
                </h3>
                <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                  <button
                    className={`btn btn-sm ${article.status === "published" ? "btn-outline-primary" : "btn-success"}`}
                    onClick={() => toggleArticleStatus(article.id)}
                    title={
                      article.status === "draft" ? "Marcar como pronto" : 
                      article.status === "scheduled" ? "Marcar como pronto" :
                      "Voltar para rascunho"
                    }
                  >
                    {article.status === "draft" || article.status === "scheduled" ? "✅" : "📝"}
                  </button>
                  <button
                    className="btn btn-danger btn-sm"
                    onClick={() => deleteArticle(article.id)}
                    title="Excluir"
                  >
                    🗑️
                  </button>
                </div>
              </div>

              <div className="article-card-status">
                <span className={`badge ${
                  article.status === "draft" ? "badge-draft" : 
                  article.status === "scheduled" ? "badge-scheduled" : "badge-published"
                }`}>
                  {article.status === "draft" ? "📝 Rascunho" : 
                   article.status === "scheduled" ? "📅 Agendado" : "✅ Pronto"}
                </span>
                {article.project_name && (
                  <span className="badge badge-progress">
                    📁 {article.project_name}
                  </span>
                )}
                <span style={{ fontSize: "0.8rem", color: "var(--text-secondary)" }}>
                  {new Date(article.created_at).toLocaleDateString("pt-BR")}
                </span>
              </div>

              {article.scheduled_date && (
                <div className="scheduled-date-info">
                  📅 Agendado para: {new Date(article.scheduled_date + "T12:00:00").toLocaleDateString("pt-BR", {
                    weekday: "long",
                    year: "numeric",
                    month: "long",
                    day: "numeric",
                  })}
                </div>
              )}

              <p
                style={{
                  color: "var(--text-secondary)",
                  fontSize: "0.85rem",
                  display: "-webkit-box",
                  WebkitLineClamp: 3,
                  WebkitBoxOrient: "vertical",
                  overflow: "hidden",
                  marginBottom: 12,
                  lineHeight: 1.5,
                }}
              >
                {article.content || "Sem conteúdo"}
              </p>

              <div style={{ display: "flex", gap: 8 }}>
                {article.content && (
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => setViewingArticle(article)}
                  >
                    📖 Ler mais
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal: View Article */}
      {viewingArticle && (
        <div
          className="modal-overlay"
          onClick={() => setViewingArticle(null)}
        >
          <div
            className="modal"
            onClick={(e) => e.stopPropagation()}
            style={{ maxWidth: 700 }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "flex-start",
                marginBottom: 16,
              }}
            >
              <div>
                <h2 style={{ marginBottom: 4 }}>{viewingArticle.title}</h2>
                <div className="article-card-status">
                  <span className={`badge ${
                    viewingArticle.status === "draft" ? "badge-draft" : 
                    viewingArticle.status === "scheduled" ? "badge-scheduled" : "badge-published"
                  }`}>
                    {viewingArticle.status === "draft" ? "📝 Rascunho" : 
                     viewingArticle.status === "scheduled" ? "📅 Agendado" : "✅ Pronto"}
                  </span>
                  {viewingArticle.project_name && (
                    <span className="badge badge-progress">
                      📁 {viewingArticle.project_name}
                    </span>
                  )}
                  <span
                    style={{
                      fontSize: "0.8rem",
                      color: "var(--text-secondary)",
                    }}
                  >
                    {new Date(viewingArticle.created_at).toLocaleDateString(
                      "pt-BR"
                    )}
                  </span>
                </div>
              </div>
              <button
                className="btn btn-secondary btn-sm"
                onClick={() => setViewingArticle(null)}
              >
                ✕
              </button>
            </div>
            <div
              style={{
                whiteSpace: "pre-wrap",
                lineHeight: 1.7,
                fontSize: "0.95rem",
                maxHeight: "50vh",
                overflowY: "auto",
                padding: "16px 0",
              }}
            >
              {viewingArticle.content || "Sem conteúdo"}
            </div>
          </div>
        </div>
      )}

      {/* Modal: New Article */}
      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>📄 Novo Documento</h2>
            <div className="form-group">
              <label>Título do Documento *</label>
              <input
                className="form-input"
                placeholder="Ex: Introdução às Redes Neurais"
                value={newArticle.title}
                onChange={(e) =>
                  setNewArticle({ ...newArticle, title: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Disciplina / Projeto</label>
              <input
                className="form-input"
                placeholder="Ex: Inteligência Artificial"
                value={newArticle.project_name}
                onChange={(e) =>
                  setNewArticle({
                    ...newArticle,
                    project_name: e.target.value,
                  })
                }
              />
            </div>
            <div className="form-group">
              <label>Conteúdo</label>
              <textarea
                className="form-textarea"
                placeholder="Cole o conteúdo do documento aqui..."
                value={newArticle.content}
                onChange={(e) =>
                  setNewArticle({ ...newArticle, content: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>📅 Agendar para (opcional)</label>
              <input
                className="form-input"
                type="date"
                value={newArticle.scheduled_date}
                onChange={(e) =>
                  setNewArticle({ ...newArticle, scheduled_date: e.target.value })
                }
              />
              <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)", marginTop: 4 }}>
                Selecione uma data futura para agendar a entrega deste documento.
              </p>
            </div>
            <div className="form-group">
              <p style={{ fontSize: "0.8rem", color: "var(--text-secondary)" }}>
                💡 {newArticle.scheduled_date 
                  ? `O documento será salvo como <strong>Agendado</strong> para ${new Date(newArticle.scheduled_date + "T12:00:00").toLocaleDateString("pt-BR")}.`
                  : `O documento será salvo como <strong>Rascunho</strong>. Você poderá marcá-lo como <strong>Pronto</strong> depois.`
                }
              </p>
            </div>
            <div className="modal-actions">
              <button
                className="btn btn-secondary"
                onClick={() => setShowModal(false)}
              >
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={createArticle}>
                Salvar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ArticleManager;
