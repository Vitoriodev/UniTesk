use std::io::Write;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use serde::{Serialize, Deserialize};

use crate::models::*;

// ===================== Export/Import de Dados =====================

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

pub async fn export_all_data(pool: &PgPool) -> Result<ExportedData, String> {
    let projects = get_projects(pool).await.map_err(|_| "Erro ao exportar projetos".to_string())?;
    let articles = get_articles(pool).await.map_err(|_| "Erro ao exportar artigos".to_string())?;
    let assignments = get_assignments(pool).await.map_err(|_| "Erro ao exportar atividades".to_string())?;

    // Buscar arquivos de projeto com dados binários
    let project_files_raw: Vec<(i32, String, String, i64, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT project_id, original_name, stored_name, file_size, mime_type, created_at::text, file_data FROM project_files"
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Erro ao exportar arquivos de projeto".to_string())?;

    let project_files: Vec<ExportedProjectFile> = project_files_raw
        .into_iter()
        .map(|(pid, on, sn, fs, mt, ca, fd)| ExportedProjectFile {
            project_id: pid,
            original_name: on,
            stored_name: sn,
            file_size: fs,
            mime_type: mt,
            created_at: ca,
            file_data_base64: base64_encode(&fd),
        })
        .collect();

    // Buscar arquivos de atividade com dados binários
    let assignment_files_raw: Vec<(i32, String, String, i64, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT assignment_id, original_name, stored_name, file_size, mime_type, created_at::text, file_data FROM assignment_files"
    )
    .fetch_all(pool)
    .await
    .map_err(|_| "Erro ao exportar arquivos de atividade".to_string())?;

    let assignment_files: Vec<ExportedAssignmentFile> = assignment_files_raw
        .into_iter()
        .map(|(aid, on, sn, fs, mt, ca, fd)| ExportedAssignmentFile {
            assignment_id: aid,
            original_name: on,
            stored_name: sn,
            file_size: fs,
            mime_type: mt,
            created_at: ca,
            file_data_base64: base64_encode(&fd),
        })
        .collect();

    Ok(ExportedData {
        version: "2.0.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        projects,
        articles,
        assignments,
        project_files,
        assignment_files,
    })
}

pub async fn import_all_data(pool: &PgPool, data: &ExportedData) -> Result<String, String> {
    // Importar projetos (mapear IDs antigos para novos)
    let mut project_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for project in &data.projects {
        let new_project = sqlx::query_as::<_, Project>(
            "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id, name, description, NULL::int as client_id, NULL::text as client_name, created_at::text as created_at"
        )
        .bind(&project.name)
        .bind(&project.description)
        .fetch_one(pool)
        .await
        .map_err(|_| "Erro ao importar projeto".to_string())?;
        project_id_map.insert(project.id, new_project.id);
    }

    // Importar artigos
    for article in &data.articles {
        let new_project_id = article.project_id.and_then(|pid| project_id_map.get(&pid).copied());
        sqlx::query(
            "INSERT INTO articles (title, content, project_name, project_id, scheduled_date) VALUES ($1, $2, $3, $4, $5::date)"
        )
        .bind(&article.title)
        .bind(&article.content)
        .bind(&article.project_name)
        .bind(new_project_id)
        .bind(&article.scheduled_date)
        .execute(pool)
        .await
        .map_err(|_| "Erro ao importar artigo".to_string())?;
    }

    // Importar atividades (mapear IDs antigos para novos)
    let mut assignment_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for assignment in &data.assignments {
        let new_assignment = sqlx::query_as::<_, Assignment>(
            "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, status) VALUES ($1, $2, $3::date, $4::time, $5::time, $6, $7) RETURNING id, title, description, due_date::text as due_date, due_time::text as due_time, notification_time::text as notification_time, project_name, status, 'medium' as priority, created_at::text as created_at"
        )
        .bind(&assignment.title)
        .bind(&assignment.description)
        .bind(&assignment.due_date)
        .bind(&assignment.due_time)
        .bind(&assignment.notification_time)
        .bind(&assignment.project_name)
        .bind(&assignment.status)
        .fetch_one(pool)
        .await
        .map_err(|_| "Erro ao importar atividade".to_string())?;
        assignment_id_map.insert(assignment.id, new_assignment.id);
    }

    // Importar arquivos de projeto
    for pf in &data.project_files {
        if let Some(&new_pid) = project_id_map.get(&pf.project_id) {
            let file_data = base64_decode(&pf.file_data_base64)?;
            add_project_file(pool, new_pid, &pf.original_name, &pf.stored_name, &file_data, &pf.mime_type)
                .await
                .map_err(|_| "Erro ao importar arquivo de projeto".to_string())?;
        }
    }

    // Importar arquivos de atividade
    for af in &data.assignment_files {
        if let Some(&new_aid) = assignment_id_map.get(&af.assignment_id) {
            let file_data = base64_decode(&af.file_data_base64)?;
            add_assignment_file(pool, new_aid, &af.original_name, &af.stored_name, &file_data, &af.mime_type)
                .await
                .map_err(|_| "Erro ao importar arquivo de atividade".to_string())?;
        }
    }

    let summary = format!(
        "Importação concluída: {} projetos, {} artigos, {} atividades, {} arquivos de projeto, {} arquivos de atividade",
        data.projects.len(),
        data.articles.len(),
        data.assignments.len(),
        data.project_files.len(),
        data.assignment_files.len(),
    );

    Ok(summary)
}

fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ' { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
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

/// Inicializa a conexão com o banco PostgreSQL
pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // ===================== Tabelas Existentes =====================

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id SERIAL PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            content TEXT,
            project_name VARCHAR(255),
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS assignments (
            id SERIAL PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            due_date DATE NOT NULL,
            due_time TIME,
            notification_time TIME,
            project_name VARCHAR(255),
            status VARCHAR(20) DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS project_files (
            id SERIAL PRIMARY KEY,
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            original_name VARCHAR(500) NOT NULL,
            stored_name VARCHAR(500) NOT NULL,
            file_data BYTEA NOT NULL,
            file_size BIGINT NOT NULL,
            mime_type VARCHAR(100) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS assignment_files (
            id SERIAL PRIMARY KEY,
            assignment_id INTEGER NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
            original_name VARCHAR(500) NOT NULL,
            stored_name VARCHAR(500) NOT NULL,
            file_data BYTEA NOT NULL,
            file_size BIGINT NOT NULL,
            mime_type VARCHAR(100) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // ===================== Migrações Existentes =====================

    sqlx::query("ALTER TABLE assignments ADD COLUMN IF NOT EXISTS due_time TIME")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE assignments ADD COLUMN IF NOT EXISTS notification_time TIME")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE articles ADD COLUMN IF NOT EXISTS scheduled_date DATE")
        .execute(&pool)
        .await?;

    // ===================== Novas Tabelas Empresariais =====================

    // Clientes
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS clients (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            phone VARCHAR(50),
            company VARCHAR(255),
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Usuários
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            role VARCHAR(50) DEFAULT 'member',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Equipes
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS teams (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Membros da equipe
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS team_members (
            id SERIAL PRIMARY KEY,
            team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role VARCHAR(50) DEFAULT 'member',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(team_id, user_id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // ===================== Migrações Empresariais =====================

    // Adicionar client_id aos projetos
    sqlx::query(
        "ALTER TABLE projects ADD COLUMN IF NOT EXISTS client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL"
    )
    .execute(&pool)
    .await?;

    // Adicionar prioridade às atividades
    sqlx::query(
        "ALTER TABLE assignments ADD COLUMN IF NOT EXISTS priority VARCHAR(20) DEFAULT 'medium'"
    )
    .execute(&pool)
    .await?;

    // ===================== Tabela de Registro de Horas =====================

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS time_entries (
            id SERIAL PRIMARY KEY,
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            description TEXT,
            start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            end_time TIMESTAMP,
            duration_minutes INTEGER,
            billable BOOLEAN DEFAULT true,
            hourly_rate DECIMAL(10,2),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // ===================== Tabela de Faturas =====================

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invoices (
            id SERIAL PRIMARY KEY,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            number VARCHAR(50) UNIQUE NOT NULL,
            description TEXT,
            amount DECIMAL(12,2) NOT NULL,
            tax DECIMAL(12,2) DEFAULT 0,
            total DECIMAL(12,2) NOT NULL,
            status VARCHAR(20) DEFAULT 'draft',
            issue_date DATE NOT NULL,
            due_date DATE,
            paid_date DATE,
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // ===================== Índices =====================

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_assignments_due_date ON assignments(due_date)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_assignments_status ON assignments(status)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_articles_project_id ON articles(project_id)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_files_project_id ON project_files(project_id)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_client_id ON projects(client_id)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_time_entries_project_id ON time_entries(project_id)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_time_entries_start_time ON time_entries(start_time)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_invoices_issue_date ON invoices(issue_date)")
        .execute(&pool).await?;

    // ===================== Tabela de Notificações =====================

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notifications (
            id SERIAL PRIMARY KEY,
            type VARCHAR(50) NOT NULL,
            title VARCHAR(255) NOT NULL,
            message TEXT NOT NULL,
            is_read BOOLEAN DEFAULT false,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read)")
        .execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at)")
        .execute(&pool).await?;

    Ok(pool)
}

// ===================== Projetos =====================

pub async fn get_projects(pool: &PgPool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT p.id, p.name, p.description, p.client_id, c.name as client_name, p.created_at::text as created_at \
         FROM projects p \
         LEFT JOIN clients c ON c.id = p.client_id \
         ORDER BY p.created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_project(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_project(
    pool: &PgPool,
    id: i32,
    name: &str,
    description: &str,
    client_id: Option<i32>,
) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = $1, description = $2, client_id = $3 WHERE id = $4 \
         RETURNING id, name, description, client_id, \
         (SELECT name FROM clients WHERE id = $3) as client_name, \
         created_at::text as created_at"
    )
    .bind(name)
    .bind(description)
    .bind(client_id)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_project(
    pool: &PgPool,
    name: &str,
    description: &str,
    client_id: Option<i32>,
) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description, client_id) VALUES ($1, $2, $3) \
         RETURNING id, name, description, client_id, \
         (SELECT name FROM clients WHERE id = $3) as client_name, \
         created_at::text as created_at"
    )
    .bind(name)
    .bind(description)
    .bind(client_id)
    .fetch_one(pool)
    .await
}

// ===================== Arquivos de Projeto =====================

pub async fn get_project_files(pool: &PgPool, project_id: i32) -> Result<Vec<ProjectFile>, sqlx::Error> {
    sqlx::query_as::<_, ProjectFile>(
        "SELECT id, project_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at FROM project_files WHERE project_id = $1 ORDER BY created_at DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn add_project_file(
    pool: &PgPool,
    project_id: i32,
    original_name: &str,
    stored_name: &str,
    file_data: &[u8],
    mime_type: &str,
) -> Result<ProjectFile, sqlx::Error> {
    sqlx::query_as::<_, ProjectFile>(
        "INSERT INTO project_files (project_id, original_name, stored_name, file_data, file_size, mime_type) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, project_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at"
    )
    .bind(project_id)
    .bind(original_name)
    .bind(stored_name)
    .bind(file_data)
    .bind(file_data.len() as i64)
    .bind(mime_type)
    .fetch_one(pool)
    .await
}

pub async fn get_project_file_data(pool: &PgPool, file_id: i32) -> Result<(String, String, Vec<u8>), sqlx::Error> {
    let row: (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_project_file(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM project_files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===================== Artigos =====================

pub async fn get_articles(pool: &PgPool) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "SELECT id, title, content, project_name, project_id, created_at::text as created_at, scheduled_date::text as scheduled_date FROM articles ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn create_article(
    pool: &PgPool,
    title: &str,
    content: &str,
    project_name: &str,
    project_id: Option<i32>,
    scheduled_date: Option<String>,
) -> Result<Article, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "INSERT INTO articles (title, content, project_name, project_id, scheduled_date) VALUES ($1, $2, $3, $4, $5::date) RETURNING id, title, content, project_name, project_id, created_at::text as created_at, scheduled_date::text as scheduled_date"
    )
    .bind(title)
    .bind(content)
    .bind(project_name)
    .bind(project_id)
    .bind(&scheduled_date)
    .fetch_one(pool)
    .await
}

pub async fn delete_article(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM articles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===================== Atividades (Assignments) =====================

pub async fn get_assignments(pool: &PgPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool)
    .await
}

pub async fn create_assignment(
    pool: &PgPool,
    title: &str,
    description: &str,
    due_date: &str,
    due_time: Option<String>,
    project_name: &str,
    priority: &str,
) -> Result<Assignment, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, priority) \
         VALUES ($1, $2, $3::date, $4::time, $5::time, $6, $7) \
         RETURNING id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, created_at::text as created_at"
    )
    .bind(title)
    .bind(description)
    .bind(due_date)
    .bind(&due_time)
    .bind(&due_time)
    .bind(project_name)
    .bind(priority)
    .fetch_one(pool)
    .await
}

pub async fn mark_assignment_done(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE assignments SET status = 'done' WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_assignment(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM assignments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_overdue_assignments(pool: &PgPool) -> Result<Vec<Assignment>, sqlx::Error> {
    // Atualizar status de atividades atrasadas
    sqlx::query(
        "UPDATE assignments SET status = 'overdue' WHERE due_date < CURRENT_DATE AND status = 'pending'"
    )
    .execute(pool)
    .await?;

    // Retornar atividades que vencem hoje ou estão atrasadas
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date <= CURRENT_DATE AND status != 'done' ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool)
    .await
}

// ===================== Arquivos de Atividades (Assignment Files) =====================

pub async fn get_assignment_files(pool: &PgPool, assignment_id: i32) -> Result<Vec<AssignmentFile>, sqlx::Error> {
    sqlx::query_as::<_, AssignmentFile>(
        "SELECT id, assignment_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at FROM assignment_files WHERE assignment_id = $1 ORDER BY created_at DESC"
    )
    .bind(assignment_id)
    .fetch_all(pool)
    .await
}

pub async fn add_assignment_file(
    pool: &PgPool,
    assignment_id: i32,
    original_name: &str,
    stored_name: &str,
    file_data: &[u8],
    mime_type: &str,
) -> Result<AssignmentFile, sqlx::Error> {
    sqlx::query_as::<_, AssignmentFile>(
        "INSERT INTO assignment_files (assignment_id, original_name, stored_name, file_data, file_size, mime_type) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, assignment_id, original_name, stored_name, file_size, mime_type, created_at::text as created_at"
    )
    .bind(assignment_id)
    .bind(original_name)
    .bind(stored_name)
    .bind(file_data)
    .bind(file_data.len() as i64)
    .bind(mime_type)
    .fetch_one(pool)
    .await
}

pub async fn get_assignment_file_data(pool: &PgPool, file_id: i32) -> Result<(String, String, Vec<u8>), sqlx::Error> {
    let row: (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM assignment_files WHERE id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_assignment_file(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM assignment_files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Obter atividades que vencem hoje para notificações
pub async fn get_today_assignments(pool: &PgPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date = CURRENT_DATE AND status = 'pending' \
         AND notification_time BETWEEN CURRENT_TIME - INTERVAL '30 seconds' AND CURRENT_TIME + INTERVAL '30 seconds'"
    )
    .fetch_all(pool)
    .await
}

// ===================== Clientes =====================

pub async fn get_clients(pool: &PgPool) -> Result<Vec<Client>, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "SELECT id, name, email, phone, company, notes, created_at::text as created_at FROM clients ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
}

pub async fn create_client(
    pool: &PgPool,
    name: &str,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
) -> Result<Client, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "INSERT INTO clients (name, email, phone, company, notes) VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, name, email, phone, company, notes, created_at::text as created_at"
    )
    .bind(name)
    .bind(email)
    .bind(phone)
    .bind(company)
    .bind(notes)
    .fetch_one(pool)
    .await
}

pub async fn update_client(
    pool: &PgPool,
    id: i32,
    name: &str,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
) -> Result<Client, sqlx::Error> {
    sqlx::query_as::<_, Client>(
        "UPDATE clients SET name = $1, email = $2, phone = $3, company = $4, notes = $5 WHERE id = $6 \
         RETURNING id, name, email, phone, company, notes, created_at::text as created_at"
    )
    .bind(name)
    .bind(email)
    .bind(phone)
    .bind(company)
    .bind(notes)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_client(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    // Primeiro desvincula projetos deste cliente (SET NULL)
    sqlx::query("UPDATE projects SET client_id = NULL WHERE client_id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===================== Usuários =====================

pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, created_at::text as created_at FROM users ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
}

pub async fn create_user(
    pool: &PgPool,
    name: &str,
    email: &str,
    role: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, role) VALUES ($1, $2, $3) \
         RETURNING id, name, email, role, created_at::text as created_at"
    )
    .bind(name)
    .bind(email)
    .bind(role)
    .fetch_one(pool)
    .await
}

pub async fn update_user(
    pool: &PgPool,
    id: i32,
    name: &str,
    email: &str,
    role: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE users SET name = $1, email = $2, role = $3 WHERE id = $4 \
         RETURNING id, name, email, role, created_at::text as created_at"
    )
    .bind(name)
    .bind(email)
    .bind(role)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_user(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    // Remove membro de equipes primeiro
    sqlx::query("DELETE FROM team_members WHERE user_id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===================== Equipes =====================

pub async fn get_teams(pool: &PgPool) -> Result<Vec<Team>, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "SELECT id, name, description, created_at::text as created_at FROM teams ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
}

pub async fn create_team(
    pool: &PgPool,
    name: &str,
    description: Option<String>,
) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "INSERT INTO teams (name, description) VALUES ($1, $2) \
         RETURNING id, name, description, created_at::text as created_at"
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await
}

pub async fn update_team(
    pool: &PgPool,
    id: i32,
    name: &str,
    description: Option<String>,
) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "UPDATE teams SET name = $1, description = $2 WHERE id = $3 \
         RETURNING id, name, description, created_at::text as created_at"
    )
    .bind(name)
    .bind(description)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_team(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    // Membros são deletados em cascata (ON DELETE CASCADE)
    sqlx::query("DELETE FROM teams WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===================== Membros da Equipe =====================

pub async fn get_team_members(pool: &PgPool, team_id: i32) -> Result<Vec<TeamMember>, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "SELECT tm.id, tm.team_id, tm.user_id, u.name as user_name, u.email as user_email, \
         tm.role, tm.created_at::text as created_at \
         FROM team_members tm \
         JOIN users u ON u.id = tm.user_id \
         WHERE tm.team_id = $1 ORDER BY u.name ASC"
    )
    .bind(team_id)
    .fetch_all(pool)
    .await
}

pub async fn add_team_member(
    pool: &PgPool,
    team_id: i32,
    user_id: i32,
    role: &str,
) -> Result<TeamMember, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3) \
         RETURNING id, team_id, user_id, \
         (SELECT name FROM users WHERE id = $2) as user_name, \
         (SELECT email FROM users WHERE id = $2) as user_email, \
         role, created_at::text as created_at"
    )
    .bind(team_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await
}

