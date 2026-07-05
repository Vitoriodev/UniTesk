// =============================================================================
// 🎓 Unitesk — Operações de Banco de Dados
// =============================================================================
//
// Suporte a duas plataformas:
//   Linux  → PostgreSQL (via PgPool)
//   Windows → SQLite     (via SqlitePool)
//
// A compilação condicional (#[cfg(target_os)]) seleciona a implementação
// correta para cada plataforma.
// =============================================================================

use serde::{Serialize, Deserialize};

use crate::models::*;

// =============================================================================
// Pool Type — selecionado por plataforma
// =============================================================================

#[cfg(target_os = "linux")]
use sqlx::postgres::PgPoolOptions;
#[cfg(target_os = "linux")]
pub type DbPool = sqlx::PgPool;

#[cfg(target_os = "windows")]
use sqlx::sqlite::SqlitePoolOptions;
#[cfg(target_os = "windows")]
pub type DbPool = sqlx::SqlitePool;

// =============================================================================
// Tipos comuns (compartilhados entre plataformas)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedData {
    pub version: String,
    pub exported_at: String,
    pub projects: Vec<Project>,
    pub articles: Vec<Article>,
    pub assignments: Vec<Assignment>,
    pub project_files: Vec<ExportedProjectFile>,
    pub assignment_files: Vec<ExportedAssignmentFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedProjectFile {
    pub project_id: i32,
    pub original_name: String,
    pub stored_name: String,
    pub file_data_base64: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedAssignmentFile {
    pub assignment_id: i32,
    pub original_name: String,
    pub stored_name: String,
    pub file_data_base64: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
}

// =============================================================================
// Helpers comuns
// =============================================================================

fn sanitize_filename(name: &str) -> String {
    let basename = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let mut safe = basename.to_string();
    while safe.contains("..") {
        safe = safe.replace("..", "");
    }
    safe = safe.replace('/', "_").replace('\\', "_").replace('~', "_");
    let clean: String = safe
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == ' ')
        .collect();
    let truncated: String = clean.chars().take(200).collect();
    let trimmed = truncated.trim().to_string();
    if trimmed.is_empty() { "arquivo_sem_nome".to_string() } else { trimmed }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(encoded)
        .map_err(|_| "Erro ao decodificar arquivo: formato inválido".to_string())
}

fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ' { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
}

