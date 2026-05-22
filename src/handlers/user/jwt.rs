use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

use crate::{
    AppState,
    auth::{Claims, JWTAuthUser},
    errors::AppError,
    handlers::user::LoginUser,
    models::user::User,
};

/// example: `curl -X POST http://localhost:8000/jwt/new-token -H "Content-type:application/json" \
/// -d '{"id": 1, "password": "testuserpass"}'`
pub async fn create_access_token(
    State(state): State<Arc<AppState>>,
    Json(login_user): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", login_user.id)
        .fetch_one(&state.db_pool)
        .await?;
    // Validate login password
    if !user.verify_password(&login_user.password).map_err(|e| {
        tracing::error!("Failed to verify: {e}");
        AppError::Unauthorized
    })? {
        return Err(AppError::Unauthorized);
    }

    let header = Header::new(Algorithm::HS256);
    // Use env var for jwt secret key in production env!
    let encoding_key = EncodingKey::from_secret("supersecret".as_ref());
    let claim = Claims {
        sub: user.id.to_string(),
        exp: Utc::now().timestamp() + 3600,
    };
    let token = encode(&header, &claim, &encoding_key).map_err(|e| {
        tracing::error!("Failed to create token: {e}");
        AppError::Internal("Failed to create token.".to_string())
    })?;

    Ok((StatusCode::OK, token))
}

/// First get auth token with login function.
/// example: `curl http://localhost:8000/jwt/profile -H "Authorization: Bearer eyJ0..."`
pub async fn profile(
    State(state): State<Arc<AppState>>,
    auth_user: JWTAuthUser,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", auth_user.user_id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user))
}
