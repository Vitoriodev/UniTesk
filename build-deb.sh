#!/bin/bash
# ============================================================
# 🎓 Unitesk — Construtor de Pacote .deb
# ============================================================
# Compila o aplicativo e gera um pacote .deb para
# distribuição em outras máquinas Linux (Debian/Ubuntu).
#
# Pré-requisitos na máquina de build:
#   Rust, Node.js, npm, todas as dependências Tauri
#
# Uso:
#   chmod +x build-deb.sh && ./build-deb.sh
#
# O .deb gerado estará em:
#   src-tauri/target/release/bundle/deb/
# ============================================================

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUNDLE_DIR="$PROJECT_DIR/src-tauri/target/release/bundle/deb"

echo "╔══════════════════════════════════════════════════╗"
echo "║   🎓 Unitesk — Construtor de Pacote .deb        ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# ---------------------------------------
# 1. Verificar pré-requisitos
# ---------------------------------------
echo "🔍 Verificando pré-requisitos..."

command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo não encontrado"; exit 1; }
echo "  ✅ Rust $(cargo --version | head -1 | awk '{print $2}')"

command -v node >/dev/null 2>&1 || { echo "❌ Node.js não encontrado"; exit 1; }
echo "  ✅ Node.js $(node --version)"

command -v npm >/dev/null 2>&1 || { echo "❌ npm não encontrado"; exit 1; }
echo "  ✅ npm $(npm --version)"

echo ""

# ---------------------------------------
# 2. Instalar dependências npm
# ---------------------------------------
echo "📦 Instalando dependências npm..."
cd "$PROJECT_DIR"
npm install 2>&1 | tail -1
echo "  ✅ npm install concluído"
echo ""

# ---------------------------------------
# 3. Compilar frontend
# ---------------------------------------
echo "🏗️  Compilando frontend..."
npm run build 2>&1 | tail -3
echo "  ✅ Frontend compilado"
echo ""

# ---------------------------------------
# 4. Compilar Tauri + gerar .deb
# ---------------------------------------
echo "📦 Compilando aplicativo e gerando pacote .deb..."
echo "   (Isso pode levar vários minutos na primeira vez)"
echo ""

cd "$PROJECT_DIR"
npx tauri build 2>&1

echo ""

# ---------------------------------------
# 5. Verificar resultado
# ---------------------------------------
echo "🔍 Verificando pacote gerado..."

DEB_FILE=$(ls "$BUNDLE_DIR"/*.deb 2>/dev/null | head -1)

if [ -n "$DEB_FILE" ]; then
    DEB_SIZE=$(du -h "$DEB_FILE" | cut -f1)
    echo ""
    echo "╔══════════════════════════════════════════════════╗"
    echo "║   ✅ Pacote .deb gerado com sucesso!             ║"
    echo "╚══════════════════════════════════════════════════╝"
    echo ""
    echo "📦 Arquivo: $DEB_FILE"
    echo "📏 Tamanho: $DEB_SIZE"
    echo ""
    echo "📌 Para instalar em outra máquina:"
    echo "   sudo dpkg -i \"$DEB_FILE\""
    echo ""
    echo "📌 Se faltar dependências:"
    echo "   sudo apt-get install -f"
    echo ""
    echo "📌 Após instalar, configure o banco de dados:"
    echo "   sudo -u postgres psql -c \"CREATE DATABASE academic_manager;\""
    echo "   sudo -u postgres psql -c \"ALTER USER postgres PASSWORD 'postgres';\""
else
    echo "❌ Erro: Pacote .deb não foi encontrado em:"
    echo "   $BUNDLE_DIR"
    echo ""
    echo "Verifique os erros de compilação acima."
    exit 1
fi
