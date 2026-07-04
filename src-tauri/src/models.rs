use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: Option<String>,
    pub project_name: Option<String>,
    pub project_id: Option<i32>,
    pub created_at: String,
}

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
    pub created_at: String,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    #[serde(rename = "totalProjects")]
    pub total_projects: i64,
    #[serde(rename = "totalArticles")]
    pub total_articles: i64,
    #[serde(rename = "pendingAssignments")]
    pub pending_assignments: i64,
    #[serde(rename = "overdueAssignments")]
    pub overdue_assignments: i64,
    #[serde(rename = "nextDeadline")]
    pub next_deadline: Option<String>,
    #[serde(rename = "nextDeadlineName")]
    pub next_deadline_name: Option<String>,
}


