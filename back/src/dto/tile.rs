use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TileDto {
    pub id: i64,
    pub planet_id: i64,
    pub face: i32,
    pub u: i32,
    pub v: i32,
    pub tile_type: String,
    pub yield_quality: f64,
    pub rare_deposit: Option<String>,
    pub owner_player_id: Option<i64>,
}