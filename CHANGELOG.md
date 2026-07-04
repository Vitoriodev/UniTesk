# 📋 CHANGELOG — Unitesk

Todas as alterações, correções e melhorias feitas no Unitesk.

---

## [1.2.0] — Julho 2026

### 🆕 Novas Funcionalidades

#### 📅 Agendamento de Documentos (Artigos)
- **Status "Agendado"** — Documentos agora podem ser agendados para uma data futura 📅
- **Campo de data** — Novo campo "Agendar para" no formulário de criação de documentos
- **Filtro "Agendados"** — Aba dedicada para filtrar documentos agendados
- **Indicador visual** — Data de agendamento exibida nos cards de documentos
- **Migração automática** — Status de versões anteriores são migrados automaticamente

#### 📅 Navegação Rápida no Calendário
- **Seletor combinado mês/ano** — Novo input `<input type="month">` para selecionar mês e ano simultaneamente
- **Navegação mais rápida** — Permite ir diretamente para qualquer mês/ano em um único clique

#### 📊 Dashboard Aprimorado
- **Removido botão "Verificar Atualizações"** — Simplificação das ações rápidas
- **Design mais limpo** — Ações essenciais em destaque

### 🐛 Correções

#### Tema (UI)
- **Hitbox do botão de tema corrigida** — Agora o botão inteiro (44×44px) é clicável, não apenas um canto específico
- **Botão reposicionado** — Centralizado verticalmente no header com design mais destacado
- **Contraste nos selects do calendário** — Corrigido fundo branco com texto branco no tema Dracula
- **Estilo `option` nos selects** — Cores agora seguem o tema (claro/escuro) corretamente

### 🔧 Alterações de Configuração

#### Instalação Otimizada
- **npm ci** — Instalação mais rápida usando `npm ci` quando `package-lock.json` existe
- **Flags de desempenho** — `--no-fund --no-audit` para acelerar instalação
- **Build incremental** — Melhor aproveitamento do cache do Cargo

---

## [1.1.0] — Julho 2026

### 🆕 Novas Funcionalidades

#### 📄 Documentos (Artigos)
- **Sistema de status** — Documentos agora possuem status **Rascunho** 📝 ou **Pronto** ✅
- **Alternância de status** — Um clique para marcar/desmarcar documento como pronto
- **Filtros rápidos** — Abas "Todos", "📝 Rascunhos" e "✅ Prontos" para filtrar documentos
- **Busca aprimorada** — Busca por título, conteúdo e disciplina, com botão "Limpar"
- **Confirmação ao excluir** — Confirmação antes de excluir documentos
- **Contagem por status** — Badges mostrando quantos documentos em cada status

#### 📅 Calendário
- **Seletores de mês/ano** — Dropdowns para selecionar diretamente o mês e o ano desejados
- **Botão "Hoje"** — Navegação rápida para a data atual
- **Prevenção de datas obsoletas** — Funções getter que sempre retornam a data atual
- **Dias passados com opacidade** — Dias anteriores ao atual aparecem com tom mais suave
- **Pontos coloridos** — 🔴 atrasado, 🟡 pendente, 🟢 concluído, 🟣 agendado

#### 📊 Dashboard
- **Card de boas-vindas** — Saudação personalizada com resumo das atividades
- **Cards de estatísticas animados** — 4 cards com ícones e cores temáticas
- **Barra de progresso** — Indicador visual de atividades concluídas vs pendentes
- **Indicadores de desempenho** — Métricas "Em dia" e "Atrasados"
- **Linha do tempo** — Atividades recentes em formato timeline
- **Ações rápidas em grid** — Layout 2×2 para acesso rápido às funcionalidades

#### 🎨 Interface
- **Header com efeito shimmer** — Animação sutil no cabeçalho
- **Sombra elevada ao hover** — Cards ganham destaque ao passar o mouse
- **Scrollbar customizada** — Estilizada para combinar com o tema
- **Responsividade aprimorada** — Melhor adaptação para diferentes tamanhos de tela
- **Tema Dracula consistente** — Todas as novas cores adaptadas para o tema escuro

### 🐛 Correções

#### Dashboard
- **Contador animado corrigido** — Uso de `setInterval` em vez de `requestAnimationFrame` para compatibilidade
- **Cálculo de progresso corrigido** — Métrica mais precisa de atividades concluídas
- **Título "📊 Dashboard" restaurado** — Estava faltando na versão anterior

#### ArticleManager
- **Persistência de status** — Status agora armazenado em chave única do localStorage (`unitesk_article_statuses`)
- **Limpeza ao deletar** — Status removido do mapa ao excluir documento

#### CalendarView
- **Data fixa corrigida** — Substituídas variáveis `today`/`todayStr` por funções getter para evitar dados obsoletos
- **Botão "Nova Atividade"** — Função extraída para `handleNewAssignmentClick`

#### CSS
- **Cores hardcoded corrigidas** — `calendar-day--has-events` agora usa variável CSS `--calendar-event-bg`
- **Cores do tema Dracula** — Fundo para eventos no calendário agora escuro

### 🔧 Alterações de Configuração

- **Versão** atualizada para `1.1.0` no `package.json` e rodapé do App

### 📚 Documentação Atualizada

- **`docs/DEVELOPER.md`** — Documentação detalhada das novas funcionalidades
- **`CHANGELOG.md`** — Este documento com todas as alterações da v1.1.0

### 📊 Resumo de Testes (v1.1.0)

