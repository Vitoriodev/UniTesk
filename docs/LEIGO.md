╔═══════════════════════════════════════════════════════╗
║                                                   ║
║          🎓 UNITESK - GUIA RÁPIDO                   ║
║     Gerenciador de Projetos Acadêmicos              ║
║                                                   ║
╚═══════════════════════════════════════════════════════╝


📌 O QUE É O UNITESK?
   Um programa para organizar seus projetos,
   artigos e prazos da faculdade.


═══════════════════════════════════════════════════════════
   ✅ COMO INSTALAR
═══════════════════════════════════════════════════════════

   PASSO 1 - Obtenha o arquivo .deb
      • Peça para alguém que já compilou o Unitesk
        te enviar o arquivo .deb
      • O arquivo se parece com:
        Unitesk_1.3.0_amd64.deb

   PASSO 2 - Instale o pacote
      Clique duas vezes no arquivo .deb:
      • A loja de aplicativos (Ubuntu Software) vai abrir
      • Clique em "Instalar"
      • Digite sua senha quando pedir

      Ou, se preferir, use o Terminal (Ctrl + Alt + T):
      
      cd ~/Downloads
      sudo dpkg -i Unitesk_1.3.0_amd64.deb
      sudo apt-get install -f

   PASSO 3 - Pronto! 🎉
      • Procure por "Unitesk" no menu de aplicativos
      • Clique no ícone e pronto!

   🔧 Se algo não funcionar
      O instalador já tenta configurar o banco de dados
      automaticamente. Se falhar:
      
      sudo systemctl start postgresql
      sudo -u postgres createdb academic_manager


═══════════════════════════════════════════════════════════
   ❌ COMO DESINSTALAR (SEM USAR TERMINAL)
═══════════════════════════════════════════════════════════

   🖱️  Método 1 — Pela loja de aplicativos (recomendado)

      No Ubuntu (Ubuntu Software):
      1. Abra a "Ubuntu Software" no menu
      2. Clique na aba "Instalados"
      3. Procure por "Unitesk"
      4. Clique em "Remover"
      5. Digite sua senha quando pedir

      No Linux Mint (Gerenciador de Programas):
      1. Abra o "Gerenciador de Programas"
      2. Vá em "Gerenciar" → "Instalados"
      3. Encontre "Unitesk"
      4. Clique em "Remover"

      ✅ Pronto! O Unitesk foi removido.


   ⌨️  Método 2 — Pelo Terminal

      PASSO 1 - Abra o Terminal (Ctrl + Alt + T)

      PASSO 2 - Digite:
         
         sudo apt remove unitesk

      PASSO 3 - Pronto! O Unitesk foi removido.


   💾 Seus dados (projetos, artigos, atividades) ficam
      salvos no banco de dados e NÃO são apagados.

   ☠️  Para apagar TUDO (inclusive o banco de dados):
         sudo apt purge unitesk
         sudo -u postgres psql -c "DROP DATABASE academic_manager;"


═══════════════════════════════════════════════════════════
   ❓ AJUDA — O Programa Não Abre
═══════════════════════════════════════════════════════════

   Se o Unitesk não abrir, verifique:

   1. O PostgreSQL está rodando?
      sudo systemctl status postgresql
      Se não estiver: sudo systemctl start postgresql

   2. O banco de dados existe?
      sudo -u postgres psql -c "SELECT 1 FROM academic_manager.articles;"
      Se der erro: sudo -u postgres createdb academic_manager

   3. Execute pelo terminal para ver erros:
      unitesk


═══════════════════════════════════════════════════════════
   💡 DICAS
═══════════════════════════════════════════════════════════

   • Sempre feche o Unitesk antes de desligar o computador
   • Se precisar de ajuda, procure pelos arquivos da
     pasta "docs/"
   • Para reinstalar, basta baixar a nova versão .deb
     e instalar por cima (sudo dpkg -i novo_unitesk.deb)

───────────────────────────────────────────────────────────
   🎓 Unitesk v1.3 — Gerenciador de Projetos Acadêmicos
───────────────────────────────────────────────────────────
