# 📡 API de Comandos Tauri — Unitesk

## 📁 Projetos

### `get_projects`
Retorna todos os projetos acadêmicos.
```typescript
const projects = await invoke("get_projects");
// Retorno: Project[]
```

### `create_project`
Cria um novo projeto.
```typescript
await invoke("create_project", {
  name: "Redes de Computadores",
  description: "Trabalho semestral sobre TCP/IP"
});
```

### `update_project`
Edita um projeto existente.
```typescript
await invoke("update_project", {
  id: 1,
  name: "Novo Nome",
  description: "Nova descrição"
});
```

### `delete_project`
Remove um projeto pelo ID (remove arquivos associados em cascata).
```typescript
await invoke("delete_project", { id: 1 });
```

## 📄 Artigos

### `get_articles`
Retorna todos os artigos.
```typescript
const articles = await invoke("get_articles");
// Retorno: Article[]
```

### `create_article`
Cria um novo artigo.
```typescript
await invoke("create_article", {
  title: "Introdução ao TCP/IP",
  content: "Conteúdo do artigo...",
  projectName: "Redes",
  projectId: 1  // opcional
});
```

### `delete_article`
Remove um artigo pelo ID.
```typescript
await invoke("delete_article", { id: 1 });
```

## 📅 Atividades

### `get_assignments`
Retorna todas as atividades ordenadas por data.
```typescript
const assignments = await invoke("get_assignments");
// Retorno: Assignment[]
```

### `create_assignment`
Cria uma nova atividade com prazo.
```typescript
await invoke("create_assignment", {
  title: "Entrega Artigo Redes",
  description: "Artigo sobre TCP/IP",
  dueDate: "2026-07-15",
  projectName: "Redes de Computadores"
});
```

### `mark_assignment_done`
Marca atividade como concluída.
```typescript
await invoke("mark_assignment_done", { id: 1 });
```

## 📎 Arquivos de Projeto

### `get_project_files`
Lista todos os arquivos anexados a um projeto.
```typescript
const files = await invoke("get_project_files", { projectId: 1 });
// Retorno: ProjectFile[]
```

### `add_project_file`
Anexa um arquivo a um projeto (dados em base64/array).
```typescript
await invoke("add_project_file", {
  projectId: 1,
  originalName: "artigo.pdf",
  fileData: [/* byte array */],
  mimeType: "application/pdf"
});
```

### `get_project_file_data`
Obtém os dados de um arquivo específico para download.
```typescript
const [name, mime, data] = await invoke("get_project_file_data", { fileId: 1 });
// Retorno: [string, string, number[]]
```

### `delete_project_file`
Remove um arquivo anexado pelo ID.
```typescript
await invoke("delete_project_file", { id: 1 });
```

## 📦 Exportar Projeto

### `export_project_zip`
Exporta o projeto completo (artigos + arquivos) como um arquivo ZIP.
```typescript
const [filename, zipData] = await invoke("export_project_zip", { projectId: 1 });
// Retorno: [string, number[]]
// Exemplo: ["Redes_Computadores.zip", [byte array]]
```

## 📊 Dashboard

### `get_dashboard_stats`
Retorna estatísticas do dashboard.
```typescript
const stats = await invoke("get_dashboard_stats");
// Retorno: { totalProjects, totalArticles, pendingAssignments, ... }
```

## 🔔 Notificações

### `check_today_assignments`
Verifica atividades com prazo hoje e dispara notificações.
```typescript
const todayAssignments = await invoke("check_today_assignments");
// Dispara notificações nativas automaticamente
```

---

## 📦 Modelos

```typescript
interface Project {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
}

interface Article {
  id: number;
  title: string;
  content: string | null;
  project_name: string | null;
  project_id: number | null;
  created_at: string;
}

interface Assignment {
  id: number;
  title: string;
  description: string | null;
  due_date: string;
  project_name: string | null;
  status: "pending" | "done" | "overdue";
  created_at: string;
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

interface DashboardStats {
  totalProjects: number;
  totalArticles: number;
  pendingAssignments: number;
  overdueAssignments: number;
  nextDeadline: string | null;
  nextDeadlineName: string | null;
}
```
