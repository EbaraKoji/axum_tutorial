use std::{io, sync::Arc};

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();
    // run `export DATABASE_URL=sqlite:sqlite.db` and `sqlx migrate run` before starting the server.

    let db_url = "sqlite:sqlite.db";
    let db_pool = SqlitePool::connect(db_url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let app_state = Arc::new(AppState { db_pool });
    let app = Router::new()
        .route("/fail", get(not_works))
        .route("/user", post(create_user))
        .route("/user/{id}", get(get_user))
        .route("/login", post(login_user))
        .route("/profile", get(profile))
        .with_state(app_state);
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                )
            }
        };

        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

async fn not_works() -> impl IntoResponse {
    AppError::Internal("This always fails!".to_string()).into_response()
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Internal(err.to_string()),
        }
    }
}

#[derive(Clone)]
struct AppState {
    db_pool: SqlitePool,
}

#[derive(Serialize, Deserialize, FromRow)]
struct User {
    id: i64,
    username: String,
    email: String,
}

/// example: curl http://localhost:8000/user/1'
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

/// example: curl -X POST http://localhost:8000/user -H "Content-type:application/json" \
/// -d '{"username": "Ebara", "email": "ebara@example.com"}'
async fn create_user(
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

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

struct AuthUser {
    user_id: String,
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

#[derive(Deserialize)]
struct LoginUser {
    id: i64,
    password: String,
}

/// example: `curl -X POST http://localhost:8000/login -H "Content-type:application/json" \
/// -d '{"id": 1, "password": "testuserpass"}'`
async fn login_user(
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

/// First get auth token with login function.
/// example: `curl http://localhost:8000/profile -H "Authorization: Bearer eyJ0..."`
async fn profile(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", auth_user.user_id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user))
}
