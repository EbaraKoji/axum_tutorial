use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{AppState, errors::AppError, models::user::User};

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    email: String,
    password: String,
}

/// example: curl -X POST http://localhost:8000/user -H "Content-type:application/json" \
/// -d '{"username": "Ebara", "email": "ebara@example.com", "password": "testuserpass"}'
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let password_hash = bcrypt::hash(&user.password, 5).map_err(|e| {
        tracing::error!("Bcrypt hashing failed: {e}");
        AppError::Internal("Failed to create user".to_string())
    })?;
    sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind(user.username)
        .bind(user.email)
        .bind(password_hash)
        .execute(&state.db_pool)
        .await?;

    Ok((StatusCode::CREATED, "New user created."))
}

/// example: curl http://localhost:8000/user/1
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user))
}
