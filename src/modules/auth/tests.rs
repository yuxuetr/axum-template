#[cfg(test)]
mod util_tests {
  pub use crate::AppState;
  pub use anyhow::Result;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn get_token_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let token = state.get_token("alice", "123456").await?;
    assert!(!token.token.is_empty());
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn verify_user_test() -> Result<()> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let user = state.verify_user("alice", "123456").await?;
    assert_eq!(user.user_info.username, "alice");
    Ok(())
  }
}

#[cfg(test)]
mod integration_tests {
  use crate::{get_router, AppState};
  use anyhow::Result;
  use axum::{http::StatusCode, Router};
  use reqwest::Client;
  use serde_json::json;
  use serial_test::serial;
  use sqlx_db_tester::TestPg;
  use tokio::net::TcpListener;
  use tokio::sync::oneshot;

  async fn setup_test_app() -> Result<(TestPg, Router)> {
    let (_tdb, state) = AppState::init_test_state().await?;
    let app = get_router(state).await?;
    Ok((_tdb, app))
  }

  #[tokio::test]
  #[serial]
  async fn signup_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:29998").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
      axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    });

    let client = Client::new();
    let response = client
      .post(format!("http://{}/auth/signup", addr))
      .json(&json!({"username": "xuetrdi", "password": "123456"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn signin_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:29999").await?;
    let addr = listener.local_addr()?;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
      axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = Client::new();

    let response = client
      .post(format!("http://{}/auth/signin", addr))
      .json(&json!({"username": "alice", "password": "123456"}))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await?.contains("token"));

    tx.send(()).unwrap();

    Ok(())
  }
}
