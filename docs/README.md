# 🎓 Unitesk

Gerenciador de Projetos — Aplicativo desktop para organizar
artigos, atividades, arquivos e prazos.

## ✨ Funcionalidades

- 📁 **Projetos** — Crie, edite e exclua projetos
- 📄 **Artigos** — Armazene artigos e materiais de estudo
- 📎 **Arquivos** — Anexe PDFs, imagens e documentos aos projetos
- 📅 **Calendário** — Visualize prazos em calendário interativo com 🗑️ exclusão de atividades
- 🔔 **Notificações** — Alertas nativos no dia da entrega
- 📊 **Dashboard** — Visão geral com estatísticas em tempo real
- 🔍 **Busca** — Pesquise artigos pelo conteúdo
- 📦 **Exportar ZIP** — Exporte projetos completos (artigos + arquivos) em um arquivo ZIP
- 📤 **Exportar/Importar Backup** — Transfira todos os dados entre máquinas via arquivo `.unitesk`
- 🌙 **Tema Dracula** — Alternne entre tema claro e escuro com um clique

## 📦 Instalação via Pacote .deb (Único Método)

> O Unitesk é distribuído **exclusivamente** como pacote `.deb`
> para sistemas Debian, Ubuntu, Linux Mint e derivados.

### Requisitos

| Requisito     | Versão Mínima |
|--------------|---------------|
| PostgreSQL   | 14+           |

### Instalação

```bash
# 1. Instalar o pacote .deb
sudo dpkg -i Unitesk_1.3.0_amd64.deb

# 2. Corrigir dependências (se necessário)
sudo apt-get install -f

# 3. Pronto! Procure por "Unitesk" no menu de aplicativos
```

A instalação configura automaticamente:
- ✅ Binário em `/usr/bin/unitesk`
- ✅ Atalho no menu de aplicativos
- ✅ Banco de dados PostgreSQL configurado
- ✅ Arquivo de configuração em `/etc/unitesk/unitesk.conf`

### Desinstalação

#### 🖱️ Pela loja de aplicativos (sem terminal)

**Ubuntu (Ubuntu Software):**
1. Abra a "Ubuntu Software" no menu
2. Clique na aba "Instalados"
3. Procure por "Unitesk"
4. Clique em "Remover"

**Linux Mint (Gerenciador de Programas):**
1. Abra o "Gerenciador de Programas"
2. Vá em "Gerenciar" → "Instalados"
3. Encontre "Unitesk"
4. Clique em "Remover"

#### ⌨️ Pelo Terminal

```bash
# Remover o Unitesk (preserva o banco de dados)
sudo apt remove unitesk

# Remover completamente (incluindo configurações)
sudo apt purge unitesk
```

> 💾 Seus dados (projetos, artigos, atividades) ficam no banco PostgreSQL
> e **não são removidos** ao desinstalar. Para removê-los manualmente:
> ```bash
> sudo -u postgres psql -c "DROP DATABASE unitesk;"
> ```

## 🔧 Para Desenvolvedores: Build do Pacote .deb

Se você deseja **compilar o Unitesk** e gerar o pacote `.deb`:

### Pré-requisitos de Build

```bash
# Dependências de sistema
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libjavascriptcoregtk-4.1-dev \
  libssl-dev \
  postgresql postgresql-client

# Rust (via https://rustup.rs)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 18+ (recomendado via nvm: https://nvm.sh)
```

### Build

```bash
./build-deb.sh
```

O `.deb` será gerado em `src-tauri/target/release/bundle/deb/`.

## 📂 Estrutura do Projeto

```
unitesk/
├── public/                  # Assets estáticos (favicon)
├── src/                     # Frontend React
├── src-tauri/               # Backend Rust + ícone personalizado
│   ├── deb-scripts/         # Scripts de manutenção do pacote .deb
│   │   ├── postinst         # Configuração pós-instalação
│   │   ├── prerm            # Pré-remoção
│   │   └── postrm           # Pós-remoção (purge)
│   └── ...
├── build-deb.sh              # Script para gerar pacote .deb
├── CHANGELOG.md              # Histórico de alterações
└── docs/                    # Documentação completa
    ├── README.md            # Guia principal
    ├── DEVELOPER.md         # Documentação para desenvolvedores
    ├── ARCHITECTURE.md      # Arquitetura do projeto
    ├── DATABASE.md          # Documentação do banco
    ├── API.md               # API de comandos
    ├── LEIGO.md             # Guia rápido para usuários
    ├── WINDOWS.md           # Plano para versão Windows
    └── setup.sql            # Schema SQL do banco
```

Consulte a [documentação para desenvolvedores](./DEVELOPER.md) para mais detalhes.

## 🗄️ Banco de Dados

PostgreSQL com as tabelas: `projects`, `articles`, `assignments`, `project_files`, `assignment_files`.

Veja [DATABASE.md](./DATABASE.md) para setup e schema completo.

## 📡 API

Comandos Tauri disponíveis no arquivo [API.md](./API.md).

## 🧪 Tech Stack

| Frontend | Backend | Database | Pacote |
|---------|---------|----------|--------|
| React   | Rust    | PostgreSQL | .deb   |
| Vite    | Tauri 2 | SQLx     | dpkg   |
| TypeScript | Tokio | —       | apt    |

---

> Organize seus projetos com eficiência! 📚✨
