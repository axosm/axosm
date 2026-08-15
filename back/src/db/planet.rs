use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct PlanetRow {
    pub id: i64,
    pub seed: i64,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub created_at: String,
    pub updated_at: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
// pub struct Player {
//     pub id: i64,
//     pub username: String,
//     pub email: String,
//     pub password_hash: String,
//     pub created_at: String,
//     pub updated_at: String,
//     pub last_login_at: Option<String>,
// }
