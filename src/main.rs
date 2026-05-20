use std::io;

use axum::{
    Router,
    extract::{FromRef, State},
    routing::get,
};

#[derive(Clone)]
struct ApiState {
    app_name: String,
}

#[derive(Clone)]
struct AdminState {
    config: String,
}

#[derive(Clone)]
struct AppState {
    api_state: ApiState,
    admin_state: AdminState,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let app_state = AppState {
        api_state: ApiState {
            app_name: "Awsome Axum".to_string(),
        },
        admin_state: AdminState {
            config: "Secret Config".to_string(),
        },
    };
    let app = Router::new()
        .route("/public", get(public_endpoint))
        .route("/admin", get(admin_endpoint))
        .with_state(app_state);
    let endpoint = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    println!("Listening on {endpoint}...");

    axum::serve(listener, app).await
}

async fn public_endpoint(State(api): State<ApiState>) -> String {
    format!("App Name: {}", api.app_name)
}

async fn admin_endpoint(State(admin): State<AdminState>) -> String {
    format!("App Config: {}", admin.config)
}

impl FromRef<AppState> for ApiState {
    fn from_ref(app: &AppState) -> Self {
        app.api_state.clone()
    }
}

impl FromRef<AppState> for AdminState {
    fn from_ref(app: &AppState) -> Self {
        app.admin_state.clone()
    }
}