pub async fn remove_team_member(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM team_members WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_team_member_role(
    pool: &PgPool,
    id: i32,
    role: &str,
) -> Result<TeamMember, sqlx::Error> {
    sqlx::query_as::<_, TeamMember>(
        "UPDATE team_members SET role = $1 WHERE id = $2 \
         RETURNING id, team_id, user_id, \
         (SELECT name FROM users WHERE id = user_id) as user_name, \
         (SELECT email FROM users WHERE id = user_id) as user_email, \
         role, created_at::text as created_at"
    )
    .bind(role)
    .bind(id)
    .fetch_one(pool)
    .await
}

// ===================== Registro de Horas =====================

pub async fn get_time_entries(
    pool: &PgPool,
    project_id: Option<i32>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<Vec<TimeEntry>, sqlx::Error> {
    // Usamos COALESCE para condições opcionais de forma segura (sem SQL injection)
    // pois todos os valores são passados como bind parameters, nunca concatenados
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
    .bind(project_id)
    .bind(&date_from)
    .bind(&date_to)
    .fetch_all(pool)
    .await
}

pub async fn start_time_entry(
    pool: &PgPool,
    project_id: i32,
    user_id: Option<i32>,
    description: Option<String>,
) -> Result<TimeEntry, sqlx::Error> {
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
    .bind(project_id)
    .bind(user_id)
    .bind(description)
    .fetch_one(pool)
    .await
}

pub async fn stop_time_entry(
    pool: &PgPool,
    id: i32,
) -> Result<TimeEntry, sqlx::Error> {
    // Calcular duração em minutos e atualizar
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
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn add_manual_time_entry(
    pool: &PgPool,
    project_id: i32,
    user_id: Option<i32>,
    description: Option<String>,
    duration_minutes: i32,
    entry_date: String,
    billable: bool,
    hourly_rate: Option<f64>,
) -> Result<TimeEntry, sqlx::Error> {
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
    .bind(project_id)
    .bind(user_id)
    .bind(description)
    .bind(&entry_date)
    .bind(duration_minutes)
    .bind(billable)
    .bind(hourly_rate)
    .fetch_one(pool)
    .await
}

pub async fn delete_time_entry(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM time_entries WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_active_time_entry(pool: &PgPool) -> Result<Option<TimeEntry>, sqlx::Error> {
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
    .fetch_optional(pool)
    .await
}

pub async fn get_hours_summary(pool: &PgPool) -> Result<(i64, f64), sqlx::Error> {
    // Horas do dia
    let today_minutes: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_entries \
         WHERE start_time::date = CURRENT_DATE AND end_time IS NOT NULL"
    )
    .fetch_one(pool)
    .await?;

    // Horas da semana
    let week_minutes: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes::float), 0) FROM time_entries \
         WHERE start_time >= date_trunc('week', CURRENT_DATE) \
         AND end_time IS NOT NULL"
    )
    .fetch_one(pool)
    .await?;

    Ok((today_minutes.0.unwrap_or(0), week_minutes.0.unwrap_or(0.0)))
}

