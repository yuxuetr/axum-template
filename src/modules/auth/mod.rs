pub mod dto;
pub mod handlers;
pub mod middleware;
pub mod services;
pub mod tests;

pub use dto::{TokenRequest, TokenResponse};
pub use handlers::{signin_handler, signup_handler};
pub use middleware::auth_middleware;

use crate::AppState;
use axum::routing::post;
use axum::Router;

pub fn auth_router(state: AppState) -> Router {
  Router::new()
    .route("/signup", post(signup_handler))
    .route("/signin", post(signin_handler))
    .with_state(state)
}
