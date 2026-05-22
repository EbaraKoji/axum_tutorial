use std::sync::Arc;

use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    AppState, auth::AuthSessionUser, errors::AppError, handlers::user::LoginUser,
    models::user::User,
};

// If using curl, you need to store the cookie in some file such as cookie.txt.

/// example: `curl -c cookie.txt http://localhost:8000/session/login \
/// -H "Content-type:application/json" -d '{"id": 1, "password": "testuserpass"}'`
pub async fn login(
    auth: AuthSessionUser,
    State(state): State<Arc<AppState>>,
    Json(user_req): Json<LoginUser>,
) -> impl IntoResponse {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", user_req.id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user id {}: {e}", user_req.id);
            AppError::Unauthorized
        })?;
    if !user.verify_password(&user_req.password).map_err(|e| {
        tracing::error!("Failed to verify user password: {e}");
        AppError::Internal("Failed to login.".to_string())
    })? {
        return Err(AppError::Unauthorized);
    }

    auth.login_user(user.id);
    Ok((StatusCode::OK, "Successfully Logged in"))
}

/// example: `curl -b cookie.txt http://localhost:8000/session/profile`
pub async fn profile(Extension(user): Extension<User>) -> impl IntoResponse {
    Json(user)
}

/// example: `curl -X POST -b cookie.txt http://localhost:8000/session/logout`
pub async fn logout(auth: AuthSessionUser) -> impl IntoResponse {
    auth.logout_user();
    (StatusCode::OK, "Log out successful!").into_response()
}
