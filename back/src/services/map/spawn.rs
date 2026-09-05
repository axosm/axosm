// src/spawns.rs
use crate::dto::state::GameStateDto;
use crate::game::game_init;
use crate::maths::goldberg;
use crate::game::proc_gen::seed::WORLD_SEED;
use crate::repositories::{buildings_repo, player_state_repo, players_repo, tiles_repo, units_repo};
use anyhow::Result;
use sqlx::SqlitePool;

const VISION_RADIUS: u32 = 2; // Radius around units/buildings to reveal

pub async fn load_or_initialize_player(pool: &SqlitePool, player_id: i64) -> Result<GameStateDto> {
    let player = players_repo::fetch_player_by_id(pool, player_id).await?;

    let mut units = units_repo::fetch_player_units(pool, player_id).await?;
    let mut buildings = buildings_repo::fetch_player_buildings(pool, player_id).await?;

    // 1. Initial Spawn Path
    if units.is_empty() && buildings.is_empty() {
        let spawn = game_init::find_starting_location(WORLD_SEED);
        let mut tx = pool.begin().await?;

        player_state_repo::insert_initial_player_state(&mut tx, player_id, &spawn).await?;
        tx.commit().await?;

        // Fast-path: Populate directly from memory without querying the DB again
        units = units_repo::fetch_player_units(pool, player_id).await?;
        buildings = buildings_repo::fetch_player_buildings(pool, player_id).await?;
    }

    // 2. Resolve Active Planet Target
    let primary_planet_id = units
        .first()
        .and_then(|u| u.planet_id)
        .or_else(|| buildings.first().map(|b| b.planet_id));

    // 3. Resolve Visible Coordinates around Units and Buildings on Primary Planet
    let mut visible_coords = Vec::new();

    if let Some(target_planet_id) = primary_planet_id {
        // Collect coordinates from units on this planet
        for unit in units.iter() {
            if let (Some(planet_id), Some(face), Some(u), Some(v)) = (
                unit.planet_id,
                unit.planet_face,
                unit.planet_u,
                unit.planet_v,
            ) {
                if planet_id == target_planet_id {
                    let neighbors = goldberg::get_tile_neighbors_in_radius(
                        face as u8,
                        u as u32,
                        v as u32,
                        VISION_RADIUS,
                    );
                    visible_coords.extend(neighbors);
                }
            }
        }

        // Collect coordinates from buildings on this planet
        for building in buildings.iter() {
            if let (Some(face), Some(u), Some(v)) = (building.face, building.u, building.v) {
                if building.planet_id == target_planet_id {
                    let neighbors = goldberg::get_tile_neighbors_in_radius(
                        face as u8,
                        u as u32,
                        v as u32,
                        VISION_RADIUS,
                    );
                    visible_coords.extend(neighbors);
                }
            }
        }

        visible_coords.sort_unstable();
        visible_coords.dedup();
    }

    // 4. Fetch Tiles for active coords
    let tiles = match primary_planet_id {
        Some(planet_id) => tiles_repo::fetch_tiles_by_coordinates(pool, planet_id, &visible_coords).await?,
        None => Vec::new(),
    };

    Ok(GameStateDto {
        player_id,
        username: player.username,
        units: units.into_iter().map(Into::into).collect(),
        buildings: buildings.into_iter().map(Into::into).collect(),
        tiles: tiles.into_iter().map(|tile| tile.into()).collect(), // Or implement From<TileRow> for TileDto
    })
}