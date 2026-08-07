use crate::models::PlayerRow;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn fetch_player_by_id(pool: &SqlitePool, player_id: i64) -> Result<PlayerRow> {
    let player = sqlx::query_as::<_, PlayerRow>("SELECT * FROM players WHERE id = ?")
        .bind(player_id)
        .fetch_one(pool)
        .await?;
    Ok(player)
}
