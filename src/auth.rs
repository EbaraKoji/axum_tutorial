use axum::{extract::FromRequestParts, http::StatusCode};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub struct AuthUser {
    pub user_id: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Use env var for jwt secret key in production env!
        let token_data = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(b"supersecret"),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("Authorization failed: {e}");
            StatusCode::UNAUTHORIZED
        })?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
        })
    }
}
