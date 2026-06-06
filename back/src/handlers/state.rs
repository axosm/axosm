use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::sync::Arc;

use crate::{
    app::AppState,
    // game::{init_new_player, reveal_fog},
    auth::AuthPlayer,
    proc_gen::seed::{
        GALAXY_TAG, PLANET_SUBDIVISION_TAG, PLANET_TAG, SYSTEM_TAG, WORLD_SEED, derive_seed,
    },
    // dto::state::GameStateDto,
};

#[derive(FromRow, Debug, Serialize)]
pub struct UnitRow {
    pub id: i64,
    pub unit_type: String,
    pub is_squad: bool,
    pub count: i64,
    pub hp: i64,
    pub player_id: i64,
    pub in_battle: bool,
    pub location_mode: String,
    pub planet_id: Option<i64>, // Using Option in case these can be null
    pub planet_face: Option<i64>,
    pub planet_u: Option<f64>,
    pub planet_v: Option<f64>,
    pub customization: Option<String>,
}

#[derive(FromRow, Debug, Serialize)]
pub struct BuildingRow {
    pub id: i64,
    pub player_id: i64,
    pub building_type: String,
    pub tile_id: i64,
    pub level: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub under_attack: i64,
    pub destroyed_at: Option<String>,
    pub can_fly: i64,
    pub flight_state: Option<String>,
    pub construction_done_at: Option<String>,
}

// Ensure your GameStateDto includes everything your frontend needs
#[derive(serde::Serialize)]
pub struct GameStateDto {
    pub player_id: i64,
    pub units: Vec<UnitRow>,
    pub buildings: Vec<BuildingRow>,
}

pub async fn get_game_state(
    State(state): State<Arc<AppState>>,
    auth: AuthPlayer,
) -> Result<Json<GameStateDto>, (StatusCode, String)> {
    let gs = load_game_state(&state.db, auth.0)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(gs))
}



continue from here
https://gemini.google.com/share/f1d952395636
add function galacy cluster / densisty to proc gen module

then (see last response ):
https://gemini.google.com/app/cae28d3bd06a3e96
or https://gemini.google.com/share/b5b85508d117


