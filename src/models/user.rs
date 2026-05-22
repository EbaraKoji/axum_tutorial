use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        bcrypt::verify(password, &self.password_hash)
    }
}
