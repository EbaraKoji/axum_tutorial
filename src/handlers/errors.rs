use axum::response::IntoResponse;

use crate::errors::AppError;

pub async fn not_works() -> impl IntoResponse {
    AppError::Internal("This always fails!".to_string()).into_response()
}
