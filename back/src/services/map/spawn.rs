use crate::game::game_init;
use crate::models::GameStateDto;
use crate::repositories::buildings_repo;
use crate::repositories::units_repo;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn load_or_initialize_player(pool: &SqlitePool, player_id: i64) -> Result<GameStateDto> {
    let mut units = units_repo::fetch_player_units(pool, player_id).await?;
    let mut buildings = buildings_repo::fetch_player_buildings(pool, player_id).await?;

    // If completely empty, trigger initialization using deterministic generator paths
    if units.is_empty() && buildings.is_empty() {
        let spawn = game_init::find_starting_location(player_id);

        let mut tx = pool.begin().await?;

        player_state::insert_initial_player_state(
            &mut tx,
            player_id,
            spawn.galaxy_id,
            spawn.coords,
            spawn.planet_seed,
            spawn.target_orbit,
            spawn.safe_tile,
        )
        .await?;

        tx.commit().await?;

        // Re-fetch elements cleanly to populate DTO surface mapping
        units = player_state::fetch_player_units(pool, player_id).await?;
        buildings = player_state::fetch_player_buildings(pool, player_id).await?;
    }

    Ok(GameStateDto {
        player_id,
        units,
        buildings,
    })
}
