use sqlx::SqlitePool;

use crate::db::models::UnitWithPlanetLocationRow;

pub async fn find_units(
    db: &SqlitePool,
    player_id: i64,
) -> sqlx::Result<Vec<UnitWithPlanetLocationRow>> {
    sqlx::query_as::<_, UnitWithPlanetLocationRow>(
        r#"
        SELECT
            u.id,
            u.player_id,
            u.unit_type,
            u.location_type,
            pl.planet_id,
            pl.face,
            pl.u,
            pl.v
        FROM units u
        LEFT JOIN unit_planet_locations pl
            ON pl.unit_id = u.id
        WHERE u.player_id = ?
        "#
    )
    .bind(player_id)
    .fetch_all(db)
    .await
}



// pub async fn find_unit(
//     db: &SqlitePool,
//     unit_id: i64,
// ) -> sqlx::Result<Option<Unit>> {
//     sqlx::query_as(
//         "SELECT id, player_id, x, y FROM units WHERE id = ?"
//     )
//     .bind(unit_id)
//     .fetch_optional(db)
//     .await
// }