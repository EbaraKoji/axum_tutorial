use std::io;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new().route("/", get(|| async { "hello, world!" }));

    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}