// ===================== Faturas / Financeiro =====================

pub async fn get_invoices(
    pool: &PgPool,
    status_filter: Option<String>,
) -> Result<Vec<Invoice>, sqlx::Error> {
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
    )
    .bind(&status_filter)
    .fetch_all(pool)
    .await
}

pub async fn get_invoice(pool: &PgPool, id: i32) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, p.name as project_name, i.client_id, c.name as client_name, \
         i.number, i.description, i.amount::float8 as amount, i.tax::float8 as tax, i.total::float8 as total, i.status, \
         i.issue_date::text as issue_date, i.due_date::text as due_date, \
         i.paid_date::text as paid_date, i.notes, i.created_at::text as created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE i.id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_invoice(
    pool: &PgPool,
    project_id: Option<i32>,
    client_id: Option<i32>,
    number: &str,
    description: Option<String>,
    amount: f64,
    tax: f64,
    total: f64,
    status: &str,
    issue_date: &str,
    due_date: Option<String>,
    notes: Option<String>,
) -> Result<Invoice, sqlx::Error> {
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
    .bind(project_id)
    .bind(client_id)
    .bind(number)
    .bind(description)
    .bind(amount)
    .bind(tax)
    .bind(total)
    .bind(status)
    .bind(issue_date)
    .bind(&due_date)
    .bind(notes)
    .fetch_one(pool)
    .await
}

