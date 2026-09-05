// src/repositories/tiles_repo.rs
use anyhow::Result;
use sqlx::{FromRow, SqlitePool};

use crate::db::tile::TileRow;

// #[derive(Debug, Clone, FromRow)]
// pub struct TileRow {
//     pub id: i64,
//     pub planet_id: i64,
//     pub face: i32,
//     pub u: i32,
//     pub v: i32,
//     pub tile_type: String,
//     pub yield_quality: f64,
//     pub rare_deposit: Option<String>,
//     pub owner_player_id: Option<i64>,
// }

pub async fn fetch_tiles_by_coordinates(
    pool: &SqlitePool,
    planet_id: i64,
    coords: &[(u8, u32, u32)],
) -> Result<Vec<TileRow>> {
    if coords.is_empty() {
        return Ok(Vec::new());
    }

    // Build a dynamic query matching (face, u, v) tuples
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, planet_id, face, u, v, tile_type, yield_quality, rare_deposit, owner_player_id FROM planet_tiles WHERE planet_id = "
    );
    query_builder.push_bind(planet_id);
    query_builder.push(" AND (face, u, v) IN ");

    query_builder.push_tuples(coords, |mut b, &(face, u, v)| {
        b.push_bind(face as i32)
         .push_bind(u as i32)
         .push_bind(v as i32);
    });

    let tiles = query_builder.build_query_as::<TileRow>()
        .fetch_all(pool)
        .await?;

    Ok(tiles)
}