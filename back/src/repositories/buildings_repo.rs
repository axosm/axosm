use crate::db::building::BuildingRow;
use anyhow::Result;
use sqlx::{Sqlite, SqlitePool, Transaction};

pub async fn fetch_player_buildings(
    pool: &SqlitePool,
    player_id: i64,
) -> Result<Vec<BuildingRow>> {
    let buildings = sqlx::query_as::<_, BuildingRow>(
        r#"
        SELECT 
            b.id,
            b.player_id,
            b.building_type,
            b.planet_id,
            b.tile_id,
            b.level,
            b.hp,
            b.max_hp,
            b.under_attack,
            b.can_fly,
            b.flight_state,
            b.created_at,
            b.updated_at,
            b.destroyed_at,
            t.face,
            t.u,
            t.v
        FROM buildings b
        INNER JOIN planet_tiles t ON b.tile_id = t.id
        WHERE b.player_id = ? AND b.destroyed_at IS NULL
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await
    .inspect_err(|e| tracing::error!("Database error in fetch_player_buildings: {:?}", e))?;

    Ok(buildings)
}

pub async fn create_building(
    tx: &mut Transaction<'_, Sqlite>,
    player_id: i64,
    building_type: &str,
    planet_id: i64,
    tile_id: i64,
    hp: i32,
    max_hp: i32,
) -> Result<i64> {
    let res = sqlx::query(
        "INSERT INTO buildings (player_id, building_type, planet_id, tile_id, hp, max_hp)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(player_id)
    .bind(building_type)
    .bind(planet_id)
    .bind(tile_id)
    .bind(hp)
    .bind(max_hp)
    .execute(&mut **tx)
    .await
    .inspect_err(|e| tracing::error!("Database error in create_building: {:?}", e))?;

    Ok(res.last_insert_rowid())
}



// pub async fn fetch_planet_buildings(
//     pool: &SqlitePool, 
//     planet_id: i64
// ) -> Result<Vec<BuildingDto>> {
//     let buildings = sqlx::query!(
//         r#"
//         SELECT 
//             b.id, b.player_id, b.building_type, b.level, b.hp, b.max_hp, 
//             b.can_fly, b.flight_state,
//             t.face, t.u, t.v
//         FROM buildings b
//         INNER JOIN planet_tiles t ON b.tile_id = t.id
//         WHERE b.planet_id = ? AND b.destroyed_at IS NULL
//         "#,
//         planet_id
//     )
//     .fetch_all(pool)
//     .await?;

//     // Map to DTOs...
//     Ok(vec![])
// }