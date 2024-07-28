// #[allow(unused_imports)]
#[cfg(test)]
mod util_tests {
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

  #[tokio::test]
  #[serial]
  async fn create_user_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:39999").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
      axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    });

    let client = Client::new();
    let response = client
      .post(format!("http://{}/users", addr))
      .json(&json!({"username": "alice", "password": "alice_password"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    Ok(())
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
    let response = client
      .post(format!("http://{}/users", addr))
      .json(&json!({"username": "bob", "password": "bob_password"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let user: serde_json::Value = response.json().await?;
    let user_id = user["id"].as_i64().expect("user id not found");

    let response = client
      .delete(format!("http://{}/users/{}", addr, user_id))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let get_response = client
      .get(format!("http://{}/users/{}", addr, user_id))
      .send()
      .await?;
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
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
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let client = Client::new();

    for i in 1..=2 {
      let response = client
        .post(format!("http://{}/users", addr))
        .json(&json!({"username": format!("user{}", i), "password": "password"}))
        .send()
        .await?;
      assert_eq!(response.status(), StatusCode::CREATED);
    }

    let response = client
      .get(format!("http://{}/users?limit=1&offset=1", addr))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let users_page: serde_json::Value = response.json().await?;
    assert_eq!(users_page["users"].as_array().unwrap().len(), 1);
    assert_eq!(users_page["total_count"].as_i64().unwrap(), 2);

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
    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = Client::new();

    let response = client
      .post(format!("http://{}/users", addr))
      .json(&json!({"username": "alice", "password": "alice_password"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    let user: serde_json::Value = response.json().await?;
    let user_id = user["id"].as_i64().expect("user id not found");

    let response = client
      .get(format!("http://{}/users/{}", addr, user_id))
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

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = Client::new();

    let response = client
      .post(format!("http://{}/users", addr))
      .json(&json!({"username": "alice", "password": "alice_password"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let user: serde_json::Value = response.json().await?;
    let user_id = user["id"].as_i64().expect("user id not found");

    let response = client
      .patch(format!("http://{}/users/{}", addr, user_id))
      .json(&json!({"username": "alice_updated", "password": "alice_password_updated"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_user: serde_json::Value = response.json().await?;
    assert_eq!(&updated_user["username"], "alice_updated");

    tx.send(()).unwrap();

    Ok(())
  }
}
