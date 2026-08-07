use crate::dto::unit::UnitDto;
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct UnitRow {
    pub id: i64,
    pub unit_type: String,
    pub is_squad: i32,
    pub count: i32,
    pub hp: i32,
    pub player_id: i64,
    pub in_battle: i32,
    pub location_mode: String,
    pub planet_id: Option<i64>,
    pub planet_face: Option<i32>,
    pub planet_u: Option<i32>,
    pub planet_v: Option<i32>,
    pub orbit_planet_id: Option<i64>,
    pub star_system_id: Option<i64>,
    pub star_system_x: Option<f64>,
    pub star_system_y: Option<f64>,
    pub star_system_z: Option<f64>,
}

impl From<UnitRow> for UnitDto {
    fn from(row: UnitRow) -> Self {
        Self {
            id: row.id,
            unit_type: row.unit_type,
            count: row.count,
            hp: row.hp,
            location_mode: row.location_mode,
            planet_id: row.planet_id,
            planet_face: row.planet_face,
            planet_u: row.planet_u,
            planet_v: row.planet_v,
        }
    }
}

// #[derive(FromRow, Debug, Serialize)]
// pub struct UnitRow {
//     pub id: i64,
//     pub unit_type: String,
//     pub is_squad: bool,
//     pub count: i64,
//     pub hp: i64,
//     pub player_id: i64,
//     pub in_battle: bool,
//     pub location_mode: String,
//     pub planet_id: Option<i64>, // Using Option in case these can be null
//     pub planet_face: Option<i64>,
//     pub planet_u: Option<f64>,
//     pub planet_v: Option<f64>,
//     pub customization: Option<String>,
// }
