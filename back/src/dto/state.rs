use serde::Serialize;

#[derive(Serialize)]
pub struct GameStateDto {
    pub player_id: i64,
    pub username: String,
    pub units: Vec<UnitsDto>,
}
