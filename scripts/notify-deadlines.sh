#!/bin/bash
# ============================================================
# Unitesk — Verificador de Prazos (cron job)
# ============================================================
# Este script consulta o PostgreSQL e envia notificações
# desktop para atividades com prazo vencendo hoje.
#
# Configuração no crontab (executar: crontab -e):
#   Todos os dias às 08:00, 12:00 e 18:00:
#     0 8,12,18 * * * /caminho/para/unitesk/scripts/notify-deadlines.sh
# ============================================================

# PATH para o cron (ambiente mínimo)
PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# === Caminhos (AJUSTE para o seu diretório do Unitesk) ===
UNITESK_DIR="/caminho/para/unitesk"
LOG_FILE="$UNITESK_DIR/scripts/notifications.log"
DB_NAME="academic_manager"

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
            -i "$UNITESK_DIR/src-tauri/icons/icon.png" \
            "$title" "$message" 2>/dev/null || true
    fi
}

log "=== Verificação de prazos iniciada ==="

PSQL="psql -U postgres -d $DB_NAME -t -A"

# --- 1. Atividades com prazo HOJE ---
TODAY=$($PSQL -c "SELECT title, COALESCE(project_name, 'Sem projeto') FROM assignments WHERE due_date = CURRENT_DATE AND status = 'pending';" 2>/dev/null)

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
OVERDUE=$($PSQL -c "SELECT title, COALESCE(project_name, 'Sem projeto') FROM assignments WHERE due_date < CURRENT_DATE AND status = 'pending' ORDER BY due_date ASC;" 2>/dev/null)

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
$PSQL -c "UPDATE assignments SET status = 'overdue' WHERE due_date < CURRENT_DATE AND status = 'pending';" 2>/dev/null

log "=== Verificação de prazos finalizada ==="
