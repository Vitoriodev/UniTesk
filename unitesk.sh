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
