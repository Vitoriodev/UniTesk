# 🎓 Unitesk

Gerenciador de Projetos Acadêmicos — Aplicativo desktop para organizar
artigos, atividades, arquivos e prazos da faculdade.

## ✨ Funcionalidades

- 📁 **Projetos** — Crie, edite e exclua projetos acadêmicos
- 📄 **Artigos** — Armazene artigos e materiais de estudo
- 📎 **Arquivos** — Anexe PDFs, imagens e documentos aos projetos
- 📅 **Calendário** — Visualize prazos em calendário interativo com 🗑️ exclusão de atividades
- 🔔 **Notificações** — Alertas nativos no dia da entrega
- 📊 **Dashboard** — Visão geral com estatísticas em tempo real
- 🔍 **Busca** — Pesquise artigos pelo conteúdo
- 📦 **Exportar ZIP** — Exporte projetos completos (artigos + arquivos) em um arquivo ZIP
- 📤 **Exportar/Importar Backup** — Transfira todos os dados entre máquinas via arquivo `.unitesk`
- 🌙 **Tema Dracula** — Alternne entre tema claro e escuro com um clique

## 🛠️ Requisitos

| Requisito     | Versão Mínima |
|--------------|---------------|
| Rust         | 1.70+         |
| Node.js      | 18+           |
| PostgreSQL   | 14+           |
| npm          | 9+            |

### Dependências de Sistema (Linux)

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libjavascriptcoregtk-4.1-dev \
  libssl-dev
```

## 🚀 Instalação Rápida (Recomendado)

O assistente gráfico de instalação configura tudo para você:

```bash
# Torne executável e execute
chmod +x setup.sh && ./setup.sh
```

Ou use o binário compilado (não precisa de terminal):

```bash
./unitesk-setup
```

O assistente mostra uma janela com botões para:
- **📦 Instalar** — Configura dependências, banco e compila o app
- **🗑️ Desinstalar** — Remove todos os dados e arquivos
- **🔍 Verificar** — Checa pré-requisitos do sistema

> 💡 O `unitesk-setup` executa o `setup.sh` em segundo plano, sem mostrar terminal.

## 📦 Instalação via .deb (Distribuição)

Para instalar o Unitesk em **outras máquinas**, use o pacote `.deb` gerado:

```bash
# Na máquina de destino, copie o .deb e instale:
sudo dpkg -i Unitesk_1.0.0_amd64.deb
sudo apt-get install -f        # corrige dependências faltantes

# Configure o banco de dados:
sudo -u postgres psql -c "CREATE DATABASE academic_manager;"
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"
```

Depois é só procurar "Unitesk" no menu de aplicativos. 🎉

Para reconstruir o pacote:
```bash
./build-deb.sh
```

---

## 🚀 Instalação Manual

### 1. Navegue até a pasta do projeto

```bash
# Substitua pelo caminho da pasta onde você salvou o Unitesk
cd ~/caminho/para/unitesk
```

### 2. Configure o banco de dados

```bash
# Criar banco
sudo -u postgres createdb academic_manager

# Configurar variável de ambiente
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/academic_manager"
```

### 3. Instale as dependências

```bash
npm install
```

### 4. Compilar para produção

```bash
npm run tauri build
```

O executável será gerado em `src-tauri/target/release/unitesk`.

### 5. Executar

```bash
./unitesk.sh
```

Ou clique no ícone do Unitesk no menu de aplicativos.

## 📂 Estrutura do Projeto

```
projetos/unitesk/
├── public/                  # Assets estáticos (favicon)
├── src/                     # Frontend React
├── src-tauri/               # Backend Rust + ícone personalizado
├── setup.sh                 # Assistente de instalação (GUI)
├── unitesk-setup            # Binário executável do assistente
├── install.sh               # Instalador direto (terminal)
├── uninstall.sh             # Desinstalador direto (terminal)
├── unitesk.sh               # Script para executar o app
├── build-deb.sh              # Script para gerar pacote .deb
├── CHANGELOG.md              # Histórico de alterações
└── docs/                    # Documentação completa
    ├── README.md            # Guia principal
    ├── DEVELOPER.md       mentação completa
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

> 📦 **NOVO!** O Unitesk agora pode ser distribuído como pacote `.deb`. Veja a [seção de instalação via .deb](#-instalação-via-deb-distribuição) acima.

> 🖼️ **NOVO!** Ícone personalizado incluso no pacote e no menu de aplicativos.

## 🗄️ Banco de Dados

PostgreSQL com as tabelas: `projects`, `articles`, `assignments`, `project_files`.

Veja [DATABASE.md](./DATABASE.md) para setup e schema completo.

## 📡 API

Comandos Tauri disponíveis no arquivo [API.md](./API.md).

## 🧪 Tech Stack

| Frontend | Backend | Database |
|---------|---------|----------|
| React   | Rust    | PostgreSQL |
| Vite    | Tauri 2 | SQLx     |
| TypeScript | Tokio | —       |

---

> Desenvolvido para manter os projetos da faculdade organizados! 📚✨
