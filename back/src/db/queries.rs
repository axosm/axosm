pub async fn find_unit(
    db: &SqlitePool,
    unit_id: i64,
) -> sqlx::Result<Option<Unit>> {
    sqlx::query_as(
        "SELECT id, player_id, x, y FROM units WHERE id = ?"
    )
    .bind(unit_id)
    .fetch_optional(db)
    .await
}