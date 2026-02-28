use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub id: String,
    pub app_id: String,
    pub name: String,
    #[serde(default)]
    pub config: Option<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueCreate {
    pub name: String,
    #[serde(default)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub app_id: String,
    #[serde(default)]
    pub queue_id: Option<String>,
    pub job_type: String,
    #[serde(default)]
    pub payload: Option<String>,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    #[serde(default)]
    pub run_at: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCreate {
    pub job_type: String,
    #[serde(default)]
    pub queue_id: Option<String>,
    #[serde(default)]
    pub payload: Option<String>,
    #[serde(default)]
    pub max_attempts: Option<i32>,
    #[serde(default)]
    pub run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub app_id: String,
    pub name: String,
    pub cron_expression: String,
    pub job_type: String,
    #[serde(default)]
    pub job_payload: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub last_run: Option<String>,
    #[serde(default)]
    pub next_run: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleCreate {
    pub name: String,
    pub cron_expression: String,
    pub job_type: String,
    #[serde(default)]
    pub job_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cron_expression: Option<String>,
    #[serde(default)]
    pub job_type: Option<String>,
    #[serde(default)]
    pub job_payload: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}
