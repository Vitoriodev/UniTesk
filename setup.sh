#!/bin/bash
# ============================================================
# 🎓 Unitesk — Assistente de Instalação (GUI)
# ============================================================
# Instala ou desinstala o Unitesk com uma interface gráfica
# nativa usando Zenity (GTK).
#
# Uso:
#   chmod +x setup.sh && ./setup.sh
# ============================================================

set -eo pipefail

# ============================================================
# Carregar NVM (Node Version Manager) se existir
# Necessário para encontrar Node.js/npm quando executado
# fora do terminal (ex: pelo launcher compilado)
# ============================================================
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" 2>/dev/null || true
# Fallback: tentar PATH comum do nvm
if ! command -v node &>/dev/null && [ -d "$NVM_DIR/versions/node" ]; then
    export PATH="$NVM_DIR/versions/node/$(ls "$NVM_DIR/versions/node" | sort -V | tail -1)/bin:$PATH"
fi

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
DB_NAME="academic_manager"
DB_USER="postgres"
ENV_FILE="$PROJECT_DIR/.env"
DESKTOP_FILE="$HOME/.local/share/applications/unitesk.desktop"
WRAPPER="$PROJECT_DIR/unitesk.sh"
BINARY="$PROJECT_DIR/src-tauri/target/release/unitesk"
TAURI_TARGET="$PROJECT_DIR/src-tauri/target"
DIST_DIR="$PROJECT_DIR/dist"
NODE_MODULES="$PROJECT_DIR/node_modules"
LOG_FILE="/tmp/unitesk_setup.log"

# Limpar log anterior
echo "" > "$LOG_FILE"

log() {
    echo "[$(date '+%H:%M:%S')] $*" >> "$LOG_FILE"
}

# ============================================================
# Utilitários de interface Zenity
# ============================================================

# Verificar se zenity está disponível
check_zenity() {
    if ! command -v zenity &>/dev/null; then
        echo "❌ Zenity não encontrado. Instale com:"
        echo "   sudo apt-get install zenity"
        exit 1
    fi
}

# Tela de boas-vindas
show_welcome() {
    zenity --info \
        --title="🎓 Unitesk" \
        --width=480 \
        --text="<b>Bem-vindo ao Assistente de Instalação do Unitesk!</b>\n\n
Este programa instala ou remove o Unitesk,\n
seu <b>Gerenciador de Projetos Acadêmicos</b>.\n
━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n
📦 <b>Instalar</b> — Configura tudo para você:\n
• Dependências do sistema\n
• Banco de dados PostgreSQL\n
• Compilação do aplicativo\n
• Atalho no menu\n\n
🗑️ <b>Desinstalar</b> — Remove completamente:\n
• Banco de dados e seus dados\n
• Binário e caches de build\n
• Atalhos e configurações\n
━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n
Clique em <b>OK</b> para começar!" \
        --ok-label="▶  Começar"
}

# Menu principal com lista
show_menu() {
    local choice
    choice=$(zenity --list \
        --title="🎓 Unitesk — Assistente de Instalação" \
        --width=520 --height=280 \
        --text="<b>O que você deseja fazer?</b>" \
        --column="Código" --column="Opção" --column="Descrição" \
        --hide-column=1 \
        --print-column=1 \
        1 "📦 Instalar" "Instalar o Unitesk completo no sistema" \
        2 "🗑️  Desinstalar" "Remover o Unitesk e todos os dados" \
        3 "🔍 Verificar" "Verificar pré-requisitos do sistema" \
        4 "❌ Sair" "Fechar o assistente" 2>/dev/null
    )

    echo "$choice"
}

# Diálogo de confirmação com warning
confirm_action() {
    local title="$1"
    local text="$2"
    local extra_btn="$3"

    if zenity --question \
        --title="$title" \
        --width=450 \
        --text="$text" \
        --ok-label="✅ Sim, continuar" \
        --cancel-label="$extra_btn"; then
        return 0
    else
        return 1
    fi
}

# Mostrar mensagem de sucesso
show_success() {
    local title="$1"
    local msg="$2"

    zenity --info \
        --title="$title" \
        --width=450 \
        --text="$msg" \
        --ok-label="OK"
}

