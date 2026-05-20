use std::io;

use axum::{
    Json, Router,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new()
        .route("/plain", get(plain))
        .route("/no-content", get(no_content))
        .route("/json", get(json))
        .route("/page", get(page))
        .route("/create", post(created));
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

/// Returns plain text.
async fn plain() -> &'static str {
    "Hello"
}

/// Returns Status code only.
async fn no_content() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Returns JSON.
async fn json() -> Json<serde_json::Value> {
    Json(serde_json::json!({"message": "Hello"}))
}

/// Returns HTML.
async fn page() -> Html<&'static str> {
    Html("<h1>Hello</h1>")
}

/// Returns Tuple: (StatusCode, Body)
async fn created() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::CREATED, Json(serde_json::json!({"id": 1})))
}
