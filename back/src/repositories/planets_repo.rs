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

    let g_id: i64 = sqlx::query_scalar(
        "INSERT INTO galaxies (seed, x, y, z) VALUES (?, 0.0, 0.0, 0.0) ON CONFLICT DO UPDATE SET id=id RETURNING id"
    )
    .bind(target_galaxy_id)
    .fetch_one(&mut **tx)
    .await?;

    let sys_id: i64 = sqlx::query_scalar(
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
