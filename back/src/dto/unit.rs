use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UnitDto {
    pub id: i64,
    pub unit_type: String,
    pub count: i32,
    pub hp: i32,
    pub location_mode: String,
    pub planet_id: Option<i64>,
    pub planet_face: Option<i32>,
    pub planet_u: Option<i32>,
    pub planet_v: Option<i32>,
}
