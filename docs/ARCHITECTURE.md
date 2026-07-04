# 🏗️ Arquitetura — Unitesk

## 📐 Stack Tecnológica

| Componente        | Tecnologia                          |
|------------------|-------------------------------------|
| **Frontend**      | React 18 + TypeScript + Vite       |
| **Backend**       | Rust + Tauri 2                     |
| **Banco**         | PostgreSQL + SQLx                  |
| **Notificações**  | Tauri Plugin Notification          |
| **UI**            | CSS Custom Properties              |
| **Instalador**    | Zenity (GUI) + Bash + C launcher   |

## 📁 Estrutura do Projeto

```
projetos/unitesk/
├── src/                         # Frontend React
│   ├── main.tsx                 # Entry point
│   ├── App.tsx                  # Componente principal com navegação
│   ├── components/
│   │   ├── Dashboard.tsx        # Visão geral com estatísticas
│   │   ├── ProjectList.tsx      # CRUD de projetos + artigos + arquivos + ZIP
│   │   ├── CalendarView.tsx     # Calendário + atividades (com localStorage)
│   │   └── ArticleManager.tsx   # Gerenciador de artigos
│   └── styles/
│       └── global.css           # Estilos globais
├── src-tauri/                   # Backend Rust
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── lib.rs               # Comandos Tauri + setup
│   │   ├── db.rs                # Operações de banco (CRUD + export ZIP)
│   │   ├── models.rs            # Estruturas de dados
│   │   └── setup_launcher.c     # Código-fonte do launcher binário
│   ├── Cargo.toml               # Dependências Rust
│   └── tauri.conf.json          # Configuração Tauri
├── docs/                        # Documentação
│   ├── README.md                # Guia principal
│   ├── ARCHITECTURE.md          # Este arquivo
│   ├── DATABASE.md              # Documentação do banco
│   ├── API.md                   # API de comandos Tauri
│   ├── LEIGO.md                 # Guia rápido para usuários
│   ├── WINDOWS.md               # Plano para versão Windows
│   └── setup.sql                # Schema SQL do banco
├── setup.sh                     # Assistente de instalação (GUI com Zenity)
├── unitesk-setup                # Binário do assistente (compilado com gcc)
├── install.sh                   # Instalador direto (terminal)
├── uninstall.sh                 # Desinstalador direto (terminal)
├── unitesk.sh                   # Script para executar o app
├── build-deb.sh                 # Script para gerar pacote .deb
├── CHANGELOG.md                 # Histórico de alterações
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tsconfig.node.json
```

## 🔄 Fluxo de Dados

```
React (Frontend)
    │
    │  invoke("comando", { args })
    │
    ▼
Tauri IPC (Rust)
    │
    │  Comandos em lib.rs
    │
    ▼
SQLx (PostgreSQL)
    │
    │  Queries em db.rs
    │
    ▼
PostgreSQL Database
```

### Exportação ZIP
```
React → invoke("export_project_zip", { projectId })
    │
    ▼
Rust → db::export_project_zip()
    │  Busca projeto + artigos + arquivos do DB
    │  Cria ZIP em memória com zip crate
    ▼
React → Recebe Vec<u8> → Cria blob → Download
```

## 🎯 Funcionalidades

| Funcionalidade          | Frontend                   | Backend                          |
|------------------------|----------------------------|----------------------------------|
| **Dashboard**          | Dashboard.tsx              | `get_dashboard_stats`            |
| **Projetos**           | ProjectList.tsx            | CRUD: get/create/update/delete   |
| **Artigos**            | ArticleManager.tsx         | CRUD: get/create/delete          |
| **Calendário**         | CalendarView.tsx           | CRUD: get/create/mark_done       |
| **Arquivos**           | ProjectList.tsx            | CRUD: get/add/get_data/delete    |
| **Exportar ZIP**       | ProjectList.tsx            | `export_project_zip`             |
| **Notificações**       | (automático)               | `check_today_assignments`        |

## 🖥️ Instalação

O projeto possui múltiplas formas de instalação:

| Método                  | Interface       | Quando usar                        |
|------------------------|----------------|------------------------------------|
| `./setup.sh`           | Zenity (GUI)   | Usuário com terminal               |
| `./unitesk-setup`      | Zenity (GUI)   | Usuário leigo (duplo clique)       |
| `./install.sh`         | Terminal       | Instalação direta/depuração        |
| `npx tauri build`      | —              | Desenvolvedores                    |
| `sudo dpkg -i *.deb`   | —              | Distribuição para outras máquinas  |

O `unitesk-setup` é um binário ELF compilado de `setup_launcher.c` que:
- Encontra o `setup.sh` no mesmo diretório
- Executa em segundo plano via `fork()` + `setsid()` (sem mostrar terminal)
- Redireciona saídas para `/tmp/unitesk_setup.log`

## 🔐 Segurança

- Rust garante memory safety
- SQLx com queries compiladas em tempo de compilação
- Tipagem forte em toda comunicação Frontend ↔ Backend
- Notificações com permissão explícita do usuário
- Validação de tamanho de arquivo (10 MB max) no upload
- Confirmação em 2 etapas na desinstalação
