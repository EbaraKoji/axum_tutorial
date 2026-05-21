#![allow(unused)]
use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    BoxError, Json, Router,
    error_handling::HandleErrorLayer,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, prelude::FromRow};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();
    // run `export DATABASE_URL=sqlite:sqlite.db` and `sqlx migrate run` before starting the server.

    let db_url = "sqlite:sqlite.db";
    // if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
    //     println!("Creating database {db_url}");
    //     match Sqlite::create_database(db_url).await {
    //         Ok(_) => println!("Create db success"),
    //         Err(e) => panic!("error: {e}"),
    //     }
    // }
    let db_pool = SqlitePool::connect(db_url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    // sqlx::migrate!("./migrations")
    //     .run(&db_pool)
    //     .await
    //     .expect("could not run sqlx migration");
    let app_state = Arc::new(AppState { db_pool });
    let app = Router::new()
        .route("/fail", get(not_works))
        .route("/user", post(create_user))
        .route("/user/{id}", get(get_user))
        .layer(axum::middleware::from_fn(timing_middleware))
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

async fn timing_middleware(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    // some time-consuming respnose...
    let response = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!("{method} {uri} -> {} in {duration:?}", response.status());

    response
}
