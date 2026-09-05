// // src/game/building_flight.rs
// use anyhow::{bail, Result};
// use sqlx::{Sqlite, Transaction};

// pub async fn lift_off(tx: &mut Transaction<'_, Sqlite>, building_id: i64) -> Result<()> {
//     // Transition from grounded -> lifting_off -> flying
//     let rows_affected = sqlx::query!(
//         r#"
//         UPDATE buildings 
//         SET flight_state = 'flying', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
//         WHERE id = ? AND can_fly = 1 AND flight_state = 'grounded'
//         "#,
//         building_id
//     )
//     .execute(&mut **tx)
//     .await?
//     .rows_affected();

//     if rows_affected == 0 {
//         bail!("Building cannot lift off (either cannot fly or already airborne)");
//     }

//     Ok(())
// }

// pub async fn land_building(
//     tx: &mut Transaction<'_, Sqlite>, 
//     building_id: i64, 
//     target_tile_id: i64
// ) -> Result<()> {
//     // 1. Ensure no other building is currently GROUNDED on the destination tile
//     let occupied = sqlx::query_scalar!(
//         r#"
//         SELECT EXISTS(
//             SELECT 1 FROM buildings 
//             WHERE tile_id = ? AND flight_state = 'grounded' AND destroyed_at IS NULL
//         )
//         "#,
//         target_tile_id
//     )
//     .fetch_one(&mut **tx)
//     .await?;

//     if occupied == 1 {
//         bail!("Cannot land: Target tile already has a grounded building");
//     }

//     // 2. Land the building on the target tile
//     sqlx::query!(
//         r#"
//         UPDATE buildings 
//         SET tile_id = ?, flight_state = 'grounded', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
//         WHERE id = ? AND flight_state = 'flying'
//         "#,
//         target_tile_id,
//         building_id
//     )
//     .execute(&mut **tx)
//     .await?;

//     Ok(())
// }