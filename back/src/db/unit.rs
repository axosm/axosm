use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(FromRow, Debug, Serialize)]
pub struct UnitRow {
    pub id: i64,
    pub unit_type: String,
    pub is_squad: bool,
    pub count: i64,
    pub hp: i64,
    pub player_id: i64,
    pub in_battle: bool,
    pub location_mode: String,
    pub planet_id: Option<i64>, // Using Option in case these can be null
    pub planet_face: Option<i64>,
    pub planet_u: Option<f64>,
    pub planet_v: Option<f64>,
    pub customization: Option<String>,
}
