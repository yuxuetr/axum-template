use serde::Deserialize;

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