pub async fn update_invoice(
    pool: &PgPool,
    id: i32,
    project_id: Option<i32>,
    client_id: Option<i32>,
    number: &str,
    description: Option<String>,
    amount: f64,
    tax: f64,
    total: f64,
    status: &str,
    issue_date: &str,
    due_date: Option<String>,
    paid_date: Option<String>,
    notes: Option<String>,
) -> Result<Invoice, sqlx::Error> {
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
    .bind(project_id)
    .bind(client_id)
    .bind(number)
    .bind(description)
    .bind(amount)
    .bind(tax)
    .bind(total)
    .bind(status)
    .bind(issue_date)
    .bind(&due_date)
    .bind(&paid_date)
    .bind(notes)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_invoice(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invoices WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===================== Notificações =====================

pub async fn create_notification(
    pool: &PgPool,
    notif_type: &str,
    title: &str,
    message: &str,
) -> Result<Notification, sqlx::Error> {
    sqlx::query_as::<_, Notification>(
        "INSERT INTO notifications (type, title, message) VALUES ($1, $2, $3) \
         RETURNING id, type, title, message, is_read, created_at::text as created_at"
    )
    .bind(notif_type)
    .bind(title)
    .bind(message)
    .fetch_one(pool)
    .await
}

pub async fn get_notifications(
    pool: &PgPool,
    unread_only: bool,
    limit: i64,
) -> Result<Vec<Notification>, sqlx::Error> {
    if unread_only {
        sqlx::query_as::<_, Notification>(
            "SELECT id, type, title, message, is_read, created_at::text as created_at \
             FROM notifications WHERE is_read = false \
             ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Notification>(
            "SELECT id, type, title, message, is_read, created_at::text as created_at \
             FROM notifications ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

pub async fn get_unread_notifications_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE is_read = false"
    )
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}

pub async fn mark_notification_read(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_all_notifications_read(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE is_read = false")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_notification(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notifications WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Remove notificações mais antigas que o número de dias especificado.
/// Retorna a quantidade de notificações removidas.
pub async fn cleanup_old_notifications(pool: &PgPool, days: i32) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM notifications WHERE created_at < CURRENT_DATE - ($1 || ' days')::INTERVAL"
    )
    .bind(days)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() as i64)
}

/// Gera notificações automáticas para:
/// - Atividades com prazo hoje
/// - Atividades atrasadas
/// - Faturas próximas do vencimento
pub async fn auto_generate_notifications(pool: &PgPool) -> Result<Vec<Notification>, sqlx::Error> {
    let mut created = Vec::new();

    // Notificações de atividades com prazo hoje
    let today_assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date = CURRENT_DATE AND status = 'pending'"
    )
    .fetch_all(pool)
    .await?;

    for a in &today_assignments {
        let project = a.project_name.as_deref().unwrap_or("");
        let msg = if project.is_empty() {
            format!("A atividade '{}' vence hoje!", a.title)
        } else {
            format!("A atividade '{}' do projeto '{}' vence hoje!", a.title, project)
        };
        // Evitar duplicatas — verificar se já existe notificação similar hoje
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications \
             WHERE type = 'assignment_due' AND message LIKE $1 \
             AND created_at::date = CURRENT_DATE"
        )
        .bind(format!("%{}%", a.title))
        .fetch_one(pool)
        .await?;

        if exists.0 == 0 {
            let n = create_notification(pool, "assignment_due", "📅 Prazo Hoje!", &msg).await?;
            created.push(n);
        }
    }

    // Notificações de atividades atrasadas
    let overdue_assignments: Vec<Assignment> = sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, \
         notification_time::text as notification_time, project_name, status, priority, \
         created_at::text as created_at FROM assignments \
         WHERE due_date < CURRENT_DATE AND status = 'pending'"
    )
    .fetch_all(pool)
    .await?;

    for a in &overdue_assignments {
        let project = a.project_name.as_deref().unwrap_or("");
        let msg = if project.is_empty() {
            format!("A atividade '{}' está atrasada!", a.title)
        } else {
            format!("A atividade '{}' do projeto '{}' está atrasada!", a.title, project)
        };
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications \
             WHERE type = 'assignment_overdue' AND message LIKE $1 \
             AND created_at::date = CURRENT_DATE"
        )
        .bind(format!("%{}%", a.title))
        .fetch_one(pool)
        .await?;

        if exists.0 == 0 {
            let n = create_notification(pool, "assignment_overdue", "🔴 Atividade Atrasada!", &msg).await?;
            created.push(n);
        }
    }

    // Notificações de faturas próximas do vencimento (7 dias)
    let soon_invoices: Vec<Invoice> = sqlx::query_as::<_, Invoice>(
        "SELECT i.id, i.project_id, p.name as project_name, i.client_id, c.name as client_name, \
         i.number, i.description, i.amount::float8 as amount, i.tax::float8 as tax, \
         i.total::float8 as total, i.status, \
         i.issue_date::text as issue_date, i.due_date::text as due_date, \
         i.paid_date::text as paid_date, i.notes, i.created_at::text as created_at \
         FROM invoices i \
         LEFT JOIN projects p ON p.id = i.project_id \
         LEFT JOIN clients c ON c.id = i.client_id \
         WHERE i.due_date BETWEEN CURRENT_DATE AND (CURRENT_DATE + INTERVAL '7 days') \
         AND i.status IN ('draft', 'sent')"
    )
    .fetch_all(pool)
    .await?;

    for inv in &soon_invoices {
        let msg = format!(
            "A fatura {} de R$ {:.2} vence em breve!",
            inv.number, inv.total
        );
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications \
             WHERE type = 'invoice_due' AND message LIKE $1 \
             AND created_at::date = CURRENT_DATE"
        )
        .bind(format!("%{}%", inv.number))
        .fetch_one(pool)
        .await?;

        if exists.0 == 0 {
            let n = create_notification(pool, "invoice_due", "💰 Fatura Próxima do Vencimento!", &msg).await?;
            created.push(n);
        }
    }

    Ok(created)
}

