use std::{io, sync::Arc};

use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post},
};
use axum_session::SessionLayer;
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionSqlitePool;
use sqlx::SqlitePool;

use axum_tutorial::{
    AppState,
    auth::{auth_session, create_session_store},
    handlers::{
        errors::not_works,
        user::{
            crud::{create_user, get_user},
            jwt::{create_access_token, profile as profile_with_jwt},
            session::{login, logout, profile as profile_with_session},
        },
    },
    models::user::User,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();
    // run `export DATABASE_URL=sqlite:sqlite.db` and `sqlx migrate run` before starting the server.

    let auth_config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(0));

    let db_url = "sqlite:sqlite.db";
    let db_pool = SqlitePool::connect(db_url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let session_store = create_session_store(db_pool.clone()).await;

    let app_state = Arc::new(AppState {
        db_pool: db_pool.clone(),
    });
    let app = Router::new()
        .route("/fail", get(not_works))
        .route("/user", post(create_user))
        .route("/user/{id}", get(get_user))
        .route("/jwt/new-token", post(create_access_token))
        .route("/jwt/profile", get(profile_with_jwt))
        .route("/session/login", post(login))
        .route(
            "/session/profile",
            get(profile_with_session).route_layer(from_fn(auth_session)),
        )
        .route("/session/logout", post(logout))
        // auth session layer
        .layer(
            AuthSessionLayer::<User, i64, SessionSqlitePool, SqlitePool>::new(Some(db_pool))
                .with_config(auth_config),
        )
        // session layer
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}