# Mostrar mensagem de erro
show_error() {
    local title="$1"
    local msg="$2"

    zenity --error \
        --title="$title" \
        --width=450 \
        --text="$msg"
}

# ============================================================
# Funções de verificação
# ============================================================

check_prereqs() {
    local issues=""
    local all_ok=true

    # Node.js
    if command -v node &>/dev/null; then
        issues+="✅ Node.js $(node --version)\n"
    else
        issues+="❌ Node.js — Não encontrado\n"
        all_ok=false
    fi

    # npm
    if command -v npm &>/dev/null; then
        issues+="✅ npm $(npm --version)\n"
    else
        issues+="❌ npm — Não encontrado\n"
        all_ok=false
    fi

    # Rust/Cargo
    if command -v cargo &>/dev/null; then
        issues+="✅ Rust $(cargo --version | head -1 | awk '{print $2}')\n"
    else
        issues+="❌ Rust — Não encontrado\n"
        all_ok=false
    fi

    # PostgreSQL
    if command -v psql &>/dev/null; then
        issues+="✅ PostgreSQL $(psql --version | head -1 | awk '{print $3}')\n"
    else
        issues+="❌ PostgreSQL — Não encontrado\n"
        all_ok=false
    fi

    # PostgreSQL rodando?
    if pg_isready &>/dev/null; then
        issues+="✅ PostgreSQL server — Rodando\n"
    else
        issues+="❌ PostgreSQL server — Parado\n"
        all_ok=false
    fi

    # Dependências Tauri
    if [ "$(uname -s)" = "Linux" ]; then
        local missing_count=0
        for pkg in libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libjavascriptcoregtk-4.1-dev libssl-dev; do
            dpkg -l "$pkg" 2>/dev/null | grep -q "^ii" || missing_count=$((missing_count + 1))
        done
        if [ "$missing_count" -eq 0 ]; then
            issues+="✅ Dependências Tauri — Todas instaladas\n"
        else
            issues+="❌ Dependências Tauri — Faltam $missing_count pacote(s)\n"
            all_ok=false
        fi
    fi

    if $all_ok; then
        zenity --info \
            --title="✅ Pré-requisitos OK" \
            --width=450 \
            --text="<b>Todos os pré-requisitos estão ok!</b>\n\n$issues" \
            --ok-label="Continuar"
        return 0
    else
        zenity --question \
            --title="⚠️  Problemas Encontrados" \
            --width=450 \
            --text="<b>Alguns pré-requisitos não estão satisfeitos:</b>\n\n$issues\n\nDeseja continuar mesmo assim?" \
            --ok-label="Ignorar e continuar" \
            --cancel-label="Cancelar"
        return $?
    fi
}

# ============================================================
# Instalação com barra de progresso real
# ============================================================

