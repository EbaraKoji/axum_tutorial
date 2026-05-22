use serde::Deserialize;

pub mod crud;
pub mod jwt;
pub mod session;

#[derive(Deserialize)]
pub struct LoginUser {
    id: i64,
    password: String,
}