// =============================================================================
//  INIT_DB — INICIALIZAÇÃO DO BANCO
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    use sqlx::Executor;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // ===================== Tabelas =====================
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS projects (
            id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL,
            description TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS articles (
            id SERIAL PRIMARY KEY, title VARCHAR(255) NOT NULL,
            content TEXT, project_name VARCHAR(255),
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS assignments (
            id SERIAL PRIMARY KEY, title VARCHAR(255) NOT NULL,
            description TEXT, due_date DATE NOT NULL,
            due_time TIME, notification_time TIME,
            project_name VARCHAR(255), status VARCHAR(20) DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS project_files (
            id SERIAL PRIMARY KEY,
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            original_name VARCHAR(500) NOT NULL,
            stored_name VARCHAR(500) NOT NULL,
            file_data BYTEA NOT NULL, file_size BIGINT NOT NULL,
            mime_type VARCHAR(100) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS assignment_files (
            id SERIAL PRIMARY KEY,
            assignment_id INTEGER NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
            original_name VARCHAR(500) NOT NULL,
            stored_name VARCHAR(500) NOT NULL,
            file_data BYTEA NOT NULL, file_size BIGINT NOT NULL,
            mime_type VARCHAR(100) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    // Migrações
    sqlx::query("ALTER TABLE assignments ADD COLUMN IF NOT EXISTS due_time TIME")
        .execute(&pool).await?;
    sqlx::query("ALTER TABLE assignments ADD COLUMN IF NOT EXISTS notification_time TIME")
        .execute(&pool).await?;
    sqlx::query("ALTER TABLE articles ADD COLUMN IF NOT EXISTS scheduled_date DATE")
        .execute(&pool).await?;

    // Tabelas empresariais
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS clients (
            id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL,
            email VARCHAR(255), phone VARCHAR(50), company VARCHAR(255),
            notes TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            role VARCHAR(50) DEFAULT 'member',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS teams (
            id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL,
            description TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS team_members (
            id SERIAL PRIMARY KEY,
            team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role VARCHAR(50) DEFAULT 'member',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(team_id, user_id))"#
    ).await?;

    sqlx::query(
        "ALTER TABLE projects ADD COLUMN IF NOT EXISTS client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL"
    ).execute(&pool).await?;
    sqlx::query(
        "ALTER TABLE assignments ADD COLUMN IF NOT EXISTS priority VARCHAR(20) DEFAULT 'medium'"
    ).execute(&pool).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS time_entries (
            id SERIAL PRIMARY KEY,
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            description TEXT,
            start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            end_time TIMESTAMP, duration_minutes INTEGER,
            billable BOOLEAN DEFAULT true,
            hourly_rate DECIMAL(10,2),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS invoices (
            id SERIAL PRIMARY KEY,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            number VARCHAR(50) UNIQUE NOT NULL,
            description TEXT, amount DECIMAL(12,2) NOT NULL,
            tax DECIMAL(12,2) DEFAULT 0, total DECIMAL(12,2) NOT NULL,
            status VARCHAR(20) DEFAULT 'draft',
            issue_date DATE NOT NULL, due_date DATE, paid_date DATE,
            notes TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS notifications (
            id SERIAL PRIMARY KEY, type VARCHAR(50) NOT NULL,
            title VARCHAR(255) NOT NULL, message TEXT NOT NULL,
            is_read BOOLEAN DEFAULT false,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"#
    ).await?;

    // ===================== Índices =====================
    for idx in &[
        "CREATE INDEX IF NOT EXISTS idx_assignments_due_date ON assignments(due_date)",
        "CREATE INDEX IF NOT EXISTS idx_assignments_status ON assignments(status)",
        "CREATE INDEX IF NOT EXISTS idx_articles_project_id ON articles(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_project_files_project_id ON project_files(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_projects_client_id ON projects(client_id)",
        "CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id)",
        "CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_time_entries_project_id ON time_entries(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_time_entries_start_time ON time_entries(start_time)",
        "CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status)",
        "CREATE INDEX IF NOT EXISTS idx_invoices_issue_date ON invoices(issue_date)",
        "CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read)",
        "CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at)",
    ] {
        sqlx::query(idx).execute(&pool).await?;
    }

    Ok(pool)
}

// =============================================================================
//  INIT_DB — SQLite (Windows)
// =============================================================================

#[cfg(target_os = "windows")]
pub async fn init_db(_database_url: &str) -> Result<DbPool, sqlx::Error> {
    use sqlx::Executor;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("unitesk.db")
        .await?;

    // Ativar suporte a chaves estrangeiras
    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

    // ===================== Tabelas =====================
    pool.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            client_id INTEGER,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT,
            project_name TEXT,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            scheduled_date TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            due_date TEXT NOT NULL,
            due_time TEXT,
            notification_time TEXT,
            project_name TEXT,
            status TEXT DEFAULT 'pending',
            priority TEXT DEFAULT 'medium',
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS project_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            original_name TEXT NOT NULL,
            stored_name TEXT NOT NULL,
            file_data BLOB NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS assignment_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            assignment_id INTEGER NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
            original_name TEXT NOT NULL,
            stored_name TEXT NOT NULL,
            file_data BLOB NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    // ===================== Tabelas Empresariais =====================
    pool.execute(
        "CREATE TABLE IF NOT EXISTS clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT,
            phone TEXT,
            company TEXT,
            notes TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            role TEXT DEFAULT 'member',
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS teams (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS team_members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT DEFAULT 'member',
            created_at TEXT DEFAULT (datetime('now')),
            UNIQUE(team_id, user_id)
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS time_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            description TEXT,
            start_time TEXT NOT NULL DEFAULT (datetime('now')),
            end_time TEXT,
            duration_minutes INTEGER,
            billable INTEGER DEFAULT 1,
            hourly_rate REAL,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS invoices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            number TEXT UNIQUE NOT NULL,
            description TEXT,
            amount REAL NOT NULL,
            tax REAL DEFAULT 0,
            total REAL NOT NULL,
            status TEXT DEFAULT 'draft',
            issue_date TEXT NOT NULL,
            due_date TEXT,
            paid_date TEXT,
            notes TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            is_read INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        )"
    ).await?;

    // ===================== Índices =====================
    for idx in &[
        "CREATE INDEX IF NOT EXISTS idx_assignments_due_date ON assignments(due_date)",
        "CREATE INDEX IF NOT EXISTS idx_assignments_status ON assignments(status)",
        "CREATE INDEX IF NOT EXISTS idx_articles_project_id ON articles(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_project_files_project_id ON project_files(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_projects_client_id ON projects(client_id)",
        "CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id)",
        "CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_time_entries_project_id ON time_entries(project_id)",
        "CREATE INDEX IF NOT EXISTS idx_time_entries_start_time ON time_entries(start_time)",
        "CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status)",
        "CREATE INDEX IF NOT EXISTS idx_invoices_issue_date ON invoices(issue_date)",
        "CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read)",
        "CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at)",
    ] {
        sqlx::query(idx).execute(&pool).await?;
    }

    Ok(pool)
}

// =============================================================================
//  EXPORTAÇÃO / IMPORTAÇÃO (compartilhado — chama funções específicas)
// =============================================================================

pub async fn export_all_data(pool: &DbPool) -> Result<ExportedData, String> {
    let projects = get_projects(pool).await.map_err(|_| "Erro ao exportar projetos".to_string())?;
    let articles = get_articles(pool).await.map_err(|_| "Erro ao exportar artigos".to_string())?;
    let assignments = get_assignments(pool).await.map_err(|_| "Erro ao exportar atividades".to_string())?;

    let project_files_raw: Vec<(i32, String, String, i64, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT project_id, original_name, stored_name, file_size, mime_type, created_at, file_data FROM project_files"
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Erro ao exportar arquivos de projeto".to_string())?;

    let project_files: Vec<ExportedProjectFile> = project_files_raw
        .into_iter()
        .map(|(pid, on, sn, fs, mt, ca, fd)| ExportedProjectFile {
            project_id: pid, original_name: on, stored_name: sn,
            file_size: fs, mime_type: mt, created_at: ca,
            file_data_base64: base64_encode(&fd),
        })
        .collect();

    let assignment_files_raw: Vec<(i32, String, String, i64, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT assignment_id, original_name, stored_name, file_size, mime_type, created_at, file_data FROM assignment_files"
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Erro ao exportar arquivos de atividade".to_string())?;

    let assignment_files: Vec<ExportedAssignmentFile> = assignment_files_raw
        .into_iter()
        .map(|(aid, on, sn, fs, mt, ca, fd)| ExportedAssignmentFile {
            assignment_id: aid, original_name: on, stored_name: sn,
            file_size: fs, mime_type: mt, created_at: ca,
            file_data_base64: base64_encode(&fd),
        })
        .collect();

    Ok(ExportedData {
        version: "2.0.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        projects, articles, assignments,
        project_files, assignment_files,
    })
}

#[cfg(target_os = "linux")]
pub async fn import_all_data(pool: &DbPool, data: &ExportedData) -> Result<String, String> {
    let mut project_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for project in &data.projects {
        let new_project = sqlx::query_as::<_, Project>(
            "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id, name, description, NULL::int as client_id, NULL::text as client_name, created_at::text as created_at"
        )
        .bind(&project.name).bind(&project.description)
        .fetch_one(pool).await
        .map_err(|_| "Erro ao importar projeto".to_string())?;
        project_id_map.insert(project.id, new_project.id);
    }

    for article in &data.articles {
        let new_project_id = article.project_id.and_then(|pid| project_id_map.get(&pid).copied());
        sqlx::query(
            "INSERT INTO articles (title, content, project_name, project_id, scheduled_date) VALUES ($1, $2, $3, $4, $5::date)"
        )
        .bind(&article.title).bind(&article.content).bind(&article.project_name)
        .bind(new_project_id).bind(&article.scheduled_date)
        .execute(pool).await
        .map_err(|_| "Erro ao importar artigo".to_string())?;
    }

    let mut assignment_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for assignment in &data.assignments {
        let new_assignment = sqlx::query_as::<_, Assignment>(
            "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, status) \
             VALUES ($1, $2, $3::date, $4::time, $5::time, $6, $7) \
             RETURNING id, title, description, due_date::text as due_date, due_time::text as due_time, \
             notification_time::text as notification_time, project_name, status, 'medium' as priority, created_at::text as created_at"
        )
        .bind(&assignment.title).bind(&assignment.description).bind(&assignment.due_date)
        .bind(&assignment.due_time).bind(&assignment.notification_time)
        .bind(&assignment.project_name).bind(&assignment.status)
        .fetch_one(pool).await
        .map_err(|_| "Erro ao importar atividade".to_string())?;
        assignment_id_map.insert(assignment.id, new_assignment.id);
    }

    for pf in &data.project_files {
        if let Some(&new_pid) = project_id_map.get(&pf.project_id) {
            let file_data = base64_decode(&pf.file_data_base64)?;
            add_project_file(pool, new_pid, &pf.original_name, &pf.stored_name, &file_data, &pf.mime_type).await
                .map_err(|_| "Erro ao importar arquivo de projeto".to_string())?;
        }
    }

    for af in &data.assignment_files {
        if let Some(&new_aid) = assignment_id_map.get(&af.assignment_id) {
            let file_data = base64_decode(&af.file_data_base64)?;
            add_assignment_file(pool, new_aid, &af.original_name, &af.stored_name, &file_data, &af.mime_type).await
                .map_err(|_| "Erro ao importar arquivo de atividade".to_string())?;
        }
    }

    Ok(format!("Importação concluída: {} projetos, {} artigos, {} atividades, {} arquivos de projeto, {} arquivos de atividade",
        data.projects.len(), data.articles.len(), data.assignments.len(),
        data.project_files.len(), data.assignment_files.len()))
}

#[cfg(target_os = "windows")]
pub async fn import_all_data(pool: &DbPool, data: &ExportedData) -> Result<String, String> {
    let mut project_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for project in &data.projects {
        let new_project = sqlx::query_as::<_, Project>(
            "INSERT INTO projects (name, description) VALUES (?1, ?2) RETURNING id, name, description, client_id, '' as client_name, created_at"
        )
        .bind(&project.name).bind(&project.description)
        .fetch_one(pool).await
        .map_err(|_| "Erro ao importar projeto".to_string())?;
        project_id_map.insert(project.id, new_project.id);
    }

    for article in &data.articles {
        let new_project_id = article.project_id.and_then(|pid| project_id_map.get(&pid).copied());
        sqlx::query(
            "INSERT INTO articles (title, content, project_name, project_id, scheduled_date) VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(&article.title).bind(&article.content).bind(&article.project_name)
        .bind(new_project_id).bind(&article.scheduled_date)
        .execute(pool).await
        .map_err(|_| "Erro ao importar artigo".to_string())?;
    }

    let mut assignment_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for assignment in &data.assignments {
        let new_assignment = sqlx::query_as::<_, Assignment>(
            "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, status) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
             RETURNING id, title, description, due_date, due_time, notification_time, project_name, status, priority, created_at"
        )
        .bind(&assignment.title).bind(&assignment.description).bind(&assignment.due_date)
        .bind(&assignment.due_time).bind(&assignment.notification_time)
        .bind(&assignment.project_name).bind(&assignment.status)
        .fetch_one(pool).await
        .map_err(|_| "Erro ao importar atividade".to_string())?;
        assignment_id_map.insert(assignment.id, new_assignment.id);
    }

    for pf in &data.project_files {
        if let Some(&new_pid) = project_id_map.get(&pf.project_id) {
            let file_data = base64_decode(&pf.file_data_base64)?;
            add_project_file(pool, new_pid, &pf.original_name, &pf.stored_name, &file_data, &pf.mime_type).await
                .map_err(|_| "Erro ao importar arquivo de projeto".to_string())?;
        }
    }

    for af in &data.assignment_files {
        if let Some(&new_aid) = assignment_id_map.get(&af.assignment_id) {
            let file_data = base64_decode(&af.file_data_base64)?;
            add_assignment_file(pool, new_aid, &af.original_name, &af.stored_name, &file_data, &af.mime_type).await
                .map_err(|_| "Erro ao importar arquivo de atividade".to_string())?;
        }
    }

    Ok(format!("Importação concluída: {} projetos, {} artigos, {} atividades, {} arquivos de projeto, {} arquivos de atividade",
        data.projects.len(), data.articles.len(), data.assignments.len(),
        data.project_files.len(), data.assignment_files.len()))
}

// =============================================================================
//  PROJETOS
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_projects(pool: &DbPool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT p.id, p.name, p.description, p.client_id, c.name as client_name, p.created_at::text as created_at \
         FROM projects p LEFT JOIN clients c ON c.id = p.client_id ORDER BY p.created_at DESC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_projects(pool: &DbPool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT p.id, p.name, p.description, p.client_id, c.name as client_name, p.created_at \
         FROM projects p LEFT JOIN clients c ON c.id = p.client_id ORDER BY p.created_at DESC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_project(pool: &DbPool, name: &str, description: &str, client_id: Option<i32>) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description, client_id) VALUES ($1, $2, $3) \
         RETURNING id, name, description, client_id, \
         (SELECT name FROM clients WHERE id = $3) as client_name, created_at::text as created_at"
    )
    .bind(name).bind(description).bind(client_id)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_project(pool: &DbPool, name: &str, description: &str, client_id: Option<i32>) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description, client_id) VALUES (?1, ?2, ?3) \
         RETURNING id, name, description, client_id, \
         (SELECT name FROM clients WHERE id = ?3) as client_name, created_at"
    )
    .bind(name).bind(description).bind(client_id)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn update_project(pool: &DbPool, id: i32, name: &str, description: &str, client_id: Option<i32>) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = $1, description = $2, client_id = $3 WHERE id = $4 \
         RETURNING id, name, description, client_id, \
         (SELECT name FROM clients WHERE id = $3) as client_name, created_at::text as created_at"
    )
    .bind(name).bind(description).bind(client_id).bind(id)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_project(pool: &DbPool, id: i32, name: &str, description: &str, client_id: Option<i32>) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = ?1, description = ?2, client_id = ?3 WHERE id = ?4 \
         RETURNING id, name, description, client_id, \
         (SELECT name FROM clients WHERE id = ?3) as client_name, created_at"
    )
    .bind(name).bind(description).bind(client_id).bind(id)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_project(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_project(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM projects WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  ARQUIVOS DE PROJETO
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_project_files(pool: &DbPool, project_id: i32) -> Result<Vec<ProjectFile>, sqlx::Error> {
    sqlx::query_as::<_, ProjectFile>(
        "SELECT id, project_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at \
         FROM project_files WHERE project_id = $1 ORDER BY created_at DESC"
    )
    .bind(project_id).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_project_files(pool: &DbPool, project_id: i32) -> Result<Vec<ProjectFile>, sqlx::Error> {
    sqlx::query_as::<_, ProjectFile>(
        "SELECT id, project_id, original_name, stored_name, file_size, mime_type, created_at \
         FROM project_files WHERE project_id = ?1 ORDER BY created_at DESC"
    )
    .bind(project_id).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn add_project_file(pool: &DbPool, project_id: i32, original_name: &str, stored_name: &str, file_data: &[u8], mime_type: &str) -> Result<ProjectFile, sqlx::Error> {
    sqlx::query_as::<_, ProjectFile>(
        "INSERT INTO project_files (project_id, original_name, stored_name, file_data, file_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, project_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at"
    )
    .bind(project_id).bind(original_name).bind(stored_name).bind(file_data)
    .bind(file_data.len() as i64).bind(mime_type)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn add_project_file(pool: &DbPool, project_id: i32, original_name: &str, stored_name: &str, file_data: &[u8], mime_type: &str) -> Result<ProjectFile, sqlx::Error> {
    sqlx::query_as::<_, ProjectFile>(
        "INSERT INTO project_files (project_id, original_name, stored_name, file_data, file_size, mime_type) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         RETURNING id, project_id, original_name, stored_name, file_size, mime_type, created_at"
    )
    .bind(project_id).bind(original_name).bind(stored_name).bind(file_data)
    .bind(file_data.len() as i64).bind(mime_type)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn get_project_file_data(pool: &DbPool, file_id: i32) -> Result<(String, String, Vec<u8>), sqlx::Error> {
    let row: (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE id = $1"
    )
    .bind(file_id).fetch_one(pool).await?;
    Ok(row)
}

#[cfg(target_os = "windows")]
pub async fn get_project_file_data(pool: &DbPool, file_id: i32) -> Result<(String, String, Vec<u8>), sqlx::Error> {
    let row: (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE id = ?1"
    )
    .bind(file_id).fetch_one(pool).await?;
    Ok(row)
}

#[cfg(target_os = "linux")]
pub async fn delete_project_file(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM project_files WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_project_file(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM project_files WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  ARTIGOS
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_articles(pool: &DbPool) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "SELECT id, title, content, project_name, project_id, created_at::text as created_at, \
         scheduled_date::text as scheduled_date FROM articles ORDER BY created_at DESC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_articles(pool: &DbPool) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "SELECT id, title, content, project_name, project_id, created_at, \
         scheduled_date FROM articles ORDER BY created_at DESC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_article(pool: &DbPool, title: &str, content: &str, project_name: &str, project_id: Option<i32>, scheduled_date: Option<String>) -> Result<Article, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "INSERT INTO articles (title, content, project_name, project_id, scheduled_date) \
         VALUES ($1, $2, $3, $4, $5::date) \
         RETURNING id, title, content, project_name, project_id, created_at::text as created_at, scheduled_date::text as scheduled_date"
    )
    .bind(title).bind(content).bind(project_name).bind(project_id).bind(&scheduled_date)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_article(pool: &DbPool, title: &str, content: &str, project_name: &str, project_id: Option<i32>, scheduled_date: Option<String>) -> Result<Article, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "INSERT INTO articles (title, content, project_name, project_id, scheduled_date) \
         VALUES (?1, ?2, ?3, ?4, ?5) \
         RETURNING id, title, content, project_name, project_id, created_at, scheduled_date"
    )
    .bind(title).bind(content).bind(project_name).bind(project_id).bind(&scheduled_date)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_article(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM articles WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_article(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM articles WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  ATIVIDADES (ASSIGNMENTS)
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_assignments(pool: &DbPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_assignments(pool: &DbPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date, due_time, \
         notification_time, project_name, status, priority, \
         created_at FROM assignments ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_assignment(pool: &DbPool, title: &str, description: &str, due_date: &str, due_time: Option<String>, project_name: &str, priority: &str) -> Result<Assignment, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, priority) \
         VALUES ($1, $2, $3::date, $4::time, $5::time, $6, $7) \
         RETURNING id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, created_at::text as created_at"
    )
    .bind(title).bind(description).bind(due_date).bind(&due_time).bind(&due_time).bind(project_name).bind(priority)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_assignment(pool: &DbPool, title: &str, description: &str, due_date: &str, due_time: Option<String>, project_name: &str, priority: &str) -> Result<Assignment, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, priority) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         RETURNING id, title, description, due_date, due_time, \
         notification_time, project_name, status, priority, created_at"
    )
    .bind(title).bind(description).bind(due_date).bind(&due_time).bind(&due_time).bind(project_name).bind(priority)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn mark_assignment_done(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE assignments SET status = 'done' WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn mark_assignment_done(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE assignments SET status = 'done' WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn delete_assignment(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM assignments WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_assignment(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM assignments WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn update_overdue_assignments(pool: &DbPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query("UPDATE assignments SET status = 'overdue' WHERE due_date < CURRENT_DATE AND status = 'pending'")
        .execute(pool).await?;
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date <= CURRENT_DATE AND status != 'done' ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_overdue_assignments(pool: &DbPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query("UPDATE assignments SET status = 'overdue' WHERE due_date < date('now') AND status = 'pending'")
        .execute(pool).await?;
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date, due_time, \
         notification_time, project_name, status, priority, \
         created_at FROM assignments \
         WHERE due_date <= date('now') AND status != 'done' ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn get_today_assignments(pool: &DbPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date = CURRENT_DATE AND status = 'pending' \
         AND notification_time BETWEEN CURRENT_TIME - INTERVAL '30 seconds' AND CURRENT_TIME + INTERVAL '30 seconds'"
    )
    .fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_today_assignments(pool: &DbPool) -> Result<Vec<Assignment>, sqlx::Error> {
    // SQLite não suporta INTERVAL — busca todas as atividades de hoje e filtra em Rust
    let assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date, due_time, \
         notification_time, project_name, status, priority, \
         created_at FROM assignments \
         WHERE due_date = date('now') AND status = 'pending'"
    )
    .fetch_all(pool).await?;

    // Filtrar por janela de 30 segundos (em Rust)
    let now = chrono::Local::now().time();
    Ok(assignments.into_iter().filter(|a| {
        if let Some(ref nt) = a.notification_time {
            if let Ok(notif_time) = chrono::NaiveTime::parse_from_str(nt, "%H:%M:%S") {
                let diff = (notif_time - now).num_seconds().abs();
                return diff <= 30;
            }
        }
        false
    }).collect())
}

// =============================================================================
//  ARQUIVOS DE ATIVIDADES (ASSIGNMENT FILES)
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_assignment_files(pool: &DbPool, assignment_id: i32) -> Result<Vec<AssignmentFile>, sqlx::Error> {
    sqlx::query_as::<_, AssignmentFile>(
        "SELECT id, assignment_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at \
         FROM assignment_files WHERE assignment_id = $1 ORDER BY created_at DESC"
    )
    .bind(assignment_id).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_assignment_files(pool: &DbPool, assignment_id: i32) -> Result<Vec<AssignmentFile>, sqlx::Error> {
    sqlx::query_as::<_, AssignmentFile>(
        "SELECT id, assignment_id, original_name, stored_name, file_size, mime_type, created_at \
         FROM assignment_files WHERE assignment_id = ?1 ORDER BY created_at DESC"
    )
    .bind(assignment_id).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn add_assignment_file(pool: &DbPool, assignment_id: i32, original_name: &str, stored_name: &str, file_data: &[u8], mime_type: &str) -> Result<AssignmentFile, sqlx::Error> {
    sqlx::query_as::<_, AssignmentFile>(
        "INSERT INTO assignment_files (assignment_id, original_name, stored_name, file_data, file_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, assignment_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at"
    )
    .bind(assignment_id).bind(original_name).bind(stored_name).bind(file_data)
    .bind(file_data.len() as i64).bind(mime_type)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn add_assignment_file(pool: &DbPool, assignment_id: i32, original_name: &str, stored_name: &str, file_data: &[u8], mime_type: &str) -> Result<AssignmentFile, sqlx::Error> {
    sqlx::query_as::<_, AssignmentFile>(
        "INSERT INTO assignment_files (assignment_id, original_name, stored_name, file_data, file_size, mime_type) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         RETURNING id, assignment_id, original_name, stored_name, file_size, mime_type, created_at"
    )
    .bind(assignment_id).bind(original_name).bind(stored_name).bind(file_data)
    .bind(file_data.len() as i64).bind(mime_type)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn get_assignment_file_data(pool: &DbPool, file_id: i32) -> Result<(String, String, Vec<u8>), sqlx::Error> {
    let row: (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM assignment_files WHERE id = $1"
    ).bind(file_id).fetch_one(pool).await?;
    Ok(row)
}

#[cfg(target_os = "windows")]
pub async fn get_assignment_file_data(pool: &DbPool, file_id: i32) -> Result<(String, String, Vec<u8>), sqlx::Error> {
    let row: (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM assignment_files WHERE id = ?1"
    ).bind(file_id).fetch_one(pool).await?;
    Ok(row)
}

#[cfg(target_os = "linux")]
pub async fn delete_assignment_file(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM assignment_files WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_assignment_file(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM assignment_files WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  CLIENTES
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_clients(pool: &DbPool) -> Result<Vec<Client>, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "SELECT id, name, email, phone, company, notes, created_at::text as created_at FROM clients ORDER BY name ASC"
    ).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_clients(pool: &DbPool) -> Result<Vec<Client>, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "SELECT id, name, email, phone, company, notes, created_at FROM clients ORDER BY name ASC"
    ).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_client(pool: &DbPool, name: &str, email: Option<String>, phone: Option<String>, company: Option<String>, notes: Option<String>) -> Result<Client, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "INSERT INTO clients (name, email, phone, company, notes) VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, name, email, phone, company, notes, created_at::text as created_at"
    )
    .bind(name).bind(email).bind(phone).bind(company).bind(notes)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_client(pool: &DbPool, name: &str, email: Option<String>, phone: Option<String>, company: Option<String>, notes: Option<String>) -> Result<Client, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "INSERT INTO clients (name, email, phone, company, notes) VALUES (?1, ?2, ?3, ?4, ?5) \
         RETURNING id, name, email, phone, company, notes, created_at"
    )
    .bind(name).bind(email).bind(phone).bind(company).bind(notes)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn update_client(pool: &DbPool, id: i32, name: &str, email: Option<String>, phone: Option<String>, company: Option<String>, notes: Option<String>) -> Result<Client, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "UPDATE clients SET name = $1, email = $2, phone = $3, company = $4, notes = $5 WHERE id = $6 \
         RETURNING id, name, email, phone, company, notes, created_at::text as created_at"
    )
    .bind(name).bind(email).bind(phone).bind(company).bind(notes).bind(id)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_client(pool: &DbPool, id: i32, name: &str, email: Option<String>, phone: Option<String>, company: Option<String>, notes: Option<String>) -> Result<Client, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "UPDATE clients SET name = ?1, email = ?2, phone = ?3, company = ?4, notes = ?5 WHERE id = ?6 \
         RETURNING id, name, email, phone, company, notes, created_at"
    )
    .bind(name).bind(email).bind(phone).bind(company).bind(notes).bind(id)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_client(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE projects SET client_id = NULL WHERE client_id = $1").bind(id).execute(pool).await?;
    sqlx::query("DELETE FROM clients WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_client(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE projects SET client_id = NULL WHERE client_id = ?1").bind(id).execute(pool).await?;
    sqlx::query("DELETE FROM clients WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  USUÁRIOS
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_users(pool: &DbPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, created_at::text as created_at FROM users ORDER BY name ASC"
    ).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_users(pool: &DbPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, created_at FROM users ORDER BY name ASC"
    ).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_user(pool: &DbPool, name: &str, email: &str, role: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, role) VALUES ($1, $2, $3) \
         RETURNING id, name, email, role, created_at::text as created_at"
    ).bind(name).bind(email).bind(role).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_user(pool: &DbPool, name: &str, email: &str, role: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, role) VALUES (?1, ?2, ?3) \
         RETURNING id, name, email, role, created_at"
    ).bind(name).bind(email).bind(role).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn update_user(pool: &DbPool, id: i32, name: &str, email: &str, role: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE users SET name = $1, email = $2, role = $3 WHERE id = $4 \
         RETURNING id, name, email, role, created_at::text as created_at"
    ).bind(name).bind(email).bind(role).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_user(pool: &DbPool, id: i32, name: &str, email: &str, role: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE users SET name = ?1, email = ?2, role = ?3 WHERE id = ?4 \
         RETURNING id, name, email, role, created_at"
    ).bind(name).bind(email).bind(role).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_user(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM team_members WHERE user_id = $1").bind(id).execute(pool).await?;
    sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_user(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM team_members WHERE user_id = ?1").bind(id).execute(pool).await?;
    sqlx::query("DELETE FROM users WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  EQUIPES
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_teams(pool: &DbPool) -> Result<Vec<Team>, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "SELECT id, name, description, created_at::text as created_at FROM teams ORDER BY name ASC"
    ).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_teams(pool: &DbPool) -> Result<Vec<Team>, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "SELECT id, name, description, created_at FROM teams ORDER BY name ASC"
    ).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_team(pool: &DbPool, name: &str, description: Option<String>) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "INSERT INTO teams (name, description) VALUES ($1, $2) \
         RETURNING id, name, description, created_at::text as created_at"
    ).bind(name).bind(description).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_team(pool: &DbPool, name: &str, description: Option<String>) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "INSERT INTO teams (name, description) VALUES (?1, ?2) \
         RETURNING id, name, description, created_at"
    ).bind(name).bind(description).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn update_team(pool: &DbPool, id: i32, name: &str, description: Option<String>) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "UPDATE teams SET name = $1, description = $2 WHERE id = $3 \
         RETURNING id, name, description, created_at::text as created_at"
    ).bind(name).bind(description).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_team(pool: &DbPool, id: i32, name: &str, description: Option<String>) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "UPDATE teams SET name = ?1, description = ?2 WHERE id = ?3 \
         RETURNING id, name, description, created_at"
    ).bind(name).bind(description).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_team(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM teams WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_team(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM teams WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  MEMBROS DA EQUIPE
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_team_members(pool: &DbPool, team_id: i32) -> Result<Vec<TeamMember>, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "SELECT tm.id, tm.team_id, tm.user_id, u.name as user_name, u.email as user_email, \
         tm.role, tm.created_at::text as created_at \
         FROM team_members tm JOIN users u ON u.id = tm.user_id WHERE tm.team_id = $1 ORDER BY u.name ASC"
    ).bind(team_id).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_team_members(pool: &DbPool, team_id: i32) -> Result<Vec<TeamMember>, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "SELECT tm.id, tm.team_id, tm.user_id, u.name as user_name, u.email as user_email, \
         tm.role, tm.created_at \
         FROM team_members tm JOIN users u ON u.id = tm.user_id WHERE tm.team_id = ?1 ORDER BY u.name ASC"
    ).bind(team_id).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn add_team_member(pool: &DbPool, team_id: i32, user_id: i32, role: &str) -> Result<TeamMember, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3) \
         RETURNING id, team_id, user_id, \
         (SELECT name FROM users WHERE id = $2) as user_name, \
         (SELECT email FROM users WHERE id = $2) as user_email, \
         role, created_at::text as created_at"
    ).bind(team_id).bind(user_id).bind(role).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn add_team_member(pool: &DbPool, team_id: i32, user_id: i32, role: &str) -> Result<TeamMember, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "INSERT INTO team_members (team_id, user_id, role) VALUES (?1, ?2, ?3) \
         RETURNING id, team_id, user_id, \
         (SELECT name FROM users WHERE id = ?2) as user_name, \
         (SELECT email FROM users WHERE id = ?2) as user_email, \
         role, created_at"
    ).bind(team_id).bind(user_id).bind(role).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn remove_team_member(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM team_members WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn remove_team_member(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM team_members WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn update_team_member_role(pool: &DbPool, id: i32, role: &str) -> Result<TeamMember, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "UPDATE team_members SET role = $1 WHERE id = $2 \
         RETURNING id, team_id, user_id, \
         (SELECT name FROM users WHERE id = user_id) as user_name, \
         (SELECT email FROM users WHERE id = user_id) as user_email, \
         role, created_at::text as created_at"
    ).bind(role).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_team_member_role(pool: &DbPool, id: i32, role: &str) -> Result<TeamMember, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "UPDATE team_members SET role = ?1 WHERE id = ?2 \
         RETURNING id, team_id, user_id, \
         (SELECT name FROM users WHERE id = user_id) as user_name, \
         (SELECT email FROM users WHERE id = user_id) as user_email, \
         role, created_at"
    ).bind(role).bind(id).fetch_one(pool).await
}

// =============================================================================
//  REGISTRO DE HORAS (TIME ENTRIES)
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_time_entries(pool: &DbPool, project_id: Option<i32>, date_from: Option<String>, date_to: Option<String>) -> Result<Vec<TimeEntry>, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "SELECT te.id, te.project_id, p.name as project_name, te.user_id, u.name as user_name, \
         te.description, te.start_time::text as start_time, te.end_time::text as end_time, \
         te.duration_minutes, te.billable, te.hourly_rate, te.created_at::text as created_at \
         FROM time_entries te \
         LEFT JOIN projects p ON p.id = te.project_id \
         LEFT JOIN users u ON u.id = te.user_id \
         WHERE ($1::int IS NULL OR te.project_id = $1) \
         AND ($2::date IS NULL OR te.start_time >= $2::date) \
         AND ($3::date IS NULL OR te.start_time <= ($3::date + INTERVAL '1 day')) \
         ORDER BY te.start_time DESC"
    )
    .bind(project_id).bind(&date_from).bind(&date_to)
    .fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_time_entries(pool: &DbPool, project_id: Option<i32>, date_from: Option<String>, date_to: Option<String>) -> Result<Vec<TimeEntry>, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "SELECT te.id, te.project_id, COALESCE(p.name, '') as project_name, te.user_id, COALESCE(u.name, '') as user_name, \
         te.description, te.start_time, te.end_time, \
         te.duration_minutes, te.billable, te.hourly_rate, te.created_at \
         FROM time_entries te \
         LEFT JOIN projects p ON p.id = te.project_id \
         LEFT JOIN users u ON u.id = te.user_id \
         WHERE (?1 IS NULL OR te.project_id = ?1) \
         AND (?2 IS NULL OR te.start_time >= ?2) \
         AND (?3 IS NULL OR te.start_time <= ?3 || 'T23:59:59') \
         ORDER BY te.start_time DESC"
    )
    .bind(project_id).bind(&date_from).bind(&date_to)
    .fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn start_time_entry(pool: &DbPool, project_id: i32, user_id: Option<i32>, description: Option<String>) -> Result<TimeEntry, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "INSERT INTO time_entries (project_id, user_id, description, start_time) VALUES ($1, $2, $3, NOW()) \
         RETURNING id, project_id, \
         (SELECT name FROM projects WHERE id = $1) as project_name, \
         $2 as user_id, \
         (SELECT name FROM users WHERE id = $2) as user_name, \
         description, start_time::text as start_time, NULL::text as end_time, \
         NULL::int as duration_minutes, true as billable, NULL::float as hourly_rate, \
         created_at::text as created_at"
    )
    .bind(project_id).bind(user_id).bind(description)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn start_time_entry(pool: &DbPool, project_id: i32, user_id: Option<i32>, description: Option<String>) -> Result<TimeEntry, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "INSERT INTO time_entries (project_id, user_id, description, start_time) VALUES (?1, ?2, ?3, datetime('now')) \
         RETURNING id, project_id, \
         COALESCE((SELECT name FROM projects WHERE id = ?1), '') as project_name, \
         ?2 as user_id, \
         COALESCE((SELECT name FROM users WHERE id = ?2), '') as user_name, \
         description, start_time, NULL as end_time, \
         NULL as duration_minutes, 1 as billable, NULL as hourly_rate, \
         created_at"
    )
    .bind(project_id).bind(user_id).bind(description)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn stop_time_entry(pool: &DbPool, id: i32) -> Result<TimeEntry, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "UPDATE time_entries SET \
         end_time = NOW(), \
         duration_minutes = EXTRACT(EPOCH FROM (NOW() - start_time))::int / 60 \
         WHERE id = $1 \
         RETURNING id, project_id, \
         (SELECT name FROM projects WHERE id = project_id) as project_name, \
         user_id, \
         (SELECT name FROM users WHERE id = user_id) as user_name, \
         description, start_time::text as start_time, end_time::text as end_time, \
         duration_minutes, billable, hourly_rate, created_at::text as created_at"
    )
    .bind(id).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn stop_time_entry(pool: &DbPool, id: i32) -> Result<TimeEntry, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "UPDATE time_entries SET \
         end_time = datetime('now'), \
         duration_minutes = CAST((strftime('%s', 'now') - strftime('%s', start_time)) / 60 AS INTEGER) \
         WHERE id = ?1 \
         RETURNING id, project_id, \
         COALESCE((SELECT name FROM projects WHERE id = project_id), '') as project_name, \
         user_id, \
         COALESCE((SELECT name FROM users WHERE id = user_id), '') as user_name, \
         description, start_time, end_time, \
         duration_minutes, billable, hourly_rate, created_at"
    )
    .bind(id).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn add_manual_time_entry(pool: &DbPool, project_id: i32, user_id: Option<i32>, description: Option<String>, duration_minutes: i32, entry_date: String, billable: bool, hourly_rate: Option<f64>) -> Result<TimeEntry, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "INSERT INTO time_entries (project_id, user_id, description, start_time, end_time, duration_minutes, billable, hourly_rate) \
         VALUES ($1, $2, $3, $4::date, $4::date, $5, $6, $7) \
         RETURNING id, project_id, \
         (SELECT name FROM projects WHERE id = $1) as project_name, \
         $2 as user_id, \
         (SELECT name FROM users WHERE id = $2) as user_name, \
         description, start_time::text as start_time, end_time::text as end_time, \
         duration_minutes, billable, hourly_rate, created_at::text as created_at"
    )
    .bind(project_id).bind(user_id).bind(description).bind(&entry_date)
    .bind(duration_minutes).bind(billable).bind(hourly_rate)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn add_manual_time_entry(pool: &DbPool, project_id: i32, user_id: Option<i32>, description: Option<String>, duration_minutes: i32, entry_date: String, billable: bool, hourly_rate: Option<f64>) -> Result<TimeEntry, sqlx::Error> {
    let billable_int: i32 = if billable { 1 } else { 0 };
    sqlx::query_as::<_, TimeEntry>(
        "INSERT INTO time_entries (project_id, user_id, description, start_time, end_time, duration_minutes, billable, hourly_rate) \
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, ?7) \
         RETURNING id, project_id, \
         COALESCE((SELECT name FROM projects WHERE id = ?1), '') as project_name, \
         ?2 as user_id, \
         COALESCE((SELECT name FROM users WHERE id = ?2), '') as user_name, \
         description, start_time, end_time, \
         duration_minutes, billable, hourly_rate, created_at"
    )
    .bind(project_id).bind(user_id).bind(description).bind(&entry_date)
    .bind(duration_minutes).bind(billable_int).bind(hourly_rate)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_time_entry(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM time_entries WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_time_entry(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM time_entries WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn get_active_time_entry(pool: &DbPool) -> Result<Option<TimeEntry>, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "SELECT te.id, te.project_id, p.name as project_name, te.user_id, u.name as user_name, \
         te.description, te.start_time::text as start_time, te.end_time::text as end_time, \
         te.duration_minutes, te.billable, te.hourly_rate, te.created_at::text as created_at \
         FROM time_entries te \
         LEFT JOIN projects p ON p.id = te.project_id \
         LEFT JOIN users u ON u.id = te.user_id \
         WHERE te.end_time IS NULL \
         ORDER BY te.start_time DESC LIMIT 1"
    )
    .fetch_optional(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_active_time_entry(pool: &DbPool) -> Result<Option<TimeEntry>, sqlx::Error> {
    sqlx::query_as::<_, TimeEntry>(
        "SELECT te.id, te.project_id, COALESCE(p.name, '') as project_name, te.user_id, COALESCE(u.name, '') as user_name, \
         te.description, te.start_time, te.end_time, \
         te.duration_minutes, te.billable, te.hourly_rate, te.created_at \
         FROM time_entries te \
         LEFT JOIN projects p ON p.id = te.project_id \
         LEFT JOIN users u ON u.id = te.user_id \
         WHERE te.end_time IS NULL \
         ORDER BY te.start_time DESC LIMIT 1"
    )
    .fetch_optional(pool).await
}

#[cfg(target_os = "linux")]
pub async fn get_hours_summary(pool: &DbPool) -> Result<(i64, f64), sqlx::Error> {
    let today_minutes: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_entries \
         WHERE start_time::date = CURRENT_DATE AND end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    let week_minutes: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes::float), 0) FROM time_entries \
         WHERE start_time >= date_trunc('week', CURRENT_DATE) AND end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    Ok((today_minutes.0.unwrap_or(0), week_minutes.0.unwrap_or(0.0)))
}

#[cfg(target_os = "windows")]
pub async fn get_hours_summary(pool: &DbPool) -> Result<(i64, f64), sqlx::Error> {
    let today_minutes: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_entries \
         WHERE date(start_time) = date('now') AND end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    let week_minutes: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(CAST(SUM(duration_minutes) AS REAL), 0) FROM time_entries \
         WHERE start_time >= date('now', 'weekday 1', '-7 days') AND end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    Ok((today_minutes.0.unwrap_or(0), week_minutes.0.unwrap_or(0.0)))
}

// =============================================================================
//  FATURAS / FINANCEIRO
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_invoices(pool: &DbPool, status_filter: Option<String>) -> Result<Vec<Invoice>, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, p.name as project_name, i.client_id, c.name as client_name, \
         i.number, i.description, i.amount::float8 as amount, i.tax::float8 as tax, i.total::float8 as total, i.status, \
         i.issue_date::text as issue_date, i.due_date::text as due_date, \
         i.paid_date::text as paid_date, i.notes, i.created_at::text as created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE ($1::text IS NULL OR i.status = $1) \
         ORDER BY i.issue_date DESC"
    ).bind(&status_filter).fetch_all(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_invoices(pool: &DbPool, status_filter: Option<String>) -> Result<Vec<Invoice>, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, COALESCE(p.name, '') as project_name, i.client_id, COALESCE(c.name, '') as client_name, \
         i.number, i.description, i.amount as amount, i.tax as tax, i.total as total, i.status, \
         i.issue_date, i.due_date, \
         i.paid_date, i.notes, i.created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE (?1 IS NULL OR i.status = ?1) \
         ORDER BY i.issue_date DESC"
    ).bind(&status_filter).fetch_all(pool).await
}

#[cfg(target_os = "linux")]
pub async fn get_invoice(pool: &DbPool, id: i32) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, p.name as project_name, i.client_id, c.name as client_name, \
         i.number, i.description, i.amount::float8 as amount, i.tax::float8 as tax, i.total::float8 as total, i.status, \
         i.issue_date::text as issue_date, i.due_date::text as due_date, \
         i.paid_date::text as paid_date, i.notes, i.created_at::text as created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE i.id = $1"
    ).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn get_invoice(pool: &DbPool, id: i32) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, p.name as project_name, i.client_id, c.name as client_name, \
         i.number, i.description, i.amount, i.tax, i.total, i.status, \
         i.issue_date, i.due_date, \
         i.paid_date, i.notes, i.created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE i.id = ?1"
    ).bind(id).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn create_invoice(pool: &DbPool, project_id: Option<i32>, client_id: Option<i32>, number: &str, description: Option<String>, amount: f64, tax: f64, total: f64, status: &str, issue_date: &str, due_date: Option<String>, notes: Option<String>) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "INSERT INTO invoices (project_id, client_id, number, description, amount, tax, total, status, issue_date, due_date, notes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::date, $10::date, $11) \
         RETURNING id, project_id, \
         (SELECT name FROM projects WHERE id = $1) as project_name, \
         client_id, \
         (SELECT name FROM clients WHERE id = $2) as client_name, \
         number, description, amount::float8 as amount, tax::float8 as tax, total::float8 as total, status, \
         issue_date::text as issue_date, due_date::text as due_date, \
         NULL::text as paid_date, notes, created_at::text as created_at"
    )
    .bind(project_id).bind(client_id).bind(number).bind(description)
    .bind(amount).bind(tax).bind(total).bind(status).bind(issue_date).bind(&due_date).bind(notes)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_invoice(pool: &DbPool, project_id: Option<i32>, client_id: Option<i32>, number: &str, description: Option<String>, amount: f64, tax: f64, total: f64, status: &str, issue_date: &str, due_date: Option<String>, notes: Option<String>) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "INSERT INTO invoices (project_id, client_id, number, description, amount, tax, total, status, issue_date, due_date, notes) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) \
         RETURNING id, project_id, \
         COALESCE((SELECT name FROM projects WHERE id = ?1), '') as project_name, \
         client_id, \
         COALESCE((SELECT name FROM clients WHERE id = ?2), '') as client_name, \
         number, description, amount, tax, total, status, \
         issue_date, due_date, \
         NULL as paid_date, notes, created_at"
    )
    .bind(project_id).bind(client_id).bind(number).bind(description)
    .bind(amount).bind(tax).bind(total).bind(status).bind(issue_date).bind(&due_date).bind(notes)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn update_invoice(pool: &DbPool, id: i32, project_id: Option<i32>, client_id: Option<i32>, number: &str, description: Option<String>, amount: f64, tax: f64, total: f64, status: &str, issue_date: &str, due_date: Option<String>, paid_date: Option<String>, notes: Option<String>) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "UPDATE invoices SET \
         project_id = $1, client_id = $2, number = $3, description = $4, \
         amount = $5, tax = $6, total = $7, status = $8, \
         issue_date = $9::date, due_date = $10::date, paid_date = $11::date, notes = $12 \
         WHERE id = $13 \
         RETURNING id, project_id, \
         (SELECT name FROM projects WHERE id = $1) as project_name, \
         client_id, \
         (SELECT name FROM clients WHERE id = $2) as client_name, \
         number, description, amount::float8 as amount, tax::float8 as tax, total::float8 as total, status, \
         issue_date::text as issue_date, due_date::text as due_date, \
         paid_date::text as paid_date, notes, created_at::text as created_at"
    )
    .bind(project_id).bind(client_id).bind(number).bind(description)
    .bind(amount).bind(tax).bind(total).bind(status).bind(issue_date).bind(&due_date).bind(&paid_date).bind(notes).bind(id)
    .fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn update_invoice(pool: &DbPool, id: i32, project_id: Option<i32>, client_id: Option<i32>, number: &str, description: Option<String>, amount: f64, tax: f64, total: f64, status: &str, issue_date: &str, due_date: Option<String>, paid_date: Option<String>, notes: Option<String>) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "UPDATE invoices SET \
         project_id = ?1, client_id = ?2, number = ?3, description = ?4, \
         amount = ?5, tax = ?6, total = ?7, status = ?8, \
         issue_date = ?9, due_date = ?10, paid_date = ?11, notes = ?12 \
         WHERE id = ?13 \
         RETURNING id, project_id, \
         COALESCE((SELECT name FROM projects WHERE id = ?1), '') as project_name, \
         client_id, \
         COALESCE((SELECT name FROM clients WHERE id = ?2), '') as client_name, \
         number, description, amount, tax, total, status, \
         issue_date, due_date, \
         paid_date, notes, created_at"
    )
    .bind(project_id).bind(client_id).bind(number).bind(description)
    .bind(amount).bind(tax).bind(total).bind(status).bind(issue_date).bind(&due_date).bind(&paid_date).bind(notes).bind(id)
    .fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn delete_invoice(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invoices WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_invoice(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invoices WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// =============================================================================
//  NOTIFICAÇÕES
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn create_notification(pool: &DbPool, notif_type: &str, title: &str, message: &str) -> Result<Notification, sqlx::Error> {
    sqlx::query_as::<_, Notification>(
        "INSERT INTO notifications (type, title, message) VALUES ($1, $2, $3) \
         RETURNING id, type, title, message, is_read, created_at::text as created_at"
    ).bind(notif_type).bind(title).bind(message).fetch_one(pool).await
}

#[cfg(target_os = "windows")]
pub async fn create_notification(pool: &DbPool, notif_type: &str, title: &str, message: &str) -> Result<Notification, sqlx::Error> {
    sqlx::query_as::<_, Notification>(
        "INSERT INTO notifications (type, title, message) VALUES (?1, ?2, ?3) \
         RETURNING id, type, title, message, is_read, created_at"
    ).bind(notif_type).bind(title).bind(message).fetch_one(pool).await
}

#[cfg(target_os = "linux")]
pub async fn get_notifications(pool: &DbPool, unread_only: bool, limit: i64) -> Result<Vec<Notification>, sqlx::Error> {
    if unread_only {
        sqlx::query_as::<_, Notification>(
            "SELECT id, type, title, message, is_read, created_at::text as created_at \
             FROM notifications WHERE is_read = false ORDER BY created_at DESC LIMIT $1"
        ).bind(limit).fetch_all(pool).await
    } else {
        sqlx::query_as::<_, Notification>(
            "SELECT id, type, title, message, is_read, created_at::text as created_at \
             FROM notifications ORDER BY created_at DESC LIMIT $1"
        ).bind(limit).fetch_all(pool).await
    }
}

#[cfg(target_os = "windows")]
pub async fn get_notifications(pool: &DbPool, unread_only: bool, limit: i64) -> Result<Vec<Notification>, sqlx::Error> {
    if unread_only {
        sqlx::query_as::<_, Notification>(
            "SELECT id, type, title, message, is_read, created_at \
             FROM notifications WHERE is_read = 0 ORDER BY created_at DESC LIMIT ?1"
        ).bind(limit).fetch_all(pool).await
    } else {
        sqlx::query_as::<_, Notification>(
            "SELECT id, type, title, message, is_read, created_at \
             FROM notifications ORDER BY created_at DESC LIMIT ?1"
        ).bind(limit).fetch_all(pool).await
    }
}

#[cfg(target_os = "linux")]
pub async fn get_unread_notifications_count(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE is_read = false"
    ).fetch_one(pool).await?;
    Ok(count.0)
}

#[cfg(target_os = "windows")]
pub async fn get_unread_notifications_count(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE is_read = 0"
    ).fetch_one(pool).await?;
    Ok(count.0)
}

