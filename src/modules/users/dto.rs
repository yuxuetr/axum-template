use super::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateUser {
  pub username: String,
  pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
  pub username: Option<String>,
  pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
  pub limit: i64,
  pub offset: i64,
}

impl Default for PaginationParams {
  fn default() -> Self {
    Self {
      limit: 10,
      offset: 0,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct PaginatedUsers {
  pub users: Vec<User>,
  pub total_count: i64,
}
