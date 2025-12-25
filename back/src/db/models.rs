#[derive(Serialize, Deserialize, FromRow)]
pub struct Unit {
    pub id: i64,
    pub player_id: i64,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct MoveOrder {
    pub id: i64,
    pub unit_id: i64,
    pub from_x: i32,
    pub from_y: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub arrival_time: i64,
}