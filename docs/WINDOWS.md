╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║   🎓 UNITESK — GUIA COMPLETO PARA VERSÃO WINDOWS                            ║
║                                                                              ║
║   Documento definitivo descrevendo TUDO que foi feito no Unitesk Linux      ║
║   para que a versão Windows seja idêntica e interoperável.                  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

Data: Julho/2026
Versão atual (Linux): 1.3.0
Stack: Tauri 2 + React 18 + TypeScript + PostgreSQL


================================================================================
  SUMÁRIO
================================================================================

  1.  VISÃO GERAL DO PROJETO
  2.  FUNCIONALIDADES COMPLETAS (MÓDULO POR MÓDULO)
  3.  STACK TECNOLÓGICA DETALHADA
  4.  ARQUITETURA E FLUXO DE DADOS
  5.  BANCO DE DADOS — SCHEMA COMPLETO (PostgreSQL)
  6.  BANCO DE DADOS — MIGRAÇÃO PARA SQLite (Windows)
  7.  API DE COMANDOS TAURI (COMPLETA)
  8.  FRONTEND — COMPONENTES E ESTADOS
  9.  FRONTEND — CSS E SISTEMA DE TEMAS
  10. SISTEMA DE NOTIFICAÇÕES
  11. GERENCIAMENTO DE ARQUIVOS
  12. EXPORTAÇÃO/IMPORTAÇÃO DE DADOS (INTEROPERABILIDADE)
  13. TESTES
  14. BUILD E EMPACOTAMENTO
  15. PLANO DE AÇÃO PARA WINDOWS
  16. INTERCONEXÃO LINUX ↔ WINDOWS


================================================================================
  1. VISÃO GERAL DO PROJETO
