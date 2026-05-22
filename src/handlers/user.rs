use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Deserialize;

use crate::{
    AppState,
    auth::{AuthUser, Claims},
    errors::AppError,
    models::user::User,
};

#[derive(Deserialize)]
pub struct LoginUser {
    id: i64,
    password: String,
}

/// example: `curl -X POST http://localhost:8000/login -H "Content-type:application/json" \
/// -d '{"id": 1, "password": "testuserpass"}'`
pub async fn login_user(
    State(state): State<Arc<AppState>>,
    Json(login_user): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    // Vaidate password: use hashed password in production env!
    if login_user.password != "testuserpass" {
        return Err(AppError::Unauthorized);
    }
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", login_user.id)
        .fetch_one(&state.db_pool)
        .await?;
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

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    email: String,
}

/// example: curl -X POST http://localhost:8000/user -H "Content-type:application/json" \
/// -d '{"username": "Ebara", "email": "ebara@example.com"}'
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("INSERT INTO users (username, email) VALUES (?, ?)")
        .bind(user.username)
        .bind(user.email)
        .execute(&state.db_pool)
        .await?;

    Ok((StatusCode::CREATED, "New user created."))
}

/// example: curl http://localhost:8000/user/1'
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user))
}

pub async fn profile(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", auth_user.user_id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user))
}
