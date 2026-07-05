<div align="center">

# 🎓 Unitesk

**Gerencie seus projetos com eficiência**

<p align="center">
  <img alt="Plataforma" src="https://img.shields.io/badge/plataforma-Linux%20%7C%20Ubuntu%20%7C%20Mint-blue?style=flat-square">
  <img alt="Pacote" src="https://img.shields.io/badge/pacote-.deb-orange?style=flat-square">
  <img alt="Versão" src="https://img.shields.io/badge/versão-1.3.0-purple?style=flat-square">
  <img alt="Frontend" src="https://img.shields.io/badge/frontend-React%20%7C%20TypeScript-61dafb?style=flat-square">
  <img alt="Backend" src="https://img.shields.io/badge/backend-Rust%20%7C%20Tauri%202-dea584?style=flat-square">
  <img alt="Banco" src="https://img.shields.io/badge/banco-PostgreSQL%20%7C%20SQLite-336791?style=flat-square">
</p>

**Organize seus projetos, artigos, atividades e prazos em um só lugar.**

<br>

</div>

---

## ✨ Funcionalidades

| | | |
|---|---|---|
| 📁 **Projetos** — Crie, edite e exclua projetos | 📄 **Artigos** — Armazene artigos e materiais | 📎 **Arquivos** — Anexe PDFs, imagens e docs |
| 📅 **Calendário** — Prazos em calendário interativo | 🔔 **Notificações** — Alertas nativos no dia | 📊 **Dashboard** — Estatísticas em tempo real |
| 🔍 **Busca** — Pesquise artigos pelo conteúdo | 📦 **Exportar ZIP** — Projetos completos em ZIP | 📤 **Backup** — Exporte/importe dados (`.unitesk`) |
| 🌙 **Tema Dracula** — Alternne entre claro e escuro | 🗑️ **Exclusão** — Confirmação em 2 cliques | 🎯 **Status** — Rascunho / Pronto / Atrasado |

---

## 📦 Instalação

> O Unitesk é distribuído **exclusivamente** como pacote `.deb` para **Ubuntu, Debian, Linux Mint** e derivados.

### ⚡ Instalação Rápida

```bash
# 1. Instalar o pacote
sudo dpkg -i Unitesk_1.3.0_amd64.deb

# 2. Corrigir dependências (se necessário)
sudo apt-get install -f

# 3. Pronto! Busque por "Unitesk" no menu 🎉
```

### ✅ O que é configurado automaticamente

| Item | Descrição |
|------|-----------|
| 🖥️ **Binário** | `/usr/bin/unitesk` |
| 📌 **Atalho** | Aparece no menu de aplicativos |
| 🗄️ **Banco** | PostgreSQL configurado automaticamente |
| ⚙️ **Config** | `/etc/unitesk/unitesk.conf` com `DATABASE_URL` |
| 🔔 **Notificações** | Script para cron em `/usr/share/unitesk/scripts/` |

### 🖱️ Desinstalação (sem terminal)

**Ubuntu:** Ubuntu Software → Instalados → Unitesk → Remover

**Mint:** Gerenciador de Programas → Gerenciar → Instalados → Unitesk → Remover

### ⌨️ Desinstalação (terminal)

```bash
sudo apt remove unitesk       # Preserva seus dados
sudo apt purge unitesk        # Remove também configs
```

> 💾 Seus projetos, artigos e atividades **não são perdidos** ao desinstalar.
> O banco de dados permanece intacto.

---

## 🛠️ Para Desenvolvedores

### Build do .deb

```bash
# Dependências de sistema
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  libjavascriptcoregtk-4.1-dev libssl-dev \
  postgresql postgresql-client

# Build
./build-deb.sh
```

O `.deb` será gerado em `src-tauri/target/release/bundle/deb/`.

### Documentação

| Documento | Conteúdo |
|-----------|----------|
| [`docs/README.md`](docs/README.md) | Guia completo de instalação e uso |
| [`docs/DEVELOPER.md`](docs/DEVELOPER.md) | Documentação técnica para devs |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Arquitetura do projeto |
| [`docs/API.md`](docs/API.md) | API de comandos Tauri |
| [`docs/DATABASE.md`](docs/DATABASE.md) | Schema do banco de dados |
| [`docs/LEIGO.md`](docs/LEIGO.md) | Guia rápido para usuários |
| [`CHANGELOG.md`](CHANGELOG.md) | Histórico de versões |

---

## 🧪 Tech Stack

| Camada | Tecnologia |
|--------|-----------|
| 🎨 **Frontend** | React 18 + TypeScript + Vite 6 |
| 🦀 **Backend** | Rust + Tauri 2 |
| 🗄️ **Banco** | PostgreSQL (Linux) / SQLite (Windows) + SQLx |
| 🌙 **Temas** | Claro + Dracula (escuro) |
| 📊 **Gráficos** | Recharts 3.x |
| 📦 **Pacote** | .deb (dpkg/apt) |

---

<div align="center">

**🎓 Unitesk v1.3** — Mantenha seus projetos organizados!

<br>

</div>
