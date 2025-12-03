use crate::{AppError, AppState};
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
  pub status: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub uptime_seconds: u64,
  pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
  pub status: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub database: DatabaseStatus,
  pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStatus {
  pub status: String,
  pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LivenessResponse {
  pub status: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Store application start time
static APP_START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn get_app_start_time() -> Instant {
  APP_START_TIME.get_or_init(Instant::now).to_owned()
}

pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
  info!("Health check endpoint accessed");

  let response = HealthResponse {
    status: "healthy".to_string(),
    timestamp: chrono::Utc::now(),
    uptime_seconds: get_app_start_time().elapsed().as_secs(),
    version: env!("CARGO_PKG_VERSION").to_string(),
  };

  (StatusCode::OK, Json(response))
}

pub async fn readiness_check(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
  info!("Readiness check endpoint accessed");

  // Check database connectivity
  let db_start = Instant::now();
  let db_status = match sqlx::query("SELECT 1").fetch_one(&state.pool).await {
    Ok(_) => DatabaseStatus {
      status: "healthy".to_string(),
      response_time_ms: Some(db_start.elapsed().as_millis() as u64),
    },
    Err(e) => {
      tracing::error!("Database health check failed: {:?}", e);
      DatabaseStatus {
        status: "unhealthy".to_string(),
        response_time_ms: None,
      }
    }
  };

  let status = if db_status.status == "healthy" {
    "ready"
  } else {
    "not_ready"
  };

  let response = ReadinessResponse {
    status: status.to_string(),
    timestamp: chrono::Utc::now(),
    database: db_status,
    uptime_seconds: get_app_start_time().elapsed().as_secs(),
  };

  let status_code = if status == "ready" {
    StatusCode::OK
  } else {
    StatusCode::SERVICE_UNAVAILABLE
  };

  Ok((status_code, Json(response)))
}

pub async fn liveness_check(State(_state): State<AppState>) -> impl IntoResponse {
  info!("Liveness check endpoint accessed");

  let response = LivenessResponse {
    status: "alive".to_string(),
    timestamp: chrono::Utc::now(),
  };

  (StatusCode::OK, Json(response))
}
