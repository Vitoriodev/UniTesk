use serde::{Deserialize, Serialize};

// ===================== Projetos =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub client_id: Option<i32>,
    pub client_name: Option<String>,
    pub created_at: String,
}

// ===================== Artigos / Documentos =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: Option<String>,
    pub project_name: Option<String>,
    pub project_id: Option<i32>,
    pub created_at: String,
    pub scheduled_date: Option<String>,
}

// ===================== Atividades =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Assignment {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub due_date: String,
    pub due_time: Option<String>,
    pub notification_time: Option<String>,
    pub project_name: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: String,
}

// ===================== Arquivos =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssignmentFile {
    pub id: i32,
    pub assignment_id: i32,
    pub original_name: String,
    pub stored_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectFile {
    pub id: i32,
    pub project_id: i32,
    pub original_name: String,
    pub stored_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: String,
}

// ===================== Clientes =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

// ===================== Usuários =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

// ===================== Equipes =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamMember {
    pub id: i32,
    pub team_id: i32,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub role: String,
    pub created_at: String,
}

// ===================== Faturas / Financeiro =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: i32,
    pub project_id: Option<i32>,
    pub project_name: Option<String>,
    pub client_id: Option<i32>,
    pub client_name: Option<String>,
    pub number: String,
    pub description: Option<String>,
    pub amount: f64,
    pub tax: f64,
    pub total: f64,
    pub status: String,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub paid_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

// ===================== Registro de Horas =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimeEntry {
    pub id: i32,
    pub project_id: i32,
    pub project_name: Option<String>,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_minutes: Option<i32>,
    pub billable: bool,
    pub hourly_rate: Option<f64>,
    pub created_at: String,
}

// ===================== Notificações =====================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: i32,
    #[serde(rename = "type")]
    pub notif_type: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: String,
}

// ===================== Dashboard =====================

// ===================== Relatórios =====================

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyCount {
    pub month: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyAmount {
    pub month: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectHours {
    pub project_name: String,
    pub hours: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportStats {
    #[serde(rename = "totalProjects")]
    pub total_projects: i64,
    #[serde(rename = "totalArticles")]
    pub total_articles: i64,
    #[serde(rename = "totalClients")]
    pub total_clients: i64,
    #[serde(rename = "totalTeams")]
    pub total_teams: i64,
    #[serde(rename = "totalUsers")]
    pub total_users: i64,
    #[serde(rename = "assignmentsByMonth")]
    pub assignments_by_month: Vec<MonthlyCount>,
    #[serde(rename = "assignmentsPending")]
    pub assignments_pending: i64,
    #[serde(rename = "assignmentsDone")]
    pub assignments_done: i64,
    #[serde(rename = "assignmentsOverdue")]
    pub assignments_overdue: i64,
    #[serde(rename = "revenueByMonth")]
    pub revenue_by_month: Vec<MonthlyAmount>,
    #[serde(rename = "totalRevenue")]
    pub total_revenue: f64,
    #[serde(rename = "pendingAmount")]
    pub pending_amount: f64,
    #[serde(rename = "hoursByProject")]
    pub hours_by_project: Vec<ProjectHours>,
    #[serde(rename = "totalHours")]
    pub total_hours: f64,
    #[serde(rename = "invoicesDraft")]
    pub invoices_draft: i64,
    #[serde(rename = "invoicesSent")]
    pub invoices_sent: i64,
    #[serde(rename = "invoicesPaid")]
    pub invoices_paid: i64,
    #[serde(rename = "invoicesOverdue")]
    pub invoices_overdue: i64,
    #[serde(rename = "invoicesCancelled")]
    pub invoices_cancelled: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    #[serde(rename = "totalProjects")]
    pub total_projects: i64,
    #[serde(rename = "totalArticles")]
    pub total_articles: i64,
    #[serde(rename = "totalClients")]
    pub total_clients: i64,
    #[serde(rename = "totalTeams")]
    pub total_teams: i64,
    #[serde(rename = "totalUsers")]
    pub total_users: i64,
    #[serde(rename = "pendingAssignments")]
    pub pending_assignments: i64,
    #[serde(rename = "overdueAssignments")]
    pub overdue_assignments: i64,
    #[serde(rename = "nextDeadline")]
    pub next_deadline: Option<String>,
    #[serde(rename = "nextDeadlineName")]
    pub next_deadline_name: Option<String>,
    #[serde(rename = "hoursToday")]
    pub hours_today: i64,
    #[serde(rename = "hoursWeek")]
    pub hours_week: f64,
    #[serde(rename = "totalRevenue")]
    pub total_revenue: f64,
    #[serde(rename = "pendingInvoices")]
    pub pending_invoices: i64,
    #[serde(rename = "pendingAmount")]
    pub pending_amount: f64,
    #[serde(rename = "unreadNotifications")]
    pub unread_notifications: i64,
}


