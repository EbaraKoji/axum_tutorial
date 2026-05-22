use axum::{
    extract::{FromRequestParts, Request},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use axum_session::{Key, SessionConfig, SessionStore};
use axum_session_auth::AuthSession;
use axum_session_sqlx::SessionSqlitePool;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub struct JWTAuthUser {
    pub user_id: String,
}

impl<S> FromRequestParts<S> for JWTAuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Use env var for jwt secret key in production env!
        let token_data = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(b"supersecret"),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("Authorization failed: {e}");
            StatusCode::UNAUTHORIZED
        })?;

        Ok(Self {
            user_id: token_data.claims.sub,
        })
    }
}

pub async fn create_session_store(pool: Pool<Sqlite>) -> SessionStore<SessionSqlitePool> {
    let session_config = SessionConfig::default()
        .with_table_name("session_table")
        .with_key(Key::generate());
    let session_store =
        SessionStore::<SessionSqlitePool>::new(Some(pool.clone().into()), session_config)
            .await
            .unwrap();

    session_store
}

pub type AuthSessionUser = AuthSession<User, i64, SessionSqlitePool, SqlitePool>;

pub async fn auth_session(
    auth: AuthSessionUser,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    if auth.is_authenticated() {
        let user = auth.current_user.unwrap().clone();
        req.extensions_mut().insert(user);
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED, "Not logged in.").into_response()
    }
}
