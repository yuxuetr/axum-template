use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct TokenRequest {
  #[validate(length(min = 3, max = 50))]
  pub username: String,
  #[validate(length(min = 6, max = 50))]
  pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
  pub token: String,
  pub token_type: Option<String>,
}

impl Default for TokenResponse {
  fn default() -> Self {
    Self {
      token: "".to_string(),
      token_type: Some("Bearer".to_string()),
    }
  }
}

impl TokenRequest {
  pub fn new(username: &str, password: &str) -> Self {
    Self {
      username: username.to_string(),
      password: password.to_string(),
    }
  }
}

impl TokenResponse {
  pub fn new(token: &str) -> Self {
    Self {
      token: token.to_string(),
      token_type: Some("Bearer".to_string()),
    }
  }
}
