use anyhow::Result;

use super::TokenResponse;
use crate::AppState;
use crate::common::{sign, verify_password};
use crate::{AppError, modules::users::User};

impl AppState {
  pub async fn get_token(&self, username: &str, password: &str) -> Result<TokenResponse> {
    let user = self.verify_user(username, password).await?;
    let token = sign(user, &self.config)?;
    Ok(TokenResponse::new(&token))
  }

  pub async fn verify_user(&self, username: &str, password: &str) -> Result<User, AppError> {
    let user = self.verify_user_by_username(username).await?;
    if verify_password(password, &user.user_info.password)? {
      Ok(user)
    } else {
      Err(AppError::PasswordError("Invalid password".to_string()))
    }
  }
}
