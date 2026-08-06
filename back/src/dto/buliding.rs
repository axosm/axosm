#[derive(Debug, Serialize)]
pub struct BuildingDto {
    pub id: i64,
    pub building_type: String,
    pub tile_id: i64,
    pub level: i32,
    pub hp: i32,
    pub max_hp: i32,
}
