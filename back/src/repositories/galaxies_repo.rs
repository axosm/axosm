use crate::db::galaxy::GalaxyRow;
use anyhow::Result;
use sqlx::{Sqlite, Transaction};

pub async fn insert_galaxy(
    tx: &mut Transaction<'_, Sqlite>,
    seed: i64,
    coords: (i64, i64, i64),
) -> Result<GalaxyRow> {
    let (x, y, z) = coords;

    let galaxy = sqlx::query_as::<_, GalaxyRow>(
        "INSERT INTO galaxies (seed, x, y, z) VALUES (?, ?, ?, ?) ON CONFLICT DO UPDATE SET id=id RETURNING id"
    )
    .bind(seed)
    .bind(x)
    .bind(y)
    .bind(z)
    .fetch_one(&mut **tx)
    .await
    .inspect_err(|e| tracing::error!("Database error in insert_galaxy: {:?}", e))?;

    Ok(galaxy)
}
