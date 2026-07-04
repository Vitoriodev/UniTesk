mod db;
mod models;

use models::*;
use sqlx::PgPool;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

/// Estado compartilhado da aplicação
struct AppState {
    pool: PgPool,
}

// ===================== Comandos Tauri =====================

#[tauri::command]
async fn get_projects(state: tauri::State<'_, AppState>) -> Result<Vec<Project>, String> {
    db::get_projects(&state.pool).await.map_err(|_| "Erro ao carregar projetos".to_string())
}

#[tauri::command]
async fn delete_project(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    db::delete_project(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir projeto".to_string())
}

#[tauri::command]
async fn update_project(
    state: tauri::State<'_, AppState>,
    id: i32,
    name: String,
    description: Option<String>,
) -> Result<Project, String> {
    db::update_project(&state.pool, id, &name, &description.unwrap_or_default())
        .await
        .map_err(|_| "Erro ao atualizar projeto".to_string())
}

#[tauri::command]
async fn create_project(
    state: tauri::State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Project, String> {
    db::create_project(&state.pool, &name, &description.unwrap_or_default())
        .await
        .map_err(|_| "Erro ao criar projeto".to_string())
}

#[tauri::command]
async fn get_articles(state: tauri::State<'_, AppState>) -> Result<Vec<Article>, String> {
    db::get_articles(&state.pool).await.map_err(|_| "Erro ao carregar artigos".to_string())
}

#[tauri::command]
async fn create_article(
    state: tauri::State<'_, AppState>,
    title: String,
    content: Option<String>,
    project_name: Option<String>,
    project_id: Option<i32>,
) -> Result<Article, String> {
    db::create_article(
        &state.pool,
        &title,
        &content.unwrap_or_default(),
        &project_name.unwrap_or_default(),
        project_id,
    )
    .await
    .map_err(|_| "Erro ao criar artigo".to_string())
}

#[tauri::command]
async fn delete_article(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    db::delete_article(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir artigo".to_string())
}

#[tauri::command]
async fn get_assignments(state: tauri::State<'_, AppState>) -> Result<Vec<Assignment>, String> {
    db::get_assignments(&state.pool).await.map_err(|_| "Erro ao carregar atividades".to_string())
}

#[tauri::command]
async fn create_assignment(
    state: tauri::State<'_, AppState>,
    title: String,
    description: Option<String>,
    due_date: String,
    due_time: Option<String>,
    project_name: Option<String>,
) -> Result<Assignment, String> {
    db::create_assignment(
        &state.pool,
        &title,
        &description.unwrap_or_default(),
        &due_date,
        due_time,
        &project_name.unwrap_or_default(),
    )
    .await
    .map_err(|_| "Erro ao criar atividade".to_string())
}

#[tauri::command]
async fn mark_assignment_done(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::mark_assignment_done(&state.pool, id)
        .await
        .map_err(|_| "Erro ao marcar atividade como concluída".to_string())
}

#[tauri::command]
async fn get_project_files(
    state: tauri::State<'_, AppState>,
    project_id: i32,
) -> Result<Vec<ProjectFile>, String> {
    db::get_project_files(&state.pool, project_id)
        .await
        .map_err(|_| "Erro ao carregar arquivos".to_string())
}

#[tauri::command]
async fn add_project_file(
    state: tauri::State<'_, AppState>,
    project_id: i32,
    original_name: String,
    file_data: Vec<u8>,
    mime_type: String,
) -> Result<ProjectFile, String> {
    // Validação de segurança: limite de 10 MB no backend também
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err(format!("Arquivo muito grande! Máximo permitido: 10 MB (tamanho enviado: {} MB)", file_data.len() / (1024 * 1024)));
    }

    let stored_name = format!("{}_{}", uuid::Uuid::new_v4(), &original_name);
    db::add_project_file(
        &state.pool,
        project_id,
        &original_name,
        &stored_name,
        &file_data,
        &mime_type,
    )
    .await
    .map_err(|_| "Erro ao anexar arquivo".to_string())
}

#[tauri::command]
async fn get_project_file_data(
    state: tauri::State<'_, AppState>,
    file_id: i32,
) -> Result<(String, String, Vec<u8>), String> {
    db::get_project_file_data(&state.pool, file_id)
        .await
        .map_err(|_| "Erro ao baixar arquivo".to_string())
}

#[tauri::command]
async fn delete_project_file(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_project_file(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir arquivo".to_string())
}

#[tauri::command]
async fn export_project_zip(
    state: tauri::State<'_, AppState>,
    project_id: i32,
) -> Result<(String, Vec<u8>), String> {
    let (project, zip_data) = db::export_project_zip(&state.pool, project_id).await?;
    // Nome sanitizado para download
    let safe_name: String = project.name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
        .collect();
    let filename = format!("{}.zip", safe_name.trim());
    Ok((filename, zip_data))
}

#[tauri::command]
async fn get_dashboard_stats(
    state: tauri::State<'_, AppState>,
) -> Result<DashboardStats, String> {
    db::get_dashboard_stats(&state.pool)
        .await
        .map_err(|_| "Erro ao carregar estatísticas".to_string())
}

// ===================== Arquivos de Atividades =====================

#[tauri::command]
async fn get_assignment_files(
    state: tauri::State<'_, AppState>,
    assignment_id: i32,
) -> Result<Vec<AssignmentFile>, String> {
    db::get_assignment_files(&state.pool, assignment_id)
        .await
        .map_err(|_| "Erro ao carregar arquivos da atividade".to_string())
}

#[tauri::command]
async fn add_assignment_file(
    state: tauri::State<'_, AppState>,
    assignment_id: i32,
    original_name: String,
    file_data: Vec<u8>,
    mime_type: String,
) -> Result<AssignmentFile, String> {
    // Validação de segurança: limite de 10 MB no backend também
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err(format!("Arquivo muito grande! Máximo permitido: 10 MB (tamanho enviado: {} MB)", file_data.len() / (1024 * 1024)));
    }

    let stored_name = format!("{}_{}", uuid::Uuid::new_v4(), &original_name);
    db::add_assignment_file(
        &state.pool,
        assignment_id,
        &original_name,
        &stored_name,
        &file_data,
        &mime_type,
    )
    .await
    .map_err(|_| "Erro ao anexar arquivo à atividade".to_string())
}

#[tauri::command]
async fn get_assignment_file_data(
    state: tauri::State<'_, AppState>,
    file_id: i32,
) -> Result<(String, String, Vec<u8>), String> {
    db::get_assignment_file_data(&state.pool, file_id)
        .await
        .map_err(|_| "Erro ao baixar arquivo da atividade".to_string())
}

#[tauri::command]
async fn delete_assignment_file(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_assignment_file(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir arquivo da atividade".to_string())
}

/// Verificar atividades com prazo hoje e disparar notificação
#[tauri::command]
async fn delete_assignment(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_assignment(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir atividade".to_string())
}

#[tauri::command]
async fn export_all_data(
    state: tauri::State<'_, AppState>,
) -> Result<db::ExportedData, String> {
    db::export_all_data(&state.pool).await
}

#[tauri::command]
async fn import_all_data(
    state: tauri::State<'_, AppState>,
    data: db::ExportedData,
) -> Result<String, String> {
    db::import_all_data(&state.pool, &data).await
}

#[tauri::command]
async fn check_today_assignments(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Assignment>, String> {
    let assignments = db::get_today_assignments(&state.pool)
        .await
        .map_err(|_| "Erro ao verificar prazos".to_string())?;

    // Disparar notificação para cada atividade
    for assignment in &assignments {
        let time_str = assignment.notification_time.as_deref()
            .or(assignment.due_time.as_deref())
            .map(|t| format!(" às {}", t))
            .unwrap_or_default();
        let project_str = assignment.project_name.as_deref()
            .map(|p| format!(" ({})", p))
            .unwrap_or_default();
        let _ = app.notification()
            .builder()
            .title("📚 Prazo Hoje!")
            .body(format!(
                "A atividade '{}'{} vence hoje{}!",
                assignment.title, project_str, time_str
            ))
            .show();
    }

    Ok(assignments)
}

// ===================== Configuração do App =====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let database_url =
                std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres@localhost:5432/academic_manager".to_string()
                });

            let pool = tauri::async_runtime::block_on(async {
                match db::init_db(&database_url).await {
                    Ok(pool) => {
                        println!("✅ Banco de dados conectado com sucesso!");
                        // Verificar atividades vencidas em background
                        let pool_clone = pool.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = db::update_overdue_assignments(&pool_clone).await;
                            println!("✅ Status de atividades atualizado!");
                        });
                        Some(pool)
                    }
                    Err(e) => {
                        eprintln!("❌ Erro ao conectar ao banco: {}", e);
                        None
                    }
                }
            });

            match pool {
                Some(pool) => {
                    app.manage(AppState { pool });
                    Ok(())
                }
                None => {
                    eprintln!("❌ App não pode iniciar sem banco de dados");
                    // Ainda assim gerencia um AppState dummy para não quebrar comandos
                    // Na prática, os comandos vão retornar erro de conexão
                    std::process::exit(1);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_projects,
            create_project,
            delete_project,
            update_project,
            get_articles,
            create_article,
            delete_article,
            get_assignments,
            create_assignment,
            mark_assignment_done,
            get_dashboard_stats,
            check_today_assignments,
            get_project_files,
            add_project_file,
            get_project_file_data,
            delete_project_file,
            export_project_zip,
            get_assignment_files,
            add_assignment_file,
            get_assignment_file_data,
            delete_assignment_file,
            delete_assignment,
            export_all_data,
            import_all_data,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Unitesk");
}
