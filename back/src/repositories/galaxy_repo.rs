use crate::models::GalaxyRow;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn insert_galaxy(
    tx: &mut Transaction<'_, Sqlite>,
    seed: i64,
    coords: (i64, i64, i64),
) -> Result<(GalaxyRow)> {
    let (x, y, z) = coords;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO galaxies (seed, x, y, z) VALUES (?, ?, ?, ?) ON CONFLICT DO UPDATE SET id=id RETURNING id"
    )
    .bind(seed)
    .bind(x)
    .bind(y)
    .bind(z)
    .fetch_one(&mut **tx)
    .await?;

    Ok(id)
}
