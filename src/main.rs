use std::{io, path::PathBuf};

use axum::{Router, extract::Path, routing::get};

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .route("/files/{*path}", get(serve_file));

    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

async fn get_user(Path(id): Path<String>) -> String {
    format!("User id: {id}")
}

async fn serve_file(Path(file_path): Path<PathBuf>) -> String {
    format!("Serving file {:?}", file_path.as_os_str())
}
