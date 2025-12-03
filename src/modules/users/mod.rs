pub mod dto;
pub mod entity;
pub mod handlers;
pub mod services;
pub mod tests;

pub use dto::{
  CreateUser, IsWho, PaginationParams, PermissionIn, RoleIn, UpdateUser, UpdateUserOptions, User,
};
pub use entity::{Permission, PermissionName, Role, RoleName, UserInfo, VecExtensions};
pub use handlers::{delete_user_handler, get_user_handler, get_users_handler, update_user_handler};

use crate::AppState;

use axum::Router;
use axum::routing::get;

pub fn users_router(state: AppState) -> Router {
  Router::new()
    .route("/", get(get_users_handler))
    .route(
      "/{id}",
      get(get_user_handler)
        .patch(update_user_handler)
        .delete(delete_user_handler),
    )
    .with_state(state)
}
