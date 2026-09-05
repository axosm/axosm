use crate::dto::building::BuildingDto;
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct BuildingRow {
    pub id: i64,
    pub player_id: i64,
    pub building_type: String,
    pub tile_id: i64,
    pub planet_id: i64,
    pub level: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub under_attack: i32,
    pub can_fly: i32,
    pub flight_state: Option<String>,
    pub construction_done_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,

    // Joined from planet_tiles
    pub face: Option<i32>,
    pub u: Option<i32>,
    pub v: Option<i32>,
}

impl From<BuildingRow> for BuildingDto {
    fn from(row: BuildingRow) -> Self {
        Self {
            id: row.id,
            building_type: row.building_type,
            tile_id: row.tile_id,
            planet_id: row.planet_id,
            level: row.level,
            hp: row.hp,
            max_hp: row.max_hp,
            can_fly: row.can_fly != 0,
            flight_state: row.flight_state,
            face: row.face,
            u: row.u,
            v: row.v,
        }
    }
}