use crate::db::planet::PlanetRow;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn fetch_player_by_id(pool: &SqlitePool, player_id: i64) -> Result<PlanetRow> {
    let player = sqlx::query_as::<_, PlanetRow>("SELECT * FROM players WHERE id = ?")
        .bind(player_id)
        .fetch_one(pool)
        .await?;
    Ok(player)
}
