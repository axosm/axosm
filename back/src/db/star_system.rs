use sqlx::prelude::FromRow;

// src/db/galaxy.rs

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct StarSystemRow {
    pub id: i64,
    pub seed: i64,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub created_at: String,
    pub updated_at: String,
}
// #[derive(FromRow, Debug, Serialize)]
// pub struct BuildingRow {
//     pub id: i64,
//     pub player_id: i64,
//     pub building_type: String,
//     pub tile_id: i64,
//     pub level: i64,
//     pub hp: i64,
//     pub max_hp: i64,
//     pub under_attack: i64,
//     pub destroyed_at: Option<String>,
//     pub can_fly: i64,
//     pub flight_state: Option<String>,
//     pub construction_done_at: Option<String>,
// }
