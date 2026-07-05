#!/bin/bash
# ============================================================
# 🎓 Unitesk — Construtor de Pacote .deb
# ============================================================
# Única forma de instalação do Unitesk.
# Compila o aplicativo e gera um pacote .deb para
# distribuição em máquinas Linux (Debian/Ubuntu/Mint).
#
# Pré-requisitos na máquina de build:
#   Rust, Node.js, npm, dependências Tauri
#
# Uso:
#   chmod +x build-deb.sh && ./build-deb.sh
#
# O .deb gerado estará em:
#   src-tauri/target/release/bundle/deb/
#
# Instalação na máquina de destino:
#   sudo dpkg -i Unitesk_*.deb
#   sudo apt-get install -f
# ============================================================

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUNDLE_DIR="$PROJECT_DIR/src-tauri/target/release/bundle/deb"
DEB_SCRIPTS="$PROJECT_DIR/src-tauri/deb-scripts"

echo "╔══════════════════════════════════════════════════╗"
echo "║   🎓 Unitesk — Construtor de Pacote .deb        ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# ---------------------------------------
# 1. Limpar .deb antigos
# ---------------------------------------
echo "🧹 Limpando pacotes .deb antigos..."
rm -f "$BUNDLE_DIR"/*.deb
echo "  ✅ Diretório limpo"
echo ""

# ---------------------------------------
# 2. Verificar pré-requisitos
# ---------------------------------------
echo "🔍 Verificando pré-requisitos..."

command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo não encontrado"; exit 1; }
echo "  ✅ Rust $(cargo --version | head -1 | awk '{print $2}')"

command -v node >/dev/null 2>&1 || { echo "❌ Node.js não encontrado"; exit 1; }
echo "  ✅ Node.js $(node --version)"

command -v npm >/dev/null 2>&1 || { echo "❌ npm não encontrado"; exit 1; }
echo "  ✅ npm $(npm --version)"

command -v dpkg-deb >/dev/null 2>&1 || { echo "❌ dpkg-deb não encontrado. Instale: sudo apt-get install dpkg-dev"; exit 1; }
echo "  ✅ dpkg-deb disponível"
echo ""

# ---------------------------------------
# 3. Instalar dependências npm
# ---------------------------------------
echo "📦 Instalando dependências npm..."
cd "$PROJECT_DIR"
if [ -f "package-lock.json" ]; then
  npm ci --no-fund --no-audit 2>&1 | tail -1
else
  npm install --no-fund --no-audit 2>&1 | tail -1
fi
echo "  ✅ npm install concluído"
echo ""

# ---------------------------------------
# 4. Compilar Tauri + gerar .deb base
# ---------------------------------------
echo "📦 Compilando aplicativo e gerando pacote .deb..."
echo "   (Isso compila frontend + Rust e empacota em .deb)"
echo "   ⏱ Pode levar vários minutos na primeira vez"
echo ""

cd "$PROJECT_DIR"
npx tauri build 2>&1

echo ""

# ---------------------------------------
# 5. Encontrar o .deb gerado
# ---------------------------------------
echo "🔍 Localizando pacote gerado..."

DEB_FILE=$(ls -t "$BUNDLE_DIR"/*.deb 2>/dev/null | head -1)

if [ -z "$DEB_FILE" ]; then
    echo "❌ Erro: Nenhum pacote .deb foi gerado em:"
    echo "   $BUNDLE_DIR"
    exit 1
fi

DEB_NAME=$(basename "$DEB_FILE")
echo "  ✅ .deb base gerado: $DEB_NAME"
echo ""

# ---------------------------------------
# 6. Injetar scripts de manutenção via dpkg-deb
# ---------------------------------------
echo "🔧 Injetando scripts de manutenção no pacote..."

WORKDIR=$(mktemp -d "/tmp/unitesk_deb_XXXXXX")

# Extrair o .deb para um diretório
# -e extrai os scripts de controle para DEBIAN/
# -x extrai os dados
dpkg-deb -e "$DEB_FILE" "$WORKDIR/DEBIAN/"
dpkg-deb -x "$DEB_FILE" "$WORKDIR/data/"
echo "  ✅ Pacote extraído"

# Copiar scripts de manutenção (postinst, prerm, postrm) para DEBIAN/
for script in postinst prerm postrm; do
    if [ -f "$DEB_SCRIPTS/$script" ]; then
        cp "$DEB_SCRIPTS/$script" "$WORKDIR/DEBIAN/$script"
        chmod 755 "$WORKDIR/DEBIAN/$script"
        echo "  ✅ Script $script adicionado"
    fi
done

# Copiar script de notificações para usr/share/unitesk/scripts/
NOTIFY_DEST="$WORKDIR/data/usr/share/unitesk/scripts"
mkdir -p "$NOTIFY_DEST"
if [ -f "$PROJECT_DIR/scripts/notify-deadlines.sh" ]; then
    cp "$PROJECT_DIR/scripts/notify-deadlines.sh" "$NOTIFY_DEST/notify-deadlines.sh"
    chmod 755 "$NOTIFY_DEST/notify-deadlines.sh"
    echo "  ✅ Script notify-deadlines.sh adicionado"
fi

# Copiar setup.sql para usr/share/unitesk/ (se existir)
if [ -f "$PROJECT_DIR/docs/setup.sql" ]; then
    cp "$PROJECT_DIR/docs/setup.sql" "$WORKDIR/data/usr/share/unitesk/setup.sql"
    chmod 644 "$WORKDIR/data/usr/share/unitesk/setup.sql"
    echo "  ✅ Schema setup.sql adicionado"
fi

# Recriar o .deb com os scripts incluídos
# O dpkg-deb --build espera a estrutura: dir/DEBIAN/ (controle) + dir/* (dados)
# Vamos mesclar DEBIAN/ dentro de data/ para o build funcionar
mv "$WORKDIR/DEBIAN" "$WORKDIR/data/DEBIAN"
DEB_TEMP=$(mktemp "/tmp/unitesk_final_XXXXXX.deb")
dpkg-deb --build "$WORKDIR/data" "$DEB_TEMP" 2>&1

# Mover o .deb final de volta
mv "$DEB_TEMP" "$DEB_FILE"

# Limpar
rm -rf "$WORKDIR"
echo "  ✅ Pacote remontado com scripts"
echo ""

# ---------------------------------------
# 7. Verificar scripts no .deb final
# ---------------------------------------
echo "🔍 Verificando scripts no pacote..."

VERIFY_DIR=$(mktemp -d "/tmp/unitesk_verify_XXXXXX")
dpkg-deb -e "$DEB_FILE" "$VERIFY_DIR/DEBIAN/"

MISSING=false
for script in postinst prerm postrm; do
    if [ -f "$VERIFY_DIR/DEBIAN/$script" ]; then
        echo "  ✅ $script presente"
    else
        echo "  ⚠ $script ausente"
        MISSING=true
    fi
done

# Verificar notify script
dpkg-deb -x "$DEB_FILE" "$VERIFY_DIR/data/"
if [ -f "$VERIFY_DIR/data/usr/share/unitesk/scripts/notify-deadlines.sh" ]; then
    echo "  ✅ notify-deadlines.sh presente"
else
    echo "  ⚠ notify-deadlines.sh ausente"
fi

rm -rf "$VERIFY_DIR"

if [ "$MISSING" = true ]; then
    echo ""
    echo "⚠  Atenção: Alguns scripts de manutenção não foram incluídos."
    echo "   Verifique os arquivos em: $DEB_SCRIPTS/"
fi

echo ""

# ---------------------------------------
# 8. Resultado final
# ---------------------------------------
DEB_SIZE=$(du -h "$DEB_FILE" | cut -f1)

echo "╔══════════════════════════════════════════════════╗"
echo "║   ✅ Pacote .deb gerado com sucesso!             ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "📦 Arquivo: $DEB_FILE"
echo "📏 Tamanho: $DEB_SIZE"
echo ""
echo "📌 Para instalar em outra máquina:"
echo "   sudo dpkg -i \"$DEB_NAME\""
echo "   sudo apt-get install -f"
echo ""
echo "📌 Scripts de manutenção incluídos:"
echo "   • postinst — Configura PostgreSQL e ambiente"
echo "   • prerm    — Avisa sobre preservação de dados"
echo "   • postrm   — Remove configs em /etc/unitesk/"
echo "   • notify-deadlines.sh — Notificações via cron"
echo "   • setup.sql — Schema do banco de dados"
echo ""