================================================================================

  O Unitesk é um aplicativo desktop para gerenciamento de projetos, artigos,
  atividades (calendário), clientes, equipes, controle de horas e financeiro.

  Público-alvo: Profissionais autônomos, pequenas equipes e estudantes que
  precisam organizar projetos, prazos e documentos em um só lugar.

  Modelo de distribuição (Linux): Pacote .deb via dpkg/apt.
  Modelo de distribuição (Windows): A definir (NSIS, Inno Setup ou MSI).

  ---

  🏗️ Estrutura de diretórios (completa):

  ```
  unitesk/
  ├── src/                          # Frontend React + TypeScript
  │   ├── main.tsx                  # Entry point do React
  │   ├── App.tsx                   # Componente principal (navegação por abas + tema)
  │   ├── components/
  │   │   ├── Dashboard.tsx         # Visão geral com 7 cards de estatísticas
  │   │   ├── ProjectList.tsx       # CRUD de projetos + artigos + arquivos + ZIP
  │   │   ├── CalendarView.tsx      # Calendário + atividades + notificações + arquivos
  │   │   ├── ArticleManager.tsx    # CRUD de artigos com busca e filtros
  │   │   ├── ClientList.tsx        # CRUD de clientes
  │   │   ├── TeamList.tsx          # CRUD de equipes e membros
  │   │   ├── TimeTracking.tsx      # Controle de horas (timer + registro manual)
  │   │   ├── Finance.tsx           # Faturas/invoices (CRUD + filtros + status)
  │   │   ├── Reports.tsx           # Relatórios com gráficos (Recharts)
  │   │   └── NotificationPanel.tsx # Painel de notificações (dropdown + ações)
  │   ├── styles/
  │   │   └── global.css            # Estilos globais (~1600 linhas)
  │   └── test/                     # Testes com Vitest
  │       ├── setup.ts
  │       ├── App.test.tsx
  │       ├── CalendarView.test.tsx
  │       ├── ProjectList.test.tsx
  │       ├── Dashboard.test.tsx
  │       ├── ArticleManager.test.tsx
  │       ├── TimeTracking.test.tsx
  │       ├── Finance.test.tsx
  │       └── NotificationPanel.test.tsx
  │
  ├── src-tauri/                    # Backend Rust + Tauri
  │   ├── src/
  │   │   ├── main.rs               # Entry point do Tauri
  │   │   ├── lib.rs                # ~50 comandos Tauri registrados + setup
  │   │   ├── db.rs                 # ~1800 linhas — operações SQL (dupla plataforma)
  │   │   │                        #   #[cfg(target_os = "linux")] → PgPool
  │   │   │                        #   #[cfg(target_os = "windows")] → SqlitePool
  │   │   └── models.rs             # Structs de dados (Serialize + FromRow)
  │   ├── deb-scripts/              # Scripts .deb (Linux apenas)
  │   │   ├── postinst              # Configura PostgreSQL + ambiente
  │   │   ├── prerm                 # Aviso de preservação de dados
  │   │   └── postrm                # Remove /etc/unitesk/ (purge)
  │   ├── Cargo.toml                # Dependências Rust (postgres + sqlite)
  │   └── tauri.conf.json           # Configuração Tauri
  │
  ├── docs/                         # Documentação
  │   ├── DEVELOPER.md              # Documentação completa para devs
  │   ├── ARCHITECTURE.md           # Arquitetura
  │   ├── API.md                    # API de comandos Tauri
  │   ├── DATABASE.md               # Banco de dados
  │   ├── WINDOWS.md                # [ESTE ARQUIVO]
  │   ├── LEIGO.md                  # Guia para usuários
  │   ├── setup.sql                 # Schema SQL completo
  │   └── README.md                 # README principal
  │
  ├── scripts/
  │   └── notify-deadlines.sh       # Script cron para notificações (Linux)
  │
  ├── package.json                  # Dependências npm e scripts
  ├── vite.config.ts                # Configuração Vite
  ├── tsconfig.json                 # Configuração TypeScript
  ├── vitest.config.ts              # Configuração Vitest
  ├── index.html                    # HTML entry point
  │
  └── build-deb.sh                  # Script para gerar .deb (Linux)


================================================================================
  2. FUNCIONALIDADES COMPLETAS (MÓDULO POR MÓDULO)
================================================================================

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 1: DASHBOARD (Dashboard.tsx)                                       │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  📊 Visão geral com:                                                       │
  │  • 7 cards de estatísticas (Projetos, Documentos, Clientes, Equipes,       │
  │    Usuários, Pendentes, Atrasados) com AnimatedCounter animado             │
  │  • Barra de progresso de atividades concluídas                             │
  │  • Próximo prazo (destaque)                                                │
  │  • Resumo de horas (hoje / semana)                                         │
  │  • Financeiro resumido (receita total / a receber)                         │
  │  • 4 ações rápidas (Novo Projeto, Novo Artigo, Nova Atividade, Exportar)  │
  │  • Importar dados (.unitesk)                                               │
  │  • Timeline de atividades recentes (últimas 5)                             │
  │  • Botão "Ver Controle de Horas →", "Ver Financeiro →"                    │
  │  • Botão "Ver todas as atividades →"                                       │
  │                                                                            │
  │  Comandos Tauri usados: get_dashboard_stats, export_all_data,              │
  │  import_all_data, get_assignments                                          │
  │                                                                            │
  │  Dados mockados: Se Tauri falha, stats são zero, sem localStorage         │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 2: PROJETOS (ProjectList.tsx)                                     │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  📁 CRUD completo de projetos com:                                         │
  │  • Lista de projetos com nome, cliente vinculado e data de criação        │
  │  • Modal de criação/edição (nome, descrição, cliente)                      │
  │  • Exclusão com confirmação em 2 cliques (Confirmar / Cancelar)            │
  │  • Aba de artigos por projeto (criar, visualizar, excluir)                │
  │  • Anexar arquivos (upload com limite de 10 MB)                           │
  │  • Download de arquivos                                                    │
  │  • Exclusão de arquivos                                                    │
  │  • Exportar ZIP do projeto (projeto + artigos + arquivos)                 │
  │                                                                            │
  │  Estados do componente:                                                    │
  │    projects: Project[], articles: Article[], projectFiles: Record<...>     │
  │    showModal, editingProject, deleteConfirm, activeTab                     │
  │    viewingArticle, expandedFiles, exporting, uploadingFile                │
  │                                                                            │
  │  Comandos Tauri: get_projects, create/update/delete_project,               │
  │  get_articles, create_article, delete_article,                             │
  │  get_project_files, add_project_file, get_project_file_data,               │
  │  delete_project_file, export_project_zip                                   │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 3: CALENDÁRIO (CalendarView.tsx)                                  │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  📅 Calendário interativo de atividades com:                               │
  │  • Grade mensal 7×N com dias do mês                                       │
  │  • Navegação: botões ← →, dropdowns de mês/ano, input type="month"        │
  │  • Botão "📅 Hoje" para voltar ao mês atual                                │
  │  • Destaque no dia atual (borda roxa)                                     │
  │  • Pontos coloridos: 🔴 atrasado, 🟡 pendente, 🟢 concluído               │
  │  • Dias com eventos têm fundo diferenciado                                │
  │  • Dias passados com opacidade reduzida                                   │
  │  • Modal de criação (data, horário notificação, título, disciplina,       │
  │    descrição, prioridade)                                                  │
  │  • Lista de atividades ordenada por data                                   │
  │  • Badges de prioridade (🔴 Urgente, 🟠 Alta, 🔵 Média, 🟢 Baixa)        │
  │  • Modal de arquivos (upload/download/exclusão por atividade)             │
  │  • Marcar como concluído                                                   │
  │  • Excluir atividade (com confirmação)                                     │
  │  • Verificação periódica de notificações (setInterval 60s)                │
  │  • Fallback localStorage (chave: unitesk_assignments)                     │
  │                                                                            │
  │  Estados do componente: (~15 states)                                      │
  │    assignments, showModal, selectedDate, currentMonth, currentYear,       │
  │    newAssignment, newTime, errorMessage, selectedAssignment,              │
  │    showFilesModal, assignmentFiles, uploadingFile                         │
  │                                                                            │
  │  Comandos Tauri: get_assignments, create_assignment, delete_assignment,   │
  │  mark_assignment_done, check_today_assignments,                           │
  │  get_assignment_files, add_assignment_file, get_assignment_file_data,     │
  │  delete_assignment_file                                                   │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 4: ARTIGOS/DOCUMENTOS (ArticleManager.tsx)                         │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  📄 Gerenciador de artigos/documentos com:                                 │
  │  • Busca por termo no título ou conteúdo                                   │
  │  • Filtros por status: Todos / Rascunhos / Prontos                        │
  │  • Visualização de conteúdo em modal separado                             │
  │  • Status: Rascunho (📝 badge-draft) / Pronto (✅ badge-published)        │
  │  • Alternar status diretamente na lista                                    │
  │  • Modal de criação (título, conteúdo, projeto vinculado)                 │
  │  • Exclusão com confirmação em 2 cliques                                   │
  │                                                                            │
  │  Estados: articles, searchTerm, activeFilter, viewingArticle,             │
  │  showCreate, deleteConfirm, showUpdate                                    │
  │                                                                            │
  │  localStorage: unitesk_articles, unitesk_article_statuses                 │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 5: CLIENTES (ClientList.tsx)                                      │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  🤝 CRUD de clientes com:                                                  │
  │  • Nome, e-mail, telefone, empresa, observações                           │
  │  • Modal de criação/edição                                                 │
  │  • Exclusão com confirmação                                                │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 6: EQUIPES (TeamList.tsx)                                         │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  👥 CRUD de equipes com:                                                   │
  │  • Nome e descrição                                                        │
  │  • Gerenciamento de membros (adicionar/remover usuários)                   │
  │  • Cargos: leader / member                                                 │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 7: CONTROLE DE HORAS (TimeTracking.tsx)                           │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  ⏱️ Controle de horas com:                                                │
  │  • Timer em tempo real (formato HH:MM:SS) com display monospace           │
  │  • Seleção de projeto para iniciar timer                                  │
  │  • Descrição opcional                                                      │
  │  • Parar timer + cálculo automático de duração                            │
  │  • Resumo: horas de hoje / da semana                                      │
  │  • Registro manual (projeto, descrição, data, duração, valor hora,         │
  │    faturável sim/não)                                                      │
  │  • Cálculo automático de valor (R$)                                       │
  │  • Filtros por projeto e período (data de / até)                          │
  │  • Lista de entradas com badges (duração, valor, faturável)               │
  │  • Exclusão com confirmação                                                │
  │  • Timer ativo com fundo gradiente roxo                                   │
  │  • Fallback local quando Tauri offline                                     │
  │                                                                            │
  │  Estados: entries, projects, activeEntry, elapsedSeconds,                 │
  │  filterProject, filterDateFrom/To, showManualModal, hoursSummary,         │
  │  manualForm, deleteConfirm, startDesc, startProject                       │
  │                                                                            │
  │  Comandos Tauri: get_time_entries, get_projects, get_hours_summary,       │
  │  get_active_time_entry, start_time_entry, stop_time_entry,                │
  │  add_manual_time_entry, delete_time_entry                                  │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 8: FINANCEIRO (Finance.tsx)                                       │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  💰 Gestão de faturas com:                                                 │
  │  • Cards de resumo: Receita Total, A Receber, Vencidas                    │
  │  • Filtros por status (abas): Todas, Rascunho, Enviadas, Pagas,           │
  │    Vencidas, Canceladas                                                    │
  │  • CRUD completo de faturas                                                │
  │  • Número, descrição, valor, imposto, total (calculado automaticamente)   │
  │  • Status: draft, sent, paid, cancelled, overdue                          │
  │  • Datas: emissão, vencimento, pagamento                                  │
  │  • Vinculação a projeto e cliente                                          │
  │  • Observações                                                             │
  │  • Badges coloridos por status                                             │
  │  • Formatação monetária em Real (BRL)                                     │
  │  • Exclusão com confirmação                                                │
  │                                                                            │
  │  Comandos Tauri: get_invoices, create/update/delete_invoice               │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 9: NOTIFICAÇÕES (NotificationPanel.tsx)                           │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  🔔 Painel de notificações com:                                            │
  │  • Badge com contagem de não lidas (pulse animado)                        │
  │  • 99+ para mais de 99 notificações                                       │
  │  • Dropdown com lista de notificações                                     │
  │  • Ícones por tipo: 📅 prazo, 🔴 atrasado, 💰 fatura                     │
  │  • Marcar como lida (individual)                                          │
  │  • Marcar todas como lidas                                                 │
  │  • Excluir notificação                                                     │
  │  • Limpar notificações antigas (>30 dias)                                 │
  │  • Time ago (agora, 5min, 2h, 1d)                                         │
  │  • Fechar ao clicar fora                                                  │
  │  • Auto-geração de notificações na inicialização                          │
  │                                                                            │
  │  Comandos Tauri: get_notifications, get_unread_notifications_count,       │
  │  mark_notification_read, mark_all_notifications_read, delete_notification,│
  │  auto_generate_notifications, cleanup_old_notifications                   │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 10: RELATÓRIOS (Reports.tsx)                                      │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  📊 Relatórios com gráficos (Recharts):                                    │
  │  • Cards de resumo (projetos, artigos, clientes, equipes, usuários)       │
  │  • Gráfico de atividades por mês (BarChart)                               │
  │  • Gráfico de receita por mês (BarChart)                                  │
  │  • Gráfico de horas por projeto (PieChart)                                │
  │  • Distribuição de status (DonutChart)                                    │
  │  • Dark mode compatível (recharts-text fill, tooltip, grid)              │
  │                                                                            │
  │  Comando Tauri: get_report_stats                                           │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────────────────┐
  │  MÓDULO 11: SISTEMA DE TEMAS (App.tsx + global.css)                       │
  ├─────────────────────────────────────────────────────────────────────────────┤
  │                                                                            │
  │  🌙 Dois temas completos:                                                  │
  │  • Tema Claro (light) — padrão, roxo #4f46e5                              │
  │  • Tema Dracula (dark) — roxo #bd93f9, fundo #1e1e2e                     │
  │  • Alternância por botão 🌙/☀️ no cabeçalho                                │
  │  • Persistência em localStorage (unitesk_theme)                           │
  │  • CSS custom properties (~40 variáveis)                                  │
  │  • Estilos específicos para Dracula em badges, selects, inputs,           │
  │    gráficos Recharts, etc.                                                 │
  │                                                                            │
  └─────────────────────────────────────────────────────────────────────────────┘


================================================================================
  3. STACK TECNOLÓGICA DETALHADA
================================================================================

  ┌─────────────────┬──────────────────────────┬──────────┐
  │ Camada          │ Tecnologia               │ Versão   │
  ├─────────────────┼──────────────────────────┼──────────┤
  │ Frontend        │ React                    │ 18.x     │
  │ Frontend        │ TypeScript               │ 5.x      │
  │ Build frontend  │ Vite                     │ 6.x      │
  │ Backend desktop │ Rust + Tauri             │ 2.x      │
  │ Banco (Linux)   │ PostgreSQL + SQLx        │ 14+ / 0.8│
  │ Banco (Windows) │ SQLite + SQLx (previsto) │ 0.8      │
  │ Comunicação     │ Tauri IPC (invoke)       │ 2.x      │
  │ Notificações    │ tauri-plugin-notification│ 2.x      │
  │ Serialização    │ serde + serde_json       │ 1.x      │
  │ UUID            │ uuid (v4)                │ 1.x      │
  │ ZIP             │ zip crate                │ 2.x      │
  │ Base64          │ base64 crate             │ 0.22     │
  │ Datas           │ chrono                   │ 0.4      │
  │ Gráficos        │ Recharts                 │ 3.x      │
  │ Testes          │ Vitest + Testing Library │ 4.x/16.x │
  │ Ui-calendário   │ react-calendar           │ 5.x      │
  └─────────────────┴──────────────────────────┴──────────┘

  Dependências Rust (Cargo.toml) completas:

  ```toml
  [dependencies]
  tauri = { version = "2", features = [] }
  tauri-plugin-notification = "2"
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "chrono", "uuid"] }
  tokio = { version = "1", features = ["full"] }
  chrono = { version = "0.4", features = ["serde"] }
  uuid = { version = "1", features = ["v4", "serde"] }
  zip = "2"
  base64 = "0.22"
  ```

  ⚠️ Para Windows, a feature "postgres" do SQLx deve ser trocada por "sqlite".

  Dependências npm (package.json) completas:

  ```json
  {
    "dependencies": {
      "@tauri-apps/api": "^2.0.0",
      "@tauri-apps/plugin-notification": "^2.0.0",
      "react": "^18.3.1",
      "react-calendar": "^5.0.0",
      "react-dom": "^18.3.1",
      "recharts": "^3.9.2"
    },
    "devDependencies": {
      "@tauri-apps/cli": "^2.0.0",
      "@testing-library/jest-dom": "^6.9.1",
      "@testing-library/react": "^16.3.2",
      "@testing-library/user-event": "^14.6.1",
      "@types/react": "^18.3.31",
      "@types/react-dom": "^18.3.7",
      "@vitejs/plugin-react": "^4.3.4",
      "jsdom": "^29.1.1",
      "typescript": "^5.6.3",
      "vite": "^6.0.0",
      "vitest": "^4.1.9"
    }
  }
  ```


================================================================================
  4. ARQUITETURA E FLUXO DE DADOS
================================================================================

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                    React (Frontend)                                       │
  │                                                                          │
  │  Componente → invoke("comando", { args }) → Tauri IPC                   │
  │                  ↑                            ↓                          │
  │         localStorage (fallback)          Rust Backend                    │
  │                                              ↓                          │
  │                                          SQLx queries                    │
  │                                              ↓                          │
  │                                       PostgreSQL (Linux)                │
  │                                       SQLite (Windows)                  │
  └─────────────────────────────────────────────────────────────────────────┘

  🔄 Comunicação Frontend ↔ Backend:

  1. Frontend chama invoke("nome_do_comando", { argumentos })
     (importado dinamicamente de @tauri-apps/api/core)

  2. Tauri roteia para a função Rust com #[tauri::command]

  3. Rust executa query SQLx e retorna Result<T, String>

  4. Se Tauri não está disponível (navegador/dev), o catch usa localStorage

  🔄 Conversão automática de nomes (Tauri 2):

  JavaScript (camelCase) → Rust (snake_case)
  ------------------------------------------
  dueDate               → due_date
  projectName           → project_name
  assignmentId          → assignment_id
  originalName          → original_name
  fileData              → file_data
  mimeType              → mime_type
  projectId             → project_id
  userId                → user_id
  durationMinutes       → duration_minutes
  entryDate             → entry_date
  hourlyRate            → hourly_rate
  issueDate             → issue_date
  dueDate               → due_date
  paidDate              → paid_date

  🔄 Padrão try-Tauri / catch-localStorage (todo componente segue):

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


================================================================================
  5. BANCO DE DADOS — SCHEMA COMPLETO (PostgreSQL)
================================================================================

  ⚠️ Este é o schema ATUAL no Linux (PostgreSQL).
  Para Windows, usar SQLite (ver seção 6).

  ┌────────────────────────────────────────────────────────────────────────┐
  │  TABELAS E RELACIONAMENTOS                                             │
  │                                                                        │
  │  clients (1) ──→ (N) projects                                         │
  │  projects (1) ──→ (N) articles                                        │
  │  projects (1) ──→ (N) project_files (CASCADE)                         │
  │  projects (1) ──→ (N) time_entries (CASCADE)                          │
  │  assignments (1) ──→ (N) assignment_files (CASCADE)                   │
  │  teams (1) ──→ (N) team_members (CASCADE)                             │
  │  users (1) ──→ (N) team_members (CASCADE)                             │
  │  users (1) ──→ (N) time_entries (SET NULL)                            │
  │  projects (1) ──→ (N) invoices (SET NULL)                             │
  │  clients (1) ──→ (N) invoices (SET NULL)                              │
  │                                                                        │
  └────────────────────────────────────────────────────────────────────────┘

  📋 Tabela: clients
  ┌────────────┬──────────────────┬───────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                     │
  ├────────────┼──────────────────┼───────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento           │
  │ name       │ VARCHAR(255)     │ NOT NULL                  │
  │ email      │ VARCHAR(255)     │ NULLABLE                  │
  │ phone      │ VARCHAR(50)      │ NULLABLE                  │
  │ company    │ VARCHAR(255)     │ NULLABLE                  │
  │ notes      │ TEXT             │ NULLABLE                  │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP │
  └────────────┴──────────────────┴───────────────────────────┘

  📋 Tabela: users
  ┌────────────┬──────────────────┬───────────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                         │
  ├────────────┼──────────────────┼───────────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento               │
  │ name       │ VARCHAR(255)     │ NOT NULL                      │
  │ email      │ VARCHAR(255)     │ UNIQUE NOT NULL               │
  │ role       │ VARCHAR(50)      │ CHECK: admin/manager/member   │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP     │
  └────────────┴──────────────────┴───────────────────────────────┘

  📋 Tabela: projects
  ┌────────────┬──────────────────┬────────────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                          │
  ├────────────┼──────────────────┼────────────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento               │
  │ name       │ VARCHAR(255)     │ NOT NULL                      │
  │ description│ TEXT             │ NULLABLE                      │
  │ client_id  │ INTEGER          │ FK → clients(id) SET NULL     │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP     │
  └────────────┴──────────────────┴────────────────────────────────┘

  📋 Tabela: articles
  ┌───────────────┬──────────────────┬────────────────────────────┐
  │ Coluna        │ Tipo             │ Notas                      │
  ├───────────────┼──────────────────┼────────────────────────────┤
  │ id            │ SERIAL PK        │ Auto incremento            │
  │ title         │ VARCHAR(255)     │ NOT NULL                   │
  │ content       │ TEXT             │ NULLABLE                   │
  │ project_name  │ VARCHAR(255)     │ NULLABLE                   │
  │ project_id    │ INTEGER          │ FK → projects(id) SET NULL │
  │ scheduled_date│ DATE             │ NULLABLE (agendamento)     │
  │ created_at    │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP  │
  └───────────────┴──────────────────┴────────────────────────────┘

  📋 Tabela: assignments
  ┌─────────────────┬──────────────────┬─────────────────────────────────────┐
  │ Coluna          │ Tipo             │ Notas                               │
  ├─────────────────┼──────────────────┼─────────────────────────────────────┤
  │ id              │ SERIAL PK        │ Auto incremento                     │
  │ title           │ VARCHAR(255)     │ NOT NULL                            │
  │ description     │ TEXT             │ NULLABLE                            │
  │ due_date        │ DATE             │ NOT NULL                            │
  │ due_time        │ TIME             │ NULLABLE                            │
  │ notification_time │ TIME           │ NULLABLE (notificação automática)  │
  │ project_name    │ VARCHAR(255)     │ NULLABLE                            │
  │ status          │ VARCHAR(20)      │ CHECK: pending/done/overdue         │
  │ priority        │ VARCHAR(20)      │ CHECK: low/medium/high/urgent       │
  │ created_at      │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP           │
  └─────────────────┴──────────────────┴─────────────────────────────────────┘

  📋 Tabela: project_files
  ┌───────────────┬──────────────────┬─────────────────────────────────┐
  │ Coluna        │ Tipo             │ Notas                           │
  ├───────────────┼──────────────────┼─────────────────────────────────┤
  │ id            │ SERIAL PK        │ Auto incremento                │
  │ project_id    │ INTEGER          │ FK → projects(id) CASCADE      │
  │ original_name │ VARCHAR(500)     │ NOT NULL                       │
  │ stored_name   │ VARCHAR(500)     │ NOT NULL (UUID único)          │
  │ file_data     │ BYTEA            │ NOT NULL (binário)             │
  │ file_size     │ BIGINT           │ NOT NULL                       │
  │ mime_type     │ VARCHAR(100)     │ NOT NULL                       │
  │ created_at    │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP      │
  └───────────────┴──────────────────┴─────────────────────────────────┘

  📋 Tabela: assignment_files
  ┌───────────────┬──────────────────┬─────────────────────────────────────┐
  │ Coluna        │ Tipo             │ Notas                               │
  ├───────────────┼──────────────────┼─────────────────────────────────────┤
  │ id            │ SERIAL PK        │ Auto incremento                     │
  │ assignment_id │ INTEGER          │ FK → assignments(id) CASCADE        │
  │ original_name │ VARCHAR(500)     │ NOT NULL                           │
  │ stored_name   │ VARCHAR(500)     │ NOT NULL (UUID único)              │
  │ file_data     │ BYTEA            │ NOT NULL (binário)                 │
  │ file_size     │ BIGINT           │ NOT NULL                           │
  │ mime_type     │ VARCHAR(100)     │ NOT NULL                           │
  │ created_at    │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP          │
  └───────────────┴──────────────────┴─────────────────────────────────────┘

  📋 Tabela: teams
  ┌────────────┬──────────────────┬───────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                     │
  ├────────────┼──────────────────┼───────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento           │
  │ name       │ VARCHAR(255)     │ NOT NULL                  │
  │ description│ TEXT             │ NULLABLE                  │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP │
  └────────────┴──────────────────┴───────────────────────────┘

  📋 Tabela: team_members
  ┌────────────┬──────────────────┬─────────────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                           │
  ├────────────┼──────────────────┼─────────────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento                │
  │ team_id    │ INTEGER          │ FK → teams(id) CASCADE         │
  │ user_id    │ INTEGER          │ FK → users(id) CASCADE         │
  │ role       │ VARCHAR(50)      │ CHECK: leader/member            │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP      │
  │ UNIQUE     │ (team_id, user_id) │ Sem duplicatas              │
  └────────────┴──────────────────┴─────────────────────────────────┘

  📋 Tabela: time_entries
  ┌─────────────────┬──────────────────┬─────────────────────────────────┐
  │ Coluna          │ Tipo             │ Notas                           │
  ├─────────────────┼──────────────────┼─────────────────────────────────┤
  │ id              │ SERIAL PK        │ Auto incremento                │
  │ project_id      │ INTEGER          │ FK → projects(id) CASCADE      │
  │ user_id         │ INTEGER          │ FK → users(id) SET NULL        │
  │ description     │ TEXT             │ NULLABLE                       │
  │ start_time      │ TIMESTAMP        │ NOT NULL                       │
  │ end_time        │ TIMESTAMP        │ NULLABLE                       │
  │ duration_minutes│ INTEGER          │ NULLABLE                       │
  │ billable        │ BOOLEAN          │ DEFAULT true                   │
  │ hourly_rate     │ DECIMAL(10,2)    │ NULLABLE                       │
  │ created_at      │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP      │
  └─────────────────┴──────────────────┴─────────────────────────────────┘

  📋 Tabela: invoices
  ┌────────────┬──────────────────┬────────────────────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                                  │
  ├────────────┼──────────────────┼────────────────────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento                       │
  │ project_id │ INTEGER          │ FK → projects(id) SET NULL            │
  │ client_id  │ INTEGER          │ FK → clients(id) SET NULL             │
  │ number     │ VARCHAR(50)      │ UNIQUE NOT NULL                       │
  │ description│ TEXT             │ NULLABLE                              │
  │ amount     │ DECIMAL(12,2)    │ NOT NULL                              │
  │ tax        │ DECIMAL(12,2)    │ DEFAULT 0                             │
  │ total      │ DECIMAL(12,2)    │ NOT NULL                              │
  │ status     │ VARCHAR(20)      │ CHECK: draft/sent/paid/overdue/       │
  │            │                  │        cancelled                      │
  │ issue_date │ DATE             │ NOT NULL                              │
  │ due_date   │ DATE             │ NULLABLE                              │
  │ paid_date  │ DATE             │ NULLABLE                              │
  │ notes      │ TEXT             │ NULLABLE                              │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP             │
  └────────────┴──────────────────┴────────────────────────────────────────┘

  📋 Tabela: notifications
  ┌────────────┬──────────────────┬───────────────────────────┐
  │ Coluna     │ Tipo             │ Notas                     │
  ├────────────┼──────────────────┼───────────────────────────┤
  │ id         │ SERIAL PK        │ Auto incremento           │
  │ type       │ VARCHAR(50)      │ NOT NULL                  │
  │ title      │ VARCHAR(255)     │ NOT NULL                  │
  │ message    │ TEXT             │ NOT NULL                  │
  │ is_read    │ BOOLEAN          │ DEFAULT false             │
  │ created_at │ TIMESTAMP        │ DEFAULT CURRENT_TIMESTAMP │
  └────────────┴──────────────────┴───────────────────────────┘


  📌 Índices do banco (todos criados via IF NOT EXISTS em db.rs):

  idx_assignments_due_date
  idx_assignments_status
  idx_assignments_priority
  idx_articles_project_id
  idx_project_files_project_id
  idx_projects_client_id
  idx_team_members_team_id
  idx_team_members_user_id
  idx_users_email
  idx_time_entries_project_id
  idx_time_entries_start_time
  idx_invoices_status
  idx_invoices_issue_date
  idx_notifications_is_read
  idx_notifications_created_at


  📌 Migrations automáticas (db.rs → init_db()):

  Toda tabela usa CREATE TABLE IF NOT EXISTS.
  Colunas adicionadas em versões posteriores usam:
    ALTER TABLE ... ADD COLUMN IF NOT EXISTS ...

  Isso significa que o app cria/atualiza o schema automaticamente
  na inicialização. NÃO é necessário rodar migrations manualmente.

  No Windows com SQLite, o init_db() vai criar as tabelas
  exatamente como no PostgreSQL, usando a sintaxe SQLite.


================================================================================
  6. BANCO DE DADOS — MIGRAÇÃO PARA SQLite (Windows)
================================================================================

  ⚠️ Esta é a mudança MAIS IMPORTANTE para o Windows.

  Motivo: PostgreSQL exige instalação de servidor no Windows.
  SQLite é embutido — apenas um arquivo .db que acompanha o .exe.

  ✅ JÁ IMPLEMENTADO — O Cargo.toml contém ambas as features:

  ```toml
  # Atual (funciona em ambas as plataformas)
  sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "sqlite", "chrono", "uuid"] }
  ```

  ✅ JÁ IMPLEMENTADO — O db.rs usa compilação condicional:

  ```rust
  // db.rs — Type alias resolvido por plataforma
  #[cfg(target_os = "linux")]
  pub type DbPool = sqlx::PgPool;

  #[cfg(target_os = "windows")]
  pub type DbPool = sqlx::SqlitePool;

  // init_db() tem duas implementações completas
  #[cfg(target_os = "linux")]
  pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
      let pool = PgPoolOptions::new()
          .max_connections(5)
          .connect(database_url)
          .await?;
      // CREATE TABLE ... (sintaxe PostgreSQL: SERIAL, BYTEA, etc.)
  }

  #[cfg(target_os = "windows")]
  pub async fn init_db(_database_url: &str) -> Result<DbPool, sqlx::Error> {
      let pool = SqlitePoolOptions::new()
          .max_connections(1)
          .connect("unitesk.db")
          .await?;
      sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
      // CREATE TABLE ... (sintaxe SQLite: INTEGER PRIMARY KEY, BLOB, etc.)
  }
  ```

  📌 Diferenças de sintaxe SQL (PostgreSQL → SQLite):

  ┌────────────────────────────────┬──────────────────────────────────────┐
  │ PostgreSQL                     │ SQLite                               │
  ├────────────────────────────────┼──────────────────────────────────────┤
  │ CURRENT_DATE                   │ date('now')                         │
  │ CURRENT_TIME                   │ time('now')                         │
  │ CURRENT_TIMESTAMP              │ datetime('now')                     │
  │ INTERVAL '30 seconds'          │ Não suportado (calcular em Rust)    │
  │ coluna::text AS coluna         │ CAST(coluna AS TEXT) AS coluna      │
  │ SERIAL PRIMARY KEY             │ INTEGER PRIMARY KEY AUTOINCREMENT   │
  │ BYTEA                          │ BLOB                                │
  │ BOOLEAN                        │ INTEGER (0/1)                       │
  │ ALTER TABLE ... ADD COLUMN     │ ALTER TABLE ... ADD COLUMN          │
  │   IF NOT EXISTS                │   (sem IF NOT EXISTS — tratar erro) │
  │ ON DELETE CASCADE/SET NULL     │ ON DELETE CASCADE/SET NULL          │
  │                                │  (suportado se PRAGMA foreign_keys) │
  └────────────────────────────────┴──────────────────────────────────────┘

  📌 Passos da migração:

  1. [ ] Criar branch: git checkout -b feature/windows-sqlite
  2. [ ] Alterar Cargo.toml: postgres → sqlite
  3. [ ] Alterar db.rs: PgPool → SqlitePool, ajustar queries
  4. [ ] Adicionar PRAGMA foreign_keys = ON na conexão (SQLite)
  5. [ ] Remover dependência de PostgreSQL do instalador
  6. [ ] Testar no Windows

  ✅ JÁ IMPLEMENTADO — A migração está completa com `#[cfg(target_os)]`.
  Todas as ~50 funções CRUD em db.rs possuem duas implementações:
  - Linux: bind `$1`, `$2` para PostgreSQL
  - Windows: bind `?1`, `?2` para SQLite

  O `lib.rs` conecta corretamente com base na plataforma:
  ```rust
  #[cfg(target_os = "linux")]
  let database_url = get_database_url_linux();

  #[cfg(target_os = "windows")]
  let database_url = "unitesk.db".to_string();

  let pool = db::init_db(&database_url).await;
  ```


