use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug, Serialize)]
pub struct BuildingRow {
    pub id: i64,
    pub player_id: i64,
    pub building_type: String,
    pub tile_id: i64,
    pub level: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub under_attack: i64,
    pub destroyed_at: Option<String>,
    pub can_fly: i64,
    pub flight_state: Option<String>,
    pub construction_done_at: Option<String>,
}
