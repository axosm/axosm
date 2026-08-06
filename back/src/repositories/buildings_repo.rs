use crate::models::BuildingRow;
use anyhow::Result;
use sqlx::{Sqlite, SqlitePool, Transaction};

pub async fn fetch_player_buildings(pool: &SqlitePool, player_id: i64) -> Result<Vec<BuildingRow>> {
    let buildings = sqlx::query_as::<_, BuildingRow>("SELECT * FROM buildings WHERE player_id = ?")
        .bind(player_id)
        .fetch_all(pool)
        .await?;
    Ok(buildings)
}

pub async fn create_building(
    tx: &mut Transaction<'_, Sqlite>,
    player_id: i64,
    building_type: &str,
    tile_id: i64,
    hp: i32,
    max_hp: i32,
) -> Result<i64> {
    let res = sqlx::query(
        "INSERT INTO buildings (player_id, building_type, tile_id, hp, max_hp)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(player_id)
    .bind(building_type)
    .bind(tile_id)
    .bind(hp)
    .bind(max_hp)
    .execute(&mut **tx)
    .await?;

    Ok(res.last_insert_rowid())
}
