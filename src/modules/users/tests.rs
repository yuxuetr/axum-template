// #[allow(unused_imports)]
#[cfg(test)]
mod util_tests {
  pub use crate::common::auth::*;
  pub use crate::modules::users::*;
  pub use crate::AppState;
  pub use anyhow::Result;
  use serial_test::serial;

  #[test]
  #[serial]
  fn hash_and_verify_password_test() -> Result<()> {
    let input = "password";
    let hashed_password = hash_password(input).unwrap();
    assert_ne!(input, hashed_password);
    assert!(verify_password(input, &hashed_password)?);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn create_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("alice", "alice_password");
    let user = state.create_user(user).await?;
    assert_eq!(user.username, "alice");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn delete_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("bob", "bob_password");
    let user = state.create_user(user).await?;
    state.delete_user(user.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
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
  #[serial]
  async fn get_users_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("dave", "123456");
    state.create_user(user).await?;
    let user = CreateUser::new("jonh", "123456");
    state.create_user(user).await?;
    let ret = state.get_users(2, 0).await?;
    assert_eq!(ret.users.len(), 2);
    let ret = state.get_users(1, 1).await?;
    assert_eq!(ret.users.len(), 1);
    assert_eq!(ret.total_count, 12);
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_user_by_id_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("mike", "mike_password");
    let user = state.create_user(user).await?;
    let user = state.get_user_by_id(user.id).await?;
    assert_eq!(user.username, "mike");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn verify_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = CreateUser::new("nancy", "nancy_password");
    let user = state.create_user(user).await?;
    let user = state.verify_user(&user.username, "nancy_password").await?;
    assert_eq!(user.username, "nancy");
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
}

#[cfg(test)]
mod integration_tests {
  use crate::{get_router, AppState};
  use anyhow::Result;
  use axum::http::StatusCode;
  use axum::Router;
  use reqwest::Client;
  use serde_json::json;
  use serial_test::serial;
  use sqlx_db_tester::TestPg;
  use tokio::net::TcpListener;
  use tokio::sync::oneshot;
  use tokio::time::Duration;

  async fn setup_test_app() -> Result<(TestPg, Router)> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let app = get_router(state).await?;
    Ok((_tdb, app))
  }

  async fn get_token(client: &Client, addr: &str, username: &str) -> Result<String> {
    let response = client
      .post(format!("http://{}/auth/signin", addr))
      .json(&json!({"username": username, "password": "123456"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let token: serde_json::Value = response.json().await?;
    let token = token["token"].as_str().unwrap().to_string();

    Ok(token)
  }

  #[tokio::test]
  #[serial]
  async fn delete_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:40000").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    // 2, bob, 123456
    let token = get_token(&client, &addr.to_string(), "bob").await?;

    // delete user
    let response = client
      .delete(format!("http://{}/users/{}", addr, 2))
      .header("Authorization", format!("Bearer {}", &token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    tx.send(()).unwrap();
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_users_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:40003").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let token = get_token(&client, &addr.to_string(), "alice").await?;

    let response = client
      .get(format!("http://{}/users?limit=1&offset=1", addr))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let users_page: serde_json::Value = response.json().await?;
    assert_eq!(users_page["users"].as_array().unwrap().len(), 1);
    assert_eq!(users_page["total_count"].as_i64().unwrap(), 10);

    tx.send(()).unwrap();
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn get_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:40002").await?;
    let addr = listener.local_addr()?;

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let token = get_token(&client, &addr.to_string(), "alice").await?;

    let response = client
      .get(format!("http://{}/users/{}", addr, 1))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let retrieved_user: serde_json::Value = response.json().await?;
    assert_eq!(&retrieved_user["username"], "alice");

    tx.send(()).unwrap();

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn update_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:40004").await?;
    let addr = listener.local_addr()?;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app)
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = Client::new();
    let token = get_token(&client, &addr.to_string(), "charlie").await?;

    let response = client
      .patch(format!("http://{}/users/{}", addr, 3))
      .json(&json!({"username": "charlie_updated", "password": "123456"}))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_user: serde_json::Value = response.json().await?;
    assert_eq!(&updated_user["username"], "charlie_updated");

    tx.send(()).unwrap();

    Ok(())
  }
}