pub async fn load_game_state(
    pool: &sqlx::SqlitePool,
    player_id: i64,
) -> anyhow::Result<GameStateDto> {
    // 1. Fetch current status
    let mut units = sqlx::query_as::<_, UnitRow>(
        "SELECT * FROM units WHERE player_id = ? AND location_mode = 'planet_surface'",
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;

    let mut buildings =
        sqlx::query_as::<_, BuildingRow>("SELECT * FROM buildings WHERE player_id = ?")
            .bind(player_id)
            .fetch_all(pool)
            .await?;

    // 2. Initialize missing player
    if units.is_empty() && buildings.is_empty() {
        let mut tx = pool.begin().await?;

        // Variables to hold our discovered safe-haven coordinates
        let mut target_galaxy_id = 0i64;
        let (mut target_sx, mut target_sy, mut target_sz) = (0i64, 0i64, 0i64);
        let mut target_orbit = 0i64;
        let (mut safe_face, mut safe_u, mut safe_v) = (0u8, 0u32, 0u32);

        let mut planet_seed = 0u64;
        let mut found_dry_land = false;

        // Pseudo-random execution counter to shift search if collision happens
        let mut search_attempt = 0i64;

        // Loop until we find a planet and tile that is NOT an ocean
        while !found_dry_land {
            // Pick a deterministic grid system location using player identity and attempt index
            let search_seed = derive_seed(
                proc_gen::WORLD_SEED,
                999, /* SEARCH_TAG. If switching to random_number instead of 999, read this : https://gemini.google.com/share/3d17aadbdbc3*/
                &[player_id, search_attempt],
            );

            // Map search_seed into large space limits
            target_galaxy_id = (search_seed % 5) as i64; // Spreads players across 5 galaxies
            target_sx = ((search_seed >> 8) % 1000) as i64 - 500; // Grid bounds X: -500 to +500
            target_sy = ((search_seed >> 16) % 1000) as i64 - 500;
            target_sz = ((search_seed >> 24) % 1000) as i64 - 500;

            // Derive seeds following your exact hierarchy chain
            let galaxy_seed = derive_seed(
                proc_gen::WORLD_SEED,
                proc_gen::GALAXY_TAG,
                &[target_galaxy_id],
            );
            let sys_seed = derive_seed(
                galaxy_seed,
                proc_gen::SYSTEM_TAG,
                &[target_sx, target_sy, target_sz],
            );

            // Roll a realistic planet count for this system
            let count_seed = derive_seed(sys_seed, 11 /* PLANET_COUNT_TAG */, &[]);
            let max_planets = 1 + (count_seed % 10) as i64; // 1 to 10 planets

            // Target the middle orbit slot for habitable safety chances
            target_orbit = max_planets / 2;
            planet_seed = derive_seed(sys_seed, proc_gen::PLANET_TAG, &[target_orbit]);

            // Pick a random surface tile to inspect (N=0 means 20 faces, m=[10,11] means max u=10, v=11)
            let tile_picker_seed =
                derive_seed(planet_seed, 888 /* PICKER_TAG */, &[search_attempt]);
            safe_face = (tile_picker_seed % 20) as u8;
            safe_u = ((tile_picker_seed >> 8) % 10) as u32;
            safe_v = ((tile_picker_seed >> 16) % 11) as u32;

            // Run your deterministic biome check
            let tile_seed = derive_seed(
                planet_seed,
                proc_gen::TILE_TAG,
                &[safe_face as i64, safe_u as i64, safe_v as i64],
            );
            let biome_seed = derive_seed(tile_seed, 101 /* TILE_BIOME_TAG */, &[]);

            let biome_type = biome_seed % 3; // 0 = Ocean, 1 = Desert, 2 = Continental
            if biome_type != 0 {
                // Land found! Break loop
                found_dry_land = true;
            } else {
                search_attempt += 1; // Try again at a completely different coordinate set
            }
        }

        // 3. Perform SQLite inserts using our verified dry-land system coordinates
        let g_id: i64 = sqlx::query_scalar(
            "INSERT INTO galaxies (seed, x, y, z) VALUES (?, 0.0, 0.0, 0.0) ON CONFLICT DO UPDATE SET id=id RETURNING id"
        ).bind(target_galaxy_id).fetch_one(&mut *tx).await?;

        let sys_id: i64 = sqlx::query_scalar(
            "INSERT INTO star_systems (galaxy_id, seed, x, y, z) VALUES (?, ?, ?, ?, ?) ON CONFLICT DO UPDATE SET id=id RETURNING id"
        ).bind(g_id).bind(target_galaxy_id).bind(target_sx).bind(target_sy).bind(target_sz).fetch_one(&mut *tx).await?;

        let planet_id: i64 = sqlx::query_scalar(
            "INSERT INTO planets (star_system_id, seed, x, y, subdivision) VALUES (?, ?, ?, ?, 0) ON CONFLICT DO UPDATE SET id=id RETURNING id"
        ).bind(sys_id).bind(planet_seed as i64).bind(target_orbit as f64).bind(target_orbit as f64).fetch_one(&mut *tx).await?;

        // 4. Spawn starter units cleanly onto the safe Goldberg surface tiles
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
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Refresh state values for DTO output
        units = sqlx::query_as::<_, UnitRow>(
            "SELECT * FROM units WHERE player_id = ? AND location_mode = 'planet_surface'",
        )
        .bind(player_id)
        .fetch_all(pool)
        .await?;

        // TODO get buildings as well
    }

    Ok(GameStateDto {
        player_id,
        units,
        buildings,
    })
}

// // Get first unit to locate player's planet
// let unit = sqlx::query_as::<_, Unit>(
//     "SELECT id, unit_type, is_squad, count, hp, player_id, in_battle, location_mode,
//             planet_id, planet_face, planet_u, planet_v, customization
//      FROM units WHERE player_id = ? AND location_mode = 'planet_surface' LIMIT 1",
// )
// .bind(player_id)
// .fetch_optional(pool)
// .await?;

// let planet_id = unit.as_ref().and_then(|u| u.planet_id).unwrap_or(0);

// let (planet_seed, subdivision, system_id): (i64, i64, i64) = if planet_id > 0 {
//     sqlx::query_as(
//         "SELECT p.seed, p.subdivision, p.star_system_id FROM planets p WHERE p.id = ?",
//     )
//     .bind(planet_id)
//     .fetch_one(pool)
//     .await?
// } else {
//     (0, 8, 0)
// };

// let galaxy_id: i64 = if system_id > 0 {
//     sqlx::query_scalar("SELECT galaxy_id FROM star_systems WHERE id = ?")
//         .bind(system_id)
//         .fetch_one(pool)
//         .await?
// } else {
//     0
// };

// let units = sqlx::query_as::<_, Unit>(
//     "SELECT id, unit_type, is_squad, count, hp, player_id, in_battle, location_mode,
//             planet_id, planet_face, planet_u, planet_v, customization
//      FROM units WHERE player_id = ?",
// )
// .bind(player_id)
// .fetch_all(pool)
// .await?;

// let visible_tile_rows: Vec<PlanetTile> = sqlx::query_as(
//     "SELECT pt.id, pt.planet_id, pt.face, pt.u, pt.v, pt.tile_type, pt.yield_quality,
//             pt.rare_deposit, pt.owner_player_id, pt.influence_recalc_needed
//      FROM planet_tiles pt
//      JOIN player_explored_tiles pet ON pet.tile_id = pt.id
//      WHERE pet.player_id = ?",
// )
// .bind(player_id)
// .fetch_all(pool)
// .await?;

// let visible_tiles = visible_tile_rows
//     .into_iter()
//     .map(|t| VisibleTile {
//         tile: t,
//         explored: true,
//     })
//     .collect();

// Ok(GameStateDto {
//     player_id,
//     // planet_id,
//     // galaxy_id,
//     // system_id,
//     // planet_seed,
//     // subdivision,
//     // units,
//     // visible_tiles,
// })
// }
