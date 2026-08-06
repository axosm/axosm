use crate::dto::state::GameStateDto;
use crate::game::game_init;
use crate::game::proc_gen::seed::WORLD_SEED;
use crate::repositories::{buildings_repo, player_state_repo, players_repo, units_repo};
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn load_or_initialize_player(pool: &SqlitePool, player_id: i64) -> Result<GameStateDto> {
    let player = players_repo::fetch_player_by_id(pool, player_id).await?;

    let mut units = units_repo::fetch_player_units(pool, player_id).await?;
    let mut buildings = buildings_repo::fetch_player_buildings(pool, player_id).await?;

    // If completely empty, trigger initialization using deterministic generator paths
    if units.is_empty() && buildings.is_empty() {
        let spawn = game_init::find_starting_location(WORLD_SEED);

        let mut tx = pool.begin().await?;

        player_state_repo::insert_initial_player_state(&mut tx, player_id, &spawn).await?;

        tx.commit().await?;

        // Re-fetch elements cleanly to populate DTO surface mapping
        units = units_repo::fetch_player_units(pool, player_id).await?;
        buildings = buildings_repo::fetch_player_buildings(pool, player_id).await?;
    }

    Ok(GameStateDto {
        player_id,
        username: player.username,
        units: units.into_iter().map(Into::into).collect(),
        buildings: buildings.into_iter().map(Into::into).collect(),
    })
}
