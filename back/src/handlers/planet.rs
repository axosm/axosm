// // handlers/planet.rs
// pub async fn get_planet(
//     State(state): State<Arc<AppState>>,
//     auth: AuthPlayer,
//     Path(planet_id): Path<i64>,
// ) -> Result<Json<PlanetResponse>, AppError> {
//     let response = planet_service::get_planet(&state.db, auth.0, planet_id).await?;
//     Ok(Json(response))
// }