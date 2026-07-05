# 🗄️ Banco de Dados — Unitesk

## 📋 Visão Geral

O Unitesk possui **duas implementações de banco** de acordo com a plataforma:

| Plataforma | Banco | Driver |
|---|---|---|
| **Linux** | PostgreSQL | `PgPool` (SQLx) |
| **Windows** | SQLite (embutido) | `SqlitePool` (SQLx) |

O schema é gerenciado automaticamente pelo Rust na inicialização da aplicação
(`db::init_db()`), com `#[cfg(target_os)]` selecionando a implementação correta.

**Interoperabilidade:** Dados podem ser transferidos entre plataformas via
formato `.unitesk` (JSON com base64 para arquivos).

### Tabelas

| Tabela          | Descrição                          |
|----------------|------------------------------------|
| `projects`     | Projetos                           |
| `articles`     | Artigos vinculados a projetos      |
| `assignments`  | Atividades com prazo (calendário)  |
| `project_files`| Arquivos anexados aos projetos     |
| `assignment_files` | Arquivos anexados às atividades |
| `clients`      | Clientes                          |
| `users`        | Usuários                          |
| `teams`        | Equipes                           |
| `team_members` | Membros de equipe                 |
| `time_entries` | Registros de horas                |
| `invoices`     | Faturas                           |
| `notifications`| Notificações                      |

## 🚀 Setup Inicial

### 1. Criar o banco de dados

```bash
sudo -u postgres psql
```

```sql
CREATE DATABASE unitesk;
CREATE USER unitesk_user WITH PASSWORD 'sua_senha';
GRANT ALL PRIVILEGES ON DATABASE unitesk TO unitesk_user;
\c unitesk
GRANT ALL ON SCHEMA public TO unitesk_user;
\q
```

### 2. Configurar variável de ambiente

```bash
export DATABASE_URL="postgres://unitesk_user:sua_senha@localhost:5432/unitesk"
```

Alternativamente, o instalador `.deb` cria automaticamente
`/etc/unitesk/unitesk.conf` com a `DATABASE_URL` configurada.
> ou `./unitesk-setup`.

> 🛠️ **Correção necessária:** Se o app não conectar ao banco, verifique se a senha do
> usuário `postgres` está definida:
> ```bash
> sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"
> ```
> O app usa `postgres:postgres@localhost` como padrão no arquivo `.env`.
> Sem essa senha, a conexão falha com `password authentication failed`.

---

## 📊 Schema do Banco

### Tabela: `projects`

| Coluna       | Tipo         | Descrição                    |
|-------------|-------------|------------------------------|
| id          | SERIAL (PK)  | Identificador único          |
| name        | VARCHAR(255) | Nome do projeto              |
| description | TEXT         | Descrição do projeto         |
| created_at  | TIMESTAMP    | Data de criação              |

### Tabela: `articles`

| Coluna       | Tipo             | Descrição                    |
|-------------|-----------------|------------------------------|
| id          | SERIAL (PK)      | Identificador único          |
| title       | VARCHAR(255)     | Título do artigo             |
| content     | TEXT             | Conteúdo do artigo           |
| project_name| VARCHAR(255)     | Nome do projeto relacionado  |
| project_id  | INT (FK)         | Referência ao projeto (ON DELETE SET NULL) |
| created_at  | TIMESTAMP        | Data de criação              |

### Tabela: `assignments`

| Coluna       | Tipo             | Descrição                    |
|-------------|-----------------|------------------------------|
| id          | SERIAL (PK)      | Identificador único          |
| title       | VARCHAR(255)     | Título da atividade          |
| description | TEXT             | Descrição da atividade       |
| due_date    | DATE             | Data de entrega              |
| project_name| VARCHAR(255)     | Nome do projeto relacionado  |
| status      | VARCHAR(20)      | Status: pending / done / overdue |
| created_at  | TIMESTAMP        | Data de criação              |

### Tabela: `project_files`

| Coluna        | Tipo             | Descrição                          |
|--------------|-----------------|------------------------------------|
| id           | SERIAL (PK)      | Identificador único                |
| project_id   | INT (FK)         | Referência ao projeto (ON DELETE CASCADE) |
| original_name| VARCHAR(500)     | Nome original do arquivo           |
| stored_name  | VARCHAR(500)     | Nome interno armazenado            |
| file_data    | BYTEA            | Conteúdo binário do arquivo        |
| file_size    | BIGINT           | Tamanho em bytes                   |
| mime_type    | VARCHAR(100)     | Tipo MIME (ex: application/pdf)    |
| created_at   | TIMESTAMP        | Data de upload                     |

> 🔗 `project_id` possui `ON DELETE CASCADE` — quando um projeto é deletado,
> todos os seus arquivos são removidos automaticamente.

### Índices

```sql
CREATE INDEX idx_assignments_due_date ON assignments(due_date);
CREATE INDEX idx_assignments_status ON assignments(status);
CREATE INDEX idx_articles_project_id ON articles(project_id);
CREATE INDEX idx_project_files_project_id ON project_files(project_id);
```

---

## ⚠️ Diferenças PostgreSQL vs SQLite

| PostgreSQL | SQLite |
|---|---|
| `CURRENT_DATE` | `date('now')` |
| `CURRENT_TIME` | `time('now')` |
| `CURRENT_TIMESTAMP` | `datetime('now')` |
| `coluna::text` | `CAST(coluna AS TEXT)` |
| `SERIAL PRIMARY KEY` | `INTEGER PRIMARY KEY AUTOINCREMENT` |
| `BYTEA` | `BLOB` |
| `BOOLEAN` | `INTEGER (0/1)` |
| Bind: `$1`, `$2` | Bind: `?1`, `?2` |
| `INTERVAL '30 seconds'` | Filtrar em Rust (não suportado) |
| `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` | Schema fixo (criado no init) |

---

## 🔄 Migrations

As tabelas são criadas automaticamente na inicialização do app
(`db::init_db()` em `src-tauri/src/db.rs`). Não é necessário rodar
migrations manualmente.

O arquivo `docs/setup.sql` contém o schema completo para setup manual no PostgreSQL.

---

## 🔍 Consultas Úteis

### PostgreSQL (Linux)

```sql
-- Atividades que vencem hoje
SELECT * FROM assignments
WHERE due_date = CURRENT_DATE AND status = 'pending';
```

### SQLite (Windows)

```sql
-- Atividades que vencem hoje
SELECT * FROM assignments
WHERE due_date = date('now') AND status = 'pending';
```

### Comuns (ambos os bancos)

```sql
-- Ver todas as atividades pendentes ordenadas por data
SELECT * FROM assignments
WHERE status = 'pending'
ORDER BY due_date ASC;

-- Ver artigos de um projeto específico
SELECT * FROM articles
WHERE project_id = 1;

-- Ver arquivos de um projeto
SELECT id, original_name, file_size, mime_type, created_at
FROM project_files
WHERE project_id = 1
ORDER BY created_at DESC;

-- Atividades que vencem hoje
SELECT * FROM assignments
WHERE due_date = CURRENT_DATE AND status = 'pending';

-- Estatísticas do dashboard
SELECT
  (SELECT COUNT(*) FROM projects) AS total_projects,
  (SELECT COUNT(*) FROM articles) AS total_articles,
  (SELECT COUNT(*) FROM assignments WHERE status = 'pending') AS pending;
```
