use serde::Serialize;

#[derive(Serialize)]
// #[derive(serde::Serialize)]
pub struct GameStateDto {
    pub player_id: i64,
    pub username: String,
    pub units: Vec<UnitsDto>,
    pub buildings: Vec<BuildingssDto>,
}

#[derive(Serialize)]
// #[derive(serde::Serialize)]
pub struct UnitsDto {}

#[derive(Serialize)]
// #[derive(serde::Serialize)]
pub struct BuildingssDto {}
