

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::sync::Arc;
use crate::AppState;

// ── JWT extractor ─────────────────────────────────────────────

pub struct AuthPlayer(pub i64);

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for AuthPlayer {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        use crate::models::auth::Claims;

        let auth_header = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

        let token = auth_header.strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Malformed Authorization header"))?;

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "change_me_in_prod".into());

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        Ok(AuthPlayer(data.claims.sub))
    }
}
