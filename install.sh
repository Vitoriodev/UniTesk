#!/bin/bash
# ============================================================
# 🎓 Unitesk — Instalador Automático
# ============================================================
# Este script configura tudo que você precisa para rodar o
# Unitesk: dependências, banco de dados, build e atalho.
#
# Uso:
#   chmod +x install.sh && ./install.sh
# ============================================================

set -eo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
DB_NAME="academic_manager"
DB_USER="postgres"
ENV_FILE="$PROJECT_DIR/.env"

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     🎓 Unitesk — Instalador              ║${NC}"
echo -e "${BLUE}║     Gerenciador de Projetos Acadêmicos   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""

# ---------------------------------------
# 1. Verificar Pré-requisitos
# ---------------------------------------
echo -e "${YELLOW}[1/7] Verificando pré-requisitos...${NC}"

OS="$(uname -s)"
echo "  • Sistema operacional: $OS"

# Node.js
if command -v node &>/dev/null; then
    echo -e "  • Node.js: ${GREEN}✓${NC} $(node --version)"
else
    echo -e "  • Node.js: ${RED}✗${NC} Não encontrado. Instale Node.js 18+ em https://nodejs.org"
    exit 1
fi

# npm
if command -v npm &>/dev/null; then
    echo -e "  • npm: ${GREEN}✓${NC} $(npm --version)"
else
    echo -e "  • npm: ${RED}✗${NC} Não encontrado."
    exit 1
fi

# Rust/Cargo
if command -v cargo &>/dev/null; then
    echo -e "  • Rust: ${GREEN}✓${NC} $(cargo --version | head -1)"
else
    echo -e "  • Rust: ${RED}✗${NC} Não encontrado. Instale em https://rustup.rs"
    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# PostgreSQL
if command -v psql &>/dev/null; then
    echo -e "  • PostgreSQL: ${GREEN}✓${NC} $(psql --version | head -1)"
else
    echo -e "  • PostgreSQL: ${RED}✗${NC} Não encontrado."
    echo "    Instale: sudo apt-get install postgresql postgresql-client"
    exit 1
fi

# Verificar se PostgreSQL está rodando
if pg_isready &>/dev/null; then
    echo -e "  • PostgreSQL server: ${GREEN}✓${NC} Rodando"
else
    echo -e "  • PostgreSQL server: ${RED}✗${NC} Não está rodando."
    echo "    Inicie: sudo systemctl start postgresql"
    echo "    ou:     sudo service postgresql start"
    exit 1
fi

echo ""

# ---------------------------------------
# 2. Instalar dependências de sistema (Linux)
# ---------------------------------------
echo -e "${YELLOW}[2/7] Verificando dependências de sistema...${NC}"

