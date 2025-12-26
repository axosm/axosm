use serde::Serialize;

use crate::db::models::UnitWithPlanetLocationRow;

#[derive(Serialize)]
#[serde(tag = "location_type")]
pub enum UnitLocation {
    PlanetSurface {
        planet_id: i64,
        face: i32,
        u: i32,
        v: i32,
    },
    Orbit,
    Space,
}

#[derive(Serialize)]
pub struct Unit {
    pub id: i64,
    pub player_id: i64,
    pub unit_type: String,
    pub location: UnitLocation,
}

impl From<UnitWithPlanetLocationRow> for Unit {
    fn from(row: UnitWithPlanetLocationRow) -> Self {
        let location = match row.location_type.as_str() {
            "PLANET_SURFACE" => UnitLocation::PlanetSurface {
                planet_id: row.planet_id.expect("planet_id missing"),
                face: row.face.expect("face missing"),
                u: row.u.expect("u missing"),
                v: row.v.expect("v missing"),
            },
            "ORBIT" => UnitLocation::Orbit,
            "SPACE" => UnitLocation::Space,
            other => panic!("Unknown location_type: {}", other),
        };

        Self {
            id: row.id,
            player_id: row.player_id,
            unit_type: row.unit_type,
            location,
        }
    }
}


// #[derive(Deserialize)]
// pub struct MoveRequest {
//     player_id: i64,
//     unit_id: i64,
//     to_x: i32,
//     to_y: i32,
// }