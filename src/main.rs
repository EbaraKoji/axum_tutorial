use std::{io, sync::Arc};

use axum::{
    Form, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new()
        .route("/user/{id}", get(get_user))
        .route("/comment/{post_id}/{comment_id}", get(get_comment))
        .route("/items", get(list_items))
        .route("/user", post(create_user))
        .route("/login", post(login));
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

async fn get_user(Path(id): Path<u64>) -> String {
    format!("User #{id}")
}

async fn get_comment(Path((post_id, comment_id)): Path<(u64, u64)>) -> String {
    format!("Post {post_id}, Comment {comment_id}")
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

/// List items by query parameters.
/// example: `curl "http://localhost:8000/items?page=2&per_page=15"
async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    format!("Page {page}, {per_page} items")
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

/// Create user using Json data.
/// exmaple: `curl -X POST http://localhost:8000/user -H "Content-Type: application/json" -d '{"name": "Ebara", "email": "ebara@example.com"}'`
async fn create_user(Json(input): Json<CreateUser>) -> String {
    format!("Created user: {} ({})", input.name, input.email)
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

/// Example login
/// exmaple: `curl -X POST http://localhost:8000/login -d 'username=Ebara&password=supersecret'`
async fn login(Form(input): Form<LoginForm>) -> impl IntoResponse {
    if input.password != "supersecret" {
        return (StatusCode::UNAUTHORIZED, "Login failed.".to_string());
    }
    (StatusCode::OK, format!("Login attempt: {}", input.username))
}

#[derive(Deserialize)]
struct UpdateItem {}

#[derive(Clone)]
struct AppState {}

/// Multiple extractors in one handler
async fn update_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Query(params): Query<Pagination>,
    Json(body): Json<UpdateItem>,
) -> impl IntoResponse {
    // ...
    StatusCode::OK
}
