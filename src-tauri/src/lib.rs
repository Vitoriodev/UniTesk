mod db;
mod models;

use models::*;
use db::DbPool;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

/// Estado compartilhado da aplicação
struct AppState {
    pool: DbPool,
}

// ===================== Comandos Tauri =====================

// ===================== Projetos =====================

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
    client_id: Option<i32>,
) -> Result<Project, String> {
    db::update_project(&state.pool, id, &name, &description.unwrap_or_default(), client_id)
        .await
        .map_err(|_| "Erro ao atualizar projeto".to_string())
}

#[tauri::command]
async fn create_project(
    state: tauri::State<'_, AppState>,
    name: String,
    description: Option<String>,
    client_id: Option<i32>,
) -> Result<Project, String> {
    db::create_project(&state.pool, &name, &description.unwrap_or_default(), client_id)
        .await
        .map_err(|_| "Erro ao criar projeto".to_string())
}

// ===================== Artigos =====================

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
    scheduled_date: Option<String>,
) -> Result<Article, String> {
    db::create_article(
        &state.pool,
        &title,
        &content.unwrap_or_default(),
        &project_name.unwrap_or_default(),
        project_id,
        scheduled_date,
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

// ===================== Atividades =====================

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
    priority: Option<String>,
) -> Result<Assignment, String> {
    db::create_assignment(
        &state.pool,
        &title,
        &description.unwrap_or_default(),
        &due_date,
        due_time,
        &project_name.unwrap_or_default(),
        &priority.unwrap_or_else(|| "medium".to_string()),
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
async fn delete_assignment(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_assignment(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir atividade".to_string())
}

// ===================== Arquivos de Projeto =====================

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
    let safe_name: String = project.name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
        .collect();
    let filename = format!("{}.zip", safe_name.trim());
    Ok((filename, zip_data))
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

// ===================== Clientes =====================

#[tauri::command]
async fn get_clients(state: tauri::State<'_, AppState>) -> Result<Vec<Client>, String> {
    db::get_clients(&state.pool).await.map_err(|_| "Erro ao carregar clientes".to_string())
}

#[tauri::command]
async fn create_client(
    state: tauri::State<'_, AppState>,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
) -> Result<Client, String> {
    db::create_client(&state.pool, &name, email, phone, company, notes)
        .await
        .map_err(|_| "Erro ao criar cliente".to_string())
}

#[tauri::command]
async fn update_client(
    state: tauri::State<'_, AppState>,
    id: i32,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
) -> Result<Client, String> {
    db::update_client(&state.pool, id, &name, email, phone, company, notes)
        .await
        .map_err(|_| "Erro ao atualizar cliente".to_string())
}

#[tauri::command]
async fn delete_client(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    db::delete_client(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir cliente".to_string())
}

// ===================== Usuários =====================

#[tauri::command]
async fn get_users(state: tauri::State<'_, AppState>) -> Result<Vec<User>, String> {
    db::get_users(&state.pool).await.map_err(|_| "Erro ao carregar usuários".to_string())
}

#[tauri::command]
async fn create_user(
    state: tauri::State<'_, AppState>,
    name: String,
    email: String,
    role: Option<String>,
) -> Result<User, String> {
    db::create_user(&state.pool, &name, &email, &role.unwrap_or_else(|| "member".to_string()))
        .await
        .map_err(|_| "Erro ao criar usuário".to_string())
}

#[tauri::command]
async fn update_user(
    state: tauri::State<'_, AppState>,
    id: i32,
    name: String,
    email: String,
    role: Option<String>,
) -> Result<User, String> {
    db::update_user(&state.pool, id, &name, &email, &role.unwrap_or_else(|| "member".to_string()))
        .await
        .map_err(|_| "Erro ao atualizar usuário".to_string())
}

#[tauri::command]
async fn delete_user(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    db::delete_user(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir usuário".to_string())
}

// ===================== Equipes =====================

#[tauri::command]
async fn get_teams(state: tauri::State<'_, AppState>) -> Result<Vec<Team>, String> {
    db::get_teams(&state.pool).await.map_err(|_| "Erro ao carregar equipes".to_string())
}

#[tauri::command]
async fn create_team(
    state: tauri::State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Team, String> {
    db::create_team(&state.pool, &name, description)
        .await
        .map_err(|_| "Erro ao criar equipe".to_string())
}

#[tauri::command]
async fn update_team(
    state: tauri::State<'_, AppState>,
    id: i32,
    name: String,
    description: Option<String>,
) -> Result<Team, String> {
    db::update_team(&state.pool, id, &name, description)
        .await
        .map_err(|_| "Erro ao atualizar equipe".to_string())
}

#[tauri::command]
async fn delete_team(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    db::delete_team(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir equipe".to_string())
}

// ===================== Membros da Equipe =====================

#[tauri::command]
async fn get_team_members(
    state: tauri::State<'_, AppState>,
    team_id: i32,
) -> Result<Vec<TeamMember>, String> {
    db::get_team_members(&state.pool, team_id)
        .await
        .map_err(|_| "Erro ao carregar membros da equipe".to_string())
}

#[tauri::command]
async fn add_team_member(
    state: tauri::State<'_, AppState>,
    team_id: i32,
    user_id: i32,
    role: Option<String>,
) -> Result<TeamMember, String> {
    db::add_team_member(&state.pool, team_id, user_id, &role.unwrap_or_else(|| "member".to_string()))
        .await
        .map_err(|_| "Erro ao adicionar membro à equipe".to_string())
}

#[tauri::command]
async fn remove_team_member(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    db::remove_team_member(&state.pool, id)
        .await
        .map_err(|_| "Erro ao remover membro da equipe".to_string())
}

#[tauri::command]
async fn update_team_member_role(
    state: tauri::State<'_, AppState>,
    id: i32,
    role: String,
) -> Result<TeamMember, String> {
    db::update_team_member_role(&state.pool, id, &role)
        .await
        .map_err(|_| "Erro ao atualizar cargo do membro".to_string())
}

// ===================== Financeiro =====================

#[tauri::command]
async fn get_invoices(
    state: tauri::State<'_, AppState>,
    status_filter: Option<String>,
) -> Result<Vec<Invoice>, String> {
    db::get_invoices(&state.pool, status_filter)
        .await
        .map_err(|_| "Erro ao carregar faturas".to_string())
}

#[tauri::command]
async fn get_invoice(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<Invoice, String> {
    db::get_invoice(&state.pool, id)
        .await
        .map_err(|_| "Fatura não encontrada".to_string())
}

#[tauri::command]
async fn create_invoice(
    state: tauri::State<'_, AppState>,
    project_id: Option<i32>,
    client_id: Option<i32>,
    number: String,
    description: Option<String>,
    amount: f64,
    tax: f64,
    total: f64,
    status: Option<String>,
    issue_date: String,
    due_date: Option<String>,
    notes: Option<String>,
) -> Result<Invoice, String> {
    db::create_invoice(
        &state.pool,
        project_id,
        client_id,
        &number,
        description,
        amount,
        tax,
        total,
        &status.unwrap_or_else(|| "draft".to_string()),
        &issue_date,
        due_date,
        notes,
    )
    .await
    .map_err(|_| "Erro ao criar fatura".to_string())
}

#[tauri::command]
async fn update_invoice(
    state: tauri::State<'_, AppState>,
    id: i32,
    project_id: Option<i32>,
    client_id: Option<i32>,
    number: String,
    description: Option<String>,
    amount: f64,
    tax: f64,
    total: f64,
    status: String,
    issue_date: String,
    due_date: Option<String>,
    paid_date: Option<String>,
    notes: Option<String>,
) -> Result<Invoice, String> {
    db::update_invoice(
        &state.pool,
        id,
        project_id,
        client_id,
        &number,
        description,
        amount,
        tax,
        total,
        &status,
        &issue_date,
        due_date,
        paid_date,
        notes,
    )
    .await
    .map_err(|_| "Erro ao atualizar fatura".to_string())
}

#[tauri::command]
async fn delete_invoice(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_invoice(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir fatura".to_string())
}

// ===================== Registro de Horas =====================

#[tauri::command]
async fn get_time_entries(
    state: tauri::State<'_, AppState>,
    project_id: Option<i32>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<Vec<TimeEntry>, String> {
    db::get_time_entries(&state.pool, project_id, date_from, date_to)
        .await
        .map_err(|_| "Erro ao carregar registros de horas".to_string())
}

#[tauri::command]
async fn start_time_entry(
    state: tauri::State<'_, AppState>,
    project_id: i32,
    user_id: Option<i32>,
    description: Option<String>,
) -> Result<TimeEntry, String> {
    db::start_time_entry(&state.pool, project_id, user_id, description)
        .await
        .map_err(|_| "Erro ao iniciar contagem de tempo".to_string())
}

#[tauri::command]
async fn stop_time_entry(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<TimeEntry, String> {
    db::stop_time_entry(&state.pool, id)
        .await
        .map_err(|_| "Erro ao parar contagem de tempo".to_string())
}

#[tauri::command]
async fn add_manual_time_entry(
    state: tauri::State<'_, AppState>,
    project_id: i32,
    user_id: Option<i32>,
    description: Option<String>,
    duration_minutes: i32,
    entry_date: String,
    billable: bool,
    hourly_rate: Option<f64>,
) -> Result<TimeEntry, String> {
    db::add_manual_time_entry(&state.pool, project_id, user_id, description, duration_minutes, entry_date, billable, hourly_rate)
        .await
        .map_err(|_| "Erro ao adicionar registro de horas".to_string())
}

#[tauri::command]
async fn delete_time_entry(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_time_entry(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir registro de horas".to_string())
}

#[tauri::command]
async fn get_active_time_entry(
    state: tauri::State<'_, AppState>,
) -> Result<Option<TimeEntry>, String> {
    db::get_active_time_entry(&state.pool)
        .await
        .map_err(|_| "Erro ao verificar timer ativo".to_string())
}

#[tauri::command]
async fn get_hours_summary(
    state: tauri::State<'_, AppState>,
) -> Result<(i64, f64), String> {
    db::get_hours_summary(&state.pool)
        .await
        .map_err(|_| "Erro ao carregar resumo de horas".to_string())
}

// ===================== Dashboard =====================

#[tauri::command]
async fn get_dashboard_stats(
    state: tauri::State<'_, AppState>,
) -> Result<DashboardStats, String> {
    db::get_dashboard_stats(&state.pool)
        .await
        .map_err(|_| "Erro ao carregar estatísticas".to_string())
}

// ===================== Export/Import =====================

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

// ===================== Notificações =====================

#[tauri::command]
async fn check_today_assignments(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Assignment>, String> {
    let assignments = db::get_today_assignments(&state.pool)
        .await
        .map_err(|_| "Erro ao verificar prazos".to_string())?;

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

#[tauri::command]
async fn get_notifications(
    state: tauri::State<'_, AppState>,
    unread_only: bool,
    limit: Option<i64>,
) -> Result<Vec<Notification>, String> {
    db::get_notifications(&state.pool, unread_only, limit.unwrap_or(50))
        .await
        .map_err(|_| "Erro ao carregar notificações".to_string())
}

#[tauri::command]
async fn get_unread_notifications_count(
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    db::get_unread_notifications_count(&state.pool)
        .await
        .map_err(|_| "Erro ao carregar contagem".to_string())
}

#[tauri::command]
async fn mark_notification_read(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::mark_notification_read(&state.pool, id)
        .await
        .map_err(|_| "Erro ao marcar notificação como lida".to_string())
}

#[tauri::command]
async fn mark_all_notifications_read(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    db::mark_all_notifications_read(&state.pool)
        .await
        .map_err(|_| "Erro ao marcar todas como lidas".to_string())
}

#[tauri::command]
async fn delete_notification(
    state: tauri::State<'_, AppState>,
    id: i32,
) -> Result<(), String> {
    db::delete_notification(&state.pool, id)
        .await
        .map_err(|_| "Erro ao excluir notificação".to_string())
}

#[tauri::command]
async fn cleanup_old_notifications(
    state: tauri::State<'_, AppState>,
    days: Option<i32>,
) -> Result<i64, String> {
    db::cleanup_old_notifications(&state.pool, days.unwrap_or(30))
        .await
        .map_err(|_| "Erro ao limpar notificações antigas".to_string())
}

#[tauri::command]
async fn auto_generate_notifications(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Notification>, String> {
    db::auto_generate_notifications(&state.pool)
        .await
        .map_err(|_| "Erro ao gerar notificações".to_string())
}

// ===================== Relatórios =====================

#[tauri::command]
async fn get_report_stats(
    state: tauri::State<'_, AppState>,
) -> Result<ReportStats, String> {
    db::get_report_stats(&state.pool)
        .await
        .map_err(|_| "Erro ao carregar relatórios".to_string())
}

// ===================== Helpers =====================

/// Obtém a URL do banco PostgreSQL no Linux
#[cfg(target_os = "linux")]
fn get_database_url_linux() -> String {
    std::env::var("DATABASE_URL").or_else(|_| {
        let config_path = std::path::Path::new("/etc/unitesk/unitesk.conf");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                for line in content.lines() {
                    if let Some(value) = line.strip_prefix("DATABASE_URL=\"") {
                        if let Some(end) = value.find('"') {
                            return Ok(value[..end].to_string());
                        }
                    } else if let Some(value) = line.strip_prefix("DATABASE_URL=") {
                        return Ok(value.to_string());
                    }
                }
            }
        }
        Err(std::env::VarError::NotPresent)
    }).unwrap_or_else(|_| {
        "postgres://postgres@localhost:5432/unitesk".to_string()
    })
}

// ===================== Configuração do App =====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Determinar URL/conexão do banco conforme a plataforma
            #[cfg(target_os = "linux")]
            let database_url = get_database_url_linux();

            #[cfg(target_os = "windows")]
            let database_url = "unitesk.db".to_string();

            let pool = tauri::async_runtime::block_on(async {
                match db::init_db(&database_url).await {
                    Ok(pool) => {
                        println!("✅ Banco de dados conectado com sucesso!");
                        #[cfg(target_os = "linux")]
                        {
                            let pool_clone = pool.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = db::update_overdue_assignments(&pool_clone).await;
                                println!("✅ Status de atividades atualizado!");
                            });
                        }
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
                    std::process::exit(1);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Projetos
            get_projects,
            create_project,
            delete_project,
            update_project,
            // Artigos
            get_articles,
            create_article,
            delete_article,
            // Atividades
            get_assignments,
            create_assignment,
            mark_assignment_done,
            delete_assignment,
            // Arquivos
            get_project_files,
            add_project_file,
            get_project_file_data,
            delete_project_file,
            export_project_zip,
            get_assignment_files,
            add_assignment_file,
            get_assignment_file_data,
            delete_assignment_file,
            // Clientes
            get_clients,
            create_client,
            update_client,
            delete_client,
            // Usuários
            get_users,
            create_user,
            update_user,
            delete_user,
            // Equipes
            get_teams,
            create_team,
            update_team,
            delete_team,
            // Membros
            get_team_members,
            add_team_member,
            remove_team_member,
            update_team_member_role,
            // Financeiro
            get_invoices,
            get_invoice,
            create_invoice,
            update_invoice,
            delete_invoice,
            // Registro de Horas
            get_time_entries,
            start_time_entry,
            stop_time_entry,
            add_manual_time_entry,
            delete_time_entry,
            get_active_time_entry,
            get_hours_summary,
            // Dashboard
            get_dashboard_stats,
            // Export/Import
            export_all_data,
            import_all_data,
            // Relatórios
            get_report_stats,
            // Limpeza de notificações
            cleanup_old_notifications,
            // Notificações
            check_today_assignments,
            get_notifications,
            get_unread_notifications_count,
            mark_notification_read,
            mark_all_notifications_read,
            delete_notification,
            auto_generate_notifications,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o Unitesk");
}
