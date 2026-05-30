use crate::app::AppState;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use std::sync::Arc;

// ALWAYS compiled: This guarantees AuthPlayer is a local type under all flag variations.
pub struct AuthPlayer(pub i64);

// ==========================================
// LOCAL BUILD ONLY: SQLite + Session Key
// ==========================================
// TODO uncomment
// #[cfg(feature = "local_mode")]
impl<S> FromRequestParts<S> for AuthPlayer
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let session_key = parts
            .headers
            .get("X-Session-Key")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing X-Session-Key header".to_string(),
                )
            })?;

        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM players WHERE session_key = ?")
                .bind(session_key)
                .fetch_optional(&app_state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(row) = existing {
            return Ok(AuthPlayer(row.0));
        }

        let new_id = sqlx::query("INSERT INTO players (session_key, username) VALUES (?, ?)")
            .bind(session_key)
            .bind(format!("Commander_{}", &session_key[..6]))
            .execute(&app_state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .last_insert_rowid();

        Ok(AuthPlayer(new_id))
    }
}

// ==========================================
// PRODUCTION BUILD ONLY: Postgres + JWT
// ==========================================
#[cfg(feature = "prod_mode")]
impl<S> FromRequestParts<S> for AuthPlayer
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing Token".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::BAD_REQUEST, "Invalid Auth Format".to_string()));
        }

        let token = &auth_header[7..];

        // Placeholder for your production JWT verification logic
        let player_id = todo!("Your production JWT verification logic");

        Ok(AuthPlayer(player_id))
    }
}
