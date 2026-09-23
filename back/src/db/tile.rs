use crate::dto::tile::TileDto;
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct TileRow {
    pub id: i64,
    pub planet_id: i64,
    pub face: i64,
    pub u: i64,
    pub v: i64,
    pub tile_type: String,
    pub yield_quality: f64,
    pub rare_deposit: Option<String>,
    pub owner_player_id: Option<i64>,
    pub influence_recalc_needed: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TileRow> for TileDto {
    fn from(row: TileRow) -> Self {
        Self {
            id: row.id,
            planet_id: row.planet_id,
            face: row.face as i32,
            u: row.u as i32,
            v: row.v as i32,
            tile_type: row.tile_type,
            yield_quality: row.yield_quality,
            rare_deposit: row.rare_deposit,
            owner_player_id: row.owner_player_id,
        }
    }
}