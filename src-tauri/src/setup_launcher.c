/*
 * 🎓 Unitesk — Assistente de Instalação
 * 
 * Launcher executável que encontra o setup.sh no mesmo diretório
 * e o executa em segundo plano (sem mostrar terminal).
 *
 * Compilação:
 *   gcc -O2 -o unitesk-setup setup_launcher.c
 *
 * Uso:
 *   ./unitesk-setup          # Executa o setup.sh em 2º plano
 *   ./unitesk-setup --help   # Mostra ajuda
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <libgen.h>
#include <limits.h>
#include <fcntl.h>

#define LOG_FILE "/tmp/unitesk_setup.log"

int main(int argc, char *argv[]) {
    char bin_path[PATH_MAX];
    char dir_path[PATH_MAX];
    char script_path[PATH_MAX];
    ssize_t len;
    pid_t pid;

    /* Mostrar ajuda se solicitado */
    if (argc > 1 && (strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0)) {
        printf("🎓 Unitesk — Assistente de Instalação\n\n");
        printf("Uso: %s [--help|-h]\n\n", argv[0]);
        printf("Abre o assistente gráfico para instalar ou desinstalar\n");
        printf("o Unitesk (Gerenciador de Projetos Acadêmicos).\n\n");
        printf("Requer: zenity, bash\n");
        return 0;
    }

    /* Obter o caminho do próprio binário */
    len = readlink("/proc/self/exe", bin_path, sizeof(bin_path) - 1);
    if (len == -1) {
        strncpy(bin_path, argv[0], sizeof(bin_path) - 1);
        bin_path[sizeof(bin_path) - 1] = '\0';
    } else {
        bin_path[len] = '\0';
    }

    /* Obter o diretório do binário */
    strncpy(dir_path, bin_path, sizeof(dir_path) - 1);
    dir_path[sizeof(dir_path) - 1] = '\0';
    char *dir = dirname(dir_path);

    /* Construir caminho para setup.sh */
    snprintf(script_path, sizeof(script_path), "%s/setup.sh", dir);

    /* Verificar se setup.sh existe */
    if (access(script_path, F_OK) != 0) {
        const char *cwd = getcwd(dir_path, sizeof(dir_path));
        if (cwd) {
            snprintf(script_path, sizeof(script_path), "%s/setup.sh", cwd);
        }
        if (!cwd || access(script_path, F_OK) != 0) {
            fprintf(stderr, "❌ Arquivo setup.sh não encontrado!\n\n");
            fprintf(stderr, "Certifique-se de que o arquivo setup.sh está no mesmo\n");
            fprintf(stderr, "diretório que este executável.\n\n");
            fprintf(stderr, "   Diretório procurado: %s\n", dir);
            return 1;
        }
    }

    /*
     * Fork para executar em segundo plano.
     * O processo filho executa o script com stdout/stderr
     * redirecionados para o log. O pai sai imediatamente
     * para que nenhuma janela de terminal fique aberta.
     */
    pid = fork();

    if (pid < 0) {
        fprintf(stderr, "❌ Erro ao criar processo em segundo plano\n");
        return 1;
    }

    if (pid == 0) {
        /* === Processo filho === */

        /* Redirecionar stdout/stderr para o log */
        int log_fd = open(LOG_FILE, O_WRONLY | O_CREAT | O_APPEND, 0644);
        if (log_fd >= 0) {
            dup2(log_fd, STDOUT_FILENO);
            dup2(log_fd, STDERR_FILENO);
            close(log_fd);
        }

        /* Desassociar do terminal */
        setsid();

        /* Executar setup.sh com bash */
        execl("/bin/bash", "bash", script_path, (char *)NULL);

        /* Se execl falhar */
        fprintf(stderr, "❌ Erro ao executar: %s\n", script_path);
        _exit(1);
    }

    /* === Processo pai === sai imediatamente (sem terminal) */
    return 0;
}
