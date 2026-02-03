use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub google_id: String,
    pub email: String,
    pub f_name: String,
    pub l_name: String,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.f_name, self.l_name)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUser {
    pub google_id: String,
    pub email: String,
    pub f_name: String,
    pub l_name: String,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateUser {
    pub f_name: Option<String>,
    pub l_name: Option<String>,
    pub photo_url: Option<String>,
}
