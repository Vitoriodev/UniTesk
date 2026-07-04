#!/bin/bash
# ============================================================
# 🎓 Unitesk — Desinstalador
# ============================================================
# Remove todos os artefatos criados pelo install.sh:
# banco de dados, atalhos, binários, caches e configurações.
#
# Uso:
#   chmod +x uninstall.sh && ./uninstall.sh
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
DESKTOP_FILE="$HOME/.local/share/applications/unitesk.desktop"
ENV_FILE="$PROJECT_DIR/.env"
WRAPPER="$PROJECT_DIR/unitesk.sh"
BINARY="$PROJECT_DIR/src-tauri/target/release/unitesk"
TAURI_TARGET="$PROJECT_DIR/src-tauri/target"
DIST_DIR="$PROJECT_DIR/dist"
NODE_MODULES="$PROJECT_DIR/node_modules"

echo -e "${RED}╔══════════════════════════════════════════╗${NC}"
echo -e "${RED}║     🎓 Unitesk — Desinstalador           ║${NC}"
echo -e "${RED}║     Gerenciador de Projetos Acadêmicos   ║${NC}"
echo -e "${RED}╚══════════════════════════════════════════╝${NC}"
echo ""

# Aviso de confirmação
echo -e "${YELLOW}⚠  ATENÇÃO: Isso removerá todos os dados do Unitesk!${NC}"
echo ""
echo "  Itens que serão removidos:"
echo "  • Banco de dados PostgreSQL '$DB_NAME'"
echo "  • Atalho do menu de aplicativos"
echo "  • Arquivo .env de configuração"
echo "  • Script wrapper unitesk.sh"
echo "  • Binário compilado"
echo "  • Cache de build do Tauri (~3.9GB)"
echo "  • Pasta dist/ (build do frontend)"
echo "  • node_modules/ (dependências npm)"
echo ""
read -p "  Tem certeza que deseja desinstalar? (s/N): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Ss]$ ]]; then
    echo ""
    echo -e "${YELLOW}❌ Desinstalação cancelada.${NC}"
    exit 0
fi
echo ""

REMOVED_COUNT=0
FAILED_COUNT=0

# ---------------------------------------
# 1. Remover banco de dados PostgreSQL
# ---------------------------------------
echo -e "${YELLOW}[1/8] Removendo banco de dados PostgreSQL...${NC}"

DB_EXISTS=$(psql -U "$DB_USER" -lqt 2>/dev/null | grep -c "$DB_NAME" || true)

if [ "$DB_EXISTS" -gt 0 ]; then
    # Tentar remover com sudo primeiro, depois como postgres
    if sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>/dev/null; then
        echo -e "  • Banco '$DB_NAME': ${GREEN}✓${NC} Removido"
        REMOVED_COUNT=$((REMOVED_COUNT + 1))
    elif psql -U "$DB_USER" -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>/dev/null; then
        echo -e "  • Banco '$DB_NAME': ${GREEN}✓${NC} Removido"
        REMOVED_COUNT=$((REMOVED_COUNT + 1))
    else
        echo -e "  • Banco '$DB_NAME': ${RED}✗${NC} Erro ao remover. Remova manualmente:"
        echo "    sudo -u postgres psql -c 'DROP DATABASE $DB_NAME;'"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
else
    echo -e "  • Banco '$DB_NAME': ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 2. Remover atalho do menu
# ---------------------------------------
echo -e "${YELLOW}[2/8] Removendo atalho do menu...${NC}"

if [ -f "$DESKTOP_FILE" ]; then
    rm -f "$DESKTOP_FILE"
    echo -e "  • Atalho: ${GREEN}✓${NC} Removido ($DESKTOP_FILE)"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • Atalho: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 3. Remover arquivo .env
# ---------------------------------------
echo -e "${YELLOW}[3/8] Removendo arquivo .env...${NC}"

if [ -f "$ENV_FILE" ]; then
    rm -f "$ENV_FILE"
    echo -e "  • .env: ${GREEN}✓${NC} Removido"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • .env: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 4. Remover wrapper unitesk.sh
# ---------------------------------------
echo -e "${YELLOW}[4/8] Removendo script wrapper...${NC}"

if [ -f "$WRAPPER" ]; then
    rm -f "$WRAPPER"
    echo -e "  • unitesk.sh: ${GREEN}✓${NC} Removido"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • unitesk.sh: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 5. Remover binário compilado
# ---------------------------------------
echo -e "${YELLOW}[5/8] Removendo binário compilado...${NC}"

if [ -f "$BINARY" ]; then
    rm -f "$BINARY"
    echo -e "  • Binário: ${GREEN}✓${NC} Removido"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • Binário: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 6. Remover cache de build do Tauri
# ---------------------------------------
echo -e "${YELLOW}[6/8] Removendo cache de build do Tauri...${NC}"

if [ -d "$TAURI_TARGET" ]; then
    TARGET_SIZE=$(du -sh "$TAURI_TARGET" 2>/dev/null | cut -f1)
    rm -rf "$TAURI_TARGET"
    echo -e "  • Cache Tauri: ${GREEN}✓${NC} Removido (~${TARGET_SIZE:-?})"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • Cache Tauri: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 7. Remover build do frontend (dist/)
# ---------------------------------------
echo -e "${YELLOW}[7/8] Removendo build do frontend (dist)...${NC}"

if [ -d "$DIST_DIR" ]; then
    rm -rf "$DIST_DIR"
    echo -e "  • dist/: ${GREEN}✓${NC} Removido"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • dist/: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# 8. Remover node_modules
# ---------------------------------------
echo -e "${YELLOW}[8/8] Removendo node_modules...${NC}"

if [ -d "$NODE_MODULES" ]; then
    rm -rf "$NODE_MODULES"
    echo -e "  • node_modules: ${GREEN}✓${NC} Removido"
    REMOVED_COUNT=$((REMOVED_COUNT + 1))
else
    echo -e "  • node_modules: ${YELLOW}—${NC} Não encontrado"
fi
echo ""

# ---------------------------------------
# ✅ Concluído!
# ---------------------------------------
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     ✅ Desinstalação concluída!           ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""

if [ "$FAILED_COUNT" -gt 0 ]; then
    echo -e "  ${RED}⚠ $FAILED_COUNT item(ns) não puderam ser removidos automaticamente.${NC}"
    echo "     Verifique as mensagens de erro acima."
    echo ""
fi

echo -e "  ${BLUE}📊 Resumo:${NC} $REMOVED_COUNT itens removidos"
echo ""
echo -e "  Para reinstalar, execute:"
echo "     ./install.sh"
echo ""