#[cfg(target_os = "linux")]
pub async fn mark_notification_read(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn mark_notification_read(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = 1 WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn mark_all_notifications_read(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE is_read = false").execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn mark_all_notifications_read(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = 1 WHERE is_read = 0").execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn delete_notification(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notifications WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn delete_notification(pool: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notifications WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn cleanup_old_notifications(pool: &DbPool, days: i32) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM notifications WHERE created_at < CURRENT_DATE - ($1 || ' days')::INTERVAL"
    ).bind(days).execute(pool).await?;
    Ok(result.rows_affected() as i64)
}

#[cfg(target_os = "windows")]
pub async fn cleanup_old_notifications(pool: &DbPool, days: i32) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM notifications WHERE created_at < datetime('now', '-' || ?1 || ' days')"
    ).bind(days).execute(pool).await?;
    Ok(result.rows_affected() as i64)
}

#[cfg(target_os = "linux")]
pub async fn auto_generate_notifications(pool: &DbPool) -> Result<Vec<Notification>, sqlx::Error> {
    let mut created = Vec::new();

    // Atividades com prazo hoje
    let today_assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date = CURRENT_DATE AND status = 'pending'"
    ).fetch_all(pool).await?;

    for a in &today_assignments {
        let project = a.project_name.as_deref().unwrap_or("");
        let msg = if project.is_empty() { format!("A atividade '{}' vence hoje!", a.title) }
                   else { format!("A atividade '{}' do projeto '{}' vence hoje!", a.title, project) };
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE type = 'assignment_due' AND message LIKE $1 AND created_at::date = CURRENT_DATE"
        ).bind(format!("%{}%", a.title)).fetch_one(pool).await?;
        if exists.0 == 0 {
            created.push(create_notification(pool, "assignment_due", "📅 Prazo Hoje!", &msg).await?);
        }
    }

    // Atividades atrasadas
    let overdue_assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date < CURRENT_DATE AND status = 'pending'"
    ).fetch_all(pool).await?;

    for a in &overdue_assignments {
        let project = a.project_name.as_deref().unwrap_or("");
        let msg = if project.is_empty() { format!("A atividade '{}' está atrasada!", a.title) }
                   else { format!("A atividade '{}' do projeto '{}' está atrasada!", a.title, project) };
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE type = 'assignment_overdue' AND message LIKE $1 AND created_at::date = CURRENT_DATE"
        ).bind(format!("%{}%", a.title)).fetch_one(pool).await?;
        if exists.0 == 0 {
            created.push(create_notification(pool, "assignment_overdue", "🔴 Atividade Atrasada!", &msg).await?);
        }
    }

    // Faturas próximas do vencimento (7 dias)
    let soon_invoices: Vec<Invoice> = sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, p.name as project_name, i.client_id, c.name as client_name, \
         i.number, i.description, i.amount::float8 as amount, i.tax::float8 as tax, i.total::float8 as total, i.status, \
         i.issue_date::text as issue_date, i.due_date::text as due_date, \
         i.paid_date::text as paid_date, i.notes, i.created_at::text as created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE i.due_date BETWEEN CURRENT_DATE AND (CURRENT_DATE + INTERVAL '7 days') \
         AND i.status IN ('draft', 'sent')"
    ).fetch_all(pool).await?;

    for inv in &soon_invoices {
        let msg = format!("A fatura {} de R$ {:.2} vence em breve!", inv.number, inv.total);
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE type = 'invoice_due' AND message LIKE $1 AND created_at::date = CURRENT_DATE"
        ).bind(format!("%{}%", inv.number)).fetch_one(pool).await?;
        if exists.0 == 0 {
            created.push(create_notification(pool, "invoice_due", "💰 Fatura Próxima do Vencimento!", &msg).await?);
        }
    }

    Ok(created)
}

