use std::io;

use axum::{Router, extract::FromRequestParts, http::StatusCode, routing::get};

#[tokio::main]
async fn main() -> io::Result<()> {
    let app = Router::new().route("/user", get(profile));
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

struct CurrentUser {
    user_id: u64,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user_id = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(CurrentUser { user_id })
    }
}

/// Get Current User.
/// example: `curl http://localhost:8000/user -H "x-user-id: 5"`
async fn profile(user: CurrentUser) -> String {
    format!("User #{}", user.user_id)
}