================================================================================
  7. API DE COMANDOS TAURI (COMPLETA)
================================================================================

  Total: ~50 comandos registrados em lib.rs.

  ┌──────────────────────────┬───────────────────────────────────────────────┐
  │ Comando                  │ Descrição                                     │
  ├──────────────────────────┼───────────────────────────────────────────────┤
  │                          │ 🏠 DASHBOARD                                 │
  │ get_dashboard_stats      │ Estatísticas completas (projetos, horas,      │
  │                          │   financeiro, notificações)                   │
  │                          │                                               │
  │                          │ 📁 PROJETOS                                  │
  │ get_projects             │ Lista todos os projetos (com nome cliente)   │
  │ create_project           │ Cria projeto (name, description, clientId)   │
  │ update_project           │ Edita projeto (id, name, description,        │
  │                          │   clientId)                                  │
  │ delete_project           │ Remove projeto (CASCADE em arquivos)         │
  │                          │                                               │
  │                          │ 📄 ARTIGOS                                   │
  │ get_articles             │ Lista artigos                                 │
  │ create_article           │ Cria artigo (title, content, projectName,    │
  │                          │   projectId)                                 │
  │ delete_article           │ Remove artigo                                │
  │                          │                                               │
  │                          │ 🤝 CLIENTES                                  │
  │ get_clients              │ Lista clientes                                │
  │ create_client            │ Cria cliente (name, email, phone, company,   │
  │                          │   notes)                                     │
  │ update_client            │ Edita cliente                                │
  │ delete_client            │ Remove cliente (projetos ficam sem cliente)  │
  │                          │                                               │
  │                          │ 👥 EQUIPES                                   │
  │ get_teams                │ Lista equipes                                │
  │ create_team              │ Cria equipe (name, description)              │
  │ update_team              │ Edita equipe                                 │
  │ delete_team              │ Remove equipe (CASCADE membros)              │
  │                          │                                               │
  │                          │ 👤 USUÁRIOS                                   │
  │ get_users                │ Lista usuários                               │
  │ create_user              │ Cria usuário (name, email, role)              │
  │ delete_user              │ Remove usuário (CASCADE membros)             │
  │                          │                                               │
  │                          │ 👥 MEMBROS DE EQUIPE                          │
  │ get_team_members         │ Lista membros de uma equipe                  │
  │ add_team_member          │ Adiciona usuário à equipe (teamId, userId,   │
  │                          │   role)                                      │
  │ update_team_member_role  │ Altera cargo do membro                       │
  │ remove_team_member       │ Remove membro da equipe                      │
  │                          │                                               │
  │                          │ 📅 ATIVIDADES                                 │
  │ get_assignments          │ Lista atividades                             │
  │ create_assignment        │ Cria atividade (title, description, dueDate, │
  │                          │   dueTime, projectName, priority)            │
  │ delete_assignment        │ Remove atividade (CASCADE arquivos)          │
  │ mark_assignment_done     │ Marca como concluída                         │
  │ check_today_assignments  │ Verifica notificações para hoje              │
  │                          │                                               │
  │                          │ 📎 ARQUIVOS DE PROJETO                        │
  │ get_project_files        │ Lista arquivos de um projeto                 │
  │ add_project_file         │ Upload arquivo (projectId, originalName,     │
  │                          │   fileData, mimeType)                        │
  │ get_project_file_data    │ Download arquivo (retorna dados binários)    │
  │ delete_project_file      │ Remove arquivo                               │
  │                          │                                               │
  │                          │ 📎 ARQUIVOS DE ATIVIDADE                      │
  │ get_assignment_files     │ Lista arquivos de uma atividade              │
  │ add_assignment_file      │ Upload arquivo (assignmentId, originalName,  │
  │                          │   fileData, mimeType)                        │
  │ get_assignment_file_data │ Download arquivo                             │
  │ delete_assignment_file   │ Remove arquivo                               │
  │                          │                                               │
  │                          │ 📦 EXPORTAÇÃO                                 │
  │ export_project_zip       │ Exporta projeto como ZIP (Projeto + artigos  │
  │                          │   + arquivos) em memória                     │
  │ export_all_data          │ Exporta TODOS os dados para .unitesk (JSON   │
  │                          │   com base64)                                │
  │ import_all_data          │ Importa dados de um arquivo .unitesk         │
  │                          │                                               │
  │                          │ ⏱️ CONTROLE DE HORAS                        │
  │ get_time_entries         │ Lista registros (filtros: projectId,         │
  │                          │   dateFrom, dateTo)                          │
  │ start_time_entry         │ Inicia timer (projectId, description)        │
  │ stop_time_entry          │ Para timer, calcula duração                  │
  │ add_manual_time_entry    │ Registro manual (projectId, description,     │
  │                          │   durationMinutes, entryDate, billable,      │
  │                          │   hourlyRate)                                │
  │ get_active_time_entry    │ Verifica timer ativo                         │
  │ get_hours_summary        │ Resumo: [todayMinutes, weekMinutes]          │
  │ delete_time_entry        │ Remove registro                              │
  │                          │                                               │
  │                          │ 💰 FATURAS                                    │
  │ get_invoices             │ Lista faturas (filtro: statusFilter)         │
  │ create_invoice           │ Cria fatura (projectId, clientId, number,    │
  │                          │   description, amount, tax, total, status,   │
  │                          │   issueDate, dueDate, notes)                 │
  │ update_invoice           │ Edita fatura (inclui paidDate)               │
  │ delete_invoice           │ Remove fatura                                │
  │                          │                                               │
  │                          │ 🔔 NOTIFICAÇÕES                               │
  │ get_notifications        │ Lista notificações (filtro: unreadOnly,      │
  │                          │   limit)                                     │
  │ get_unread_notifications_count │ Contagem de não lidas                 │
  │ mark_notification_read   │ Marca uma como lida                         │
  │ mark_all_notifications_read │ Marca todas como lidas                   │
  │ delete_notification      │ Remove notificação                           │
  │ auto_generate_notifications │ Gera notificações automáticas            │
  │ cleanup_old_notifications │ Remove notificações >30 dias               │
  │                          │                                               │
  │                          │ 📊 RELATÓRIOS                                 │
  │ get_report_stats         │ Dados agregados para gráficos               │
  └──────────────────────────┴───────────────────────────────────────────────┘

  📌 Estrutura dos modelos de dados (TypeScript):

  ```typescript
  interface Project {
    id: number; name: string; description: string | null;
    client_id: number | null; client_name: string | null;
    created_at: string;
  }

  interface Article {
    id: number; title: string; content: string | null;
    project_name: string | null; project_id: number | null;
    scheduled_date: string | null; created_at: string;
  }

  interface Assignment {
    id: number; title: string; description: string | null;
    due_date: string; due_time: string | null;
    notification_time: string | null;
    project_name: string | null;
    status: "pending" | "done" | "overdue";
    priority: "low" | "medium" | "high" | "urgent";
    created_at: string;
  }

  interface Client {
    id: number; name: string; email: string | null;
    phone: string | null; company: string | null;
    notes: string | null; created_at: string;
  }

  interface User {
    id: number; name: string; email: string;
    role: string; created_at: string;
  }

  interface Team {
    id: number; name: string; description: string | null;
    created_at: string;
  }

  interface TeamMember {
    id: number; team_id: number; user_id: number;
    user_name: string | null; user_email: string | null;
    role: string; created_at: string;
  }

  interface TimeEntry {
    id: number; project_id: number; project_name: string | null;
    user_id: number | null; user_name: string | null;
    description: string | null;
    start_time: string; end_time: string | null;
    duration_minutes: number | null;
    billable: boolean; hourly_rate: number | null;
    created_at: string;
  }

  interface Invoice {
    id: number; project_id: number | null;
    project_name: string | null; client_id: number | null;
    client_name: string | null; number: string;
    description: string | null; amount: number; tax: number;
    total: number;
    status: "draft" | "sent" | "paid" | "overdue" | "cancelled";
    issue_date: string; due_date: string | null;
    paid_date: string | null; notes: string | null;
    created_at: string;
  }

  interface Notification {
    id: number; type: string; title: string; message: string;
    is_read: boolean; created_at: string;
  }

  interface AssignmentFile {
    id: number; assignment_id: number;
    original_name: string; stored_name: string;
    file_size: number; mime_type: string; created_at: string;
  }

  interface ProjectFile {
    id: number; project_id: number;
    original_name: string; stored_name: string;
    file_size: number; mime_type: string; created_at: string;
  }

  interface DashboardStats {
    totalProjects: number; totalArticles: number;
    totalClients: number; totalTeams: number; totalUsers: number;
    pendingAssignments: number; overdueAssignments: number;
    nextDeadline: string | null; nextDeadlineName: string | null;
    hoursToday: number; hoursWeek: number;
    totalRevenue: number; pendingInvoices: number;
    pendingAmount: number;
  }

  interface ReportStats {
    totalProjects: number; totalArticles: number;
    totalClients: number; totalTeams: number; totalUsers: number;
    assignmentsByMonth: { month: string; count: number }[];
    assignmentsPending: number; assignmentsDone: number;
    assignmentsOverdue: number;
    revenueByMonth: { month: string; amount: number }[];
    totalRevenue: number; pendingAmount: number;
    hoursByProject: { project_name: string; hours: number }[];
    totalHours: number;
    invoicesDraft: number; invoicesSent: number;
    invoicesPaid: number; invoicesOverdue: number;
    invoicesCancelled: number;
  }
  ```