#[cfg(target_os = "windows")]
pub async fn auto_generate_notifications(pool: &DbPool) -> Result<Vec<Notification>, sqlx::Error> {
    let mut created = Vec::new();

    // Atividades com prazo hoje
    let today_assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date, due_time, \
         notification_time, project_name, status, priority, \
         created_at FROM assignments \
         WHERE due_date = date('now') AND status = 'pending'"
    ).fetch_all(pool).await?;

    for a in &today_assignments {
        let project = a.project_name.as_deref().unwrap_or("");
        let msg = if project.is_empty() { format!("A atividade '{}' vence hoje!", a.title) }
                   else { format!("A atividade '{}' do projeto '{}' vence hoje!", a.title, project) };
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE type = 'assignment_due' AND message LIKE ?1 AND date(created_at) = date('now')"
        ).bind(format!("%{}%", a.title)).fetch_one(pool).await?;
        if exists.0 == 0 {
            created.push(create_notification(pool, "assignment_due", "📅 Prazo Hoje!", &msg).await?);
        }
    }

    // Atividades atrasadas
    let overdue_assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date, due_time, \
         notification_time, project_name, status, priority, \
         created_at FROM assignments \
         WHERE due_date < date('now') AND status = 'pending'"
    ).fetch_all(pool).await?;

    for a in &overdue_assignments {
        let project = a.project_name.as_deref().unwrap_or("");
        let msg = if project.is_empty() { format!("A atividade '{}' está atrasada!", a.title) }
                   else { format!("A atividade '{}' do projeto '{}' está atrasada!", a.title, project) };
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE type = 'assignment_overdue' AND message LIKE ?1 AND date(created_at) = date('now')"
        ).bind(format!("%{}%", a.title)).fetch_one(pool).await?;
        if exists.0 == 0 {
            created.push(create_notification(pool, "assignment_overdue", "🔴 Atividade Atrasada!", &msg).await?);
        }
    }

    // Faturas próximas do vencimento (7 dias)
    let soon_invoices: Vec<Invoice> = sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, COALESCE(p.name, '') as project_name, i.client_id, COALESCE(c.name, '') as client_name, \
         i.number, i.description, i.amount, i.tax, i.total, i.status, \
         i.issue_date, i.due_date, \
         i.paid_date, i.notes, i.created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE i.due_date BETWEEN date('now') AND date('now', '+7 days') \
         AND i.status IN ('draft', 'sent')"
    ).fetch_all(pool).await?;

    for inv in &soon_invoices {
        let msg = format!("A fatura {} de R$ {:.2} vence em breve!", inv.number, inv.total);
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE type = 'invoice_due' AND message LIKE ?1 AND date(created_at) = date('now')"
        ).bind(format!("%{}%", inv.number)).fetch_one(pool).await?;
        if exists.0 == 0 {
            created.push(create_notification(pool, "invoice_due", "💰 Fatura Próxima do Vencimento!", &msg).await?);
        }
    }

    Ok(created)
}

