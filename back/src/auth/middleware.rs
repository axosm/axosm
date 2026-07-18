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

        // 1. Get the key the TypeScript app sent
        let session_key = parts
            .headers
            .get("X-Session-Key")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing X-Session-Key".to_string(),
                )
            })?;

        // 2. FAST PATH: Check if they are already in the database (99% of requests)
        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM players WHERE session_key = ?")
                .bind(session_key)
                .fetch_optional(&app_state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(row) = existing {
            return Ok(AuthPlayer(row.0)); // Found them! Return their ID.
        }

        // 3. SLOW PATH: This is their very first connection. Automatically create them.
        let username_len = std::cmp::min(6, session_key.len());
        let default_username = format!("Player_{}", &session_key[..username_len]);

        let player_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO players (session_key, username)
            VALUES (?, ?)
            ON CONFLICT(session_key) DO UPDATE SET session_key = excluded.session_key
            RETURNING id
            "#,
        )
        .bind(session_key)
        .bind(default_username)
        .fetch_one(&app_state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(AuthPlayer(player_id))
    }
}

// TypeScript
// // On game startup:
// let sessionKey = localStorage.getItem("game_session_key");

// if (!sessionKey) {
//     // Generate a simple unique random string
//     sessionKey = Math.random().toString(36).substring(2) + Date.now().toString(36);
//     localStorage.setItem("game_session_key", sessionKey);
// }

// // Every time you talk to your Axum API:
// async function sendGameAction(actionData: any) {
//     const response = await fetch("/api/action", {
//         method: "POST",
//         headers: {
//             "Content-Type": "application/json",
//             "X-Session-Key": sessionKey // <--- Always include this!
//         },
//         body: JSON.stringify(actionData)
//     });
//     return response.json();
// }

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
