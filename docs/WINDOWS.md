╔═══════════════════════════════════════════════════════════════╗
║                                                           ║
║   🎓 UNITESK — PLANO PARA VERSÃO WINDOWS                  ║
║                                                           ║
║   Tarefas futuras para portar o Unitesk para Windows.     ║
║                                                           ║
╚═══════════════════════════════════════════════════════════════╝

Data: Julho/2026
Versão atual: 1.0.0 (Linux - Tauri 2)


═══════════════════════════════════════════════════════════════
   1. MUDANÇA PRINCIPAL: TROCAR PostgreSQL POR SQLite
═══════════════════════════════════════════════════════════════

   Motivo:
   • PostgreSQL exige instalação de servidor no Windows
   • SQLite é embutido — apenas um arquivo .db
   • Usuário leigo no Windows não sabe configurar PostgreSQL

   O que fazer:
   [ ] Alterar Cargo.toml — trocar feature "postgres" por "sqlite"
   [ ] Alterar db.rs — usar SqlitePool em vez de PgPool
   [ ] Atualizar queries SQL para sintaxe SQLite
        (CURRENT_DATE → date('now'), ::text → CAST, etc.)
   [ ] Remover setup.sql (SQLite cria tabelas automaticamente)
   [ ] Remover dependência de PostgreSQL do instalador

   Código atual (src-tauri/db.rs) — init_db():
       let pool = PgPoolOptions::new()
           .max_connections(5)
           .connect(database_url).await?;

   Código para Windows (SQLite):
       let pool = SqlitePoolOptions::new()
           .max_connections(1)
           .connect("unitesk.db").await?;


═══════════════════════════════════════════════════════════════
   2. INSTALADOR PARA WINDOWS
═══════════════════════════════════════════════════════════════

   Opção A — NSIS (Nullsoft Scriptable Install System)
   [ ] Criar script .nsi para gerar instalador .exe
   [ ] Empacotar: binário + ícones + atalhos
   [ ] Criar entrada no Menu Iniciar
   [ ] Opção de desinstalação no Painel de Controle

   Opção B — Inno Setup
   [ ] Criar script .iss para gerar instalador .exe
   [ ] Mais moderno e personalizável que NSIS

   Opção C — WiX Toolset
   [ ] Gera instalador .msi nativo do Windows
   [ ] Mais complexo, mas integração completa

   Saída do Tauri build no Windows:
       src-tauri/target/release/unitesk.exe
       src-tauri/target/release/bundle/msi/  (se WiX)
       src-tauri/target/release/bundle/nsis/ (se NSIS)


═══════════════════════════════════════════════════════════════
   3. ARQUIVOS PARA WINDOWS
═══════════════════════════════════════════════════════════════

   [ ] Criar install.bat — Instalador para Windows
        • Verificar se Node.js está instalado
        • Instalar dependências npm
        • Executar o binário compilado
        • Não precisa configurar banco (SQLite)

   [ ] Criar uninstall.bat — Desinstalador para Windows
        • Remover arquivos
        • Remover atalhos
        • Remover entrada do registro (se houver)

   [ ] Criar wrapper unitesk.bat — Inicializador
        @echo off
        start "" "%~dp0src-tauri\target\release\unitesk.exe"


═══════════════════════════════════════════════════════════════
   4. DEPENDÊNCIAS DE SISTEMA (WINDOWS)
═══════════════════════════════════════════════════════════════

   Tauri 2 no Windows precisa de:
   ✅ Microsoft Visual Studio Build Tools (C++)
   ✅ WebView2 (já vem instalado no Windows 10/11)
   ✅ Rust (via rustup-init.exe)

   O instalador .exe gerado pelo Tauri já empacota
   tudo que é necessário — o usuário só precisa
   executar o instalador.


═══════════════════════════════════════════════════════════════
   5. DATABASE_URL NO WINDOWS
═══════════════════════════════════════════════════════════════

   Com SQLite, não precisa mais de DATABASE_URL.
   O arquivo do banco fica na mesma pasta do programa:
   
   unitesk.exe + unitesk.db

   O usuário pode copiar a pasta inteira para outro PC
   que leva todos os dados junto! 📦


═══════════════════════════════════════════════════════════════
   6. COMPILAÇÃO CRUZADA (BUILD PARA WINDOWS NO LINUX)
═══════════════════════════════════════════════════════════════

   Se quiser compilar para Windows sem sair do Linux:

   [ ] Instalar toolchain cross-compile:
       rustup target add x86_64-pc-windows-gnu
       sudo apt-get install mingw-w64

   [ ] Compilar:
       cargo build --target x86_64-pc-windows-gnu --release

   ⚠  Nota: Tauri tem limitações com cross-compile.
       O ideal é compilar nativamente no Windows.


═══════════════════════════════════════════════════════════════
   PRIORIDADES (O QUE FAZER PRIMEIRO)
═══════════════════════════════════════════════════════════════

   1. 🔥 MIGRAR CÓDIGO PARA SQLITE (mudança mais crítica)
   2. 📦 COMPILAR NO WINDOWS (gerar .exe com Tauri build)
   3. 🪟 CRIAR INSTALADOR (NSIS ou Inno Setup)
   4. 📝 CRIAR SCRIPTS .bat (install, uninstall, run)
   5. 🧪 TESTAR EM WINDOWS PURO (VM ou máquina real)

───────────────────────────────────────────────────────────────
   🎓 Unitesk v1.0 — Planejamento para Windows
   Documento criado em: Julho/2026
───────────────────────────────────────────────────────────────
