pub mod handlers;

use crate::AppState;
use axum::Router;

pub fn health_router(state: AppState) -> Router {
  Router::new()
    .route("/health", axum::routing::get(handlers::health_check))
    .route("/health/live", axum::routing::get(handlers::liveness_check))
    .route(
      "/health/ready",
      axum::routing::get(handlers::readiness_check),
    )
    .with_state(state)
}