if [ "$OS" = "Linux" ]; then
    MISSING_DEPS=()

    check_dpkg() {
        dpkg -l "$1" 2>/dev/null | grep -q "^ii" || MISSING_DEPS+=("$1")
    }

    check_dpkg libwebkit2gtk-4.1-dev
    check_dpkg libgtk-3-dev
    check_dpkg libayatana-appindicator3-dev
    check_dpkg librsvg2-dev
    check_dpkg libjavascriptcoregtk-4.1-dev
    check_dpkg libssl-dev

    if [ ${#MISSING_DEPS[@]} -eq 0 ]; then
        echo -e "  • Dependências de sistema: ${GREEN}✓${NC} Todas presentes"
    else
        echo -e "  • Instalando dependências faltantes: ${MISSING_DEPS[*]}..."
        sudo apt-get update -qq
        sudo apt-get install -y -qq "${MISSING_DEPS[@]}"
        echo -e "  • Dependências de sistema: ${GREEN}✓${NC} Instaladas"
    fi
else
    echo -e "  • Sistema não é Linux. Pulei verificação de dependências Tauri."
fi

echo ""

# ---------------------------------------
# 3. Instalar dependências npm
# ---------------------------------------
echo -e "${YELLOW}[3/7] Instalando dependências npm...${NC}"
cd "$PROJECT_DIR"

if [ -d "node_modules" ]; then
    echo "  • node_modules já existe. Atualizando..."
    npm install --silent 2>&1 | tail -1
else
    npm install 2>&1 | tail -1
fi
echo -e "  • npm: ${GREEN}✓${NC} Dependências instaladas"
echo ""

# ---------------------------------------
# 4. Configurar banco de dados PostgreSQL
# ---------------------------------------
echo -e "${YELLOW}[4/7] Configurando banco de dados PostgreSQL...${NC}"

# Verificar se o banco já existe
DB_EXISTS=$(psql -U "$DB_USER" -lqt 2>/dev/null | grep -c "$DB_NAME" || true)

if [ "$DB_EXISTS" -gt 0 ]; then
    echo -e "  • Banco de dados '${DB_NAME}': ${GREEN}✓${NC} Já existe"
else
    echo "  • Criando banco de dados '${DB_NAME}'..."
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || \
        psql -U "$DB_USER" -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || {
        echo -e "  ${RED}✗ Erro ao criar banco. Tente manualmente:${NC}"
        echo "    sudo -u postgres createdb $DB_NAME"
        exit 1
    }
    echo -e "  • Banco criado: ${GREEN}✓${NC}"
fi

# Executar schema (apenas CREATE TABLE e índices, ignorando CREATE DATABASE e \c)
echo "  • Executando schema SQL..."
if [ -f "$PROJECT_DIR/docs/setup.sql" ]; then
  # Filtra apenas os comandos CREATE TABLE, CREATE INDEX e ALTER
  grep -E "^CREATE TABLE|^CREATE INDEX|^ALTER" "$PROJECT_DIR/docs/setup.sql" | \
    sudo -u postgres psql -d "$DB_NAME" -q 2>&1 | grep -v "already exists" || true
  echo -e "  • Schema: ${GREEN}✓${NC} Tabelas criadas/atualizadas"
else
  echo -e "  • Schema: ${YELLOW}⚠${NC} Arquivo setup.sql não encontrado em docs/. As tabelas serão criadas pelo app na inicialização."
fi
echo ""

# ---------------------------------------
# 5. Configurar variável de ambiente
# ---------------------------------------
echo -e "${YELLOW}[5/7] Configurando variáveis de ambiente...${NC}"

# Verificar se DATABASE_URL já está definida
if [ -n "$DATABASE_URL" ]; then
    echo -e "  • DATABASE_URL já definida no ambiente: ${GREEN}✓${NC}"
elif [ -f "$ENV_FILE" ]; then
    echo -e "  • Arquivo .env já existe: ${GREEN}✓${NC}"
    source "$ENV_FILE" 2>/dev/null || true
else
    echo "  • Configurando acesso ao banco (usuário postgres local)..."
    # Configurar senha para o postgres (necessário para conexão TCP/IP)
    psql -U postgres -c "ALTER USER postgres PASSWORD 'postgres';" 2>/dev/null || true
    DATABASE_URL="postgres://postgres:postgres@localhost:5432/${DB_NAME}"
    echo "export DATABASE_URL=\"$DATABASE_URL\"" > "$ENV_FILE"
    echo -e "  • Arquivo .env criado: ${GREEN}✓${NC}"
fi

# Garantir que DATABASE_URL esteja exportada
if [ -z "$DATABASE_URL" ] && [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE" 2>/dev/null || true
fi
echo ""

# ---------------------------------------
# 6. Build do executável Tauri
# ---------------------------------------
echo -e "${YELLOW}[6/7] Compilando o executável...${NC}"
echo "  • Isso pode levar alguns minutos na primeira execução."
echo ""

cd "$PROJECT_DIR"

# Build do frontend primeiro
echo "  • Compilando frontend (Vite)..."
if DATABASE_URL="$DATABASE_URL" npm run build 2>&1; then
  echo -e "  • Frontend: ${GREEN}✓${NC}"
else
  echo -e "  • Frontend: ${RED}✗${NC} Erro na compilação. Verifique a saída acima."
  exit 1
fi

# Build Tauri
echo "  • Compilando aplicativo desktop (Tauri)..."
echo "    (Isso pode levar alguns minutos na primeira execução)"
if DATABASE_URL="$DATABASE_URL" npx tauri build 2>&1; then
  echo -e "  • Tauri: ${GREEN}✓${NC}"
else
  echo -e "  • Tauri: ${RED}✗${NC} Erro na compilação. Verifique a saída acima."
  exit 1
fi

BINARY_PATH="$PROJECT_DIR/src-tauri/target/release/unitesk"
DEB_PATH="$PROJECT_DIR/src-tauri/target/release/bundle/deb/unitesk_1.0.0_amd64.deb"

echo ""

# ---------------------------------------
# 7. Criar atalho no menu de aplicativos
# ---------------------------------------
echo -e "${YELLOW}[7/7] Criando atalho no menu de aplicativos...${NC}"

# Criar script wrapper unitesk.sh
cat > "$PROJECT_DIR/unitesk.sh" << 'WRAPPEREOF'
#!/bin/bash
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$PROJECT_DIR/src-tauri/target/release/unitesk"
ENV_FILE="$PROJECT_DIR/.env"

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" 2>/dev/null || true
if ! command -v node &>/dev/null && [ -d "$NVM_DIR/versions/node" ]; then
    export PATH="$NVM_DIR/versions/node/$(ls "$NVM_DIR/versions/node" | sort -V | tail -1)/bin:$PATH"
fi

[ -f "$ENV_FILE" ] && source "$ENV_FILE" 2>/dev/null || true

if [ ! -f "$BINARY" ]; then
    echo "❌ Erro: Binário do Unitesk não encontrado em $BINARY"
    echo "   Execute ./setup.sh para reinstalar."
    notify-send "❌ Unitesk" "Binário não encontrado. Execute ./setup.sh para reinstalar." --icon=dialog-error 2>/dev/null || true
    exit 1
fi

exec "$BINARY"
WRAPPEREOF
chmod +x "$PROJECT_DIR/unitesk.sh"
echo -e "  • Script wrapper: ${GREEN}✓${NC} $PROJECT_DIR/unitesk.sh"

DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_DIR/unitesk.desktop" << EOF
[Desktop Entry]
Name=Unitesk
Comment=Gerenciador de Projetos Acadêmicos
Exec=${PROJECT_DIR}/unitesk.sh
Icon=${PROJECT_DIR}/src-tauri/icons/unitesk_icon_512.png
Terminal=false
Type=Application
Categories=Education;Office;
StartupNotify=true
EOF

echo -e "  • Atalho criado: ${GREEN}✓${NC} $DESKTOP_DIR/unitesk.desktop"

# ---------------------------------------
# ✅ Concluído!
# ---------------------------------------
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     ✅ Instalação concluída!              ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${BLUE}📌 Executável compilado:${NC}"
echo "     $BINARY_PATH"
echo ""

if [ -f "$DEB_PATH" ]; then
    echo -e "  ${BLUE}📦 Pacote .deb:${NC}"
    echo "     $DEB_PATH"
    echo ""
fi

echo -e "  ${BLUE}▶ Para executar o Unitesk:${NC}"
echo "     $PROJECT_DIR/unitesk.sh"
echo ""
echo "     Ou clique no ícone do Unitesk no menu de aplicativos."
echo ""
echo -e "  ${BLUE}📖 Leia o guia:${NC}"
echo "     cat docs/LEIGO.md"
echo ""

# Configurar permissões de execução
chmod +x "$BINARY_PATH" 2>/dev/null || true
chmod +x "$PROJECT_DIR/unitesk.sh" 2>/dev/null || true

echo -e "${GREEN}🎓 Unitesk pronto para uso!${NC}"
