use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::app::AppState;
use crate::api::dtos::Unit;
use crate::db;

// URL	                    Extractor
// /api/state/1	            Path<i64>
// /api/state?player_id=1	Query<StateQuery>
// /api/state/1/planet/3	Path<(i64, i64)>
pub async fn get_state(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<i64>,
) -> Result<Json<Vec<Unit>>, (StatusCode, String)> {
    let rows = db::queries::find_units(&state.db, player_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let units = rows.into_iter().map(Unit::from).collect();

    Ok(Json(units))
}




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



