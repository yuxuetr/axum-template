use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, FromRow, Serialize)]
pub struct User {
  pub id: i32,
  pub username: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl User {
  pub fn new(username: &str, password: &str) -> Self {
    Self {
      id: 0,
      username: username.to_string(),
      password: password.to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}
