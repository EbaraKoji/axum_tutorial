use sqlx::SqlitePool;

pub mod auth;
pub mod errors;
pub mod handlers;
pub mod models;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
}
