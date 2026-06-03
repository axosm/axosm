use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64_with_seed;

use crate::{
    app::AppState,
    // game::{init_new_player, reveal_fog},
    auth::AuthPlayer,
    // dto::state::GameStateDto,
};

// TODO this out of this file with function derive_seed in proc_gen module
// Core Configuration
pub const WORLD_SEED: u64 = 42;
pub const GENERATION_VERSION: u64 = 1;

// TODO this out of this file with function derive_seed in proc_gen module
// Tags
pub const GALAXY_TAG: u64 = 1;
pub const SYSTEM_TAG: u64 = 2;
pub const PLANET_TAG: u64 = 3;
pub const TILE_TAG: u64 = 4;

// TODO this out of this file with function derive_seed in proc_gen module
pub const PLANET_SUBDIVISION_TAG: u64 = 12;

// TODO this out of this file with function derive_seed in proc_gen module
pub fn derive_seed(base_seed: u64, tag: u64, components: &[i64]) -> u64 {
    // Start with a mix of our global generation version, base seed, and specific feature tag
    let mut current_hash = xxh3_64_with_seed(&tag.to_le_bytes(), base_seed);
    current_hash = xxh3_64_with_seed(&GENERATION_VERSION.to_le_bytes(), current_hash);

    // Append all location/coordinate constraints sequentially
    for &comp in components {
        current_hash = xxh3_64_with_seed(&comp.to_le_bytes(), current_hash);
    }

    current_hash
}

#[derive(Serialize, Debug, Clone)]
pub struct TileProperties {
    // Coordinate identity
    pub face: u8,
    pub u: u32,
    pub v: u32,

    // Generated physical attributes (Never saved to SQLite)
    pub height: u32,
    pub biome: String,
    pub fertility: f64,
    pub max_mineral_capacity: u32,
}

pub fn get_tile_properties(planet_seed: u64, face: u8, u: u32, v: u32) -> TileProperties {
    // Step 1: Establish the individual tile's unique deterministic seed
    let tile_seed = derive_seed(planet_seed, TILE_TAG, &[face as i64, u as i64, v as i64]);

    // Step 2: Establish feature tags isolated from each other
    const TILE_HEIGHT_TAG: u64 = 100;
    const TILE_BIOME_TAG: u64 = 101;

    let height_seed = derive_seed(tile_seed, TILE_HEIGHT_TAG, &[]);
    let biome_seed = derive_seed(tile_seed, TILE_BIOME_TAG, &[]);

    // Step 3: Run your game's deterministic rules matching the design document
    let height = (height_seed % 100) as u32; // e.g. Height map value 0-99
    let biome = match biome_seed % 3 {
        0 => "Ocean",
        1 => "Desert",
        _ => "Continental",
    };

    TileProperties {
        face,
        u,
        v,
        height,
        biome: biome.to_string(),
    }
}

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

async fn load_game_state(pool: &sqlx::SqlitePool, player_id: i64) -> anyhow::Result<GameStateDto> {
    // 1. (Fetch units and buildings exactly as done before...)
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

    // 2. Initialize a new player if they have no presence in the world
    if units.is_empty() && buildings.is_empty() {
        let mut tx = pool.begin().await?;

        // --- DERIVE SEEDS DETERMINISTICALLY ---
        // Let's settle the player in Galaxy 0, System (0, 0, 0), Planet Orbit 0
        let galaxy_id = 0i64;
        let (sys_x, sys_y, sys_z) = (0i64, 0i64, 0i64);
        let orbit_index = 0i64;

        let galaxy_seed = derive_seed(WORLD_SEED, GALAXY_TAG, &[galaxy_id]);
        let system_seed = derive_seed(galaxy_seed, SYSTEM_TAG, &[sys_x, sys_y, sys_z]);
        let planet_seed = derive_seed(system_seed, PLANET_TAG, &[orbit_index]);

        // Determine planet geometric density (Goldberg resolution N) from seed
        // We take the deterministic hash modulo a tight range (e.g., resolution between 4 and 8)
        let subdivision_seed = derive_seed(planet_seed, PLANET_SUBDIVISION_TAG, &[]);
        let subdivision = 4 + (subdivision_seed % 5) as i64;

        // --- SQLITE UPSERTS ---
        // Insert Galaxy using its deterministic seed
        let g_id: i64 = sqlx::query_scalar(
            r#"
                INSERT INTO galaxies (seed, x, y, z) VALUES (?, 0.0, 0.0, 0.0)
                ON CONFLICT(x, y, z) DO UPDATE SET id=id RETURNING id
                "#,
        )
        .bind(galaxy_seed as i64)
        .fetch_one(&mut *tx)
        .await?;

        // Insert Star System
        let sys_id: i64 = sqlx::query_scalar(
            r#"
                INSERT INTO star_systems (galaxy_id, seed, x, y, z) VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(galaxy_id, x, y, z) DO UPDATE SET id=id RETURNING id
                "#,
        )
        .bind(g_id)
        .bind(system_seed as i64)
        .bind(sys_x)
        .bind(sys_y)
        .bind(sys_z)
        .fetch_one(&mut *tx)
        .await?;

        // Insert Planet
        let planet_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO planets (star_system_id, seed, x, y, z, subdivision) VALUES (?, ?, 0.0, 0.0, ?)
                ON CONFLICT DO UPDATE SET id=id RETURNING id
                "#
            )
            .bind(sys_id).bind(planet_seed as i64).bind(subdivision)
            .fetch_one(&mut *tx).await?;

        // --- SPAWN UNITS USING GOLDBERG SURFACE COORDINATES ---
        // Per your Design Notes, units require: (face, u, v) instead of planet_u/v floats
        // Let's spawn them on Face 0, at starting grid spaces (0,0) and (1,1)
        let (face_1, u_1, v_1) = (0i64, 0i64, 0i64);
        let (face_2, u_2, v_2) = (0i64, 1i64, 1i64);

        sqlx::query(
            r#"
                INSERT INTO units (
                    unit_type, is_squad, count, hp, player_id, in_battle,
                    location_mode, planet_id, planet_face, planet_u, planet_v
                ) VALUES
                ('colonist_scout', 0, 1, 100, ?, 0, 'planet_surface', ?, ?, ?, ?),
                ('construction_drone', 0, 1, 150, ?, 0, 'planet_surface', ?, ?, ?, ?)
                "#,
        )
        .bind(player_id)
        .bind(planet_id)
        .bind(face_1)
        .bind(u_1)
        .bind(v_1)
        .bind(player_id)
        .bind(planet_id)
        .bind(face_2)
        .bind(u_2)
        .bind(v_2)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Re-fetch fresh records to pass down to DTO
        units = sqlx::query_as::<_, UnitRow>(
            "SELECT * FROM units WHERE player_id = ? AND location_mode = 'planet_surface'",
        )
        .bind(player_id)
        .fetch_all(pool)
        .await?;
    }

    Ok(GameStateDto {
        player_id,
        units,
        buildings,
    })

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
}
