use crate::db::unit::UnitRow;
use anyhow::Result;
use sqlx::{Sqlite, SqlitePool, Transaction};

pub async fn fetch_player_units(pool: &SqlitePool, player_id: i64) -> Result<Vec<UnitRow>> {
    let units = sqlx::query_as::<_, UnitRow>(
        "SELECT * FROM units WHERE player_id = ? AND location_mode = 'planet_surface'",
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(units)
}

/// Persists the generated initial entities inside a single transaction safely
pub async fn insert_initial_player_state(
    tx: &mut Transaction<'_, Sqlite>,
    player_id: i64,
    target_galaxy_id: i64,
    coords: (i64, i64, i64),
    planet_seed: u64,
    target_orbit: i64,
    safe_tile: (u8, u32, u32),
) -> Result<()> {
    let (target_sx, target_sy, target_sz) = coords;
    let (safe_face, safe_u, safe_v) = safe_tile;

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
