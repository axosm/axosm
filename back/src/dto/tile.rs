use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TileDto {
    pub id: i64,
    pub planet_id: i64,
    pub planet_face: i32,
    pub planet_u: i32,
    pub planet_v: i32,
}