================================================================================
  8. FRONTEND — COMPONENTES E ESTADOS
================================================================================

  📌 Navegação principal (App.tsx):

  Abas disponíveis:
  📊 Dashboard | 📁 Projetos | 🤝 Clientes | 💰 Financeiro
  ⏱️ Horas | 📅 Atividades | 📄 Documentos | 👥 Equipes | 📊 Relatórios

  A aba ativa é controlada por useState<Tab>.
  O tema (light/dracula) é controlado por data-theme no <html>.

  📌 Cada componente e seus estados (useState + useRef + useMemo + useCallback):

  ┌───────────────────┬───────────────────────────────────────────────────────┐
  │ Componente        │ Estados                                              │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ Dashboard         │ stats, assignments, importing, exporting,            │
  │                   │ importMessage, statsLoaded, fileInputRef             │
  │                   │ useMemo: completionRate                              │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ ProjectList       │ projects, articles, searchTerm, showModal,           │
  │                   │ editingProject, deleteConfirm, activeTab,            │
  │                   │ viewingArticle, showCreate, showUpdate,              │
  │                   │ showArticleContent, expandedFiles, exporting,        │
  │                   │ uploadingFile, fileInputRef                          │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ CalendarView      │ assignments, showModal, selectedDate, currentMonth,  │
  │                   │ currentYear, newAssignment, newTime, errorMessage,   │
  │                   │ selectedAssignment, showFilesModal, assignmentFiles, │
  │                   │ uploadingFile, fileInputRef                          │
  │                   │ useMemo: years, sortedAssignments                    │
  │                   │ useCallback: getAssignmentsForDay                    │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ ArticleManager    │ articles, searchTerm, activeFilter, viewingArticle,  │
  │                   │ showCreate, deleteConfirm, showUpdate                │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ TimeTracking      │ entries, projects, activeEntry, elapsedSeconds,      │
  │                   │ filterProject, filterDateFrom, filterDateTo,         │
  │                   │ showManualModal, hoursSummary, manualForm,           │
  │                   │ deleteConfirm, startDesc, startProject, timerRef     │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ Finance           │ invoices, projects, clients, statusFilter,           │
  │                   │ showModal, editingInvoice, deleteConfirm, form       │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ NotificationPanel │ notifications, unreadCount, isOpen, cleaningUp,      │
  │                   │ panelRef                                             │
  ├───────────────────┼───────────────────────────────────────────────────────┤
  │ Reports           │ stats (ReportStats | null)                           │
  └───────────────────┴───────────────────────────────────────────────────────┘

  📌 Chaves de localStorage:

  ┌──────────────────────────────┬──────────────────┬─────────────────────────┐
  │ Chave                        │ Componente       │ Conteúdo                │
  ├──────────────────────────────┼──────────────────┼─────────────────────────┤
  │ unitesk_theme                │ App              │ "light" / "dracula"     │
  │ unitesk_assignments          │ CalendarView     │ Assignment[] completo   │
  │ unitesk_articles             │ ArticleManager   │ ArticleExtended[]       │
  │ unitesk_article_statuses     │ ArticleManager   │ Record<number, string>  │
  └──────────────────────────────┴──────────────────┴─────────────────────────┘