// =============================================================================
//  DASHBOARD
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_dashboard_stats(pool: &DbPool) -> Result<DashboardStats, sqlx::Error> {
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects").fetch_one(pool).await?;
    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles").fetch_one(pool).await?;
    let total_clients: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clients").fetch_one(pool).await?;
    let total_teams: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams").fetch_one(pool).await?;
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;

    let pending_assignments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'pending' AND due_date >= CURRENT_DATE"
    ).fetch_one(pool).await?;

    let overdue_assignments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status IN ('pending', 'overdue') AND due_date < CURRENT_DATE"
    ).fetch_one(pool).await?;

    let next_deadline_row: Option<(String, String)> = sqlx::query_as(
        "SELECT due_date::text, title FROM assignments WHERE status != 'done' AND due_date >= CURRENT_DATE ORDER BY due_date ASC LIMIT 1"
    ).fetch_optional(pool).await?;

    let (hours_today, hours_week) = get_hours_summary(pool).await.unwrap_or((0, 0.0));

    let total_revenue: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;

    let pending_invoices: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    let pending_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    let unread = get_unread_notifications_count(pool).await.unwrap_or(0);

    Ok(DashboardStats {
        total_projects: total_projects.0, total_articles: total_articles.0,
        total_clients: total_clients.0, total_teams: total_teams.0, total_users: total_users.0,
        pending_assignments: pending_assignments.0, overdue_assignments: overdue_assignments.0,
        next_deadline: next_deadline_row.as_ref().map(|(d, _)| d.clone()),
        next_deadline_name: next_deadline_row.map(|(_, n)| n),
        hours_today, hours_week,
        total_revenue: total_revenue.0.unwrap_or(0.0),
        pending_invoices: pending_invoices.0, pending_amount: pending_amount.0.unwrap_or(0.0),
        unread_notifications: unread,
    })
}

