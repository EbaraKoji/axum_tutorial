use std::{io, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;

use axum_tutorial::{
    AppState,
    handlers::{
        errors::not_works,
        user::{create_user, get_user, login_user, profile},
    },
};

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
