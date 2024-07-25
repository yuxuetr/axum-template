pub mod dto;
pub mod entity;
pub mod handlers;
mod services;

pub use dto::{CreateUser, PaginationParams, UpdateUser};
pub use handlers::{
  create_user_handler, delete_user_handler, get_user_handler, get_users_handler,
  update_user_handler,
};

use crate::AppState;

use axum::routing::get;
use axum::Router;

pub fn users_router(state: AppState) -> Router {
  Router::new()
    .route("/", get(get_users_handler).post(create_user_handler))
    .route(
      "/:id",
      get(get_user_handler)
        .put(update_user_handler)
        .delete(delete_user_handler),
    )
    .with_state(state)
}