#[cfg(target_os = "windows")]
pub async fn get_dashboard_stats(pool: &DbPool) -> Result<DashboardStats, sqlx::Error> {
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects").fetch_one(pool).await?;
    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles").fetch_one(pool).await?;
    let total_clients: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clients").fetch_one(pool).await?;
    let total_teams: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams").fetch_one(pool).await?;
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;

    let pending_assignments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'pending' AND due_date >= date('now')"
    ).fetch_one(pool).await?;

    let overdue_assignments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status IN ('pending', 'overdue') AND due_date < date('now')"
    ).fetch_one(pool).await?;

    let next_deadline_row: Option<(String, String)> = sqlx::query_as(
        "SELECT due_date, title FROM assignments WHERE status != 'done' AND due_date >= date('now') ORDER BY due_date ASC LIMIT 1"
    ).fetch_optional(pool).await?;

    let (hours_today, hours_week) = get_hours_summary(pool).await.unwrap_or((0, 0.0));

    let total_revenue: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;

    let pending_invoices: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    let pending_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    let unread = get_unread_notifications_count(pool).await.unwrap_or(0);

    Ok(DashboardStats {
        total_projects: total_projects.0, total_articles: total_articles.0,
        total_clients: total_clients.0, total_teams: total_teams.0, total_users: total_users.0,
        pending_assignments: pending_assignments.0, overdue_assignments: overdue_assignments.0,
        next_deadline: next_deadline_row.as_ref().map(|(d, _)| d.clone()),
        next_deadline_name: next_deadline_row.map(|(_, n)| n),
        hours_today, hours_week,
        total_revenue: total_revenue.0.unwrap_or(0.0),
        pending_invoices: pending_invoices.0, pending_amount: pending_amount.0.unwrap_or(0.0),
        unread_notifications: unread,
    })
}