// ===================== Dashboard =====================

pub async fn get_dashboard_stats(pool: &PgPool) -> Result<DashboardStats, sqlx::Error> {
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM projects")
        .fetch_one(pool)
        .await?;

    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM articles")
        .fetch_one(pool)
        .await?;

    let total_clients: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM clients")
        .fetch_one(pool)
        .await?;

    let total_teams: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM teams")
        .fetch_one(pool)
        .await?;

    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;

    let pending_assignments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) as count FROM assignments WHERE status = 'pending' AND due_date >= CURRENT_DATE"
    )
    .fetch_one(pool)
    .await?;

    let overdue_assignments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) as count FROM assignments WHERE status IN ('pending', 'overdue') AND due_date < CURRENT_DATE"
    )
    .fetch_one(pool)
    .await?;

    let next_deadline_row: Option<(String, String)> = sqlx::query_as(
        "SELECT due_date::text, title FROM assignments WHERE status != 'done' AND due_date >= CURRENT_DATE ORDER BY due_date ASC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let (hours_today, hours_week) = get_hours_summary(pool).await.unwrap_or((0, 0.0));

    // Dados financeiros (CAST para float8 para compatibilidade com Rust f64)
    let total_revenue: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status = 'paid'"
    )
    .fetch_one(pool)
    .await?;

    let pending_invoices: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE status IN ('draft', 'sent')"
    )
    .fetch_one(pool)
    .await?;

    let pending_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status IN ('draft', 'sent')"
    )
    .fetch_one(pool)
    .await?;

    let unread_notifications = get_unread_notifications_count(pool).await.unwrap_or(0);

    Ok(DashboardStats {
        total_projects: total_projects.0,
        total_articles: total_articles.0,
        total_clients: total_clients.0,
        total_teams: total_teams.0,
        total_users: total_users.0,
        pending_assignments: pending_assignments.0,
        overdue_assignments: overdue_assignments.0,
        next_deadline: next_deadline_row.as_ref().map(|(d, _)| d.clone()),
        next_deadline_name: next_deadline_row.map(|(_, n)| n),
        hours_today,
        hours_week,
        total_revenue: total_revenue.0.unwrap_or(0.0),
        pending_invoices: pending_invoices.0,
        pending_amount: pending_amount.0.unwrap_or(0.0),
        unread_notifications,
    })
}

