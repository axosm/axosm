use crate::models::StarSystemRow;
use anyhow::Result;
use sqlx::SqlitePool;

/// Persists the generated initial entities inside a single transaction safely
pub async fn insert_star_system(
    tx: &mut Transaction<'_, Sqlite>,
    seed: u64,
    coords: (i64, i64, i64),
    galaxy_id: i64,
) -> Result<()> {
    let (x, y, z) = coords;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO star_systems (galaxy_id, seed, x, y, z) VALUES (?, ?, ?, ?, ?) ON CONFLICT DO UPDATE SET id=id RETURNING id"
    )
    .bind(g_id)
    .bind(target_galaxy_id)
    .bind(target_sx)
    .bind(target_sy)
    .bind(target_sz)
    .fetch_one(&mut **tx)
    .await?;

    let planet_id: i64 = sqlx::query_scalar(
        "INSERT INTO planets (star_system_id, seed, x, y, subdivision) VALUES (?, ?, ?, ?, 0) ON CONFLICT DO UPDATE SET id=id RETURNING id"
    )
    .bind(sys_id)
    .bind(planet_seed as i64)
    .bind(target_orbit as f64)
    .bind(target_orbit as f64)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO units (
            unit_type, is_squad, count, hp, player_id, in_battle,
            location_mode, planet_id, planet_face, planet_u, planet_v
        ) VALUES ('colonist_scout', 0, 1, 100, ?, 0, 'planet_surface', ?, ?, ?, ?)
        "#,
    )
    .bind(player_id)
    .bind(planet_id)
    .bind(safe_face as i64)
    .bind(safe_u as i64)
    .bind(safe_v as i64)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