// =============================================================================
//  EXPORTAÇÃO ZIP
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn export_project_zip(pool: &DbPool, project_id: i32) -> Result<(Project, Vec<u8>), String> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT p.id, p.name, p.description, p.client_id, c.name as client_name, p.created_at::text as created_at \
         FROM projects p LEFT JOIN clients c ON c.id = p.client_id WHERE p.id = $1"
    )
    .bind(project_id).fetch_optional(pool).await
    .map_err(|_| "Erro ao buscar projeto".to_string())?
    .ok_or_else(|| "Projeto não encontrado".to_string())?;

    let articles: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT title, content FROM articles WHERE project_id = $1"
    ).bind(project_id).fetch_all(pool).await
    .map_err(|_| "Erro ao buscar artigos".to_string())?;

    let files: Vec<(String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE project_id = $1"
    ).bind(project_id).fetch_all(pool).await
    .map_err(|_| "Erro ao buscar arquivos".to_string())?;

    build_zip(project, articles, files)
}

#[cfg(target_os = "windows")]
pub async fn export_project_zip(pool: &DbPool, project_id: i32) -> Result<(Project, Vec<u8>), String> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT p.id, p.name, p.description, p.client_id, c.name as client_name, p.created_at \
         FROM projects p LEFT JOIN clients c ON c.id = p.client_id WHERE p.id = ?1"
    )
    .bind(project_id).fetch_optional(pool).await
    .map_err(|_| "Erro ao buscar projeto".to_string())?
    .ok_or_else(|| "Projeto não encontrado".to_string())?;

    let articles: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT title, content FROM articles WHERE project_id = ?1"
    ).bind(project_id).fetch_all(pool).await
    .map_err(|_| "Erro ao buscar artigos".to_string())?;

    let files: Vec<(String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE project_id = ?1"
    ).bind(project_id).fetch_all(pool).await
    .map_err(|_| "Erro ao buscar arquivos".to_string())?;

    build_zip(project, articles, files)
}

fn build_zip(project: Project, articles: Vec<(String, Option<String>)>, files: Vec<(String, String, Vec<u8>)>) -> Result<(Project, Vec<u8>), String> {
    use std::io::Write;
    let buffer = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buffer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let safe_project_name = sanitize_project_name(&project.name);
    let info_content = format!(
        "Projeto: {}\n\nDescrição: {}\n\n--- Artigos ({}) ---\n",
        safe_project_name,
        project.description.as_deref().unwrap_or("Sem descrição"),
        articles.len()
    );
    zip.start_file("projeto.txt", options).map_err(|_| "Erro ao criar ZIP".to_string())?;
    zip.write_all(info_content.as_bytes()).map_err(|_| "Erro ao escrever no ZIP".to_string())?;

    for (i, (title, content)) in articles.iter().enumerate() {
        let safe_name = sanitize_filename(title);
        let filename = format!("artigos/{:03}_{}.txt", i + 1, safe_name);
        zip.start_file(&filename, options).map_err(|_| "Erro ao criar ZIP".to_string())?;
        zip.write_all(content.as_deref().unwrap_or("Sem conteúdo").as_bytes())
            .map_err(|_| "Erro ao escrever no ZIP".to_string())?;
    }

    for (original_name, _, file_data) in &files {
        let safe_name = sanitize_filename(original_name);
        zip.start_file(&format!("arquivos/{}", safe_name), options)
            .map_err(|_| "Erro ao criar ZIP".to_string())?;
        zip.write_all(file_data).map_err(|_| "Erro ao escrever no ZIP".to_string())?;
    }

    let finished = zip.finish().map_err(|_| "Erro ao finalizar ZIP".to_string())?;
    Ok((project, finished.into_inner()))
}

// =============================================================================
//  RELATÓRIOS
// =============================================================================

#[cfg(target_os = "linux")]
pub async fn get_report_stats(pool: &DbPool) -> Result<ReportStats, sqlx::Error> {
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects").fetch_one(pool).await?;
    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles").fetch_one(pool).await?;
    let total_clients: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clients").fetch_one(pool).await?;
    let total_teams: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams").fetch_one(pool).await?;
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;

    let assignments_by_month: Vec<(String, i64)> = sqlx::query_as(
        "SELECT to_char(due_date, 'YYYY-MM') as month, COUNT(*) as count FROM assignments \
         WHERE due_date >= date_trunc('month', CURRENT_DATE) - INTERVAL '11 months' \
         GROUP BY month ORDER BY month"
    ).fetch_all(pool).await?;

    let assignments_pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'pending'"
    ).fetch_one(pool).await?;
    let assignments_done: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'done'"
    ).fetch_one(pool).await?;
    let assignments_overdue: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'overdue'"
    ).fetch_one(pool).await?;

    let revenue_by_month: Vec<(String, Option<f64>)> = sqlx::query_as(
        "SELECT to_char(paid_date, 'YYYY-MM') as month, COALESCE(SUM(total::float8), 0) as amount FROM invoices \
         WHERE status = 'paid' AND paid_date >= date_trunc('month', CURRENT_DATE) - INTERVAL '11 months' \
         GROUP BY month ORDER BY month"
    ).fetch_all(pool).await?;

    let total_revenue: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;
    let pending_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    let hours_by_project: Vec<(String, Option<f64>)> = sqlx::query_as(
        "SELECT COALESCE(p.name, 'Sem projeto') as project_name, \
         COALESCE(SUM(te.duration_minutes::float8 / 60.0), 0) as hours \
         FROM time_entries te LEFT JOIN projects p ON p.id = te.project_id \
         WHERE te.end_time IS NOT NULL GROUP BY p.name ORDER BY hours DESC"
    ).fetch_all(pool).await?;

    let total_hours: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes::float8 / 60.0), 0) FROM time_entries WHERE end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    let invoices_draft: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'draft'"
    ).fetch_one(pool).await?;
    let invoices_sent: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'sent'"
    ).fetch_one(pool).await?;
    let invoices_paid: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;
    let invoices_overdue: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'overdue'"
    ).fetch_one(pool).await?;
    let invoices_cancelled: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'cancelled'"
    ).fetch_one(pool).await?;

    Ok(ReportStats {
        total_projects: total_projects.0, total_articles: total_articles.0,
        total_clients: total_clients.0, total_teams: total_teams.0, total_users: total_users.0,
        assignments_by_month: assignments_by_month.into_iter()
            .map(|(m, c)| MonthlyCount { month: m, count: c }).collect(),
        assignments_pending: assignments_pending.0, assignments_done: assignments_done.0,
        assignments_overdue: assignments_overdue.0,
        revenue_by_month: revenue_by_month.into_iter()
            .map(|(m, a)| MonthlyAmount { month: m, amount: a.unwrap_or(0.0) }).collect(),
        total_revenue: total_revenue.0.unwrap_or(0.0), pending_amount: pending_amount.0.unwrap_or(0.0),
        hours_by_project: hours_by_project.into_iter()
            .map(|(n, h)| ProjectHours { project_name: n, hours: (h.unwrap_or(0.0) * 100.0).round() / 100.0 }).collect(),
        total_hours: (total_hours.0.unwrap_or(0.0) * 100.0).round() / 100.0,
        invoices_draft: invoices_draft.0, invoices_sent: invoices_sent.0,
        invoices_paid: invoices_paid.0, invoices_overdue: invoices_overdue.0,
        invoices_cancelled: invoices_cancelled.0,
    })
}

