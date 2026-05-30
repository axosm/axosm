use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    app::AppState,
    // game::{init_new_player, reveal_fog},
    auth::AuthPlayer,
};

pub async fn get_game_state(
    State(state): State<Arc<AppState>>,
    auth: AuthPlayer,
) -> Result<Json<GameState>, (StatusCode, String)> {
    let gs = load_game_state(&state.db, auth.0)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(gs))
}

async fn load_game_state(pool: &sqlx::SqlitePool, player_id: i64) -> anyhow::Result<GameState> {
    // Get first unit to locate player's planet
    let unit = sqlx::query_as::<_, Unit>(
        "SELECT id, unit_type, is_squad, count, hp, player_id, in_battle, location_mode,
                planet_id, planet_face, planet_u, planet_v, customization
         FROM units WHERE player_id = ? AND location_mode = 'planet_surface' LIMIT 1",
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await?;

    let planet_id = unit.as_ref().and_then(|u| u.planet_id).unwrap_or(0);

    let (planet_seed, subdivision, system_id): (i64, i64, i64) = if planet_id > 0 {
        sqlx::query_as(
            "SELECT p.seed, p.subdivision, p.star_system_id FROM planets p WHERE p.id = ?",
        )
        .bind(planet_id)
        .fetch_one(pool)
        .await?
    } else {
        (0, 8, 0)
    };

    let galaxy_id: i64 = if system_id > 0 {
        sqlx::query_scalar("SELECT galaxy_id FROM star_systems WHERE id = ?")
            .bind(system_id)
            .fetch_one(pool)
            .await?
    } else {
        0
    };

    let units = sqlx::query_as::<_, Unit>(
        "SELECT id, unit_type, is_squad, count, hp, player_id, in_battle, location_mode,
                planet_id, planet_face, planet_u, planet_v, customization
         FROM units WHERE player_id = ?",
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;

    let visible_tile_rows: Vec<PlanetTile> = sqlx::query_as(
        "SELECT pt.id, pt.planet_id, pt.face, pt.u, pt.v, pt.tile_type, pt.yield_quality,
                pt.rare_deposit, pt.owner_player_id, pt.influence_recalc_needed
         FROM planet_tiles pt
         JOIN player_explored_tiles pet ON pet.tile_id = pt.id
         WHERE pet.player_id = ?",
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;

    let visible_tiles = visible_tile_rows
        .into_iter()
        .map(|t| VisibleTile {
            tile: t,
            explored: true,
        })
        .collect();

    Ok(GameState {
        player_id,
        planet_id,
        galaxy_id,
        system_id,
        planet_seed,
        subdivision,
        units,
        visible_tiles,
    })
}
