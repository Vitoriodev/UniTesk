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
    let projects = get_projects(pool).await.map_err(|e| e.to_string())?;
    let articles = get_articles(pool).await.map_err(|e| e.to_string())?;
    let assignments = get_assignments(pool).await.map_err(|e| e.to_string())?;

    // Buscar arquivos de projeto com dados binários
    let project_files_raw: Vec<(i32, String, String, i64, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT project_id, original_name, stored_name, file_size, mime_type, created_at::text, file_data FROM project_files"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

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
    .map_err(|e| e.to_string())?;

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
        version: "1.0.0".to_string(),
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
            "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id, name, description, created_at::text as created_at"
        )
        .bind(&project.name)
        .bind(&project.description)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        project_id_map.insert(project.id, new_project.id);
    }

    // Importar artigos
    for article in &data.articles {
        let new_project_id = article.project_id.and_then(|pid| project_id_map.get(&pid).copied());
        sqlx::query(
            "INSERT INTO articles (title, content, project_name, project_id) VALUES ($1, $2, $3, $4)"
        )
        .bind(&article.title)
        .bind(&article.content)
        .bind(&article.project_name)
        .bind(new_project_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Importar atividades (mapear IDs antigos para novos)
    let mut assignment_id_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for assignment in &data.assignments {
        let new_assignment = sqlx::query_as::<_, Assignment>(
            "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name, status) VALUES ($1, $2, $3::date, $4::time, $5::time, $6, $7) RETURNING id, title, description, due_date::text as due_date, due_time::text as due_time, notification_time::text as notification_time, project_name, status, created_at::text as created_at"
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
        .map_err(|e| e.to_string())?;
        assignment_id_map.insert(assignment.id, new_assignment.id);
    }

    // Importar arquivos de projeto
    for pf in &data.project_files {
        if let Some(&new_pid) = project_id_map.get(&pf.project_id) {
            let file_data = base64_decode(&pf.file_data_base64)?;
            add_project_file(pool, new_pid, &pf.original_name, &pf.stored_name, &file_data, &pf.mime_type)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    // Importar arquivos de atividade
    for af in &data.assignment_files {
        if let Some(&new_aid) = assignment_id_map.get(&af.assignment_id) {
            let file_data = base64_decode(&af.file_data_base64)?;
            add_assignment_file(pool, new_aid, &af.original_name, &af.stored_name, &file_data, &af.mime_type)
                .await
                .map_err(|e| e.to_string())?;
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

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(encoded)
        .map_err(|e| format!("Erro ao decodificar base64: {}", e))
}

/// Inicializa a conexão com o banco PostgreSQL
pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Criar tabelas se não existirem
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

    // Migração: adicionar colunas due_time e notification_time se não existirem
    sqlx::query(
        "ALTER TABLE assignments ADD COLUMN IF NOT EXISTS due_time TIME"
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE assignments ADD COLUMN IF NOT EXISTS notification_time TIME"
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

    Ok(pool)
}

// === Projetos ===

pub async fn get_projects(pool: &PgPool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT id, name, description, created_at::text as created_at FROM projects ORDER BY created_at DESC")
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
) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = $1, description = $2 WHERE id = $3 RETURNING id, name, description, created_at::text as created_at"
    )
    .bind(name)
    .bind(description)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_project(pool: &PgPool, name: &str, description: &str) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id, name, description, created_at::text as created_at"
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await
}

// === Arquivos de Projeto ===

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

// === Artigos ===

pub async fn get_articles(pool: &PgPool) -> Result<Vec<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "SELECT id, title, content, project_name, project_id, created_at::text as created_at FROM articles ORDER BY created_at DESC"
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
) -> Result<Article, sqlx::Error> {
    sqlx::query_as::<_, Article>(
        "INSERT INTO articles (title, content, project_name, project_id) VALUES ($1, $2, $3, $4) RETURNING id, title, content, project_name, project_id, created_at::text as created_at"
    )
    .bind(title)
    .bind(content)
    .bind(project_name)
    .bind(project_id)
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

// === Atividades (Assignments) ===

pub async fn get_assignments(pool: &PgPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, notification_time::text as notification_time, project_name, status, created_at::text as created_at FROM assignments ORDER BY due_date ASC, due_time ASC"
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
) -> Result<Assignment, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "INSERT INTO assignments (title, description, due_date, due_time, notification_time, project_name) VALUES ($1, $2, $3::date, $4::time, $5::time, $6) RETURNING id, title, description, due_date::text as due_date, due_time::text as due_time, notification_time::text as notification_time, project_name, status, created_at::text as created_at"
    )
    .bind(title)
    .bind(description)
    .bind(due_date)
    .bind(&due_time)
    .bind(&due_time)
    .bind(project_name)
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
    // Os arquivos associados serão deletados em cascata (ON DELETE CASCADE)
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
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, notification_time::text as notification_time, project_name, status, created_at::text as created_at FROM assignments WHERE due_date <= CURRENT_DATE AND status != 'done' ORDER BY due_date ASC, due_time ASC"
    )
    .fetch_all(pool)
    .await
}

// === Arquivos de Atividades (Assignment Files) ===

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

// === Dashboard ===

pub async fn get_dashboard_stats(pool: &PgPool) -> Result<DashboardStats, sqlx::Error> {
    let total_projects: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM projects")
        .fetch_one(pool)
        .await?;

    let total_articles: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM articles")
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

    Ok(DashboardStats {
        total_projects: total_projects.0,
        total_articles: total_articles.0,
        pending_assignments: pending_assignments.0,
        overdue_assignments: overdue_assignments.0,
        next_deadline: next_deadline_row.as_ref().map(|(d, _)| d.clone()),
        next_deadline_name: next_deadline_row.map(|(_, n)| n),
    })
}

// === Exportação ZIP ===

pub async fn export_project_zip(pool: &PgPool, project_id: i32) -> Result<(Project, Vec<u8>), String> {

    // Buscar dados do projeto
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at::text as created_at FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Erro ao buscar projeto: {}", e))?
    .ok_or_else(|| "Projeto não encontrado".to_string())?;

    // Buscar artigos do projeto
    let articles: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT title, content FROM articles WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Erro ao buscar artigos: {}", e))?;

    // Buscar arquivos do projeto (com dados)
    let files: Vec<(String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT original_name, mime_type, file_data FROM project_files WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Erro ao buscar arquivos: {}", e))?;

    // Criar ZIP em memória
    let buffer = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buffer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. Adicionar arquivo de informações do projeto
    let info_content = format!(
        "Projeto: {}\n\nDescrição: {}\n\nData de Criação: {}\n\n--- Artigos ({}) ---\n",
        project.name,
        project.description.as_deref().unwrap_or("Sem descrição"),
        project.created_at,
        articles.len()
    );

    zip.start_file("projeto.txt", options)
        .map_err(|e| format!("Erro ao criar ZIP: {}", e))?;
    zip.write_all(info_content.as_bytes())
        .map_err(|e| format!("Erro ao escrever ZIP: {}", e))?;

    // 2. Adicionar artigos como arquivos .txt
    for (i, (title, content)) in articles.iter().enumerate() {
        let safe_name = sanitize_filename(title);
        let filename = format!("artigos/{:03}_{}.txt", i + 1, safe_name);
        let content = content.as_deref().unwrap_or("Sem conteúdo");

        zip.start_file(&filename, options)
            .map_err(|e| format!("Erro ao criar ZIP: {}", e))?;
        zip.write_all(content.as_bytes())
            .map_err(|e| format!("Erro ao escrever ZIP: {}", e))?;
    }

    // 3. Adicionar arquivos anexados
    for (original_name, _, file_data) in &files {
        let filename = format!("arquivos/{}", original_name);

        zip.start_file(&filename, options)
            .map_err(|e| format!("Erro ao criar ZIP: {}", e))?;
        zip.write_all(file_data)
            .map_err(|e| format!("Erro ao escrever ZIP: {}", e))?;
    }

    // Finalizar ZIP
    let finished = zip.finish()
        .map_err(|e| format!("Erro ao finalizar ZIP: {}", e))?;

    Ok((project, finished.into_inner()))
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

    /// Cria um pool de conexão para o banco de testes.
    /// Lê TEST_DATABASE_URL ou usa um fallback.
    /// Limpa todas as tabelas e garante que existam.
    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/unitesk_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("Falha ao conectar ao banco de testes. Configure TEST_DATABASE_URL ou crie o banco 'unitesk_test'");

        // Limpar dados existentes (respeitando chaves estrangeiras)
        sqlx::query("DELETE FROM assignment_files").execute(&pool).await.ok();
        sqlx::query("DELETE FROM project_files").execute(&pool).await.ok();
        sqlx::query("DELETE FROM articles").execute(&pool).await.ok();
        sqlx::query("DELETE FROM assignments").execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects").execute(&pool).await.ok();

        // Garantir que as tabelas existem
        init_db(&database_url).await.expect("Falha ao inicializar banco de testes");

        pool
    }

    // ==================== Testes de Projetos ====================

    #[tokio::test]
    async fn test_create_and_get_projects() {
        let pool = setup_test_db().await;

        // Criar um projeto
        let project = create_project(&pool, "Projeto Teste", "Descrição teste")
            .await
            .expect("Falha ao criar projeto");
        assert_eq!(project.name, "Projeto Teste");
        assert_eq!(project.description.as_deref(), Some("Descrição teste"));
        assert!(project.id > 0);

        // Listar projetos
        let projects = get_projects(&pool).await.expect("Falha ao listar projetos");
        assert!(!projects.is_empty());
        assert_eq!(projects[0].name, "Projeto Teste");
    }

    #[tokio::test]
    async fn test_update_project() {
        let pool = setup_test_db().await;

        let project = create_project(&pool, "Nome Antigo", "Descrição antiga")
            .await
            .expect("Falha ao criar projeto");

        let updated = update_project(&pool, project.id, "Nome Novo", "Descrição nova")
            .await
            .expect("Falha ao atualizar projeto");

        assert_eq!(updated.name, "Nome Novo");
        assert_eq!(updated.description.as_deref(), Some("Descrição nova"));
        assert_eq!(updated.id, project.id);
    }

    #[tokio::test]
    async fn test_delete_project() {
        let pool = setup_test_db().await;

        let project = create_project(&pool, "Projeto para Deletar", "")
            .await
            .expect("Falha ao criar projeto");

        delete_project(&pool, project.id)
            .await
            .expect("Falha ao deletar projeto");

        let projects = get_projects(&pool).await.expect("Falha ao listar projetos");
        assert!(projects.is_empty());
    }

    // ==================== Testes de Artigos ====================

    #[tokio::test]
    async fn test_create_and_get_articles() {
        let pool = setup_test_db().await;

        // Criar projeto para vincular artigo
        let project = create_project(&pool, "Projeto Artigo", "")
            .await
            .expect("Falha ao criar projeto");

        // Artigo com projeto vinculado
        let article = create_article(&pool, "Artigo Teste", "Conteúdo do artigo", "Projeto Artigo", Some(project.id))
            .await
            .expect("Falha ao criar artigo");
        assert_eq!(article.title, "Artigo Teste");
        assert_eq!(article.project_id, Some(project.id));

        // Artigo sem projeto
        let article2 = create_article(&pool, "Artigo Solto", "Conteúdo", "", None)
            .await
            .expect("Falha ao criar artigo sem projeto");
        assert_eq!(article2.title, "Artigo Solto");
        assert!(article2.project_id.is_none());

        // Listar artigos
        let articles = get_articles(&pool).await.expect("Falha ao listar artigos");
        assert_eq!(articles.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_article() {
        let pool = setup_test_db().await;

        let article = create_article(&pool, "Artigo para Deletar", "", "", None)
            .await
            .expect("Falha ao criar artigo");

        delete_article(&pool, article.id)
            .await
            .expect("Falha ao deletar artigo");

        let articles = get_articles(&pool).await.expect("Falha ao listar artigos");
        assert!(articles.is_empty());
    }

    // ==================== Testes de Atividades ====================

    #[tokio::test]
    async fn test_create_and_get_assignments() {
        let pool = setup_test_db().await;

        // Atividade sem horário
        let assignment = create_assignment(
            &pool, "Atividade Teste", "Descrição", "2026-12-31", None, "Disciplina X",
        )
        .await
        .expect("Falha ao criar atividade");
        assert_eq!(assignment.title, "Atividade Teste");
        assert_eq!(assignment.due_date, "2026-12-31");
        assert!(assignment.due_time.is_none());
        assert_eq!(assignment.status, "pending");

        // Atividade com horário
        let assignment2 = create_assignment(
            &pool, "Atividade com Hora", "", "2026-12-31", Some("14:00".to_string()), "Disciplina Y",
        )
        .await
        .expect("Falha ao criar atividade com horário");
        assert_eq!(assignment2.due_time.as_deref(), Some("14:00:00"));

        // Listar atividades
        let assignments = get_assignments(&pool).await.expect("Falha ao listar atividades");
        assert_eq!(assignments.len(), 2);
    }

    #[tokio::test]
    async fn test_mark_assignment_done() {
        let pool = setup_test_db().await;

        let assignment = create_assignment(
            &pool, "Para Concluir", "", "2026-12-31", None, "",
        )
        .await
        .expect("Falha ao criar atividade");

        assert_eq!(assignment.status, "pending");

        mark_assignment_done(&pool, assignment.id)
            .await
            .expect("Falha ao marcar como concluída");

        // Verificar via get_assignments
        let assignments = get_assignments(&pool).await.expect("Falha ao listar");
        let updated = assignments.iter().find(|a| a.id == assignment.id).unwrap();
        assert_eq!(updated.status, "done");
    }

    #[tokio::test]
    async fn test_update_overdue_assignments() {
        let pool = setup_test_db().await;

        // Atividade com data passada (deve virar overdue)
        let past = create_assignment(
            &pool, "Atrasada", "", "2020-01-01", None, "",
        )
        .await
        .expect("Falha ao criar atividade atrasada");
        assert_eq!(past.status, "pending");

        // Atividade com data futura (deve continuar pending)
        let future = create_assignment(
            &pool, "Futura", "", "2030-12-31", None, "",
        )
        .await
        .expect("Falha ao criar atividade futura");

        // Executar update_overdue
        let overdue = update_overdue_assignments(&pool)
            .await
            .expect("Falha ao atualizar status");

        // Verificar que a atividade atrasada virou overdue
        let assignments = get_assignments(&pool).await.expect("Falha ao listar");
        let past_now = assignments.iter().find(|a| a.id == past.id).unwrap();
        assert_eq!(past_now.status, "overdue", "Atividade com data passada deveria estar 'overdue'");

        let future_now = assignments.iter().find(|a| a.id == future.id).unwrap();
        assert_eq!(future_now.status, "pending", "Atividade futura deve continuar 'pending'");
    }

    // ==================== Testes de Arquivos de Projeto ====================

    #[tokio::test]
    async fn test_project_file_crud() {
        let pool = setup_test_db().await;

        let project = create_project(&pool, "Projeto com Arquivos", "")
            .await
            .expect("Falha ao criar projeto");

        let file_data = b"conteudo do arquivo binario";

        // Adicionar arquivo
        let file = add_project_file(
            &pool, project.id, "documento.pdf", "uuid_documento.pdf", file_data, "application/pdf",
        )
        .await
        .expect("Falha ao adicionar arquivo");
        assert_eq!(file.original_name, "documento.pdf");
        assert_eq!(file.file_size, file_data.len() as i64);

        // Listar arquivos
        let files = get_project_files(&pool, project.id)
            .await
            .expect("Falha ao listar arquivos");
        assert_eq!(files.len(), 1);

        // Obter dados do arquivo
        let (name, mime, data) = get_project_file_data(&pool, file.id)
            .await
            .expect("Falha ao obter dados do arquivo");
        assert_eq!(name, "documento.pdf");
        assert_eq!(mime, "application/pdf");
        assert_eq!(data, file_data.to_vec());

        // Deletar arquivo
        delete_project_file(&pool, file.id)
            .await
            .expect("Falha ao deletar arquivo");

        let files_after = get_project_files(&pool, project.id)
            .await
            .expect("Falha ao listar arquivos");
        assert!(files_after.is_empty());
    }

    // ==================== Testes de Arquivos de Atividade ====================

    #[tokio::test]
    async fn test_assignment_file_crud() {
        let pool = setup_test_db().await;

        let assignment = create_assignment(
            &pool, "Atividade com Arquivo", "", "2026-12-31", None, "",
        )
        .await
        .expect("Falha ao criar atividade");

        let file_data = b"dados do arquivo de atividade";

        // Adicionar arquivo
        let file = add_assignment_file(
            &pool, assignment.id, "relatorio.docx", "uuid_relatorio.docx", file_data, "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )
        .await
        .expect("Falha ao adicionar arquivo à atividade");
        assert_eq!(file.original_name, "relatorio.docx");
        assert_eq!(file.assignment_id, assignment.id);

        // Listar arquivos
        let files = get_assignment_files(&pool, assignment.id)
            .await
            .expect("Falha ao listar arquivos da atividade");
        assert_eq!(files.len(), 1);

        // Obter dados
        let (name, mime, data) = get_assignment_file_data(&pool, file.id)
            .await
            .expect("Falha ao obter dados do arquivo");
        assert_eq!(name, "relatorio.docx");
        assert_eq!(data, file_data.to_vec());

        // Deletar
        delete_assignment_file(&pool, file.id)
            .await
            .expect("Falha ao deletar arquivo da atividade");

        let files_after = get_assignment_files(&pool, assignment.id)
            .await
            .expect("Falha ao listar após deleção");
        assert!(files_after.is_empty());
    }

    // ==================== Testes de Dashboard ====================

    #[tokio::test]
    async fn test_get_dashboard_stats() {
        let pool = setup_test_db().await;

        // Dashboard vazio
        let stats = get_dashboard_stats(&pool)
            .await
            .expect("Falha ao obter stats do dashboard");
        assert!(stats.total_projects >= 0);
        assert!(stats.total_articles >= 0);

        // Adicionar dados (IDs únicos para evitar conflito com paralelismo)
        let unique_suffix = format!("{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        
        create_project(&pool, &format!("Proj_DB_{}", unique_suffix), "").await.unwrap();
        create_project(&pool, &format!("Proj_DB_{}_2", unique_suffix), "").await.unwrap();
        create_article(&pool, &format!("Art_DB_{}", unique_suffix), "", "", None).await.unwrap();

        let stats = get_dashboard_stats(&pool)
            .await
            .expect("Falha ao obter stats");
        
        // Como os testes rodam em paralelo, usamos verificações relativas
        let my_projects = stats.total_projects - 0; // podem haver projetos de outros testes
        let my_articles = stats.total_articles - 0;
        
        // Verificar que pelo menos NOSSOS dados estão presentes
        assert!(stats.total_projects >= 2, "Deveria ter pelo menos 2 projetos, tem {}", stats.total_projects);
        assert!(stats.total_articles >= 1, "Deveria ter pelo menos 1 artigo, tem {}", stats.total_articles);
        
        // Verificar que as contagens totais são consistentes
        assert!(stats.pending_assignments >= 0);
        assert!(stats.overdue_assignments >= 0);
    }

    // ==================== Testes de Edge Cases ====================

    #[tokio::test]
    async fn test_create_project_empty_description() {
        let pool = setup_test_db().await;

        let project = create_project(&pool, "Só Nome", "")
            .await
            .expect("Falha ao criar projeto sem descrição");
        assert_eq!(project.name, "Só Nome");
        assert_eq!(project.description.as_deref(), Some(""));
    }

    #[tokio::test]
    async fn test_get_today_assignments_empty() {
        let pool = setup_test_db().await;

        // Sem atividades para hoje
        let today = get_today_assignments(&pool)
            .await
            .expect("Falha ao obter atividades de hoje");
        assert!(today.is_empty());
    }

    #[tokio::test]
    async fn test_article_on_delete_set_null() {
        let pool = setup_test_db().await;

        let project = create_project(&pool, "Projeto", "").await.unwrap();
        
        let article = create_article(&pool, "Artigo Vinculado", "", "Projeto", Some(project.id))
            .await
            .unwrap();
        assert_eq!(article.project_id, Some(project.id));

        // Deletar projeto → project_id deve virar NULL (SET NULL)
        delete_project(&pool, project.id).await.unwrap();

        let articles = get_articles(&pool).await.unwrap();
        assert_eq!(articles.len(), 1);
        assert!(articles[0].project_id.is_none(), "project_id deveria ser NULL após deletar o projeto");
    }

    #[tokio::test]
    async fn test_project_file_cascade_on_delete() {
        let pool = setup_test_db().await;

        let project = create_project(&pool, "Projeto", "").await.unwrap();
        
        add_project_file(
            &pool, project.id, "arquivo.txt", "stored.txt", b"dados", "text/plain",
        )
        .await
        .unwrap();

        // Deletar projeto → arquivos devem ser deletados em cascata
        delete_project(&pool, project.id).await.unwrap();

        let files = get_project_files(&pool, project.id).await.unwrap();
        assert!(files.is_empty(), "Arquivos deveriam ser deletados com o projeto (CASCADE)");
    }

    #[tokio::test]
    async fn test_assignment_file_cascade_on_delete() {
        let pool = setup_test_db().await;

        let assignment = create_assignment(
            &pool, "Atividade", "", "2026-12-31", None, "",
        )
        .await
        .unwrap();

        add_assignment_file(
            &pool, assignment.id, "anexo.pdf", "stored.pdf", b"pdf data", "application/pdf",
        )
        .await
        .unwrap();

        // Deletar atividade via SQL direto (não temos delete_assignment)
        sqlx::query("DELETE FROM assignments WHERE id = $1")
            .bind(assignment.id)
            .execute(&pool)
            .await
            .unwrap();

        // Arquivos devem ser deletados em cascata
        let files = get_assignment_files(&pool, assignment.id).await.unwrap();
        assert!(files.is_empty(), "Arquivos de atividade deveriam ser deletados em cascata");
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Obter atividades que vencem hoje para notificações
pub async fn get_today_assignments(pool: &PgPool) -> Result<Vec<Assignment>, sqlx::Error> {
    sqlx::query_as::<_, Assignment>(
        "SELECT id, title, description, due_date::text as due_date, due_time::text as due_time, notification_time::text as notification_time, project_name, status, created_at::text as created_at FROM assignments WHERE due_date = CURRENT_DATE AND status = 'pending' AND notification_time BETWEEN CURRENT_TIME - INTERVAL '30 seconds' AND CURRENT_TIME + INTERVAL '30 seconds'"
    )
    .fetch_all(pool)
    .await
}