#[cfg(target_os = "windows")]
pub async fn get_report_stats(pool: &DbPool) -> Result<ReportStats, sqlx::Error> {
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects").fetch_one(pool).await?;
    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles").fetch_one(pool).await?;
    let total_clients: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clients").fetch_one(pool).await?;
    let total_teams: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams").fetch_one(pool).await?;
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;

    let assignments_by_month: Vec<(String, i64)> = sqlx::query_as(
        "SELECT strftime('%Y-%m', due_date) as month, COUNT(*) as count FROM assignments \
         WHERE due_date >= date('now', '-11 months', 'start of month') \
         GROUP BY month ORDER BY month"
    ).fetch_all(pool).await?;

    let assignments_pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'pending'"
    ).fetch_one(pool).await?;
    let assignments_done: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'done'"
    ).fetch_one(pool).await?;
    let assignments_overdue: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'overdue'"
    ).fetch_one(pool).await?;

    let revenue_by_month: Vec<(String, Option<f64>)> = sqlx::query_as(
        "SELECT strftime('%Y-%m', paid_date) as month, COALESCE(SUM(total), 0) as amount FROM invoices \
         WHERE status = 'paid' AND paid_date >= date('now', '-11 months', 'start of month') \
         GROUP BY month ORDER BY month"
    ).fetch_all(pool).await?;

    let total_revenue: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;
    let pending_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    let hours_by_project: Vec<(String, Option<f64>)> = sqlx::query_as(
        "SELECT COALESCE(p.name, 'Sem projeto') as project_name, \
         COALESCE(CAST(SUM(te.duration_minutes) AS REAL) / 60.0, 0) as hours \
         FROM time_entries te LEFT JOIN projects p ON p.id = te.project_id \
         WHERE te.end_time IS NOT NULL GROUP BY p.name ORDER BY hours DESC"
    ).fetch_all(pool).await?;

    let total_hours: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(CAST(SUM(CAST(duration_minutes AS REAL)) AS REAL) / 60.0, 0) FROM time_entries WHERE end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    let invoices_draft: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'draft'"
    ).fetch_one(pool).await?;
    let invoices_sent: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'sent'"
    ).fetch_one(pool).await?;
    let invoices_paid: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;
    let invoices_overdue: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'overdue'"
    ).fetch_one(pool).await?;
    let invoices_cancelled: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status = 'cancelled'"
    ).fetch_one(pool).await?;

    Ok(ReportStats {
        total_projects: total_projects.0, total_articles: total_articles.0,
        total_clients: total_clients.0, total_teams: total_teams.0, total_users: total_users.0,
        assignments_by_month: assignments_by_month.into_iter()
            .map(|(m, c)| MonthlyCount { month: m, count: c }).collect(),
        assignments_pending: assignments_pending.0, assignments_done: assignments_done.0,
        assignments_overdue: assignments_overdue.0,
        revenue_by_month: revenue_by_month.into_iter()
            .map(|(m, a)| MonthlyAmount { month: m, amount: a.unwrap_or(0.0) }).collect(),
        total_revenue: total_revenue.0.unwrap_or(0.0), pending_amount: pending_amount.0.unwrap_or(0.0),
        hours_by_project: hours_by_project.into_iter()
            .map(|(n, h)| ProjectHours { project_name: n, hours: (h.unwrap_or(0.0) * 100.0).round() / 100.0 }).collect(),
        total_hours: (total_hours.0.unwrap_or(0.0) * 100.0).round() / 100.0,
        invoices_draft: invoices_draft.0, invoices_sent: invoices_sent.0,
        invoices_paid: invoices_paid.0, invoices_overdue: invoices_overdue.0,
        invoices_cancelled: invoices_cancelled.0,
    })
}

// =============================================================================
//  TESTES DE INTEGRAÇÃO (apenas Linux/PostgreSQL)
// =============================================================================

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/unitesk_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("Falha ao conectar ao banco de testes");

        // Limpar dados
        sqlx::query("DELETE FROM notifications").execute(&pool).await.ok();
        sqlx::query("DELETE FROM invoices").execute(&pool).await.ok();
        sqlx::query("DELETE FROM team_members").execute(&pool).await.ok();
        sqlx::query("DELETE FROM teams").execute(&pool).await.ok();
        sqlx::query("DELETE FROM users").execute(&pool).await.ok();
        sqlx::query("DELETE FROM clients").execute(&pool).await.ok();
        sqlx::query("DELETE FROM assignment_files").execute(&pool).await.ok();
        sqlx::query("DELETE FROM project_files").execute(&pool).await.ok();
        sqlx::query("DELETE FROM articles").execute(&pool).await.ok();
        sqlx::query("DELETE FROM assignments").execute(&pool).await.ok();
        sqlx::query("DELETE FROM time_entries").execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects").execute(&pool).await.ok();

        init_db(&database_url).await.expect("Falha ao inicializar banco de testes");
        pool
    }

    // ==================== Testes de Clientes ====================
    #[tokio::test]
    async fn test_create_and_get_clients() {
        let pool = setup_test_db().await;
        let client = create_client(&pool, "Empresa ABC",
            Some("contato@abc.com".to_string()), Some("11999999999".to_string()),
            Some("ABC Ltda".to_string()), Some("Cliente premium".to_string()))
            .await.expect("Falha ao criar cliente");
        assert_eq!(client.name, "Empresa ABC");
        assert_eq!(client.email.as_deref(), Some("contato@abc.com"));

        let clients = get_clients(&pool).await.expect("Falha ao listar clientes");
        assert!(!clients.is_empty());
    }

    #[tokio::test]
    async fn test_delete_client_unlinks_projects() {
        let pool = setup_test_db().await;
        let client = create_client(&pool, "Cliente", None, None, None, None).await.unwrap();
        let project = create_project(&pool, "Projeto", "", Some(client.id)).await.unwrap();
        delete_client(&pool, client.id).await.unwrap();
        let projects = get_projects(&pool).await.unwrap();
        assert_eq!(projects.len(), 1);
        assert!(projects[0].client_id.is_none());
    }

    // ==================== Testes de Usuários ====================
    #[tokio::test]
    async fn test_create_and_get_users() {
        let pool = setup_test_db().await;
        let user = create_user(&pool, "João Silva", "joao@email.com", "admin").await.unwrap();
        assert_eq!(user.name, "João Silva");
        let users = get_users(&pool).await.unwrap();
        assert!(!users.is_empty());
    }

    // ==================== Testes de Equipes ====================
    #[tokio::test]
    async fn test_create_and_get_teams() {
        let pool = setup_test_db().await;
        let team = create_team(&pool, "Equipe Alpha", Some("Time dev".to_string())).await.unwrap();
        assert_eq!(team.name, "Equipe Alpha");
    }

    #[tokio::test]
    async fn test_team_members() {
        let pool = setup_test_db().await;
        let user = create_user(&pool, "Maria", "maria@email.com", "member").await.unwrap();
        let team = create_team(&pool, "Equipe Beta", None).await.unwrap();
        let member = add_team_member(&pool, team.id, user.id, "leader").await.unwrap();
        assert_eq!(member.role, "leader");
        remove_team_member(&pool, member.id).await.unwrap();
        let members_after = get_team_members(&pool, team.id).await.unwrap();
        assert!(members_after.is_empty());
    }

    // ==================== Testes de Projetos com Cliente ====================
    #[tokio::test]
    async fn test_create_project_with_client() {
        let pool = setup_test_db().await;
        let client = create_client(&pool, "Cliente Proj", None, None, None, None).await.unwrap();
        let project = create_project(&pool, "Projeto", "Desc", Some(client.id)).await.unwrap();
        assert_eq!(project.client_id, Some(client.id));
    }

    // ==================== Testes de Atividades ====================
    #[tokio::test]
    async fn test_create_assignment_with_priority() {
        let pool = setup_test_db().await;
        let a = create_assignment(&pool, "Urgente!", "Alta", "2026-12-31", None, "", "high").await.unwrap();
        assert_eq!(a.priority, "high");
    }

    // ==================== Testes de Faturas ====================
    #[tokio::test]
    async fn test_create_and_get_invoices() {
        let pool = setup_test_db().await;
        let project = create_project(&pool, "Proj Fatura", "", None).await.unwrap();
        let client = create_client(&pool, "Cli Fatura", None, None, None, None).await.unwrap();
        let inv = create_invoice(&pool, Some(project.id), Some(client.id), "INV-001",
            Some("Serviços".to_string()), 1000.0, 100.0, 1100.0, "draft",
            "2026-07-01", Some("2026-07-31".to_string()), None).await.unwrap();
        assert_eq!(inv.number, "INV-001");
    }

    #[tokio::test]
    async fn test_dashboard_finance_stats() {
        let pool = setup_test_db().await;
        let stats = get_dashboard_stats(&pool).await.unwrap();
        assert_eq!(stats.total_revenue, 0.0);

        create_invoice(&pool, None, None, "INV-001", None, 1000.0, 100.0, 1100.0, "paid",
            "2026-07-01", None, None).await.unwrap();
        create_invoice(&pool, None, None, "INV-002", None, 500.0, 0.0, 500.0, "sent",
            "2026-07-01", Some("2026-07-31".to_string()), None).await.unwrap();

        let stats2 = get_dashboard_stats(&pool).await.unwrap();
        assert_eq!(stats2.total_revenue, 1100.0);
        assert_eq!(stats2.pending_invoices, 1);
    }
}