do_install() {
    log "=== INICIANDO INSTALAÇÃO ==="

    # Verificar pré-requisitos
    if ! check_prereqs; then
        return 1
    fi

    # Confirmar instalação
    if ! confirm_action \
        "📦 Confirmar Instalação" \
        "<b>Tem certeza que deseja instalar o Unitesk?</b>\n\n
Isso vai realizar os seguintes passos:\n\n
◉ Instalar dependências do sistema (se necessário)\n
◉ Baixar pacotes npm\n
◉ Configurar banco PostgreSQL\n
◉ Compilar o aplicativo (pode levar vários minutos!)\n
◉ Criar atalho no menu\n\n
⏱️ Tempo estimado: <b>5 a 15 minutos</b>" \
        "Cancelar"; then
        return 1
    fi

    # Arquivo temporário para comunicar status do subshell
    local STATUS_FILE=$(mktemp /tmp/unitesk_status.XXXXXX)
    echo "ok" > "$STATUS_FILE"
    local ERROR_MSG=""

    # Iniciar pipe de progresso
    (
        echo "0"
        echo "# 🔍 Passo 1/7: Verificando pré-requisitos..."
        log "Passo 1: Pré-requisitos OK"
        sleep 0.5

        # Passo 2: Dependências de sistema
        echo "10"
        echo "# 📦 Passo 2/7: Verificando dependências do sistema..."
        if [ "$(uname -s)" = "Linux" ]; then
            MISSING_DEPS=()
            for pkg in libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libjavascriptcoregtk-4.1-dev libssl-dev; do
                dpkg -l "$pkg" 2>/dev/null | grep -q "^ii" || MISSING_DEPS+=("$pkg")
            done
            if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
                echo "20"
                echo "# 📦 Instalando ${#MISSING_DEPS[@]} pacote(s) faltantes..."
                log "Instalando: ${MISSING_DEPS[*]}"
                if ! sudo apt-get update -qq 2>>"$LOG_FILE"; then
                    install_ok=false
                    error_msg="Falha ao atualizar pacotes"
                fi
                if ! sudo apt-get install -y -qq "${MISSING_DEPS[@]}" 2>>"$LOG_FILE"; then
                    install_ok=false
                    error_msg="Falha ao instalar dependências"
                fi
            fi
        fi
        echo "30"
        echo "# 📦 Passo 3/7: Instalando dependências npm..."
        cd "$PROJECT_DIR"
        log "npm install..."
        if [ -d "node_modules" ]; then
            if ! npm install --silent 2>>"$LOG_FILE"; then
                install_ok=false
                error_msg="Falha no npm install"
            fi
        else
            if ! npm install 2>>"$LOG_FILE"; then
                install_ok=false
                error_msg="Falha no npm install"
            fi
        fi
        log "npm install concluído"

        # Passo 4: Banco de dados
        echo "45"
        echo "# 🗄️  Passo 4/7: Configurando banco de dados PostgreSQL..."
        DB_EXISTS=$(psql -U "$DB_USER" -lqt 2>/dev/null | grep -c "$DB_NAME" || true)
        if [ "$DB_EXISTS" -eq 0 ]; then
            echo "50"
            echo "# 🗄️  Criando banco de dados '$DB_NAME'..."
            sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;" 2>>"$LOG_FILE" || \
            psql -U "$DB_USER" -c "CREATE DATABASE $DB_NAME;" 2>>"$LOG_FILE" || true
        fi
        if [ -f "$PROJECT_DIR/docs/setup.sql" ]; then
            echo "55"
            echo "# 🗄️  Executando schema SQL..."
            grep -E "^CREATE TABLE|^CREATE INDEX|^ALTER" "$PROJECT_DIR/docs/setup.sql" | \
                sudo -u postgres psql -d "$DB_NAME" -q 2>&1 | grep -v "already exists" || true
        fi
        log "Banco configurado"

        # Passo 5: Variáveis de ambiente
        echo "60"
        echo "# ⚙️  Passo 5/7: Configurando variáveis de ambiente..."
        if [ -z "$DATABASE_URL" ]; then
            if [ ! -f "$ENV_FILE" ]; then
                psql -U postgres -c "ALTER USER postgres PASSWORD 'postgres';" 2>>"$LOG_FILE" || true
                DATABASE_URL="postgres://postgres:postgres@localhost:5432/${DB_NAME}"
                echo "export DATABASE_URL=\"$DATABASE_URL\"" > "$ENV_FILE"
                log "Arquivo .env criado"
            fi
            source "$ENV_FILE" 2>/dev/null || true
        fi

        # Passo 6: Compilar frontend
        echo "68"
        echo "# 🏗️  Passo 6/7: Compilando frontend (Vite)..."
        log "npm run build..."
        if ! DATABASE_URL="$DATABASE_URL" npm run build 2>>"$LOG_FILE"; then
            install_ok=false
            error_msg="Falha ao compilar frontend"
        fi
        log "Frontend compilado"

        # Passo 7: Compilar Tauri
        echo "78"
        echo "# 🏗️  Passo 7/7: Compilando aplicativo (pode levar minutos)..."
        log "npx tauri build..."
        if ! DATABASE_URL="$DATABASE_URL" npx tauri build 2>>"$LOG_FILE"; then
            install_ok=false
            error_msg="Falha ao compilar Tauri"
        fi
        log "Tauri build concluído"

        # Finalizar: Criar wrapper e atalho
        echo "90"
        echo "# 🔗 Criando script wrapper e atalho no menu..."
        # Criar script wrapper
        cat > "$WRAPPER" << 'WRAPPEREOF'
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
        chmod +x "$WRAPPER" 2>/dev/null || true
        log "Wrapper criado: $WRAPPER"
        # Criar atalho
        mkdir -p "$HOME/.local/share/applications"
        cat > "$DESKTOP_FILE" << EOF
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
        chmod +x "$BINARY" 2>/dev/null || true
        log "Atalho criado: $DESKTOP_FILE"

        if [ -f "$BINARY" ]; then
            echo "ok" > "$STATUS_FILE"
            echo "100"
            echo "# ✅ Instalação concluída com sucesso!"
            log "=== INSTALAÇÃO CONCLUÍDA ==="
        else
            echo "fail" > "$STATUS_FILE"
            echo "100"
            echo "# ❌ Instalação encontrou problemas: ${error_msg:-Erro desconhecido}"
            log "=== INSTALAÇÃO FALHOU: ${error_msg:-Erro} ==="
        fi
        sleep 1

    ) | zenity --progress \
        --title="📦 Instalando Unitesk" \
        --width=550 \
        --text="Preparando..." \
        --percentage=0 \
        --auto-close \
        --no-cancel 2>/dev/null

    # Ler status do arquivo (escapou do subshell)
    local FINAL_STATUS=$(cat "$STATUS_FILE" 2>/dev/null || echo "fail")
    rm -f "$STATUS_FILE"

    if [ "$FINAL_STATUS" = "ok" ] && [ -f "$BINARY" ]; then
        show_success \
            "✅ Instalação Concluída!" \
            "<b>Unitesk foi instalado com sucesso! 🎉</b>\n\n
📌 <b>Para executar:</b>\n
   • Clique no ícone do Unitesk no menu de aplicativos\n
   • Ou execute no terminal: ./unitesk.sh\n\n
📖 Leia o guia rápido: cat docs/LEIGO.md\n
━━━━━━━━━━━━━━━━━━━━━━\n
💡 Dica: Você pode abrir este assistente\n
novamente a qualquer momento executando ./setup.sh"
    else
        # Tentar ler mensagem de erro do log
        local LAST_ERROR=$(tail -5 "$LOG_FILE" 2>/dev/null | grep -i "falha\|erro\|error" | tail -1 || echo "")
        show_error \
            "⚠️  Instalação Incompleta" \
            "<b>A instalação encontrou problemas.</b>\n\n
📋 Verifique o log completo:\n
<b>cat $LOG_FILE</b>\n\n
💡 Tente executar o instalador manual:\n
<b>./install.sh</b>"
    fi
}

