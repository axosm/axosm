use crate::game::proc_gen::StartingLocation;
use crate::repositories::{buildings_repo, units_repo};
use anyhow::Result;
use sqlx::{Sqlite, Transaction};

pub async fn insert_initial_player_state(
    tx: &mut Transaction<'_, Sqlite>,
    player_id: i64,
    spawn: &StartingLocation,
) -> Result<()> {
    // 1. Get or Create Galaxy
    let galaxy_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM galaxies WHERE x = ? AND y = ? AND z = ?",
    )
    .bind(spawn.galaxy.position.0)
    .bind(spawn.galaxy.position.1)
    .bind(spawn.galaxy.position.2)
    .fetch_optional(&mut **tx)
    .await?
    {
        Some(id) => id,
        None => sqlx::query("INSERT INTO galaxies (seed, x, y, z) VALUES (?, ?, ?, ?)")
            .bind(spawn.galaxy.seed as i64)
            .bind(spawn.galaxy.position.0)
            .bind(spawn.galaxy.position.1)
            .bind(spawn.galaxy.position.2)
            .execute(&mut **tx)
            .await?
            .last_insert_rowid(),
    };

    // 2. Get or Create Star System
    let star_system_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM star_systems WHERE galaxy_id = ? AND x = ? AND y = ? AND z = ?",
    )
    .bind(galaxy_id)
    .bind(spawn.star_system.position.0)
    .bind(spawn.star_system.position.1)
    .bind(spawn.star_system.position.2)
    .fetch_optional(&mut **tx)
    .await?
    {
        Some(id) => id,
        None => sqlx::query(
            "INSERT INTO star_systems (galaxy_id, seed, x, y, z) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(galaxy_id)
        .bind(spawn.star_system.seed as i64)
        .bind(spawn.star_system.position.0)
        .bind(spawn.star_system.position.1)
        .bind(spawn.star_system.position.2)
        .execute(&mut **tx)
        .await?
        .last_insert_rowid(),
    };

    // 3. Get or Create Planet
    let planet_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM planets WHERE star_system_id = ? AND x = ? AND y = ?",
    )
    .bind(star_system_id)
    .bind(spawn.planet.planet_pos.0)
    .bind(spawn.planet.planet_pos.1)
    .fetch_optional(&mut **tx)
    .await?
    {
        Some(id) => id,
        None => sqlx::query(
            "INSERT INTO planets (star_system_id, seed, x, y, subdivision) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(star_system_id)
        .bind(spawn.planet.seed as i64)
        .bind(spawn.planet.planet_pos.0)
        .bind(spawn.planet.planet_pos.1)
        .bind(spawn.planet.subdivision as i32)
        .execute(&mut **tx)
        .await?
        .last_insert_rowid(),
    };

    // 4. Claim Tile for Player
    let tile_id = sqlx::query(
        "INSERT INTO planet_tiles (planet_id, face, u, v, tile_type, yield_quality, rare_deposit, owner_player_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(planet_id, face, u, v) DO UPDATE SET owner_player_id = excluded.owner_player_id",
    )
    .bind(planet_id)
    .bind(spawn.tile.face as i32)
    .bind(spawn.tile.u as i32)
    .bind(spawn.tile.v as i32)
    .bind(spawn.tile.tile_type.as_str())
    .bind(spawn.tile.yield_quality as f64)
    .bind(spawn.tile.rare_deposit)
    .bind(player_id)
    .execute(&mut **tx)
    .await?
    .last_insert_rowid();

    // 5. Spawn Initial Headquarters Building
    buildings_repo::create_building(tx, player_id, "colony_hub", tile_id, 1000, 1000).await?;

    // 6. Spawn Initial Explorer Unit on the starting tile
    units_repo::create_surface_unit(
        tx,
        player_id,
        "scout",
        planet_id,
        spawn.tile.face,
        spawn.tile.u,
        spawn.tile.v,
        100,
    )
    .await?;

    Ok(())
}
