use crate::dto::building::BuildingDto;
use crate::dto::unit::UnitDto;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GameStateDto {
    pub player_id: i64,
    pub username: String,
    pub units: Vec<UnitDto>,
    pub buildings: Vec<BuildingDto>,
}
