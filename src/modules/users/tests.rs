// #[allow(unused_imports)]
pub use super::*;
pub use crate::AppState;
pub use anyhow::Result;

#[test]
fn hash_and_verify_password_test() -> Result<()> {
  let input = "password";
  let hashed_password = hash_password(input).unwrap();
  assert_ne!(input, hashed_password);
  assert!(verify_password(input, &hashed_password)?);
  Ok(())
}

#[tokio::test]
async fn create_user_test() -> Result<()> {
  let (_tdb, state) = AppState::init_test_state().await?;
  let user = CreateUser::new("alice", "alice_password");
  let user = state.create_user(user).await?;
  assert_eq!(user.username, "alice");
  Ok(())
}

#[tokio::test]
async fn delete_user_test() -> Result<()> {
  let (_tdb, state) = AppState::init_test_state().await?;
  let user = CreateUser::new("bob", "bob_password");
  let user = state.create_user(user).await?;
  state.delete_user(user.id).await?;
  Ok(())
}

#[tokio::test]
async fn update_user_test() -> Result<()> {
  let (_tdb, state) = AppState::init_test_state().await?;
  let user = CreateUser::new("charlie", "charlie_password");
  let user = state.create_user(user).await?;
  let updated_user = UpdateUser::new("charlie_updated", "charlie_password_updated");
  let updated_user = state.update_user(user.id, updated_user).await?;
  assert_eq!(updated_user.username, "charlie_updated");
  Ok(())
}

#[tokio::test]
async fn get_users_test() -> Result<()> {
  let (_tdb, state) = AppState::init_test_state().await?;
  let user = CreateUser::new("dave", "dave_password");
  state.create_user(user).await?;
  let user = CreateUser::new("jonh", "dave_password");
  state.create_user(user).await?;
  let ret = state.get_users(2, 0).await?;
  assert_eq!(ret.users.len(), 2);
  let ret = state.get_users(1, 1).await?;
  assert_eq!(ret.users.len(), 1);
  assert_eq!(ret.total_count, 2);
  Ok(())
}

#[tokio::test]
async fn get_user_by_id_test() -> Result<()> {
  let (_tdb, state) = AppState::init_test_state().await?;
  let user = CreateUser::new("mike", "mike_password");
  let user = state.create_user(user).await?;
  let user = state.get_user_by_id(user.id).await?;
  assert_eq!(user.username, "mike");
  Ok(())
}

#[tokio::test]
async fn verify_user_test() -> Result<()> {
  let (_tdb, state) = AppState::init_test_state().await?;
  let user = CreateUser::new("nancy", "nancy_password");
  let user = state.create_user(user).await?;
  let is_valid = state.verify_user(&user.username, "nancy_password").await?;
  assert!(is_valid);
  Ok(())
}

#[cfg(test)]
impl CreateUser {
  pub fn new(username: &str, password: &str) -> Self {
    Self {
      username: username.to_string(),
      password: password.to_string(),
    }
  }
}

#[cfg(test)]
impl UpdateUser {
  pub fn new(username: &str, password: &str) -> Self {
    Self {
      username: Some(username.to_string()),
      password: Some(password.to_string()),
    }
  }
}
