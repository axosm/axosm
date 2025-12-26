use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Unit {
    pub id: i64,
    pub player_id: i64,
    pub unit_type: String,
    pub location_type: String,
}

#[derive(Debug, FromRow)]
pub struct UnitPlanetLocation {
    pub unit_id: i64,
    pub planet_id: i64,
    pub face: i32,
    pub u: i32,
    pub v: i32,
}


#[derive(Debug, FromRow)]
pub struct UnitWithPlanetLocationRow {
    pub id: i64,
    pub player_id: i64,
    pub unit_type: String,
    pub location_type: String,

    pub planet_id: Option<i64>,
    pub face: Option<i32>,
    pub u: Option<i32>,
    pub v: Option<i32>,
}