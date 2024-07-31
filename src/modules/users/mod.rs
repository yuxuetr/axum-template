pub mod dto;
pub mod entity;
pub mod handlers;
pub mod services;
pub mod tests;

pub use dto::{CreateUser, PaginationParams, UpdateUser};
pub use entity::User;
pub use handlers::{delete_user_handler, get_user_handler, get_users_handler, update_user_handler};

use crate::AppState;

use axum::routing::get;
use axum::Router;

pub fn users_router(state: AppState) -> Router {
  Router::new()
    .route("/", get(get_users_handler))
    .route(
      "/:id",
      get(get_user_handler)
        .patch(update_user_handler)
        .delete(delete_user_handler),
    )
    .with_state(state)
}
