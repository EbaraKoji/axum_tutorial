use std::io;

use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new().route("/users", get(list_users));
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

enum ApiResponse {
    Ok,
    Created,
    JsonData(Vec<User>),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok => StatusCode::OK.into_response(),
            Self::Created => StatusCode::CREATED.into_response(),
            Self::JsonData(data) => (StatusCode::OK, Json(data)).into_response(),
        }
    }
}

async fn list_users() -> ApiResponse {
    ApiResponse::JsonData(vec![
        User {
            id: 1,
            name: "Alice".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
        },
    ])
}