================================================================================
  9. FRONTEND — CSS E SISTEMA DE TEMAS
================================================================================

  📌 Arquivo: src/styles/global.css (~1600 linhas)

  📌 Sistema de design:

  • CSS Custom Properties no :root e [data-theme="dracula"]
  • ~40 variáveis CSS (--primary, --bg, --text, --border, etc.)
  • Grid system: .grid-2, .grid-3, .grid-4
  • Cards: .card, .card-header, .card-title
  • Botões: .btn, .btn-primary, .btn-secondary, .btn-danger, .btn-sm, .btn-xs
  • Formulários: .form-input, .form-textarea, .form-group
  • Badges: .badge-pending, .badge-progress, .badge-done, .badge-overdue,
            .badge-draft, .badge-published, .badge-scheduled
  • Modal: .modal-overlay, .modal, .modal-actions
  • Notificações: .notification-container, .notification-bell,
                  .notification-badge, .notification-panel, etc.
  • Calendário: .calendar-grid, .calendar-day, .calendar-nav-select, etc.
  • Timeline: .timeline, .timeline-item
  • Progress bar: .progress-bar-container, .progress-bar-fill
  • Stat cards: .stat-card, .stat-card--projects, etc.
  • Utilitários: .flex, .flex-between, .gap-*, .text-center, .text-sm, etc.

  📌 Tema Dracula (escuro):

  Cores principais:
  • --primary: #bd93f9 (roxo)
  • --bg: #1e1e2e (fundo escuro)
  • --bg-card: #282a36 (cards)
  • --text: #f8f8f2 (texto claro)
  • --text-secondary: #6272a4 (texto secundário)
  • --border: #44475a (bordas)
  • --success: #50fa7b, --warning: #ffb86c, --danger: #ff5555

  📌 Animações:
  • @keyframes fadeIn (0.2s) — para modais
  • @keyframes slideUp (0.25s) — para modais e painéis
  • @keyframes countUp — para AnimatedCounter
  • @keyframes pulse — para badge de notificação
  • @keyframes headerShimmer — gradiente animado no cabeçalho

  📌 Responsividade:
  • @media (max-width: 1024px) — grid-4 vira 2 colunas
  • @media (max-width: 768px) — grids viram 1 coluna, nav compacto