# ============================================================
# Desinstalação
# ============================================================

do_uninstall() {
    log "=== INICIANDO DESINSTALAÇÃO ==="

    # Primeira confirmação
    if ! confirm_action \
        "⚠️  Confirmar Desinstalação" \
        "<b>ATENÇÃO! Isso removerá todos os dados do Unitesk!</b>\n\n
Itens que serão removidos:\n\n
🗄️  Banco de dados PostgreSQL '$DB_NAME'\n
📌 Atalho do menu de aplicativos\n
⚙️  Arquivo .env de configuração\n
📝 Script unitesk.sh\n
📦 Binário compilado\n
🏗️  Cache de build do Tauri (~3.9 GB)\n
📁 Pasta dist/ (frontend compilado)\n
📁 node_modules/ (dependências)\n\n
<b>Deseja realmente desinstalar?</b>" \
        "Cancelar"; then
        return 1
    fi

    # Segunda confirmação (red warning)
    if ! confirm_action \
        "🔴 Confirmação Final" \
        "<b>ÚLTIMA CHANCE — Tem CERTEZA?</b>\n\n
<span color='red'><b>Esta ação NÃO pode ser desfeita!</b></span>\n\n
Todos os seus projetos acadêmicos, artigos,\narquivos e prazos serão perdidos permanentemente.\n\n
Se você quer apenas reinstalar, mantenha os dados\ne execute a instalação novamente." \
        "Cancelar"; then
        return 1
    fi

    # Executar desinstalação
    (
        echo "0"
        echo "# 🗑️  Preparando desinstalação..."
        sleep 0.5

        # 1. Banco
        echo "8"
        echo "# 🗄️  [1/8] Removendo banco de dados..."
        DB_EXISTS=$(psql -U "$DB_USER" -lqt 2>/dev/null | grep -c "$DB_NAME" || true)
        if [ "$DB_EXISTS" -gt 0 ]; then
            sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>>"$LOG_FILE" || \
            psql -U "$DB_USER" -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>>"$LOG_FILE" || true
            log "Banco $DB_NAME removido"
        fi

        # 2. Atalho
        echo "20"
        echo "# 📌 [2/8] Removendo atalho do menu..."
        [ -f "$DESKTOP_FILE" ] && rm -f "$DESKTOP_FILE" && log "Atalho removido"

        # 3. .env
        echo "30"
        echo "# ⚙️  [3/8] Removendo arquivo .env..."
        [ -f "$ENV_FILE" ] && rm -f "$ENV_FILE" && log ".env removido"

        # 4. Wrapper
        echo "38"
        echo "# 📝 [4/8] Removendo script unitesk.sh..."
        [ -f "$WRAPPER" ] && rm -f "$WRAPPER" && log "Wrapper removido"

        # 5. Binário
        echo "45"
        echo "# 📦 [5/8] Removendo binário compilado..."
        [ -f "$BINARY" ] && rm -f "$BINARY" && log "Binário removido"

        # 6. Cache Tauri
        echo "55"
        echo "# 🏗️  [6/8] Removendo cache de build do Tauri..."
        if [ -d "$TAURI_TARGET" ]; then
            local target_size=$(du -sh "$TAURI_TARGET" 2>/dev/null | cut -f1)
            rm -rf "$TAURI_TARGET"
            log "Cache Tauri removido (~${target_size})"
        fi

        # 7. dist
        echo "70"
        echo "# 📁 [7/8] Removendo build do frontend..."
        [ -d "$DIST_DIR" ] && rm -rf "$DIST_DIR" && log "dist removido"

        # 8. node_modules
        echo "85"
        echo "# 📁 [8/8] Removendo node_modules..."
        if [ -d "$NODE_MODULES" ]; then
            rm -rf "$NODE_MODULES"
            log "node_modules removido"
        fi

        echo "100"
        echo "# ✅ Desinstalação concluída! Todos os dados foram removidos."
        log "=== DESINSTALAÇÃO CONCLUÍDA ==="
        sleep 1

    ) | zenity --progress \
        --title="🗑️  Desinstalando Unitesk" \
        --width=550 \
        --text="Preparando..." \
        --percentage=0 \
        --auto-close \
        --no-cancel 2>/dev/null

    show_success \
        "✅ Desinstalação Concluída!" \
        "<b>Unitesk foi removido com sucesso! 🗑️</b>\n\n
Todos os dados foram apagados:\n
✓ Banco de dados removido\n
✓ Atalhos removidos\n
✓ Cache e builds removidos\n
✓ Dependências removidas\n\n
📌 Para reinstalar no futuro:\n
   <b>./setup.sh</b>\n
   E escolha a opção \"Instalar\""
}

# ============================================================
# Verificador de pré-requisitos (modo direto)
# ============================================================

do_check_prereqs() {
    check_prereqs
}

# ============================================================
# Programa Principal
# ============================================================

# Verificar zenity
check_zenity

# Tela de boas-vindas
show_welcome

# Loop do menu principal
while true; do
    choice=$(show_menu)

    case "$choice" in
        1)
            do_install
            ;;
        2)
            do_uninstall
            ;;
        3)
            do_check_prereqs
            ;;
        4|"")
            zenity --info \
                --title="Saindo" \
                --width=350 \
                --text="<b>Obrigado por usar o Unitesk! 🎓</b>\n\nAté logo!" \
                --ok-label="Sair"
            exit 0
            ;;
    esac
done