// ===================== Exportação ZIP =====================

pub async fn export_project_zip(pool: &PgPool, project_id: i32) -> Result<(Project, Vec<u8>), String> {

    // Buscar dados do projeto
    let project = sqlx::query_as::<_, Project>(
        "SELECT p.id, p.name, p.description, p.client_id, c.name as client_name, p.created_at::text as created_at \
         FROM projects p LEFT JOIN clients c ON c.id = p.client_id WHERE p.id = $1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| "Erro ao buscar projeto".to_string())?
    .ok_or_else(|| "Projeto não encontrado".to_string())?;

    // Buscar artigos do projeto
    let articles: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT title, content FROM articles WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|_| "Erro ao buscar artigos".to_string())?;

    // Buscar arquivos do projeto (com dados)
    let files: Vec<(String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|_| "Erro ao buscar arquivos".to_string())?;

    // Criar ZIP em memória
    let buffer = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buffer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. Adicionar arquivo de informações do projeto (nome sanitizado)
    let safe_project_name = sanitize_project_name(&project.name);
    let client_info = project.client_name.as_deref()
        .map(|cn| format!("\nCliente: {}", cn))
        .unwrap_or_default();
    let info_content = format!(
        "Projeto: {}\n\nDescrição: {}\n\nData de Criação: {}{}\n\n--- Artigos ({}) ---\n",
        safe_project_name,
        project.description.as_deref().unwrap_or("Sem descrição"),
        project.created_at,
        client_info,
        articles.len()
    );

    zip.start_file("projeto.txt", options)
        .map_err(|_| "Erro ao criar ZIP".to_string())?;
    zip.write_all(info_content.as_bytes())
        .map_err(|_| "Erro ao escrever arquivo no ZIP".to_string())?;

    // 2. Adicionar artigos como arquivos .txt
    for (i, (title, content)) in articles.iter().enumerate() {
        let safe_name = sanitize_filename(title);
        let filename = format!("artigos/{:03}_{}.txt", i + 1, safe_name);
        let content = content.as_deref().unwrap_or("Sem conteúdo");

        zip.start_file(&filename, options)
            .map_err(|_| "Erro ao criar arquivo no ZIP".to_string())?;
        zip.write_all(content.as_bytes())
            .map_err(|_| "Erro ao escrever conteúdo no ZIP".to_string())?;
    }

    // 3. Adicionar arquivos anexados (com nome sanitizado)
    for (original_name, _, file_data) in &files {
        let safe_name = sanitize_filename(original_name);
        let filename = format!("arquivos/{}", safe_name);

        zip.start_file(&filename, options)
            .map_err(|_| "Erro ao criar arquivo no ZIP".to_string())?;
        zip.write_all(file_data)
            .map_err(|_| "Erro ao escrever arquivo no ZIP".to_string())?;
    }

    // Finalizar ZIP
    let finished = zip.finish()
        .map_err(|_| "Erro ao finalizar ZIP".to_string())?;

    Ok((project, finished.into_inner()))
}

// ===================== Relatórios =====================

pub async fn get_report_stats(pool: &PgPool) -> Result<ReportStats, sqlx::Error> {
    // Totais gerais
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects")
        .fetch_one(pool).await?;
    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles")
        .fetch_one(pool).await?;
    let total_clients: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clients")
        .fetch_one(pool).await?;
    let total_teams: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams")
        .fetch_one(pool).await?;
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool).await?;

    // Atividades por mês (últimos 12 meses)
    let assignments_by_month: Vec<MonthlyCount> = sqlx::query_as::<_, (String, i64)>(
        "SELECT to_char(due_date, 'YYYY-MM') as month, COUNT(*) as count \
         FROM assignments \
         WHERE due_date >= date_trunc('month', CURRENT_DATE) - INTERVAL '11 months' \
         GROUP BY month ORDER BY month"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(m, c)| MonthlyCount { month: m, count: c })
    .collect();

    // Atividades por status
    let assignments_pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'pending'"
    ).fetch_one(pool).await?;
    let assignments_done: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'done'"
    ).fetch_one(pool).await?;
    let assignments_overdue: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM assignments WHERE status = 'overdue'"
    ).fetch_one(pool).await?;

    // Receita por mês (últimos 12 meses, faturas pagas)
    let revenue_by_month: Vec<MonthlyAmount> = sqlx::query_as::<_, (String, Option<f64>)>(
        "SELECT to_char(paid_date, 'YYYY-MM') as month, COALESCE(SUM(total::float8), 0) as amount \
         FROM invoices \
         WHERE status = 'paid' AND paid_date >= date_trunc('month', CURRENT_DATE) - INTERVAL '11 months' \
         GROUP BY month ORDER BY month"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(m, a)| MonthlyAmount { month: m, amount: a.unwrap_or(0.0) })
    .collect();

    // Totais financeiros
    let total_revenue: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status = 'paid'"
    ).fetch_one(pool).await?;
    let pending_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total::float8), 0) FROM invoices WHERE status IN ('draft', 'sent')"
    ).fetch_one(pool).await?;

    // Horas por projeto
    let hours_by_project: Vec<ProjectHours> = sqlx::query_as::<_, (String, Option<f64>)>(
        "SELECT COALESCE(p.name, 'Sem projeto') as project_name, \
         COALESCE(SUM(te.duration_minutes::float8 / 60.0), 0) as hours \
         FROM time_entries te \
         LEFT JOIN projects p ON p.id = te.project_id \
         WHERE te.end_time IS NOT NULL \
         GROUP BY p.name ORDER BY hours DESC"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(n, h)| ProjectHours { project_name: n, hours: (h.unwrap_or(0.0) * 100.0).round() / 100.0 })
    .collect();

    // Total de horas
    let total_hours: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_minutes::float8 / 60.0), 0) FROM time_entries WHERE end_time IS NOT NULL"
    ).fetch_one(pool).await?;

    // Faturas por status
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
        total_projects: total_projects.0,
        total_articles: total_articles.0,
        total_clients: total_clients.0,
        total_teams: total_teams.0,
        total_users: total_users.0,
        assignments_by_month,
        assignments_pending: assignments_pending.0,
        assignments_done: assignments_done.0,
        assignments_overdue: assignments_overdue.0,
        revenue_by_month,
        total_revenue: total_revenue.0.unwrap_or(0.0),
        pending_amount: pending_amount.0.unwrap_or(0.0),
        hours_by_project,
        total_hours: (total_hours.0.unwrap_or(0.0) * 100.0).round() / 100.0,
        invoices_draft: invoices_draft.0,
        invoices_sent: invoices_sent.0,
        invoices_paid: invoices_paid.0,
        invoices_overdue: invoices_overdue.0,
        invoices_cancelled: invoices_cancelled.0,
    })
}

