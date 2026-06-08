use crate::models::BuildingRow;
use anyhow::Result;
use sqlx::SqlitePool; // Adjust paths to where your Rows are defined

pub async fn fetch_player_buildings(pool: &SqlitePool, player_id: i64) -> Result<Vec<BuildingRow>> {
    let buildings = sqlx::query_as::<_, BuildingRow>("SELECT * FROM buildings WHERE player_id = ?")
        .bind(player_id)
        .fetch_all(pool)
        .await?;
    Ok(buildings)
}
