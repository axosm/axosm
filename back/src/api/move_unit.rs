// pub async fn handler(
//     State(state): State<Arc<AppState>>,
//     Json(req): Json<MoveRequest>,
// ) -> ApiResult<Json<MoveResponse>> {
//     let unit = db::queries::find_unit(&state.db, req.unit_id)
//         .await?
//         .ok_or(ApiError::BadRequest("unit not found"))?;

//     domain::movement::validate_ownership(&unit, req.player_id)?;

//     let arrival = domain::movement::arrival_time();
//     db::queries::insert_move_order(&state.db, &unit, &req, arrival).await?;

//     Ok(Json(MoveResponse { arrival }))
// }