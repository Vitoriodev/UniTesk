#!/bin/bash
# ============================================================
# Unitesk — Verificador de Prazos (cron job)
# ============================================================
# Este script consulta o PostgreSQL e envia notificações
# desktop para atividades com prazo vencendo hoje.
#
# Projetado para instalação via pacote .deb.
# O binário do Unitesk está em /usr/bin/unitesk.
#
# Configuração no crontab do USUÁRIO (crontab -e):
#   Todos os dias às 08:00, 12:00 e 18:00:
#     0 8,12,18 * * * /usr/share/unitesk/scripts/notify-deadlines.sh
#
# Configuração no crontab do ROOT (sudo crontab -e):
#   Garante acesso ao PostgreSQL via sudo -u postgres:
#     0 8,12,18 * * * /usr/share/unitesk/scripts/notify-deadlines.sh
# ============================================================

# PATH para o cron (ambiente mínimo)
PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# === Caminhos do sistema (instalação .deb) ===
BINARY_PATH="/usr/bin/unitesk"
DB_NAME="academic_manager"
ICON_PATH="/usr/share/icons/hicolor/512x512/apps/unitesk.png"

# Fallback de ícone para diferentes tamanhos/temas
if [ ! -f "$ICON_PATH" ]; then
    for fallback in \
        "/usr/share/icons/hicolor/256x256/apps/unitesk.png" \
        "/usr/share/icons/hicolor/128x128/apps/unitesk.png" \
        "/usr/share/icons/hicolor/48x48/apps/unitesk.png" \
        "/usr/share/pixmaps/unitesk.png"; do
        if [ -f "$fallback" ]; then
            ICON_PATH="$fallback"
            break
        fi
    done
fi

# === Log: usa diretório com permissão de escrita do usuário ===
# Não usa /var/log/ pois o cron de usuário comum não tem escrita lá.
LOG_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/unitesk/logs"
mkdir -p "$LOG_DIR" 2>/dev/null
LOG_FILE="$LOG_DIR/notifications.log"

# Verificar se o Unitesk está instalado
if [ ! -f "$BINARY_PATH" ]; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ❌ Unitesk não encontrado em $BINARY_PATH" >> "$LOG_FILE"
    notify-send -u critical -t 10000 \
        "❌ Unitesk" "Binário não encontrado. Reinstale o pacote .deb." 2>/dev/null || true
    exit 1
fi

# Função para registrar log
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Função para enviar notificação desktop
notify() {
    local title="$1"
    local message="$2"
    local urgency="${3:-normal}"

    if command -v notify-send &>/dev/null; then
        notify-send -u "$urgency" -t 10000 \
            -i "$ICON_PATH" \
            "$title" "$message" 2>/dev/null || true
    fi
}

log "=== Verificação de prazos iniciada ==="

# Conexão PostgreSQL:
# Opção 1: sudo -u postgres (crontab root, autenticação peer via socket)
# Opção 2: psql -U postgres -h localhost (crontab de usuário, autenticação md5/password)
# Se tudo falhar, aborta com aviso

PSQL_CMD=""

# Tentar opção 1: sudo -u postgres (socket local)
PSQL_OUTPUT=$(sudo -u postgres psql -U postgres -d $DB_NAME -t -A -c "SELECT 1;" 2>/dev/null)
if [ -n "$PSQL_OUTPUT" ]; then
    PSQL_CMD="sudo -u postgres psql -U postgres -d $DB_NAME -t -A"
    log "Conectado ao PostgreSQL via sudo -u postgres (socket)."
fi

# Se opção 1 falhou, tentar opção 2: conexão TCP local com senha
if [ -z "$PSQL_CMD" ]; then
    PSQL_OUTPUT=$(psql -U postgres -h localhost -d $DB_NAME -t -A -c "SELECT 1;" 2>/dev/null)
    if [ -n "$PSQL_OUTPUT" ]; then
        PSQL_CMD="psql -U postgres -h localhost -d $DB_NAME -t -A"
        log "Conectado ao PostgreSQL via TCP local (host localhost)."
    fi
fi

# Se nenhuma opção funcionou, abortar
if [ -z "$PSQL_CMD" ]; then
    log "❌ Não foi possível conectar ao PostgreSQL."
    log "   Tente: sudo -u postgres psql -c \"SELECT 1;\""
    log "   Ou:   psql -U postgres -h localhost -c \"SELECT 1;\""
    notify "❌ Unitesk — Erro" "Não foi possível conectar ao banco de dados PostgreSQL." "critical"
    exit 1
fi

# --- 1. Atividades com prazo HOJE ---
TODAY=$($PSQL_CMD -c "SELECT title, COALESCE(project_name, 'Sem projeto') FROM assignments WHERE due_date = CURRENT_DATE AND status = 'pending';" 2>/dev/null)

if [ -n "$TODAY" ]; then
    while IFS='|' read -r title project; do
        [ -z "$title" ] && continue
        log "Prazo hoje: '$title' ($project)"
        notify "📚 Unitesk — Prazo Hoje!" \
            "A atividade '$title' ($project) vence hoje!" \
            "critical"
    done <<< "$TODAY"
else
    log "Nenhum prazo vencendo hoje."
fi

# --- 2. Atividades ATRASADAS ---
OVERDUE=$($PSQL_CMD -c "SELECT title, COALESCE(project_name, 'Sem projeto') FROM assignments WHERE due_date < CURRENT_DATE AND status = 'pending' ORDER BY due_date ASC;" 2>/dev/null)

if [ -n "$OVERDUE" ]; then
    count=$(echo "$OVERDUE" | wc -l)
    log "Atividades atrasadas: $count"
    notify "🔴 Unitesk — $count Atividade(s) Atrasada(s)!" \
        "Você tem $count atividade(s) com prazo vencido. Abra o Unitesk!" \
        "critical"
else
    log "Nenhuma atividade atrasada."
fi

# --- 3. Atualizar status de atividades atrasadas ---
$PSQL_CMD -c "UPDATE assignments SET status = 'overdue' WHERE due_date < CURRENT_DATE AND status = 'pending';" 2>/dev/null

log "=== Verificação de prazos finalizada ==="
