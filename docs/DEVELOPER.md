# 🎓 Unitesk — Documentação para Desenvolvedores

> **Propósito:** Documentação completa do projeto Unitesk (Gerenciador de Projetos Acadêmicos),  
> criada para que **desenvolvedores humanos e IAs** possam entender, manter e estender o código.

---

## 📋 Sumário

1. [Visão Geral](#-visão-geral)
2. [Repositório Oficial](#-repositório-oficial)
3. [Stack Tecnológica](#-stack-tecnológica)
4. [Estrutura do Projeto](#-estrutura-do-projeto)
5. [Arquitetura](#-arquitetura)
6. [Banco de Dados](#-banco-de-dados)
7. [Backend Rust/Tauri](#-backend-rusttauri)
8. [Frontend React](#-frontend-react)
9. [Sistema de Notificações](#-sistema-de-notificações)
10. [Gerenciamento de Arquivos](#-gerenciamento-de-arquivos)
11. [Sistema de Calendário](#-sistema-de-calendário)
12. [Fallback localStorage](#-fallback-localstorage)
13. [Testes](#-testes)
14. [Instalação e Build](#-instalação-e-build)
15. [Scripts de Instalação](#-scripts-de-instalação)
16. [Git Workflow](#-git-workflow)
17. [Como Adicionar uma Nova Funcionalidade](#-como-adicionar-uma-nova-funcionalidade)
18. [Padrões de Código](#-padrões-de-código)
19. [Troubleshooting](#-troubleshooting)

---

## 📖 Visão Geral

O **Unitesk** é um aplicativo desktop para gerenciamento de projetos acadêmicos.  
Ele permite organizar **projetos**, **artigos**, **atividades com prazo** e **arquivos**  
em uma interface amigável, com calendário interativo e notificações nativas.

### Funcionalidades Principais

| Funcionalidade | Frontend | Comando Tauri (Backend) |
|---|---|---|
| **Dashboard** com estatísticas + timeline | `Dashboard.tsx` | `get_dashboard_stats` |
| **CRUD de Projetos** | `ProjectList.tsx` | `get/create/update/delete_project` |
| **CRUD de Artigos** | `ArticleManager.tsx` | `get/create/delete_article` |
| **Status Rascunho/Pronto** (Artigos) | `ArticleManager.tsx` | localStorage `unitesk_article_statuses` |
| **Filtros por status** | `ArticleManager.tsx` | Abas Todos/Rascunhos/Prontos |
| **Calendário de Atividades** | `CalendarView.tsx` | `get/create/mark_done assignments` |
| **Seletores mês/ano** | `CalendarView.tsx` | Dropdowns + botão "Hoje" |
| **Upload/Download de Arquivos** (projetos) | `ProjectList.tsx` | `get/add/get_data/delete_project_file` |
| **Upload/Download de Arquivos** (atividades) | `CalendarView.tsx` | `get/add/get_data/delete_assignment_file` |
| **Exportar ZIP** | `ProjectList.tsx` | `export_project_zip` |
| **Excluir Atividades** | `CalendarView.tsx` | `delete_assignment` |
| **Exportar/Importar Backup** | `Dashboard.tsx` | `export_all_data` / `import_all_data` |
| **Tema Dracula** | `App.tsx` + `global.css` | data-theme="dracula" |
| **Notificações nativas** | Automático (setInterval 60s) | `check_today_assignments` |

---

---

## 🌐 Repositório Oficial

> **URL do repositório:** [https://github.com/Vitoriodev/UniTesk](https://github.com/Vitoriodev/UniTesk)

Sempre use este link para clonar, fazer push e colaborar no projeto.

### Comandos Git Rápidos

```bash
# Clonar o projeto
git clone https://github.com/Vitoriodev/UniTesk.git

# Verificar remote configurado
git remote -v
# → origin  https://github.com/Vitoriodev/UniTesk.git (fetch)
# → origin  https://github.com/Vitoriodev/UniTesk.git (push)

# Commitar e enviar alterações
git add -A
git commit -m "mensagem descritiva"
git push origin main
```

> ⚠️ **Importante:** O remote `origin` já está configurado para o URL correto.
> Não altere para outro repositório. Consulte esta seção sempre que precisar
> confirmar o endereço antes de um `git push`.

---

## 🛠 Stack Tecnológica

| Camada | Tecnologia | Versão |
|---|---|---|
| **Frontend** | React + TypeScript | 18.x / 5.x |
| **Build Frontend** | Vite | 6.x |
| **Backend Desktop** | Rust + Tauri | 2.x |
| **Banco de Dados** | PostgreSQL + SQLx | 14+ / 0.8 |
| **Comunicação** | Tauri IPC (invoke) | — |
| **Notificações Nativas** | tauri-plugin-notification | 2.x |
| **Serialização** | serde + serde_json | 1.x |
| **UUID** | uuid (v4) | 1.x |
| **ZIP** | zip crate | 2.x |
| **Testes Frontend** | Vitest + Testing Library | 4.x / 16.x |

### Dependências de Sistema (Linux)

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libjavascriptcoregtk-4.1-dev \
  libssl-dev \
  postgresql postgresql-client
```

---

## 📁 Estrutura do Projeto

```
unitesk/
├── src/                          # Frontend React + TypeScript
│   ├── main.tsx                  # Entry point do React
│   ├── App.tsx                   # Componente principal (navegação por abas)
│   ├── components/
│   │   ├── Dashboard.tsx         # Visão geral com 4 cards de estatísticas
│   │   ├── ProjectList.tsx       # CRUD de projetos + artigos + arquivos + ZIP
│   │   ├── CalendarView.tsx      # Calendário + atividades + notificações + arquivos
│   │   └── ArticleManager.tsx    # CRUD de artigos com busca
│   ├── styles/
│   │   └── global.css            # Estilos globais (CSS custom properties)
│   └── test/                     # Testes com Vitest
│       ├── setup.ts
│       ├── App.test.tsx
│       ├── CalendarView.test.tsx
│       ├── ProjectList.test.tsx
│       ├── Dashboard.test.tsx
│       └── ArticleManager.test.tsx
│
├── src-tauri/                    # Backend Rust + Tauri
│   ├── src/
│   │   ├── main.rs               # Entry point do Tauri
│   │   ├── lib.rs                # Comandos Tauri + setup do app
│   │   ├── db.rs                 # Operações de banco (CRUD + migrações)
│   │   ├── models.rs             # Structs de dados (serde + sqlx::FromRow)
│   │   └── setup_launcher.c      # Código-fonte do binário unitesk-setup
│   ├── Cargo.toml                # Dependências Rust
│   ├── tauri.conf.json           # Configuração do Tauri (janela, build, .deb)
│   ├── public/                    # Assets estáticos (favicon)
│   │   └── icon.png              # Cópia do ícone para o Vite
│   └── icons/
│       └── unitesk_icon_512.png  # Ícone personalizado do aplicativo
│
├── docs/                         # Documentação
│   ├── DEVELOPER.md              # [ESTE ARQUIVO] Documentação para devs
│   ├── README.md                 # README principal
│   ├── LEIGO.md                  # Guia rápido para usuários
│   ├── ARCHITECTURE.md           # Arquitetura do projeto
│   ├── DATABASE.md               # Documentação do banco
│   ├── API.md                    # API de comandos Tauri
│   ├── WINDOWS.md                # Plano para versão Windows
│   └── setup.sql                 # Schema SQL (setup manual)
│
├── scripts/
│   └── notify-deadlines.sh       # Script cron para notificações fora do app
│
├── package.json                  # Dependências npm e scripts
├── vite.config.ts                # Configuração do Vite
├── tsconfig.json                 # Configuração do TypeScript
├── vitest.config.ts              # Configuração do Vitest
├── index.html                    # HTML entry point
│
├── install.sh                    # Instalador via terminal
├── setup.sh                      # Assistente de instalação com Zenity (GUI)
├── uninstall.sh                  # Desinstalador via terminal
├── unitesk.sh                    # Script wrapper para executar o app
├── unitesk-setup                 # Binário compilado do assistente (C)
└── build-deb.sh                  # Script para gerar pacote .deb
```

---

## 🔄 Arquitetura

### Fluxo de Dados

```
┌─────────────────────────────────────────────────────────┐
│                    React (Frontend)                       │
│                                                          │
│  Componente → invoke("comando", { args }) → Tauri IPC   │
│                  ↑                            ↓          │
│         localStorage (fallback)          Rust Backend    │
│                                              ↓          │
│                                          SQLx queries    │
│                                              ↓          │
│                                       PostgreSQL DB      │
└─────────────────────────────────────────────────────────┘
```

### Comunicação Frontend ↔ Backend

1. O frontend chama `invoke("nome_do_comando", { argumentos })` (dinamicamente importado de `@tauri-apps/api/core`)
2. O Tauri roteia para a função Rust correspondente (anotada com `#[tauri::command]`)
3. O Rust executa a query SQLx e retorna o resultado
4. **Se o Tauri não estiver disponível** (desenvolvimento em navegador), o `catch` usa **localStorage** como fallback

> ⚠️ **Importante:** Toda comunicação é **tipada**. Os tipos TypeScript no frontend  
> devem corresponder exatamente aos structs Rust (com `#[derive(Serialize, Deserialize)]`).  
> Tauri 2 faz conversão automática `camelCase` (JS) ↔ `snake_case` (Rust).

### Conversão automática de nomes (Tauri 2)

| JavaScript (camelCase) | Rust (snake_case) |
|---|---|
| `dueDate` | `due_date` |
| `projectName` | `project_name` |
| `assignmentId` | `assignment_id` |
| `originalName` | `original_name` |
| `fileData` | `file_data` |
| `mimeType` | `mime_type` |

### Fallback automático

Cada função CRUD no frontend segue o padrão **try-Tauri / catch-localStorage**:

```typescript
async function loadData() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const data = await invoke("get_data");
    setData(data);
    localStorage.setItem("backup", JSON.stringify(data));
  } catch {
    const saved = localStorage.getItem("backup");
    if (saved) setData(JSON.parse(saved));
    else setData([]);
  }
}
```

---

## 🗄 Banco de Dados

### Tabelas

```
┌─────────────┐    ┌─────────────────┐    ┌─────────────────────┐
│   projects  │    │    articles      │    │    assignments      │
├─────────────┤    ├─────────────────┤    ├─────────────────────┤
│ id (PK)     │    │ id (PK)          │    │ id (PK)             │
│ name        │◄───│ project_id (FK)──┘    │ title               │
│ description │    │ title            │    │ description          │
│ created_at  │    │ content          │    │ due_date (DATE)     │
└─────────────┘    │ project_name     │    │ due_time (TIME)     │
       │           │ created_at       │    │ notification_time(T)│
       │           └─────────────────┘    │ project_name        │
       │                                  │ status (pending/    │
       │                                  │   done/overdue)     │
       │                                  │ created_at          │
       │                                  └─────────────────────┘
       │                                              │
       │    ┌─────────────────────┐     ┌─────────────────────────┐
       │    │   project_files     │     │   assignment_files      │
       │    ├─────────────────────┤     ├─────────────────────────┤
       └────│ project_id (FK) CASCADE   │ assignment_id (FK) CASCADE
            │ original_name      │     │ original_name          │
            │ stored_name        │     │ stored_name            │
            │ file_data (BYTEA)  │     │ file_data (BYTEA)      │
            │ file_size          │     │ file_size              │
            │ mime_type          │     │ mime_type              │
            │ created_at         │     │ created_at             │
            └─────────────────────┘     └─────────────────────────┘
```

### Migrações automáticas

O banco é gerenciado automaticamente pelo Rust em `db.rs::init_db()`:

1. `CREATE TABLE IF NOT EXISTS` para todas as tabelas
2. `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` para colunas adicionadas em versões posteriores (ex: `due_time` e `notification_time` adicionados na v1.0)

**Não é necessário rodar migrations manualmente** — o app cria/atualiza as tabelas na inicialização.

### Índices

```sql
-- Criados automaticamente pelo PostgreSQL para chaves estrangeiras
-- Índices recomendados para performance:
CREATE INDEX idx_assignments_due_date ON assignments(due_date);
CREATE INDEX idx_assignments_status ON assignments(status);
CREATE INDEX idx_articles_project_id ON articles(project_id);
CREATE INDEX idx_project_files_project_id ON project_files(project_id);
CREATE INDEX idx_assignment_files_assignment_id ON assignment_files(assignment_id);
```

---

## 🔧 Backend Rust/Tauri

### models.rs

Define os structs que representam as tabelas do banco. Cada struct:

- Deriva `Debug`, `Serialize`, `Deserialize`, `sqlx::FromRow`
- Usa `Option<T>` para campos anuláveis
- Campos de data/hora são `String` (convertidos via `::text` no SQL)

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Assignment {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub due_date: String,           // YYYY-MM-DD
    pub due_time: Option<String>,   // HH:MM
    pub notification_time: Option<String>, // HH:MM (mesmo valor que due_time)
    pub project_name: Option<String>,
    pub status: String,             // "pending" | "done" | "overdue"
    pub created_at: String,         // TIMESTAMP como string
}
```

**`DashboardStats`** usa `#[serde(rename = "camelCase")]` para expor campos  
em camelCase para o frontend JavaScript.

### db.rs — Operações de banco

Organizado por seção de comentário:

- `init_db()` — Conexão + criação de tabelas + migrações
- `// === Projetos ===` — CRUD de projetos
- `// === Arquivos de Projeto ===` — CRUD de arquivos
- `// === Artigos ===` — CRUD de artigos
- `// === Atividades (Assignments) ===` — CRUD de atividades + update_overdue
- `// === Arquivos de Atividades ===` — CRUD de arquivos de atividade
- `// === Dashboard ===` — Estatísticas agregadas
- `// === Exportação ZIP ===` — Geração de ZIP
- `get_today_assignments()` — Busca atividades para notificação

**Padrão de queries SQL:**

```rust
// INSERT com RETURNING (devolve o registro criado)
sqlx::query_as::<_, Assignment>("INSERT INTO assignments (...) VALUES ($1, $2) RETURNING id, ...")
    .bind(valor)
    .fetch_one(pool)
    .await

// SELECT com filtro
sqlx::query_as::<_, Assignment>("SELECT ... WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)  // ou .fetch_all(pool)
    .await

// Cast de tipos: use ::text para converter DATE/TIME/TIMESTAMP para string
"SELECT due_date::text as due_date, due_time::text as due_time FROM assignments"
```

### lib.rs — Comandos Tauri

Cada comando Tauri segue o padrão:

```rust
#[tauri::command]
async fn nome_do_comando(
    state: tauri::State<'_, AppState>,  // Acesso ao banco via pool
    parametro1: String,                  // snake_case
    parametro2: Option<String>,          // Opcionais = Option<T>
) -> Result<Retorno, String> {           // Erro como String
    db::funcao(&state.pool, &parametro1).await
        .map_err(|e| e.to_string())
}
```

**Registro de comandos:** Todos os comandos devem ser listados em `.invoke_handler(tauri::generate_handler![...])`.

**AppState:** Estado compartilhado contendo o `PgPool`.

**Inicialização:** Em `run()` → `setup()`:
1. Lê `DATABASE_URL` da env var (fallback: `postgres://postgres@localhost:5432/academic_manager`)
2. Conecta ao PostgreSQL via `db::init_db()`
3. Gerencia o `AppState` com o pool
4. Dispara `update_overdue_assignments` em background (spawn)

### Notificações Nativas

Usam o plugin `tauri-plugin-notification`:

```rust
app.notification()
    .builder()
    .title("📚 Prazo Hoje!")
    .body(format!("A atividade '{}' vence hoje às {}!", title, time))
    .show();
```

---

## ⚛️ Frontend React

### Componentes

| Componente | Arquivo | Estado | LocalStorage Key |
|---|---|---|---|
| Dashboard | `Dashboard.tsx` | `stats`, `assignments`, import/export | — |
| ProjectList | `ProjectList.tsx` | `projects`, `articles`, `projectFiles`, `expandedFiles` | — |
| CalendarView | `CalendarView.tsx` | `assignments`, modals, `assignmentFiles`, `currentMonth`, `currentYear` | `unitesk_assignments` |
| ArticleManager | `ArticleManager.tsx` | `articles`, `searchTerm`, `activeFilter`, `viewingArticle` | `unitesk_articles`, `unitesk_article_statuses` |
| App | `App.tsx` | `activeTab` | — |

### Padrão de import dinâmico do Tauri

Todos os componentes usam import dinâmico para evitar erros em modo browser:

```typescript
const { invoke } = await import("@tauri-apps/api/core");
const data = await invoke<Tipo>("comando", { args });
```

Isso permite que o frontend rode tanto como app Tauri desktop quanto em  
navegador (desenvolvimento com `npm run dev`).

### CalendarView.tsx — Funcionalidades

O componente mais complexo. Gerencia:

1. **Calendário interativo** — Navegação mensal com **dropdowns de mês/ano** e botão "Hoje"
2. **Prevenção de datas obsoletas** — Funções `getTodayStr()` e `getNowTime()` em vez de constantes
3. **Modal de criação** — Data, horário de notificação, título, disciplina, descrição
4. **Lista de atividades** — Ordenada por data, com badges de status
5. **Enriquecimento de status** — `enrichAssignmentStatus()` marca pendentes como atrasados
6. **Modal de arquivos** — Upload, download, exclusão de arquivos por atividade
7. **Verificação periódica de notificações** — `setInterval` a cada 60 segundos
8. **Fallback de notificações** — `Notification API` do navegador quando sem Tauri

### Navegação do Calendário (v1.1.0)

```typescript
const [currentMonth, setCurrentMonth] = useState(new Date().getMonth());
const [currentYear, setCurrentYear] = useState(new Date().getFullYear());

// Dropdowns para navegação rápida:
// - <select> para mês (Janeiro-Dezembro)
// - <select> para ano (currentYear-10 até currentYear+5)
// - Botão "📅 Hoje" para voltar ao mês atual

// Funções getter para evitar valores stale:
function getTodayStr(): string { /* retorna YYYY-MM-DD atual */ }
function getNowTime(): string { /* retorna HH:MM atual */ }
```

### Estados do CalendarView

```typescript
const [assignments, setAssignments] = useState<Assignment[]>([]);
const [showModal, setShowModal] = useState(false);
const [selectedDate, setSelectedDate] = useState<string>("");
const [newTime, setNewTime] = useState<string>("");
const [errorMessage, setErrorMessage] = useState<string | null>(null);
const [showFilesModal, setShowFilesModal] = useState(false);
const [assignmentFiles, setAssignmentFiles] = useState<AssignmentFile[]>([]);
const [uploadingFile, setUploadingFile] = useState(false);
const fileInputRef = useRef<HTMLInputElement>(null);
```

---

## 🔔 Sistema de Notificações

### Arquitetura

O sistema de notificações opera em **3 camadas**:

#### 1. Backend Rust (check_today_assignments)

Chamado pelo frontend a cada 60 segundos. A query SQL:

```sql
SELECT ... FROM assignments 
WHERE due_date = CURRENT_DATE 
  AND status = 'pending' 
  AND notification_time BETWEEN CURRENT_TIME - INTERVAL '30 seconds' 
                             AND CURRENT_TIME + INTERVAL '30 seconds'
```

- Filtra por **data de hoje** + **status pendente** + **janela de 60 segundos** (-30s a +30s)
- Se `notification_time` é NULL, a condição `BETWEEN` falha (NULL não é comparável) → **sem notificação**
- Dispara notificação nativa do sistema via `tauri-plugin-notification`

#### 2. Frontend (fallback browser)

Quando o Tauri não está disponível (desenvolvimento em navegador):

```typescript
const notifTime = a.notification_time || a.due_time;
if (nowMinutes >= activityMinutes - 5 && nowMinutes <= activityMinutes + 5) {
  new Notification("📚 Prazo Hoje!", { body: `...` });
}
```

Usa a **Notification API** do navegador, com janela de 10 minutos (-5 a +5 min).

#### 3. Script Cron (fora do app)

`scripts/notify-deadlines.sh` — executado pelo crontab do sistema.  
Faz consulta direta ao PostgreSQL e dispara `notify-send`.

### Fluxo Completo

```
App Aberto:
┌──────────┐     cada 60s      ┌───────────┐     SQL      ┌──────────┐
│ Frontend │ ────────────────► │ Rust      │ ──────────► │ PostgreSQL│
│ setInterval│                │ check_today│ ◄────────── │          │
│ 60s      │ ◄─────────────── │ assignments│     dados   └──────────┘
└──────────┘     notificação   └───────────┘
     │             nativa
     │ (fallback)
     ▼
Browser Notification API
     ou
nada (Tauri disponível)

App Fechado:
crontab ──► notify-deadlines.sh ──► psql ──► notify-send
```

### Prevenção de Notificações Duplicadas

- A janela SQL de 60 segundos (-30s a +30s) com intervalo de 60s no frontend  
  garante **no máximo 1-2 notificações** por atividade
- Atividades **sem notification_time** (NULL) nunca disparam notificação automática
- O script cron (`notify-deadlines.sh`) é independente e roda apenas quando agendado

---

## 📎 Gerenciamento de Arquivos

### Projetos (ProjectList.tsx)

- Upload via `<input type="file">` oculto, acionado por botão "Anexar Arquivo"
- Lê o arquivo como `ArrayBuffer`, converte para `Uint8Array[]`
- Envia para o Rust via `invoke("add_project_file", { projectId, originalName, fileData, mimeType })`
- **Limite:** 10 MB por arquivo (validado no frontend)
- Armazenamento: `BYTEA` no PostgreSQL
- Download: `invoke("get_project_file_data")` → `Blob` → `URL.createObjectURL` → click
- Lista expansível por projeto (botão "Ver Arquivos")

### Atividades (CalendarView.tsx)

- Mesmo padrão dos projetos, mas vinculado a `assignment_files` (FK → assignments)
- Modal dedicado "📎 Arquivos" aberto por botão em cada card de atividade
- Ícones dinâmicos por tipo MIME: 📕 PDF, 🖼️ imagem, 📦 ZIP, 📄 outros
- Tamanho formatado (B, KB, MB)
- Botões de 📥 download e 🗑️ exclusão

### Fluxo de Upload

```
1. Usuário clica "Anexar Arquivo"
2. fileInputRef.current.click() → abre seletor de arquivos
3. onChange → valida tamanho (max 10 MB)
4. file.arrayBuffer() → Uint8Array
5. invoke("add_*_file", { ..., fileData: [...uint8] })
6. Rust: gera UUID + stored_name → INSERT INTO ... RETURNING
7. Recarrega lista de arquivos do backend
```

---

## 📅 Sistema de Calendário

### Navegação

- Estado: `currentMonth` (0-11) e `currentYear`
- Botões ← → para navegar entre meses
- Grade 7×N com dias do mês
- Dia atual destacado com borda roxa (`isToday`)
- Dias com atividades têm fundo amarelo `#fefce8`
- Pequenos pontos coloridos indicam: 🔴 atrasado, 🟡 pendente, 🟢 concluído

### Criação de Atividade

1. Clique em um dia → modal abre com data preenchida
2. Preenche título (obrigatório), horário de notificação, disciplina, descrição
3. Salvar → `invoke("create_assignment", { ... })` ou fallback localStorage
4. Modal fecha → `loadAssignments()` atualiza a lista

### Estados de Atividade

| Status | Badge | Cor | Descrição |
|---|---|---|---|
| `pending` | ⏳ Pendente | badge-pending | Prazo futuro ou hoje |
| `done` | ✅ Concluído | badge-done | Marcado manualmente |
| `overdue` | 🔴 Atrasado | badge-overdue | Atualizado automaticamente |

### Atualização de Status Overdue

Quando o app inicia, uma task em background (`tauri::async_runtime::spawn`)  
chama `update_overdue_assignments()` que faz:

```sql
UPDATE assignments SET status = 'overdue' 
WHERE due_date < CURRENT_DATE AND status = 'pending'
```

---

## 💾 Fallback localStorage

### Propósito

Permitir que o frontend funcione em **modo de desenvolvimento** (navegador)  
sem o backend Tauri/PostgreSQL.

### Como funciona

Cada componente que persiste dados segue este padrão:

```
invoke() → SUCCESS → atualiza estado + salva backup no localStorage
invoke() → ERROR  → carrega do localStorage (se existir)
                    ↓
                    Se não houver backup → estado vazio []
```

### Chaves de localStorage

| Chave | Componente | Conteúdo |
|---|---|---|
| `unitesk_assignments` | CalendarView | `Assignment[]` completo |
| `unitesk_articles` | ArticleManager | `ArticleExtended[]` completo (com status) |
| `unitesk_article_statuses` | ArticleManager | `Record<number, "draft"| "published">` — mapa de status |
| `unitesk_theme` | App | `"light"` ou `"dracula"` |

### Limitação

O localStorage **não substitui** o banco de dados — é apenas um fallback  
de desenvolvimento. Em produção (Tauri desktop), os dados persistem no PostgreSQL.

---

## 🧪 Testes

### Configuração

- **Framework:** Vitest + Testing Library React
- **Setup:** `vitest.config.ts` + `src/test/setup.ts`
- **Comando:** `npx vitest run` ou `npx vitest` (modo watch)

### Cobertura (75 testes)

| Arquivo | Testes | O que testa |
|---|---|---|
| `CalendarView.test.tsx` | 14 | Renderização, modal, criação de atividades, localStorage, badges, **navegação com selects de mês/ano**, exclusão com confirmação |
| `ProjectList.test.tsx` | 18 | CRUD de projetos, edição, exclusão com confirmação, artigos por projeto, arquivos, ZIP, modais |
| `Dashboard.test.tsx` | 18 | Cards de stats, **welcome card, progresso, timeline**, ações rápidas, export/import, **contadores animados**, **backend mockado** |
| `ArticleManager.test.tsx` | 11 | CRUD de artigos, **status draft/published**, **filtros**, busca por termo, visualização de conteúdo, exclusão com confirmação |
| `App.test.tsx` | 14 | Navegação entre abas, renderização de header/footer, destaque da aba ativa, **alternância de tema Dracula** |

### Padrão de Teste

```typescript
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

describe("Componente", () => {
  beforeEach(() => {
    localStorage.clear();  // Limpa estado entre testes
  });

  it("renderiza o título", () => {
    render(<Componente />);
    expect(screen.getByText("Título")).toBeInTheDocument();
  });

  it("abre modal ao clicar no botão", async () => {
    const user = userEvent.setup();
    render(<Componente />);
    await user.click(screen.getByText("➕ Novo"));
    expect(screen.getByText("Modal Title")).toBeInTheDocument();
  });

  it("carrega dados do localStorage", async () => {
    localStorage.setItem("key", JSON.stringify(mockData));
    render(<Componente />);
    await waitFor(() => {
      expect(screen.getByText("Item")).toBeInTheDocument();
    });
  });
});
```

### Testando o Backend Rust

O backend Rust é verificado via `cargo check` (typecheck) e `cargo build` (compilação).  
Não há testes unitários Rust implementados atualmente.

### Padrões de Mock para Testes

#### Mock do `window.confirm` (exclusão de atividades)

```typescript
it("deletes assignment when confirm is accepted", async () => {
  const originalConfirm = window.confirm;
  window.confirm = vi.fn().mockReturnValue(true);

  // ... executa a ação que dispara o confirm ...

  // Verifica que o item foi removido
  await waitFor(() => {
    expect(screen.queryByText("Item")).not.toBeInTheDocument();
  });

  window.confirm = originalConfirm;  // Restaura
});
```

#### Mock do `URL.createObjectURL` (exportação de dados)

```typescript
it("calls export_all_data when export button is clicked", async () => {
  vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock");
  vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});

  await user.click(screen.getByText("📤 Exportar Dados"));

  await waitFor(() => {
    expect(invoke).toHaveBeenCalledWith("export_all_data");
  });

  vi.restoreAllMocks();  // Limpa todos os spies
});
```

#### Mock de `localStorage` para tema

```typescript
it("loads theme from localStorage", () => {
  localStorage.setItem("unitesk_theme", "dracula");
  render(<App />);
  expect(document.documentElement.getAttribute("data-theme")).toBe("dracula");
});

it("toggles theme when clicking the theme button", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByTitle("Tema Dracula"));
  expect(document.documentElement.getAttribute("data-theme")).toBe("dracula");
  expect(localStorage.getItem("unitesk_theme")).toBe("dracula");
});
```

---

### Dicas para Escrever Testes

- **`beforeEach`** deve limpar `localStorage` e mocks entre testes
- **`waitFor`** é essencial para operações assíncronas (carregamento de dados, setState)
- **`userEvent`** (não `fireEvent`) é o padrão do projeto — simula eventos reais do usuário
- **Mock apenas APIs externas**, não a lógica do componente
- **Sempre restaure mocks** ao final do teste com `vi.restoreAllMocks()` ou manualmente

---

## 🚀 Instalação e Build

### Desenvolvimento (Frontend only)

```bash
npm install        # Instalar dependências
npm run dev        # Iniciar servidor Vite (porta 1420)
```

### Build Completo (Tauri + Frontend)

```bash
# 1. Configurar banco
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/academic_manager"

# 2. Build Tauri (compila frontend + Rust + gera .deb)
npm run tauri build

# Para build mais rápido (sem gerar .deb):
npx tauri build --no-bundle

# 3. Executar
./unitesk.sh
```

### Scripts npm

| Script | Descrição |
|---|---|
| `npm run dev` | Inicia servidor Vite para desenvolvimento frontend |
| `npm run build` | `tsc && vite build` — compila frontend para produção |
| `npm run preview` | `vite preview` — pré-visualiza o build de produção |
| `npm run tauri` | Atalho para `tauri` CLI |
| `npx vitest run` | Executa todos os testes (modo CI) |
| `npx vitest` | Executa testes em modo watch (desenvolvimento) |
| `cd src-tauri && cargo test` | Testes de integração Rust (requer banco de testes) |

---

## 🗄️ Git Workflow

### Repositório

- **URL oficial:** [https://github.com/Vitoriodev/UniTesk](https://github.com/Vitoriodev/UniTesk)
- **Branch principal:** `main`

### Fluxo de Commits

```bash
# 1. Verificar o estado
git status

# 2. Adicionar arquivos modificados
git add -A

# 3. Commitar com mensagem descritiva
git commit -m "tipo: descrição clara do que foi feito"

# 4. Enviar para o repositório
git push origin main
```

### Convenção de Mensagens de Commit

| Tipo | Quando usar | Exemplo |
|---|---|---|
| `feat` | Nova funcionalidade | `feat: adiciona suporte a scheduled_date nos artigos` |
| `fix` | Correção de bug | `fix: corrige hitbox do botão de tema` |
| `ui` | Melhoria de interface | `ui: melhora design do dashboard` |
| `docs` | Documentação | `docs: atualiza DEVELOPER.md com link do repositório` |
| `perf` | Otimização | `perf: acelera instalação com npm ci` |
| `refactor` | Refatoração | `refactor: extrai lógica de notificações` |

### Antes de um Push

Sempre verifique:

1. **TypeScript:** `npx tsc --noEmit` → 0 erros
2. **Testes:** `npx vitest run` → todos passando
3. **Remote:** `git remote -v` → aponta para `https://github.com/Vitoriodev/UniTesk.git`

> ⚠️ **Nunca** faça push para um repositório diferente do oficial.
> Confirme a URL do remote antes de cada push consultando esta seção.

---

## 📜 Scripts de Instalação

### Hierarquia dos instaladores

```
unitesk-setup (binário C compilado - 17KB)
    │
    │ fork() + setsid() + execl()
    ▼
setup.sh (assistente GUI com Zenity) / install.sh (terminal)
    │
    ├── Verifica pré-requisitos (Node, Rust, PostgreSQL)
    ├── Instala dependências de sistema (apt-get)
    ├── npm ci (package-lock) ou npm install — flags --no-fund --no-audit ⚡
    ├── Configura PostgreSQL (cria banco)
    ├── Cria .env com DATABASE_URL
    ├── npx tauri build --no-bundle (frontend + backend em 1 passo) ⚡
    ├── Cria unitesk.sh (wrapper)
    └── Cria .desktop (atalho no menu)

uninstall.sh (remove tudo: banco, build, atalhos)
build-deb.sh (gera pacote .deb para distribuição)
```

### unitesk-setup (C Launcher)

- Código-fonte: `src-tauri/src/setup_launcher.c`
- Compilado com: `gcc -O2 -o unitesk-setup setup_launcher.c`
- Função: Encontra `setup.sh` no mesmo diretório e executa em background
- Usa `fork()` + `setsid()` para desassociar do terminal
- Log em `/tmp/unitesk_setup.log`

### Otimizações de Performance (v1.2.0)

Os scripts de instalação foram otimizados para reduzir o tempo total:

1. **`npm ci` no lugar de `npm install`** — Quando `package-lock.json` existe, `npm ci`
   é 2-5x mais rápido por ser determinístico (não resolve versões, só instala o lock).
   Flags `--no-fund --no-audit` eliminam verificações desnecessárias.

2. **Build duplicado eliminado** — O `tauri.conf.json` define `beforeBuildCommand: "npm run build"`,
   então `npx tauri build` já compila o frontend automaticamente. Os scripts antes rodavam
   `npm run build` separadamente, compilando o frontend **duas vezes**. Agora só compila uma.

3. **`--no-bundle` no Tauri** — `npx tauri build --no-bundle` pula a geração do pacote `.deb`,
   economizando minutos. O binário é gerado normalmente em `src-tauri/target/release/unitesk`.
   Para distribuição, use `build-deb.sh` ou rode sem `--no-bundle`.

**Ganho estimado:** 1-2 minutos em média numa instalação completa.

### unitesk.sh (Wrapper)

Script que:
1. Carrega NVM (Node Version Manager) se existir
2. Source do arquivo `.env` para `DATABASE_URL`
3. Executa o binário compilado `src-tauri/target/release/unitesk`
4. Se o binário não existir, mostra erro via `echo` + `notify-send`

---

## 🧩 Como Adicionar uma Nova Funcionalidade

### Roteiro Passo a Passo

Suponha que você quer adicionar um novo recurso, exemplo: "Campo 'prioridade' nas atividades".

#### Etapa 1: Modelo de Dados

**`models.rs`** — Adicione o campo no struct:

```rust
pub struct Assignment {
    // ... campos existentes
    pub priority: Option<String>,  // "baixa", "media", "alta"
}
```

#### Etapa 2: Banco de Dados

**`db.rs`** em `init_db()`:

```rust
// Migração
sqlx::query("ALTER TABLE assignments ADD COLUMN IF NOT EXISTS priority VARCHAR(20)")
    .execute(&pool)
    .await?;
```

#### Etapa 3: Query SQL

**`db.rs`** — Atualize todas as queries SELECT e INSERT/UPDATE:

```sql
SELECT ..., priority FROM assignments ...
INSERT INTO assignments (..., priority) VALUES ($1, ..., $7)
```

#### Etapa 4: Comando Tauri

**`lib.rs`** — Adicione ou modifique o comando:

```rust
#[tauri::command]
async fn create_assignment(
    // ... params existentes
    priority: Option<String>,
) -> Result<Assignment, String> {
    db::create_assignment(..., &priority).await...
}
```

Registre no `invoke_handler`.

#### Etapa 5: Frontend

**No componente React:**
- Adicione o campo no formulário (modal)
- Envie no `invoke`
- Exiba no card da atividade

**Interface TypeScript:**

```typescript
interface Assignment {
  // ... campos existentes
  priority: string | null;
}
```

#### Etapa 6: Testes

Adicione testes no arquivo `.test.tsx` correspondente.

#### Etapa 7: Documentação

Atualize este `DEVELOPER.md` e os docs relevantes.

#### Etapa 8: Commit e Push

```bash
git add -A
git commit -m "feat: adiciona campo prioridade nas atividades"
git push origin main
```

> 📌 O remote `origin` já aponta para o repositório oficial:
> `https://github.com/Vitoriodev/UniTesk.git`

---

## 📐 Padrões de Código

### Rust

- **Snake case** para funções e variáveis
- **`Result<T, String>`** para comandos Tauri (erro como String)
- **`sqlx::query_as::<_, Modelo>()`** para queries tipadas
- **`::text` cast** para datas nos SELECTs
- **`Option<T>`** para campos opcionais e parâmetros

### TypeScript/React

- **Camel case** para variáveis e funções
- **`interface`** para tipos de dados
- **Import dinâmico** do Tauri (`await import("@tauri-apps/api/core")`)
- **try/catch** com fallback localStorage
- **`useState`** para estado local (sem Redux/Context — app pequeno)
- **`useEffect`** para carregamento inicial e intervalos
- **`useCallback`** para funções passadas como props (ex: `getAssignmentsForDay`)
- **`useRef`** para referências a elementos DOM (ex: file input)

### CSS

- **CSS Custom Properties** (`--primary`, `--bg`, etc.) no `:root`
- **Classes utilitárias** (`.btn`, `.card`, `.form-input`, `.badge`, etc.)
- **Grid** com `.grid-2`, `.grid-3` para layouts responsivos
- **Animações** CSS (`fadeIn`, `slideUp`) para modais
- **Media query** para mobile (`max-width: 768px`)

### Estrutura de um Componente

```typescript
function Componente() {
  // 1. Estado
  const [data, setData] = useState<Tipo[]>([]);

  // 2. Efeitos
  useEffect(() => { loadData(); }, []);

  // 3. Funções auxiliares
  async function loadData() { /* try Tauri, catch localStorage */ }

  // 4. Renderização
  return (
    <div>
      {data.length === 0 ? <Empty /> : <List />}
      <Modal />
    </div>
  );
}
```

---

## 🔍 Troubleshooting

### Problemas Comuns

| Problema | Causa Provável | Solução |
|---|---|---|
| App não abre pelo menu | `unitesk.sh` não existe | Executar `./setup.sh` para recriar |
| "Não foi possível conectar ao banco" | PostgreSQL não está rodando | `sudo systemctl start postgresql` |
| "Password authentication failed" | Senha do postgres não configurada | `sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"` |
| Atividade não salva no calendário | Backend Tauri falhou (catch usou localStorage) | Verificar log do Tauri, testar conexão DB |
| Notificação não aparece no horário | `notification_time` não foi definido | Definir horário ao criar atividade |
| Notificação duplicada | Janela SQL muito larga | Verificar `BETWEEN` em `get_today_assignments` |
| Testes falham | Mudança na interface sem atualizar tests | Rodar `npx vitest run` e corrigir asserts |
| Build Tauri falha | Dependências de sistema faltando | `sudo apt-get install ...` (ver lista acima) |

### Verificações Rápidas

```bash
# Verificar sintaxe bash
bash -n install.sh && bash -n setup.sh && bash -n unitesk.sh

# Verificar compilação Rust
cd src-tauri && cargo check

# Verificar TypeScript
npx tsc --noEmit

# Rodar testes
npx vitest run

# Verificar PostgreSQL
pg_isready
psql -U postgres -d academic_manager -c "SELECT COUNT(*) FROM assignments;"
```

---

## 📚 Documentos Relacionados

| Arquivo | Conteúdo |
|---|---|
| `docs/README.md` | README principal do projeto |
| `docs/LEIGO.md` | Guia rápido para usuários (instalação, uso) |
| `docs/ARCHITECTURE.md` | Arquitetura detalhada |
| `docs/API.md` | Lista completa de comandos Tauri |
| `docs/DATABASE.md` | Schema e consultas do banco |
| `docs/WINDOWS.md` | Plano para portar para Windows |
| `docs/setup.sql` | Schema SQL para setup manual |
| `CHANGELOG.md` | Histórico de alterações do projeto |

---

## 🗄️ Exportação e Importação de Dados (Backup)

O Unitesk permite exportar todos os dados (projetos, artigos, atividades e arquivos) para um arquivo `.unitesk` e importá-los em outra máquina.

### Como funciona

#### Exportar
1. Vá para o **Dashboard**
2. Clique em **📤 Exportar Dados**
3. Um arquivo `unitesk_backup_YYYY-MM-DD.unitesk` será baixado
4. O arquivo contém **todos os dados** em formato JSON, incluindo arquivos codificados em base64

#### Importar
1. Na máquina de destino, abra o Unitesk
2. Vá para o **Dashboard**
3. Clique em **📥 Importar Dados**
4. Selecione o arquivo `.unitesk` baixado
5. Os dados serão inseridos no banco de dados local

> ⚠️ **Importante:** A importação **adiciona** os dados ao banco existente, não substitui. IDs são recriados automaticamente.

### Backend Rust

```rust
// db.rs — Estrutura de dados exportados
pub struct ExportedData {
    pub version: String,
    pub exported_at: String,
    pub projects: Vec<Project>,
    pub articles: Vec<Article>,
    pub assignments: Vec<Assignment>,
    pub project_files: Vec<ExportedProjectFile>,  // com file_data em base64
    pub assignment_files: Vec<ExportedAssignmentFile>,  // com file_data em base64
}

// Comandos Tauri:
// - export_all_data → Retorna ExportedData completo
// - import_all_data(data) → Insere todos os dados no banco
```

**Mapeamento de IDs:** Durante a importação, os IDs antigos são mapeados para os novos IDs gerados pelo PostgreSQL, preservando as relações entre projetos, artigos e arquivos.

---

## 🎨 Sistema de Temas

O Unitesk possui dois temas:

| Tema | Modo | Cor predominante |
|---|---|---|
| **Claro** (padrão) | Light | Roxo (#4f46e5) |
| **Dracula** | Dark | Roxo Dracula (#bd93f9) |

### Alternar tema

Clique no botão 🌙/☀️ no canto superior direito do cabeçalho.

### Como funciona

O tema é controlado pelo atributo `data-theme` no elemento `<html>`:

```html
<html data-theme="dracula">  <!-- Tema escuro -->
<html data-theme="light">    <!-- Tema claro -->
```

- O tema é **persistido** no `localStorage` (chave: `unitesk_theme`)
- A alternância é feita via `App.tsx` com `useEffect` e `useState`
- As cores do tema Dracula seguem a [paleta oficial do Dracula](https://draculatheme.com)

### Variáveis CSS do Dracula

```css
[data-theme="dracula"] {
  --primary: #bd93f9;        /* Roxo Dracula */
  --bg: #1e1e2e;             /* Fundo escuro */
  --bg-card: #282a36;        /* Cards */
  --text: #f8f8f2;           /* Texto claro */
  --text-secondary: #6272a4; /* Texto secundário */
  --border: #44475a;         /* Bordas */
  --success: #50fa7b;        /* Verde */
  --warning: #ffb86c;        /* Laranja */
  --danger: #ff5555;         /* Vermelho */
}
```

---

## 🗑️ Exclusão de Atividades

O calendário agora possui um botão 🗑️ em cada card de atividade para excluí-la.

**Comportamento:**
- Exibe um `confirm()` perguntando se o usuário tem certeza
- Remove a atividade e todos os seus arquivos associados (ON DELETE CASCADE no PostgreSQL)
- Fallback para localStorage quando o backend não está disponível

### Backend

```rust
#[tauri::command]
async fn delete_assignment(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_assignment(&state.pool, id).await...
}
```

---

## 🖼️ Ícone Personalizado

O Unitesk possui um ícone personalizado (`unitesk_icon_512.png`) usado em todo o sistema.

### Onde o ícone é usado

| Local | Arquivo | Referência |
|---|---|---|
| **Tauri bundle** (.deb) | `src-tauri/icons/unitesk_icon_512.png` | `tauri.conf.json` → `bundle.icon` |
| **Favicon** (desenvolvimento) | `public/icon.png` | `index.html` → `<link rel="icon">` |
| **Atalho .desktop** (menu) | `src-tauri/icons/unitesk_icon_512.png` | `install.sh` / `setup.sh` → `Icon=` |

### Como funciona

1. O arquivo original é `src-tauri/icons/unitesk_icon_512.png`
2. Durante o build Tauri, esse ícone é incluído no pacote `.deb`
3. Uma cópia é mantida em `public/icon.png` para servir como favicon no frontend (modo dev)
4. Os scripts de instalação (`install.sh` e `setup.sh`) referenciam o caminho absoluto no arquivo `.desktop`

### Para trocar o ícone

1. Substitua o arquivo `src-tauri/icons/unitesk_icon_512.png`
2. Atualize a cópia: `cp src-tauri/icons/unitesk_icon_512.png public/icon.png`
3. Recompile: `npx tauri build`

---

## 📁 Diretório public/ (Assets Estáticos)

O diretório `public/` na raiz do projeto contém assets servidos estaticamente pelo Vite.  
Arquivos em `public/` são copiados para `dist/` durante o build sem transformação.

### Conteúdo

| Arquivo | Finalidade |
|---|---|
| `icon.png` | Favicon do aplicativo (cópia do ícone principal) |

### Uso

No `index.html`, o favicon é referenciado como:

```html
<link rel="icon" type="image/png" href="/icon.png" />
```

O Vite serve o arquivo de `public/` na raiz durante `npm run dev` e o copia para `dist/` durante `npm run build`.

---

> **🎓 Unitesk v1.2** — Documentação gerada em Julho/2026  
> Mantenha este documento atualizado conforme o código evolui!
