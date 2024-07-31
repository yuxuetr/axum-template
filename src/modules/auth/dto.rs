use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenRequest {
  pub username: String,
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