// ===================== Testes de Integração =====================
//
// ⚠️  IMPORTANTE: Estes testes compartilham um ÚNICO banco de dados.
//     Para evitar interferência entre testes, execute com:
//        cargo test -- --test-threads=1
//
//     Configure o banco de testes via env var:
//        export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/unitesk_test"

#[cfg(test)]
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
            .expect("Falha ao conectar ao banco de testes. Configure TEST_DATABASE_URL ou crie o banco 'unitesk_test'");

        // Limpar dados existentes (respeitando chaves estrangeiras)
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
        sqlx::query("DELETE FROM projects").execute(&pool).await.ok();

        // Garantir que as tabelas existem
        init_db(&database_url).await.expect("Falha ao inicializar banco de testes");

        pool
    }

    // ==================== Testes de Clientes ====================

    #[tokio::test]
    async fn test_create_and_get_clients() {
        let pool = setup_test_db().await;

        let client = create_client(&pool, "Empresa ABC", Some("contato@abc.com".to_string()), Some("11999999999".to_string()), Some("ABC Ltda".to_string()), Some("Cliente premium".to_string()))
            .await
            .expect("Falha ao criar cliente");
        assert_eq!(client.name, "Empresa ABC");
        assert_eq!(client.email.as_deref(), Some("contato@abc.com"));

        let clients = get_clients(&pool).await.expect("Falha ao listar clientes");
        assert!(!clients.is_empty());
        assert_eq!(clients[0].name, "Empresa ABC");
    }

    #[tokio::test]
    async fn test_update_client() {
        let pool = setup_test_db().await;

        let client = create_client(&pool, "Nome Antigo", None, None, None, None)
            .await
            .expect("Falha ao criar cliente");

        let updated = update_client(&pool, client.id, "Nome Novo", Some("novo@email.com".to_string()), None, None, None)
            .await
            .expect("Falha ao atualizar cliente");

        assert_eq!(updated.name, "Nome Novo");
        assert_eq!(updated.email.as_deref(), Some("novo@email.com"));
    }

    #[tokio::test]
    async fn test_delete_client() {
        let pool = setup_test_db().await;

        let client = create_client(&pool, "Cliente para Deletar", None, None, None, None)
            .await
            .expect("Falha ao criar cliente");

        delete_client(&pool, client.id)
            .await
            .expect("Falha ao deletar cliente");

        let clients = get_clients(&pool).await.expect("Falha ao listar");
        assert!(clients.is_empty());
    }

    // ==================== Testes de Usuários ====================

    #[tokio::test]
    async fn test_create_and_get_users() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "João Silva", "joao@email.com", "admin")
            .await
            .expect("Falha ao criar usuário");
        assert_eq!(user.name, "João Silva");
        assert_eq!(user.role, "admin");

        let users = get_users(&pool).await.expect("Falha ao listar usuários");
        assert!(!users.is_empty());
    }

    #[tokio::test]
    async fn test_create_duplicate_user_email() {
        let pool = setup_test_db().await;

        create_user(&pool, "João", "joao@email.com", "member")
            .await
            .expect("Falha ao criar primeiro usuário");

        let result = create_user(&pool, "João Duplicado", "joao@email.com", "member").await;
        assert!(result.is_err(), "Email duplicado deveria dar erro");
    }

    // ==================== Testes de Equipes ====================

    #[tokio::test]
    async fn test_create_and_get_teams() {
        let pool = setup_test_db().await;

        let team = create_team(&pool, "Equipe Alpha", Some("Time de desenvolvimento".to_string()))
            .await
            .expect("Falha ao criar equipe");
        assert_eq!(team.name, "Equipe Alpha");

        let teams = get_teams(&pool).await.expect("Falha ao listar equipes");
        assert!(!teams.is_empty());
    }

    #[tokio::test]
    async fn test_team_members() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "Maria", "maria@email.com", "member")
            .await
            .expect("Falha ao criar usuário");
        let team = create_team(&pool, "Equipe Beta", None)
            .await
            .expect("Falha ao criar equipe");

        let member = add_team_member(&pool, team.id, user.id, "leader")
            .await
            .expect("Falha ao adicionar membro");
        assert_eq!(member.role, "leader");
        assert_eq!(member.user_name.as_deref(), Some("Maria"));

        let members = get_team_members(&pool, team.id)
            .await
            .expect("Falha ao listar membros");
        assert_eq!(members.len(), 1);

        remove_team_member(&pool, member.id)
            .await
            .expect("Falha ao remover membro");

        let members_after = get_team_members(&pool, team.id)
            .await
            .expect("Falha ao listar após remoção");
        assert!(members_after.is_empty());
    }

    // ==================== Testes de Projetos com Cliente ====================

    #[tokio::test]
    async fn test_create_project_with_client() {
        let pool = setup_test_db().await;

        let client = create_client(&pool, "Cliente Projeto", None, None, None, None)
            .await
            .expect("Falha ao criar cliente");

        let project = create_project(&pool, "Projeto com Cliente", "Descrição", Some(client.id))
            .await
            .expect("Falha ao criar projeto com cliente");
        assert_eq!(project.client_id, Some(client.id));
        assert_eq!(project.client_name.as_deref(), Some("Cliente Projeto"));
    }

    #[tokio::test]
    async fn test_delete_client_unlinks_projects() {
        let pool = setup_test_db().await;

        let client = create_client(&pool, "Cliente", None, None, None, None)
            .await
            .expect("Falha ao criar cliente");

        let project = create_project(&pool, "Projeto", "", Some(client.id))
            .await
            .expect("Falha ao criar projeto");

        // Deletar cliente
        delete_client(&pool, client.id)
            .await
            .expect("Falha ao deletar cliente");

        // Projeto deve continuar existindo com client_id = NULL
        let projects = get_projects(&pool).await.expect("Falha ao listar");
        assert_eq!(projects.len(), 1);
        assert!(projects[0].client_id.is_none(), "client_id deveria ser NULL");
    }

    // ==================== Testes de Atividades com Prioridade ====================

    #[tokio::test]
    async fn test_create_assignment_with_priority() {
        let pool = setup_test_db().await;

        let assignment = create_assignment(&pool, "Urgente!", "Alta prioridade", "2026-12-31", None, "", "high")
            .await
            .expect("Falha ao criar atividade");
        assert_eq!(assignment.priority, "high");

        let assignment2 = create_assignment(&pool, "Normal", "", "2026-12-31", None, "", "low")
            .await
            .expect("Falha ao criar atividade");
        assert_eq!(assignment2.priority, "low");
    }

    // ==================== Testes de Faturas ====================

    #[tokio::test]
    async fn test_create_and_get_invoices() {
        let pool = setup_test_db().await;

        // Criar projeto e cliente para vincular
        let project = create_project(&pool, "Projeto Fatura", "", None)
            .await
            .expect("Falha ao criar projeto");
        let client = create_client(&pool, "Cliente Fatura", None, None, None, None)
            .await
            .expect("Falha ao criar cliente");

        // Criar fatura
        let invoice = create_invoice(
            &pool,
            Some(project.id),
            Some(client.id),
            "INV-001",
            Some("Serviços de consultoria".to_string()),
            1000.0,
            100.0,
            1100.0,
            "draft",
            "2026-07-01",
            Some("2026-07-31".to_string()),
            None,
        )
        .await
        .expect("Falha ao criar fatura");

        assert_eq!(invoice.number, "INV-001");
        assert_eq!(invoice.amount, 1000.0);
        assert_eq!(invoice.tax, 100.0);
        assert_eq!(invoice.total, 1100.0);
        assert_eq!(invoice.status, "draft");
        assert_eq!(invoice.client_name.as_deref(), Some("Cliente Fatura"));

        // Listar faturas
        let invoices = get_invoices(&pool, None)
            .await
            .expect("Falha ao listar faturas");
        assert_eq!(invoices.len(), 1);

        // Listar com filtro de status
        let draft = get_invoices(&pool, Some("draft".to_string()))
            .await
            .expect("Falha ao filtrar");
        assert_eq!(draft.len(), 1);

        let paid = get_invoices(&pool, Some("paid".to_string()))
            .await
            .expect("Falha ao filtrar");
        assert!(paid.is_empty());
    }

    #[tokio::test]
    async fn test_update_invoice_status() {
        let pool = setup_test_db().await;

        let invoice = create_invoice(
            &pool,
            None, None, "INV-002", None, 500.0, 0.0, 500.0, "draft",
            "2026-07-01", None, None,
        )
        .await
        .expect("Falha ao criar fatura");

        // Atualizar para enviada
        let updated = update_invoice(
            &pool,
            invoice.id,
            None, None, "INV-002", None, 500.0, 0.0, 500.0, "sent",
            "2026-07-01", None, None, None,
        )
        .await
        .expect("Falha ao atualizar fatura");

        assert_eq!(updated.status, "sent");

        // Atualizar para paga
        let paid = update_invoice(
            &pool,
            invoice.id,
            None, None, "INV-002", None, 500.0, 0.0, 500.0, "paid",
            "2026-07-01", None, Some("2026-07-15".to_string()), None,
        )
        .await
        .expect("Falha ao marcar como paga");

        assert_eq!(paid.status, "paid");
        assert_eq!(paid.paid_date.as_deref(), Some("2026-07-15"));
    }

    #[tokio::test]
    async fn test_delete_invoice() {
        let pool = setup_test_db().await;

        let invoice = create_invoice(
            &pool,
            None, None, "INV-003", None, 100.0, 0.0, 100.0, "draft",
            "2026-07-01", None, None,
        )
        .await
        .expect("Falha ao criar fatura");

        delete_invoice(&pool, invoice.id)
            .await
            .expect("Falha ao deletar fatura");

        let invoices = get_invoices(&pool, None)
            .await
            .expect("Falha ao listar");
        assert!(invoices.is_empty());
    }

    #[tokio::test]
    async fn test_dashboard_finance_stats() {
        let pool = setup_test_db().await;

        // Dashboard sem faturas
        let stats = get_dashboard_stats(&pool)
            .await
            .expect("Falha ao obter stats");
        assert_eq!(stats.total_revenue, 0.0);
        assert_eq!(stats.pending_invoices, 0);
        assert_eq!(stats.pending_amount, 0.0);

        // Criar faturas
        create_invoice(&pool, None, None, "INV-001", None, 1000.0, 100.0, 1100.0, "paid", "2026-07-01", None, None).await.unwrap();
        create_invoice(&pool, None, None, "INV-002", None, 500.0, 0.0, 500.0, "sent", "2026-07-01", Some("2026-07-31".to_string()), None).await.unwrap();
        create_invoice(&pool, None, None, "INV-003", None, 300.0, 30.0, 330.0, "draft", "2026-07-01", None, None).await.unwrap();

        let stats2 = get_dashboard_stats(&pool)
            .await
            .expect("Falha ao obter stats");

        assert_eq!(stats2.total_revenue, 1100.0); // INV-001 paga
        assert_eq!(stats2.pending_invoices, 2); // INV-002 sent + INV-003 draft
        assert_eq!(stats2.pending_amount, 830.0); // 500 + 330
    }

    // ==================== Testes de Dashboard ====================

    #[tokio::test]
    async fn test_get_dashboard_stats_enterprise() {
        let pool = setup_test_db().await;

        let stats = get_dashboard_stats(&pool)
            .await
            .expect("Falha ao obter stats");

        // Verificar as novas métricas
        assert!(stats.total_clients >= 0);
        assert!(stats.total_teams >= 0);
        assert!(stats.total_users >= 0);

        // Adicionar dados
        create_client(&pool, "Cliente Teste", None, None, None, None).await.unwrap();
        create_team(&pool, "Equipe Teste", None).await.unwrap();
        create_user(&pool, "Usuário Teste", "user_test@email.com", "member").await.unwrap();

        let stats2 = get_dashboard_stats(&pool)
            .await
            .expect("Falha ao obter stats");
        assert!(stats2.total_clients >= 1);
        assert!(stats2.total_teams >= 1);
        assert!(stats2.total_users >= 1);
    }
}

/// Sanitiza nome de arquivo para evitar path traversal (Zip Slip, etc.)
/// Remove qualquer tentativa de ../, ~, /, \0
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