| Teste | Status |
|-------|--------|
| TypeScript `tsc --noEmit` | ✅ Passou (0 erros) |
| Vitest (75 testes) | ✅ 75/75 passaram |
| Test Files (5) | ✅ Todos passaram |

---

## [1.0.0] — Julho 2026

### 🆕 Novas Funcionalidades

#### 📁 Projetos
- **Editar projeto** — Botão ✏️ em cada cartão para editar nome e descrição
- **Excluir projeto** — Botão 🗑️ com confirmação em dois cliques para evitar acidentes
- Remoção em cascata: artigos ficam órfãos (`SET NULL`), arquivos são deletados (`CASCADE`)

#### 📎 Arquivos
- **Anexar arquivos** aos projetos (PDF, imagens, documentos, etc.)
- **Download** de arquivos diretamente pela interface
- **Exclusão** de arquivos individuais
- **Limite de 10 MB** por arquivo com validação no frontend
- Ícones dinâmicos conforme o tipo de arquivo (📕 PDF, 🖼️ imagem, 📦 ZIP, 📄 outros)

#### 📦 Exportar ZIP
- **Exportar projeto completo** como arquivo ZIP
- Estrutura do ZIP:
  - `projeto.txt` — informações do projeto
  - `artigos/001_titulo.txt` — artigos em arquivos .txt numerados
  - `arquivos/` — arquivos anexados com nomes originais
- Download automático após a geração

#### 📅 Calendário
- **Persistência com localStorage** — atividades do calendário agora ficam salvas mesmo sem o backend Tauri
- Fallback automático: tenta carregar do backend, se falhar usa localStorage

#### 🖥️ Assistente de Instalação (GUI)
- **`setup.sh`** com interface gráfica nativa usando **Zenity (GTK)**
- Janelas nativas com botões, barras de progresso e confirmações visuais
- **4 opções** no menu: Instalar, Desinstalar, Verificar Pré-requisitos, Sair
- Progresso em tempo real com descrição de cada passo
- Log completo em `/tmp/unitesk_setup.log` para debug

#### 🏃 Executável Binário
- **`unitesk-setup`** — binário ELF compilado (17 KB)
- Executa o `setup.sh` em **segundo plano** (sem mostrar terminal)
- Usa `fork()` + `setsid()` para desassociar do terminal
- Redireciona saídas para o arquivo de log
- Pode ser executado com duplo clique no gerenciador de arquivos

#### 📦 Pacote .deb
- Geração de pacote `.deb` para distribuição em outras máquinas
- Configuração no `tauri.conf.json` com dependências runtime corretas
- Script `build-deb.sh` para reconstruir o pacote
- Binário instalado em `/usr/bin/` e atalho no menu automaticamente

### 🐛 Correções

#### PostgreSQL
- **Senha do postgres configurada** — `ALTER USER postgres PASSWORD 'postgres'`
- Conexão com banco agora funciona (antes falhava com `password authentication failed`)

#### Node.js com NVM
- **Detecção automática do NVM** — script carrega `nvm.sh` se existir
- Fallback: adiciona o PATH do Node mais recente do NVM diretamente
- Agora funciona mesmo quando executado fora do terminal

#### Scripts
- **`unitesk.sh`** recriado (estava faltando, impedia o app de abrir pelo menu)
- **`update-desktop-database`** executado para atualizar o cache de aplicativos
- Caminhos corrigidos nos scripts (`setup.sql` → `docs/setup.sql`)

### 🔧 Alterações de Configuração

#### tauri.conf.json
- Adicionada seção `bundle` com configuração para geração de `.deb`
- Dependências runtime corretas (`libwebkit2gtk-4.1-0`, `libgtk-3-0`, etc.)

#### Cargo.toml
- Adicionada dependência: `zip = "2"` (para exportação ZIP)

### 📁 Organização do Projeto

| Antes | Depois |
|-------|--------|
| `README_LEIGO.txt` (raiz) | `docs/LEIGO.md` |
| `TASK_WINDOWS.txt` (raiz) | `docs/WINDOWS.md` |
| `setup.sql` (raiz) | `docs/setup.sql` |
| — | `build-deb.sh` (novo) |
| — | `unitesk-setup` (novo binário) |
| — | `src-tauri/src/setup_launcher.c` (novo) |

### 📚 Documentação Atualizada

- **`docs/README.md`** — Guia completo com novas funcionalidades, setup.sh, unitesk-setup
- **`docs/API.md`** — Todos os novos comandos Tauri documentados
- **`docs/DATABASE.md`** — Nova tabela `project_files`, índices, consultas SQL
- **`docs/ARCHITECTURE.md`** — Estrutura atualizada, fluxo de ZIP, métodos de instalação
- **`docs/LEIGO.md`** — Guia rápido (movido de `README_LEIGO.txt`)
- **`docs/WINDOWS.md`** — Plano Windows (movido de `TASK_WINDOWS.txt`)
- **`CHANGELOG.md`** — Este arquivo (novo)

### 📊 Resumo de Testes

| Teste | Status |
|-------|--------|
| TypeScript `tsc --noEmit` | ✅ Passou |
| Rust `cargo check` | ✅ Passou |
| Bash syntax (5 scripts) | ✅ Passou |
| C compilation (gcc) | ✅ Passou |
| Binary (18 MB) | ✅ Existe |
| .deb package (5.9 MB) | ✅ Gerado |
| Desktop file | ✅ OK |
| Database connection | ✅ OK |
| App startup | ✅ OK |

---

> 🎓 **Unitesk v1.0** — Gerenciador de Projetos Acadêmicos
