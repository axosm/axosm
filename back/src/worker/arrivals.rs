// pub async fn process(
//     order: MoveOrder,
//     state: Arc<AppState>,
// ) -> anyhow::Result<()> {
//     db::queries::apply_movement(&state.db, &order).await?;
//     domain::encounter::check_and_emit(&state.db, &state.notify, &order).await?;
//     Ok(())
// }