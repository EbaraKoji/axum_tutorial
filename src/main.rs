use std::io;

use axum::{Router, extract::Path, routing::get};

#[tokio::main]
async fn main() -> io::Result<()> {
    let user_routes = Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user));

    let app = Router::new().nest("/api/users", user_routes);

    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

async fn list_users() -> &'static str {
    "List users"
}

async fn create_user() -> &'static str {
    "Create user"
}

async fn get_user(Path(id): Path<String>) -> String {
    format!("User id: {id}")
}

async fn update_user(Path(id): Path<String>) -> String {
    format!("Updating user(id: {id})")
}

async fn delete_user(Path(id): Path<String>) -> String {
    format!("Delete user(id: {id})")
}
