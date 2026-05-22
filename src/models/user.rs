use async_trait::async_trait;
use axum_session_auth::Authentication;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        // Always return false with anonymous user
        if self.id == 0 {
            return Ok(false);
        }
        bcrypt::verify(password, &self.password_hash)
    }
}

#[async_trait]
impl Authentication<User, i64, SqlitePool> for User {
    async fn load_user(id: i64, pool: Option<&SqlitePool>) -> Result<Self, anyhow::Error> {
        // anonymous user
        if id == 0 {
            return Ok(Self {
                id: 0,
                username: "guest".to_string(),
                email: "guest@example.com".to_string(),
                password_hash: bcrypt::hash("guest", 4).unwrap(),
            });
        }

        match pool {
            Some(pool) => {
                let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id= $1", id)
                    .fetch_one(pool)
                    .await?;
                Ok(user)
            }
            None => Err(std::io::Error::other("Cannot find any db pool").into()),
        }
    }

    fn is_active(&self) -> bool {
        true
    }

    fn is_anonymous(&self) -> bool {
        self.id > 0
    }

    fn is_authenticated(&self) -> bool {
        true
    }
}