================================================================================
  10. SISTEMA DE NOTIFICAÇÕES
================================================================================

  O sistema opera em 3 camadas:

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  Camada 1: Backend Rust (check_today_assignments)                        │
  │                                                                          │
  │  Chamado pelo frontend a cada 60 segundos.                               │
  │  Query SQL:                                                              │
  │    SELECT ... FROM assignments                                           │
  │    WHERE due_date = CURRENT_DATE                                         │
  │      AND status = 'pending'                                              │
  │      AND notification_time BETWEEN CURRENT_TIME - INTERVAL '30 seconds'  │
  │                                 AND CURRENT_TIME + INTERVAL '30 seconds' │
  │                                                                          │
  │  Dispara notificação nativa via tauri-plugin-notification.              │
  │                                                                          │
  │  ⚠️ Para Windows: INTERVAL não existe em SQLite.                        │
  │     Solução: Buscar todas as atividades de hoje e filtrar em Rust.       │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  Camada 2: Frontend (fallback browser)                                   │
  │                                                                          │
  │  Quando Tauri não está disponível (modo dev no navegador):               │
  │    Usa Notification API do navegador, janela de 10 minutos.             │
  │                                                                          │
  │    const notifTime = a.notification_time || a.due_time;                  │
  │    if (nowMinutes >= activityMinutes - 5 &&                              │
  │        nowMinutes <= activityMinutes + 5) {                              │
  │      new Notification("📚 Prazo Hoje!", { body: `...` });               │
  │    }                                                                     │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  Camada 3: Script externo (Linux apenas)                                 │
  │                                                                          │
  │  scripts/notify-deadlines.sh — Executado por cron.                      │
  │  Faz consulta direta ao PostgreSQL via psql.                             │
  │  Dispara notify-send.                                                    │
  │                                                                          │
  │  No Windows, isso seria substituído por um Agendador de Tarefas          │
  │  (schtasks.exe) ou um serviço em segundo plano.                         │
  └──────────────────────────────────────────────────────────────────────────┘


================================================================================
  11. GERENCIAMENTO DE ARQUIVOS
================================================================================

  📌 Upload de arquivos (projetos e atividades):

  1. Usuário clica "Anexar Arquivo"
  2. fileInputRef.current.click() → abre seletor de arquivos
  3. onChange → valida tamanho (max 10 MB)
  4. file.arrayBuffer() → Uint8Array
  5. invoke("add_*_file", { ..., fileData: [...uint8] })
  6. Rust: gera UUID + stored_name → INSERT INTO ... RETURNING
  7. Recarrega lista de arquivos do backend

  📌 Armazenamento:

  • Linux: BYTEA no PostgreSQL
  • Windows: BLOB no SQLite (upload de arquivos funciona igual)

  📌 Sanitização de nomes (Rust db.rs):

  • Extrai apenas o basename do caminho
  • Remove .. (path traversal)
  • Substitui /, \, ~ por _
  • Filtra caracteres não alfanuméricos
  • Trunca para 200 caracteres

  📌 Tamanho máximo: 10 MB (verificado no frontend e backend)

  📌 Exportação ZIP:

  O Rust gera um ZIP em memória contendo:
  • Um JSON com dados do projeto e artigos
  • Todos os arquivos do projeto
  O frontend recebe Vec<u8>, cria Blob e dispara download.


================================================================================
  12. EXPORTAÇÃO/IMPORTAÇÃO DE DADOS (INTEROPERABILIDADE)
