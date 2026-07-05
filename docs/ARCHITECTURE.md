# 🏗️ Arquitetura — Unitesk

## 📐 Stack Tecnológica

| Componente        | Tecnologia                          |
|------------------|-------------------------------------|
| **Frontend**      | React 18 + TypeScript + Vite       |
| **Backend**       | Rust + Tauri 2                     |
| **Banco**         | PostgreSQL + SQLx (SQLite no Windows) |
| **Notificações**  | Tauri Plugin Notification          |
| **UI**            | CSS Custom Properties              |
| **Pacote**        | .deb (dpkg/apt)                    |

## 📁 Estrutura do Projeto

```
unitesk/
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
│   │   └── models.rs            # Estruturas de dados
│   ├── deb-scripts/             # Scripts de manutenção do pacote .deb
│   │   ├── postinst             # Configuração pós-instalação
│   │   ├── prerm                # Pré-remoção
│   │   └── postrm               # Pós-remoção (purge)
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
SQLx (dupla plataforma)
    │
    │  #[cfg(target_os = "linux")] → PgPool ($1 binds)
    │  #[cfg(target_os = "windows")] → SqlitePool (?1 binds)
    │
    ▼
┌──────────────────────┐
│ Linux: PostgreSQL    │
│ Windows: SQLite      │
└──────────────────────┘
```

### Exportação ZIP
```
React → invoke("export_project_zip", { projectId })
    │
    ▼
Rust → db::export_project_zip() (função por #[cfg])
    │  Busca projeto + artigos + arquivos
    │  build_zip() (compartilhada) cria ZIP em memória
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

## 📦 Distribuição

O Unitesk é distribuído **exclusivamente** como pacote `.deb`.

### Build

```bash
./build-deb.sh
```

O comando acima:
1. Instala dependências npm
2. Compila o frontend (React → Vite)
3. Compila o backend (Rust → Tauri)
4. Gera o pacote `.deb` em `src-tauri/target/release/bundle/deb/`

### Instalação na máquina de destino

```bash
sudo dpkg -i Unitesk_*.deb
sudo apt-get install -f
```

### O que o pacote .deb instala

| Caminho                      | Descrição                          |
|------------------------------|------------------------------------|
| `/usr/bin/unitesk`           | Binário principal                  |
| `/usr/share/applications/`   | Atalho no menu                     |
| `/usr/share/icons/`          | Ícones do aplicativo               |
| `/etc/unitesk/unitesk.conf`  | Configuração (DATABASE_URL)        |

### Scripts de manutenção (.deb)

O pacote inclui scripts que executam durante instalação/remoção:

| Script   | Execução                  | Função                                     |
|----------|---------------------------|--------------------------------------------|
| postinst | Após instalação           | Cria banco PostgreSQL, configura ambiente  |
| prerm    | Antes da remoção          | Avisa sobre preservação dos dados          |
| postrm   | Após remoção (purge)      | Remove configurações em `/etc/unitesk/`    |

## 🔐 Segurança

- Rust garante memory safety
- SQLx com queries compiladas em tempo de compilação
- Tipagem forte em toda comunicação Frontend ↔ Backend
- Notificações com permissão explícita do usuário
- Validação de tamanho de arquivo (10 MB max) no upload
