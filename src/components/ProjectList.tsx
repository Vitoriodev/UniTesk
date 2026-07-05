import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Project {
  id: number;
  name: string;
  description: string;
  client_id: number | null;
  client_name: string | null;
  created_at: string;
}

interface Article {
  id: number;
  title: string;
  project_id: number;
}

interface Client {
  id: number;
  name: string;
  company: string | null;
}

interface ProjectFile {
  id: number;
  project_id: number;
  original_name: string;
  stored_name: string;
  file_size: number;
  mime_type: string;
  created_at: string;
}

function ProjectList() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [articles, setArticles] = useState<Article[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [showArticleModal, setShowArticleModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedProject, setSelectedProject] = useState<number | null>(null);
  const [editingProject, setEditingProject] = useState<Project | null>(null);
  const [newProject, setNewProject] = useState({ name: "", description: "", client_id: null as number | null });
  const [editProject, setEditProject] = useState({ name: "", description: "", client_id: null as number | null });
  const [newArticle, setNewArticle] = useState({ title: "", content: "" });
  const [projectFiles, setProjectFiles] = useState<Record<number, ProjectFile[]>>({});
  const [expandedFiles, setExpandedFiles] = useState<Record<number, boolean>>({});
  const [uploadingProject, setUploadingProject] = useState<number | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);
  const [downloadingFile, setDownloadingFile] = useState<number | null>(null);
  const [exportingProject, setExportingProject] = useState<number | null>(null);

  useEffect(() => {
    loadProjects();
    loadArticles();
    loadClients();
  }, []);

  async function loadProjects() {
    try {
      const data = await invoke<Project[]>("get_projects");
      setProjects(data);
    } catch {
      setProjects([]);
    }
  }

  async function loadClients() {
    try {
      const data = await invoke<Client[]>("get_clients");
      setClients(data);
    } catch {
      setClients([]);
    }
  }

  async function loadArticles() {
    try {
      const data = await invoke<Article[]>("get_articles");
      setArticles(data);
    } catch {
      setArticles([]);
    }
  }

  async function loadProjectFiles(projectId: number) {
    try {
      const data = await invoke<ProjectFile[]>("get_project_files", { projectId });
      setProjectFiles((prev) => ({ ...prev, [projectId]: data }));
    } catch {
      setProjectFiles((prev) => ({ ...prev, [projectId]: [] }));
    }
  }

  async function createProject() {
    if (!newProject.name) return;
    try {
      await invoke("create_project", {
        name: newProject.name,
        description: newProject.description,
        clientId: newProject.client_id,
      });
      setNewProject({ name: "", description: "", client_id: null });
      setShowModal(false);
      loadProjects();
    } catch {
      setProjects([
        ...projects,
        {
          id: Date.now(),
          name: newProject.name,
          description: newProject.description,
          client_id: newProject.client_id,
          client_name: newProject.client_id
            ? clients.find((c) => c.id === newProject.client_id)?.name || null
            : null,
          created_at: new Date().toISOString(),
        },
      ]);
      setNewProject({ name: "", description: "", client_id: null });
      setShowModal(false);
    }
  }

  async function deleteProject(id: number) {
    try {
      await invoke("delete_project", { id });
      setDeleteConfirm(null);
      loadProjects();
    } catch {
      setProjects(projects.filter((p) => p.id !== id));
      setDeleteConfirm(null);
    }
  }

  function openEditModal(project: Project) {
    setEditingProject(project);
    setEditProject({
      name: project.name,
      description: project.description || "",
      client_id: project.client_id,
    });
    setShowEditModal(true);
  }

  async function saveEditProject() {
    if (!editingProject || !editProject.name) return;
    try {
      await invoke("update_project", {
        id: editingProject.id,
        name: editProject.name,
        description: editProject.description,
        clientId: editProject.client_id,
      });
      setShowEditModal(false);
      setEditingProject(null);
      loadProjects();
    } catch {
      setProjects(
        projects.map((p) =>
          p.id === editingProject.id
            ? {
                ...p,
                name: editProject.name,
                description: editProject.description,
                client_id: editProject.client_id,
                client_name: editProject.client_id
                  ? clients.find((c) => c.id === editProject.client_id)?.name || null
                  : null,
              }
            : p
        )
      );
      setShowEditModal(false);
      setEditingProject(null);
    }
  }

  async function createArticle() {
    if (!newArticle.title || !selectedProject) return;
    try {
      await invoke("create_article", { title: newArticle.title, content: newArticle.content, projectId: selectedProject });
      setNewArticle({ title: "", content: "" });
      setShowArticleModal(false);
      loadArticles();
    } catch {
      setArticles([...articles, { id: Date.now(), title: newArticle.title, project_id: selectedProject }]);
      setNewArticle({ title: "", content: "" });
      setShowArticleModal(false);
    }
  }

  function handleFileSelect(projectId: number) {
    setUploadingProject(projectId);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
      fileInputRef.current.click();
    }
  }

  const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10 MB

  async function uploadFile(e: React.ChangeEvent<HTMLInputElement>) {
    const files = e.target.files;
    if (!files || files.length === 0 || uploadingProject === null) return;

    const file = files[0];

    if (file.size > MAX_FILE_SIZE) {
      alert("⚠️ Arquivo muito grande! O tamanho máximo é de 10 MB.");
      setUploadingProject(null);
      return;
    }

    try {
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      await invoke("add_project_file", {
        projectId: uploadingProject,
        originalName: file.name,
        fileData: Array.from(uint8Array),
        mimeType: file.type || "application/octet-stream",
      });

      await loadProjectFiles(uploadingProject);
    } catch (err) {
      console.error("Erro ao enviar arquivo:", err);
      alert("❌ Erro ao enviar arquivo. Verifique o tamanho e tente novamente.");
    }

    setUploadingProject(null);
  }

  async function downloadFile(file: ProjectFile) {
    setDownloadingFile(file.id);
    try {
      const result = await invoke<[string, string, number[]]>("get_project_file_data", { fileId: file.id });
      const [originalName, mimeType, dataArray] = result;
      const uint8Array = new Uint8Array(dataArray);
      const blob = new Blob([uint8Array], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = originalName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Erro ao baixar arquivo:", err);
      alert("❌ Erro ao baixar arquivo. Tente novamente.");
    }
    setDownloadingFile(null);
  }

  async function deleteFile(fileId: number, projectId: number) {
    try {
      await invoke("delete_project_file", { id: fileId });
      await loadProjectFiles(projectId);
    } catch {
      setProjectFiles((prev) => ({
        ...prev,
        [projectId]: (prev[projectId] || []).filter((f) => f.id !== fileId),
      }));
    }
  }

  async function exportProjectZip(projectId: number) {
    setExportingProject(projectId);
    try {
      const result = await invoke<[string, number[]]>("export_project_zip", { projectId });
      const [filename, dataArray] = result;
      const uint8Array = new Uint8Array(dataArray);
      const blob = new Blob([uint8Array], { type: "application/zip" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Erro ao exportar projeto:", err);
      alert("❌ Erro ao exportar projeto. Tente novamente.");
    }
    setExportingProject(null);
  }

  function toggleFiles(projectId: number) {
    const willExpand = !expandedFiles[projectId];
    setExpandedFiles((prev) => ({ ...prev, [projectId]: willExpand }));
    if (willExpand && !projectFiles[projectId]) {
      loadProjectFiles(projectId);
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  const getProjectArticles = (projectId: number) =>
    articles.filter((a) => a.project_id === projectId);

  return (
    <div>
      <div className="flex-between">
        <h2 className="section-title">📁 Projetos</h2>
        <button className="btn btn-primary" onClick={() => setShowModal(true)}>
          ➕ Novo Projeto
        </button>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept=".pdf,.doc,.docx,.txt,.png,.jpg,.jpeg,.zip"
        className="hidden-input"
        onChange={uploadFile}
      />

      {projects.length === 0 ? (
        <div className="card text-center" style={{ padding: 48 }}>
          <p className="empty-icon">🎓</p>
          <p className="text-secondary" style={{ fontSize: "1.1rem" }}>
            Nenhum projeto ainda.
          </p>
          <p className="text-secondary" style={{ marginBottom: 16 }}>
            Crie seu primeiro projeto!
          </p>
          <button className="btn btn-primary" onClick={() => setShowModal(true)}>
            ➕ Criar Projeto
          </button>
        </div>
      ) : (
        <div className="grid-2">
          {projects.map((project) => {
            const projectArticles = getProjectArticles(project.id);
            const files = projectFiles[project.id] || [];
            const isExpanded = expandedFiles[project.id] || false;
            return (
              <div className="card" key={project.id}>
                <div className="flex-between" style={{ marginBottom: 12 }}>
                  <h3 className="project-name">{project.name}</h3>
                  <div className="flex gap-6">
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => openEditModal(project)}
                      title="Editar projeto"
                    >
                      ✏️
                    </button>
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={() => {
                        setSelectedProject(project.id);
                        setShowArticleModal(true);
                      }}
                    >
                      📄 + Artigo
                    </button>
                  </div>
                </div>
                <p className="text-secondary text-sm" style={{ marginBottom: 12 }}>
                  {project.description || "Sem descrição"}
                </p>
                <div className="meta-info">
                  {project.client_name && (
                    <>
                      <span className="badge badge-progress">🤝 {project.client_name}</span>
                      <span>•</span>
                    </>
                  )}
                  <span>📄 {projectArticles.length} artigo(s)</span>
                  <span>•</span>
                  <span>
                    Criado em{" "}
                    {new Date(project.created_at).toLocaleDateString("pt-BR")}
                  </span>
                </div>

                <div className="flex gap-8" style={{ marginBottom: 12, flexWrap: "wrap" }}>
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => handleFileSelect(project.id)}
                  >
                    📎 Anexar Arquivo
                  </button>
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => toggleFiles(project.id)}
                  >
                    {isExpanded ? "📂 Ocultar Arquivos" : "📂 Ver Arquivos"}
                    {files.length > 0 && (
                      <span className="file-count-badge">{files.length}</span>
                    )}
                  </button>
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => exportProjectZip(project.id)}
                    disabled={exportingProject === project.id}
                    title="Exportar projeto como ZIP"
                  >
                    {exportingProject === project.id ? "⏳" : "📦 Exportar ZIP"}
                  </button>
                  {deleteConfirm === project.id ? (
                    <div className="flex gap-6" style={{ marginLeft: "auto" }}>
                      <button
                        className="btn btn-danger btn-sm"
                        onClick={() => deleteProject(project.id)}
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
                      style={{ marginLeft: "auto" }}
                      onClick={() => setDeleteConfirm(project.id)}
                      title="Excluir projeto"
                    >
                      🗑️
                    </button>
                  )}
                </div>

                {isExpanded && (
                  <div className="files-section">
                    <p className="files-section-title">📎 Arquivos Anexados</p>
                    {files.length === 0 ? (
                      <p className="text-secondary text-italic text-xs">
                        Nenhum arquivo anexado ainda.
                      </p>
                    ) : (
                      <div className="flex-col gap-6">
                        {files.map((file) => (
                          <div key={file.id} className="file-row">
                            <div className="file-info">
                              <span>
                                {file.mime_type.includes("pdf")
                                  ? "📕"
                                  : file.mime_type.includes("image")
                                  ? "🖼️"
                                  : file.mime_type.includes("zip")
                                  ? "📦"
                                  : "📄"}
                              </span>
                              <span className="file-name">{file.original_name}</span>
                              <span className="file-size">{formatFileSize(file.file_size)}</span>
                            </div>
                            <div className="flex gap-4">
                              <button
                                className="btn btn-secondary btn-xs"
                                onClick={() => downloadFile(file)}
                                disabled={downloadingFile === file.id}
                              >
                                {downloadingFile === file.id ? "⏳" : "⬇️"}
                              </button>
                              <button
                                className="btn btn-danger btn-xs"
                                onClick={() => deleteFile(file.id, project.id)}
                              >
                                ✕
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}

                {projectArticles.length > 0 && (
                  <div className="articles-preview">
                    <p className="articles-preview-title">Artigos:</p>
                    {projectArticles.map((article) => (
                      <p key={article.id} className="article-link">
                        📄 {article.title}
                      </p>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {showModal && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>📁 Novo Projeto</h2>
            <div className="form-group">
              <label>Nome do Projeto</label>
              <input
                className="form-input"
                placeholder="Ex: Trabalho de Redes"
                value={newProject.name}
                onChange={(e) =>
                  setNewProject({ ...newProject, name: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Descrição</label>
              <textarea
                className="form-textarea"
                placeholder="Descreva o projeto..."
                value={newProject.description}
                onChange={(e) =>
                  setNewProject({ ...newProject, description: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Cliente (opcional)</label>
              <select
                className="form-input"
                value={newProject.client_id ?? ""}
                onChange={(e) =>
                  setNewProject({
                    ...newProject,
                    client_id: e.target.value ? Number(e.target.value) : null,
                  })
                }
              >
                <option value="">Sem cliente</option>
                {clients.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}{c.company ? ` (${c.company})` : ""}
                  </option>
                ))}
              </select>
            </div>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowModal(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={createProject}>
                Criar
              </button>
            </div>
          </div>
        </div>
      )}

      {showEditModal && editingProject && (
        <div className="modal-overlay" onClick={() => setShowEditModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>✏️ Editar Projeto</h2>
            <div className="form-group">
              <label>Nome do Projeto</label>
              <input
                className="form-input"
                placeholder="Ex: Trabalho de Redes"
                value={editProject.name}
                onChange={(e) =>
                  setEditProject({ ...editProject, name: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Descrição</label>
              <textarea
                className="form-textarea"
                placeholder="Descreva o projeto..."
                value={editProject.description}
                onChange={(e) =>
                  setEditProject({ ...editProject, description: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Cliente</label>
              <select
                className="form-input"
                value={editProject.client_id ?? ""}
                onChange={(e) =>
                  setEditProject({
                    ...editProject,
                    client_id: e.target.value ? Number(e.target.value) : null,
                  })
                }
              >
                <option value="">Sem cliente</option>
                {clients.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}{c.company ? ` (${c.company})` : ""}
                  </option>
                ))}
              </select>
            </div>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowEditModal(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={saveEditProject}>
                Salvar
              </button>
            </div>
          </div>
        </div>
      )}

      {showArticleModal && (
        <div className="modal-overlay" onClick={() => setShowArticleModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>📄 Novo Artigo</h2>
            <div className="form-group">
              <label>Título do Artigo</label>
              <input
                className="form-input"
                placeholder="Ex: Artigo sobre TCP/IP"
                value={newArticle.title}
                onChange={(e) =>
                  setNewArticle({ ...newArticle, title: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>Conteúdo</label>
              <textarea
                className="form-textarea"
                placeholder="Cole ou escreva o conteúdo do artigo..."
                value={newArticle.content}
                onChange={(e) =>
                  setNewArticle({ ...newArticle, content: e.target.value })
                }
              />
            </div>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowArticleModal(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={createArticle}>
                Salvar Artigo
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ProjectList;