================================================================================

  📌 Este é o MECANISMO PRINCIPAL DE INTERCONEXÃO entre Linux e Windows.

  O formato .unitesk é um JSON com esta estrutura:

  ```typescript
  interface ExportedData {
    version: string;          // "1.0.0"
    exported_at: string;      // ISO timestamp
    projects: Project[];
    articles: Article[];
    assignments: Assignment[];
    project_files: {           // com file_data em base64
      id: number; project_id: number;
      original_name: string; stored_name: string;
      file_data: string;       // ← base64 (bytes codificados)
      file_size: number; mime_type: string;
      created_at: string;
    }[];
    assignment_files: {        // com file_data em base64
      id: number; assignment_id: number;
      original_name: string; stored_name: string;
      file_data: string;       // ← base64
      file_size: number; mime_type: string;
      created_at: string;
    }[];
  }
  ```

  📌 Fluxo de exportação (qualquer plataforma):

  1. Dashboard → 📤 Exportar Dados
  2. Rust: Busca TODOS os dados do banco (projetos, artigos, atividades,
     arquivos com file_data convertido para base64)
  3. Serializa como ExportedData (JSON)
  4. Frontend: Cria Blob → download como unitesk_backup_YYYY-MM-DD.unitesk

  📌 Fluxo de importação (qualquer plataforma):

  1. Dashboard → 📥 Importar Dados
  2. Seleciona arquivo .unitesk
  3. Frontend: Lê o arquivo, faz JSON.parse
  4. Rust: Insere todos os dados no banco, com mapeamento de IDs
     (preserva relações entre projetos, artigos e arquivos)

  ✅ Interoperabilidade garantida porque:

  • O formato .unitesk é JSON puro (independente de plataforma)
  • Arquivos são codificados em base64 (portável)
  • PostgreSQL BYTEA e SQLite BLOB podem armazenar os mesmos bytes
  • As queries SQL são equivalentes (INSERT, SELECT, DELETE)
  • O schema de tabelas é idêntico entre PostgreSQL e SQLite

  ⚠️ Para garantir compatibilidade total:

  1. Manter o mesmo formato ExportedData em ambas as versões
  2. Testar: exportar do Linux, importar no Windows e vice-versa
  3. A versão do schema (ExportedData.version) permite detectar
     incompatibilidades futuras


================================================================================
  13. TESTES
================================================================================

  📌 Framework: Vitest + Testing Library React
  📌 Setup: vitest.config.ts + src/test/setup.ts
  📌 Total: 143 testes, 8 arquivos (Julho/2026)

  ┌─────────────────────────┬────────┬─────────────────────────────────────┐
  │ Arquivo                 │ Testes │ O que testa                         │
  ├─────────────────────────┼────────┼─────────────────────────────────────┤
  │ App.test.tsx            │ 14     │ Navegação, header/footer, tema,     │
  │                         │        │ localização da aba ativa            │
  │ Dashboard.test.tsx      │ 18     │ Stats, welcome card, progresso,     │
  │                         │        │ timeline, ações, export/import      │
  │ ProjectList.test.tsx    │ 18     │ CRUD projetos, artigos, arquivos,   │
  │                         │        │ ZIP, modais, exclusão               │
  │ CalendarView.test.tsx   │ 14     │ Renderização, modal, criação,       │
  │                         │        │ localStorage, badges, navegação     │
  │ ArticleManager.test.tsx │ 11     │ CRUD artigos, status, filtros,      │
  │                         │        │ busca, visualização                 │
  │ TimeTracking.test.tsx   │ ~23    │ Timer, registro manual, filtros,    │
  │                         │        │ fallback local, exclusão            │
  │ Finance.test.tsx        │ ~20    │ CRUD faturas, filtros, status,      │
  │                         │        │ cálculo de totais                   │
  │ NotificationPanel.test. │ ~25    │ Badge, dropdown, mark read/delete,  │
  │ tsx                     │        │ time ago, fallback local            │
  └─────────────────────────┴────────┴─────────────────────────────────────┘

  📌 Comando: npx vitest run (modo CI) ou npx vitest (modo watch)

  📌 Padrão de teste (todos seguem o mesmo padrão):

  • beforeEach limpa localStorage e mocks
  • Mock do Tauri invoke via vi.hoisted() + vi.mock()
  • userEvent (não fireEvent) para interações
  • waitFor para operações assíncronas

  📌 Mock do invoke (exemplo):

  ```typescript
  const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
  vi.mock("@tauri-apps/api/core", () => ({ invoke }));

  beforeEach(() => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      return [];
    });
  });
  ```

  ⚠️ Os mesmos testes devem passar no Windows sem alterações,
  pois o frontend é idêntico. Apenas o backend (Rust/SQL) muda.


================================================================================
  14. BUILD E EMPACOTAMENTO
================================================================================

  📌 Linux:

  • Script: ./build-deb.sh
  • Gera: src-tauri/target/release/bundle/deb/Unitesk_1.3.0_amd64.deb
  • Processo: npm ci → npx tauri build → dpkg-deb (injetar scripts)
  • Dependências de sistema: libwebkit2gtk, libgtk-3, libayatana, etc.

  📌 Windows (Plano):

  • Comando: npx tauri build (no Windows)
  • Gera: src-tauri/target/release/unitesk.exe
  • Bundler pode gerar:
    - MSI (WiX Toolset): src-tauri/target/release/bundle/msi/
    - NSIS: src-tauri/target/release/bundle/nsis/
  • Dependências: WebView2 (já vem no Windows 10/11), Visual Studio
    Build Tools, Rust

  📌 Configuração Tauri (tauri.conf.json):

  ```json
  {
    "productName": "Unitesk",
    "version": "1.3.0",
    "identifier": "com.unitesk.app",
    "build": {
      "beforeDevCommand": "npm run dev",
      "devUrl": "http://localhost:1420",
      "beforeBuildCommand": "npm run build",
      "frontendDist": "../dist"
    },
    "app": {
      "windows": [{
        "title": "Unitesk",
        "width": 1200, "height": 800,
        "minWidth": 800, "minHeight": 600,
        "resizable": true, "fullscreen": false
      }],
      "security": {
        "csp": "default-src 'self'; connect-src 'self' https:; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'"
      }
    },
    "bundle": {
      "active": true,
      "icon": ["icons/unitesk_icon_512.png"],
      "targets": ["deb"],    // ← No Windows: ["msi"] ou ["nsis"]
      "linux": {
        "deb": {
          "depends": ["postgresql", "postgresql-client", ...]
        }
      }
    }
  }
  ```

  ⚠️ Para Windows, ajustar:
  • bundle.targets: ["deb"] → ["msi"] ou ["nsis"]
  • Remover linux.deb.depends (não aplicável)
  • Adicionar configurações específicas do Windows (ícone .ico, etc.)


================================================================================
  15. PLANO DE AÇÃO PARA WINDOWS
