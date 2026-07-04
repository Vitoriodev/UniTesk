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

   PASSO 1 - Abra o Terminal
      • Pressione as teclas: Ctrl + Alt + T
      • Ou procure por "Terminal" no menu do sistema

   PASSO 2 - Navegue até a pasta do Unitesk
      Digite no Terminal e pressione Enter:
      (substitua pelo caminho da pasta onde você salvou)
      
      cd ~/caminho/para/unitesk

   PASSO 3 - Execute o instalador interativo
      Digite no Terminal e pressione Enter:
      
      ./setup.sh

   PASSO 4 - Escolha "Instalar" no menu
      • Use as setas do teclado para navegar
      • Pressione Enter para selecionar
      • O progresso será mostrado na tela

   PASSO 5 - Aguarde a instalação
      • O instalador vai configurar tudo sozinho
      • Pode levar vários minutos na primeira vez
      • Quando aparecer "✅ Instalação concluída!" está pronto!

   PRONTO! 🎉
   Agora você pode abrir o Unitesk pelo menu de aplicativos
   ou digitando no Terminal:
   
   ./unitesk.sh


═══════════════════════════════════════════════════════════
   📦 INSTALAÇÃO VIA PACOTE .DEB (SEM COMPILAR)
═══════════════════════════════════════════════════════════

   Essa é a forma mais fácil — não precisa compilar nada!
   Basta pegar o arquivo .deb pronto e instalar.

   ⚠️  Você precisa de alguém que já tenha compilado o
      Unitesk para gerar o pacote .deb para você.
      O arquivo se chama algo como:
      Unitesk_1.0.0_amd64.deb

   PASSO 1 - Clique duas vezes no arquivo .deb
      • A loja de aplicativos (Ubuntu Software) vai abrir
      • Clique em "Instalar"
      • Digite sua senha quando pedir

      Ou, se preferir, use o Terminal:
      
      cd ~/Downloads
      sudo dpkg -i Unitesk_1.0.0_amd64.deb
      sudo apt-get install -f

   PASSO 2 - Configure o banco de dados (uma vez só)
      Abra o Terminal (Ctrl + Alt + T) e digite:
      
      sudo -u postgres psql -c "CREATE DATABASE academic_manager;"
      sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"

   PASSO 3 - Pronto!
      • Agora procure por "Unitesk" no menu de aplicativos
      • Clique no ícone e pronto! 🎉

   📌 Para desinstalar, vá em:
      Configurações do Sistema → Aplicativos → Unitesk → Remover


═══════════════════════════════════════════════════════════
   ❌ COMO DESINSTALAR
═══════════════════════════════════════════════════════════

   PASSO 1 - Abra o Terminal (Ctrl + Alt + T)

   PASSO 2 - Navegue até a pasta do Unitesk
      (substitua pelo caminho da pasta onde você salvou)
      
      cd ~/caminho/para/unitesk

   PASSO 3 - Execute o gerenciador de instalação
      
      ./setup.sh

   PASSO 4 - Escolha "Desinstalar" no menu
      • Confirme duas vezes para garantir
      • O progresso será mostrado na tela
      • Pronto! Tudo foi removido.


═══════════════════════════════════════════════════════════
   ❓ AJUDA - O Programa Não Abre
═══════════════════════════════════════════════════════════

   Se o Unitesk não abrir, tente:

   1. Execute o instalador novamente:
      ./install.sh

   2. Se ainda assim não funcionar, execute:
      ./unitesk.sh

      e veja a mensagem de erro que aparece.


═══════════════════════════════════════════════════════════
   💡 DICAS
═══════════════════════════════════════════════════════════

   • Sempre feche o Unitesk antes de desligar o computador
   • Se precisar de ajuda, procure pelos arquivos da
     pasta "docs/"
   • Para atualizar o Unitesk, execute ./setup.sh novamente
   • Você também pode executar ./install.sh (instalação direta)
     ou ./uninstall.sh (desinstalação direta)

───────────────────────────────────────────────────────────
   🎓 Unitesk v1.0 — Gerenciador de Projetos Acadêmicos
───────────────────────────────────────────────────────────
