// uncomment this file for prod / postgres / jwt
// use axum::{Json, extract::State, http::StatusCode};
// use bcrypt::{DEFAULT_COST, hash, verify};
// use chrono::Utc;
// use jsonwebtoken::{EncodingKey, Header, encode};
// use std::sync::Arc;

// use serde::{Deserialize, Serialize};

// use crate::app::AppState;

// #[derive(Debug, Serialize)]
// pub struct AuthResponse {
//     pub token: String,
//     pub player_id: i64,
//     pub username: String,
// }

// #[derive(Debug, Deserialize)]
// pub struct RegisterRequest {
//     pub username: String,
//     pub email: String,
//     pub password: String,
// }

// pub async fn register(
//     State(state): State<Arc<AppState>>,
//     Json(req): Json<RegisterRequest>,
// ) -> Result<Json<AuthResponse>, (StatusCode, String)> {
//     let password_hash = hash(&req.password, DEFAULT_COST)
//         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     let player_id = sqlx::query_scalar::<_, i64>(
//         "INSERT INTO players (username, email, password_hash)
//          VALUES (?, ?, ?) RETURNING id",
//     )
//     .bind(&req.username)
//     .bind(&req.email)
//     .bind(&password_hash)
//     .fetch_one(&state.db)
//     .await
//     .map_err(|e| (StatusCode::CONFLICT, e.to_string()))?;

//     let token =
//         make_token(player_id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     Ok(Json(AuthResponse {
//         token,
//         player_id,
//         username: req.username,
//     }))
// }

// #[derive(Debug, Deserialize)]
// pub struct LoginRequest {
//     pub email: String,
//     pub password: String,
// }

// pub async fn login(
//     State(state): State<Arc<AppState>>,
//     Json(req): Json<LoginRequest>,
// ) -> Result<Json<AuthResponse>, (StatusCode, String)> {
//     let player =
//         sqlx::query_as::<_, crate::db::player::Player>("SELECT * FROM players WHERE email = ?")
//             .bind(&req.email)
//             .fetch_optional(&state.db)
//             .await
//             .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
//             .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

//     let valid = verify(&req.password, &player.password_hash)
//         .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

//     if !valid {
//         return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
//     }

//     // Update last_login_at
//     let _ = sqlx::query("UPDATE players SET last_login_at = ? WHERE id = ?")
//         .bind(Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
//         .bind(player.id)
//         .execute(&state.db)
//         .await;

//     let token =
//         make_token(player.id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     Ok(Json(AuthResponse {
//         token,
//         player_id: player.id,
//         username: player.username,
//     }))
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Claims {
//     pub sub: i64,   // player_id
//     pub exp: usize, // expiry unix ts
// }

// fn make_token(player_id: i64) -> anyhow::Result<String> {
//     let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "change_me_in_prod".into());
//     let exp = (Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
//     let claims = Claims {
//         sub: player_id,
//         exp,
//     };
//     let token = encode(
//         &Header::default(),
//         &claims,
//         &EncodingKey::from_secret(secret.as_bytes()),
//     )?;
//     Ok(token)
// }
