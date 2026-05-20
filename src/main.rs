use std::{
    io,
    sync::{Arc, Mutex},
};

use axum::{Router, extract::State, routing::post};

#[derive(Clone)]
struct AppState {
    count: Arc<Mutex<i32>>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let app_state = AppState {
        count: Arc::new(Mutex::new(1)),
    };
    let app = Router::new()
        .route("/increment", post(increment))
        .route("/decrement", post(decrement))
        .with_state(app_state);
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

async fn increment(State(state): State<AppState>) -> String {
    *state.count.lock().unwrap() += 1;
    format!("updated count to {}", state.count.lock().unwrap())
}

async fn decrement(State(state): State<AppState>) -> String {
    *state.count.lock().unwrap() -= 1;
    format!("updated count to {}", state.count.lock().unwrap())
}
