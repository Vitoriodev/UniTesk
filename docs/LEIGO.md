╔═══════════════════════════════════════════════════════╗
║                                                   ║
║          🚀 UNITESK - GUIA RÁPIDO                   ║
║     Gerenciador de Projetos                        ║
║                                                   ║
╚═══════════════════════════════════════════════════════╝


📌 O QUE É O UNITESK?
   Um programa para gerenciar projetos, documentos,
   prazos, clientes e equipes.


═══════════════════════════════════════════════════════════
   ✅ O QUE VOCÊ PODE FAZER
═══════════════════════════════════════════════════════════

   📊 Dashboard — Visão geral de projetos, prazos e metas
   📁 Projetos — Crie e organize projetos (com cliente!)
   🤝 Clientes — Cadastre empresas e pessoas físicas
   📅 Atividades — Prazos com prioridade (baixa/média/alta/urgente)
   📄 Documentos — Artigos, relatórios e documentos diversos
   👥 Equipes — Crie equipes e adicione membros com cargos


═══════════════════════════════════════════════════════════
   ✅ COMO INSTALAR
═══════════════════════════════════════════════════════════

   PASSO 1 - Obtenha o arquivo .deb
      • Peça para alguém que já compilou o Unitesk
        te enviar o arquivo .deb
      • O arquivo se parece com:
        Unitesk_2.0.0_amd64.deb

   PASSO 2 - Instale o pacote
      Clique duas vezes no arquivo .deb:
      • A loja de aplicativos (Ubuntu Software) vai abrir
      • Clique em "Instalar"
      • Digite sua senha quando pedir

      Ou, se preferir, use o Terminal (Ctrl + Alt + T):
      
      cd ~/Downloads
      sudo dpkg -i Unitesk_2.0.0_amd64.deb
      sudo apt-get install -f

   PASSO 3 - Pronto! 🎉
      • Procure por "Unitesk" no menu de aplicativos
      • Clique no ícone e pronto!


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


   💾 Seus dados (projetos, clientes, atividades) ficam
      salvos no banco de dados e NÃO são apagados.

   ☠️  Para apagar TUDO (inclusive o banco de dados):
         sudo apt purge unitesk
         sudo -u postgres psql -c "DROP DATABASE unitesk;"


═══════════════════════════════════════════════════════════
   ❓ AJUDA — O Programa Não Abre
═══════════════════════════════════════════════════════════

   Se o Unitesk não abrir, verifique:

   1. O PostgreSQL está rodando?
      sudo systemctl status postgresql
      Se não estiver: sudo systemctl start postgresql

   2. O banco de dados existe?
      sudo -u postgres psql -c "SELECT 1 FROM unitesk.articles;"
      Se der erro: sudo -u postgres createdb unitesk

   3. Execute pelo terminal para ver erros:
      unitesk


═══════════════════════════════════════════════════════════
   💡 DICAS
═══════════════════════════════════════════════════════════

   • Para criar um cliente: aba "Clientes" → "Novo Cliente"
   • Para criar uma equipe: aba "Equipes" → "Nova Equipe"
   • Para vincular cliente a projeto: ao criar/editar um
     projeto, selecione o cliente no campo "Cliente"
   • Para definir prioridade: ao criar uma atividade,
     escolha entre Baixa, Média, Alta ou Urgente
   • Sempre feche o Unitesk antes de desligar o computador
   • Se precisar de ajuda, procure pelos arquivos da
     pasta "docs/"
   • Para reinstalar, basta baixar a nova versão .deb
     e instalar por cima (sudo dpkg -i novo_unitesk.deb)

───────────────────────────────────────────────────────────
   🚀 Unitesk v2.0 — Gerencie seus projetos com eficiência
───────────────────────────────────────────────────────────
