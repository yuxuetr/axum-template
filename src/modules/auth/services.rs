use super::TokenResponse;
use crate::AppError;
use crate::AppState;
use crate::common::{sign, verify_password};
use crate::modules::users::User;

impl AppState {
  pub async fn get_token(&self, username: &str, password: &str) -> Result<TokenResponse, AppError> {
    let user = self.verify_user(username, password).await?;
    let token = sign(user.user_info.id, &self.config)?;
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
