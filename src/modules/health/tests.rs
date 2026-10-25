#[cfg(test)]
mod integration_tests {
  use crate::modules::health::handlers::{HealthResponse, LivenessResponse, ReadinessResponse};
  use crate::test_util::TestPg;
  use crate::{AppState, health_router};
  use anyhow::Result;
  use axum::Router;
  use axum::http::StatusCode;
  use reqwest::Client;
  use serial_test::serial;
  use tokio::net::TcpListener;
  use tokio::sync::oneshot;

  async fn setup_test_app() -> Result<(TestPg, Router)> {
    let (tdb, state) = AppState::init_test_state().await?;
    let app = health_router(state);
    Ok((tdb, app))
  }

  #[tokio::test]
  #[serial]
  async fn health_check_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
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

    let client = Client::builder().no_proxy().build().unwrap();
    let response = client.get(format!("http://{}/health", addr)).send().await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: HealthResponse = response.json().await?;
    assert_eq!(body.status, "healthy");

    let _ = tx.send(());
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn liveness_check_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
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

    let client = Client::builder().no_proxy().build().unwrap();
    let response = client
      .get(format!("http://{}/health/live", addr))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: LivenessResponse = response.json().await?;
    assert_eq!(body.status, "alive");

    let _ = tx.send(());
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn readiness_check_handler_test() -> Result<()> {
    let (_tdb, app) = setup_test_app().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
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

    let client = Client::builder().no_proxy().build().unwrap();
    let response = client
      .get(format!("http://{}/health/ready", addr))
      .send()
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: ReadinessResponse = response.json().await?;
    assert_eq!(body.status, "ready");
    assert_eq!(body.database.status, "healthy");
    assert!(body.database.response_time_ms.is_some());

    let _ = tx.send(());
    Ok(())
  }
}