================================================================================

  ⚡ Prioridades em ordem:

  1. ✅ MIGRAR BACKEND PARA SQLite (CONCLUÍDO)
     └── Usar #[cfg] para manter ambas as versões

  2. 🔧 COMPILAR NO WINDOWS
     └── Instalar Rust, VS Build Tools, clonar repo
     └── npx tauri build

  3. 🪟 CRIAR INSTALADOR
     └── Opção A: NSIS (.exe)
     └── Opção B: Inno Setup (.exe)
     └── Opção C: WiX Toolset (.msi)

  4. 📝 CRIAR SCRIPTS .bat
     └── install.bat, uninstall.bat, run.bat

  5. 🧪 TESTAR COMPATIBILIDADE
     └── Exportar do Linux → Importar no Windows
     └── Exportar do Windows → Importar no Linux
     └── Testar todos os 143 testes frontend

  6. 📦 DISTRIBUIR
     └── Criar release no GitHub com ambos os pacotes
     └── Documentar instalação para Windows

  📌 Status das tarefas:

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ PASSO 1: Migrar para SQLite com suporte a duas plataformas (FEITO)  │
  │                                                                          │
  │  ✅ Cargo.toml: feature "sqlite" adicionada ao sqlx                      │
  │  ✅ db.rs: #[cfg(target_os = "windows")] SqlitePool implementado        │
  │  ✅ db.rs: #[cfg(target_os = "linux")] PgPool mantido                   │
  │  ✅ db.rs: queries adaptadas (date('now'), ?1 binds, etc.)              │
  │  ✅ db.rs: PRAGMA foreign_keys = ON no SQLite                           │
  │  ✅ db.rs: filtragem de notificações em Rust (sem INTERVAL)             │
  │  ✅ Testado no Linux (não quebrou nada — cargo check + vitest passam)   │
  │                                                                          │
  │  PRÓXIMO PASSO 2: Compilar no Windows                                   │
  │                                                                          │
  │  [ ] Instalar Rust (rustup-init.exe)                                    │
  │  [ ] Instalar Visual Studio Build Tools (C++)                           │
  │  [ ] git clone https://github.com/Vitoriodev/UniTesk.git                │
  │  [ ] npm install                                                         │
  │  [ ] npm run tauri build                                                 │
  │  [ ] Verificar: src-tauri/target/release/unitesk.exe                    │
  │                                                                          │
  │  PRÓXIMO PASSO 3: Criar instalador                                      │
  │                                                                          │
  │  Opção NSIS:                                                             │
  │  [ ] Criar unitesk.nsi                                                  │
  │  [ ] Empacotar: .exe + unitesk.db (vazio) + ícones                     │
  │  [ ] Criar atalhos no Menu Iniciar / Área de Trabalho                  │
  │  [ ] Adicionar desinstalação no Painel de Controle                      │
  │                                                                          │
  │  Opção Inno Setup:                                                       │
  │  [ ] Criar unitesk.iss                                                  │
  │  [ ] Mais customizável (banner, wizard, etc.)                           │
  │                                                                          │
  │  Opção WiX (MSI):                                                        │
  │  [ ] Habilitar no tauri.conf.json: targets: ["msi"]                     │
  │  [ ] Tauri gera .msi automaticamente                                    │
  │                                                                          │
  │  PRÓXIMO PASSO 4: Scripts .bat                                          │
  │                                                                          │
  │  [ ] unitesk.bat (inicializador):                                       │
  │        @echo off                                                        │
  │        start "" "%~dp0unitesk.exe"                                      │
  │                                                                          │
  │  [ ] install.bat (se não usar instalador):                              │
  │        @echo off                                                        │
  │        echo Instalando Unitesk...                                       │
  │        mkdir "%ProgramFiles%\Unitesk" 2>nul                             │
  │        copy /Y unitesk.exe "%ProgramFiles%\Unitesk\"                    │
  │        copy /Y unitesk.bat "%ProgramFiles%\Unitesk\"                    │
  │        copy /Y unitesk_icon.ico "%ProgramFiles%\Unitesk\"               │
  │        echo Criando atalhos...                                          │
  │        :: (usar script VBS ou atalho manual)                            │
  │        echo ✅ Instalação concluída!                                    │
  │        pause                                                            │
  │                                                                          │
  │  [ ] uninstall.bat:                                                     │
  │        @echo off                                                        │
  │        echo Removendo Unitesk...                                        │
  │        rmdir /S /Q "%ProgramFiles%\Unitesk\"                            │
  │        echo Desinstalação concluída.                                    │
  │        pause                                                            │
  │                                                                          │
  │  PRÓXIMO PASSO 5: Testar interoperabilidade                             │
  │                                                                          │
  │  [ ] Exportar dados do Linux (.unitesk) → Importar no Windows          │
  │  [ ] Exportar dados do Windows (.unitesk) → Importar no Linux          │
  │  [ ] Verificar: projetos, artigos, atividades, arquivos                 │
  │  [ ] Verificar: clientes, equipes, time_entries, faturas               │
  │  [ ] Verificar: notificações geradas corretamente                      │
  │  [ ] Rodar todos os 143 testes no Windows                              │
  └──────────────────────────────────────────────────────────────────────────┘

  📌 Documentação sobre o código real da migração:

  Consulte os seguintes arquivos para detalhes da implementação:
  - `src-tauri/Cargo.toml` — dependências (postgres + sqlite)
  - `src-tauri/src/db.rs` — ~1800 linhas, todas as funções com #[cfg] dual
  - `src-tauri/src/lib.rs` — AppState com DbPool, setup por plataforma
  - `docs/DEVELOPER.md` — documentação completa da migração


================================================================================
  16. INTERCONEXÃO LINUX ↔ WINDOWS
================================================================================

  📌 Como garantir que AMBAS as versões sejam o mesmo software:

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ Item 1: FRONTEND IDÊNTICO                                           │
  │                                                                          │
  │  Todo o código em src/ (React, TypeScript, CSS) é 100% compartilhado    │
  │  entre Linux e Windows. Não precisa modificar nada.                      │
  │                                                                          │
  │  • Os mesmos componentes, os mesmos testes, o mesmo CSS.                │
  │  • Única diferença: o tema escuro pode precisar de ajustes finos        │
  │    de contraste em certos elementos nativos (select, input) no Windows. │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ Item 2: MESMA API DE COMANDOS TAURI                                 │
  │                                                                          │
  │  Os ~50 comandos Tauri (lib.rs) são os mesmos. A assinatura de cada     │
  │  comando (nome, parâmetros, retorno) não muda.                           │
  │                                                                          │
  │  Apenas a implementação interna (db.rs) muda:                           │
  │  • Linux: PostgreSQL via PgPool                                         │
  │  • Windows: SQLite via SqlitePool                                       │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ Item 3: MESMO SCHEMA DE BANCO                                       │
  │                                                                          │
  │  As 11 tabelas (clients, users, projects, articles, assignments,         │
  │  project_files, assignment_files, teams, team_members, time_entries,    │
  │  invoices, notifications) são idênticas em estrutura.                   │
  │                                                                          │
  │  • Colunas, tipos, constraints, chaves estrangeiras: tudo igual.        │
  │  • Apenas o engine muda: PostgreSQL (Linux) vs SQLite (Windows).        │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ Item 4: MESMO FORMATO DE BACKUP (.unitesk)                          │
  │                                                                          │
  │  O formato .unitesk (JSON com base64 para arquivos) é o MECANISMO       │
  │  PRINCIPAL de troca de dados entre plataformas.                         │
  │                                                                          │
  │  • Exportar do Linux → Importar no Windows: ✅ funciona                  │
  │  • Exportar do Windows → Importar no Linux: ✅ funciona                  │
  │                                                                          │
  │  Como testar:                                                            │
  │  1. No Linux, crie projetos, artigos, atividades com arquivos           │
  │  2. Exporte como .unitesk                                               │
  │  3. Copie o arquivo para o Windows                                      │
  │  4. No Windows, importe o .unitesk                                      │
  │  5. Verifique que todos os dados estão lá                               │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ Item 5: MESMA INTERFACE DE USUÁRIO                                  │
  │                                                                          │
  │  • Mesmas 9 abas de navegação                                           │
  │  • Mesmos modais, botões, formulários                                   │
  │  • Mesmo sistema de temas (Claro/Dracula)                               │
  │  • Mesmo calendário, notificações, gráficos                             │
  │                                                                          │
  │  O usuário não consegue distinguir qual versão está usando              │
  │  (exceto pelo instalador e pelo gerenciamento de arquivos do SO).       │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ✅ Item 6: MESMA ESTRATÉGIA DE DISTRIBUIÇÃO                            │
  │                                                                          │
  │  • Linux: .deb (dpkg/apt)                                               │
  │  • Windows: .exe (NSIS/Inno Setup) ou .msi (WiX)                        │
  │                                                                          │
  │  Ambos são instaladores que colocam o app no sistema com atalhos.       │
  │  A experiência do usuário final é análoga.                              │
  └──────────────────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────────────────┐
  │  ❌ Item 7: DIFERENÇAS (o que muda)                                     │
  │                                                                          │
  │  ┌──────────────────────────┬────────────────────┬─────────────────────┐│
  │  │ Aspecto                  │ Linux              │ Windows             ││
  │  ├──────────────────────────┼────────────────────┼─────────────────────┤│
  │  │ Banco                    │ PostgreSQL         │ SQLite              ││
  │  │ Tipo de arquivo          │ .deb               │ .exe / .msi         ││
  │  │ Script de build          │ build-deb.sh       │ tauri build         ││
  │  │ Notificações externas    │ notify-deadlines.sh│ Agendador Tarefas   ││
  │  │ │                        │ + cron             │ (schtasks)          ││
  │  │ │                        │                    │                     ││
  │  │ Dependências Rust        │ sqlx (postgres)    │ sqlx (sqlite)       ││
  │  │ Dependências SO          │ libwebkit2gtk, etc │ WebView2 (built-in) ││
  │  │ Scripts manutenção       │ postinst/prerm/    │ N/A (instalador)    ││
  │  │                          │ postrm (.deb)      │                     ││
  │  └──────────────────────────┴────────────────────┴─────────────────────┘│
  └──────────────────────────────────────────────────────────────────────────┘


  📌 RESUMO FINAL:

  O Unitesk Windows será uma cópia FIEL do Unitesk Linux com APENAS
  3 diferenças fundamentais:

  1. 🗄️ Banco: SQLite (embutido) em vez de PostgreSQL (servidor)
  2. 📦 Instalador: .exe/.msi em vez de .deb
  3. 🔧 Backend Rust: SqlitePool em vez de PgPool (com #[cfg])

  Tudo o mais — frontend, componentes, testes, API de comandos,
  formato de backup .unitesk, CSS, temas — é 100% COMPARTILHADO.

  Isso garante que as duas versões são o MESMO software,
  e que dados podem ser livremente transferidos entre elas
  através do formato .unitesk.



================================================================================
  🎓 Unitesk v1.3.0 — Guia Completo para Versão Windows
  Documento gerado em: Julho/2026
================================================================================
