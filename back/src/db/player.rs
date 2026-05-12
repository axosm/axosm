use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Player {
    pub id:            i64,
    pub username:      String,
    pub email:         String,
    pub password_hash: String,
    pub created_at:    String,
    pub updated_at:    String,
    pub last_login_at: Option<String>,
}