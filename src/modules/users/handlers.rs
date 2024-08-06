use super::{PaginationParams, UpdateUser, UpdateUserOptions, User};
use crate::common::errors::AppError;
use crate::AppState;

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
  Extension, Json,
};

pub async fn delete_user_handler(
  State(state): State<AppState>,
  Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
  state.delete_user(user_id).await?;
  Ok(StatusCode::OK)
}

pub async fn update_user_handler(
  Extension(claims): Extension<User>,
  State(state): State<AppState>,
  Path(user_id): Path<i32>,
  Json(input): Json<UpdateUserOptions>,
) -> Result<impl IntoResponse, AppError> {
  let is_who = state.get_role_by_claim(&claims, user_id).await?;
  let input = UpdateUser::new(input, is_who);

  let user = state.update_user(user_id, input).await?;
  Ok((StatusCode::OK, Json(user)))
}

pub async fn get_user_handler(
  Extension(claims): Extension<User>,
  State(state): State<AppState>,
  Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
  let is_who = state.get_role_by_claim(&claims, user_id).await?;
  if !is_who.is_own_user && !is_who.is_admin && !is_who.is_moderator {
    return Err(AppError::BadRequest("Permission denied".to_string()));
  }
  let user = state.get_user_by_id(user_id).await?;
  Ok((StatusCode::OK, Json(user)))
}

pub async fn get_users_handler(
  Extension(claims): Extension<User>,
  State(state): State<AppState>,
  Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
  let is_who = state
    .get_role_by_claim(&claims, claims.user_info.id)
    .await?;
  // Only admin and moderator can get all users
  if !is_who.is_own_user && !is_who.is_admin && !is_who.is_moderator {
    return Err(AppError::BadRequest("Permission denied".to_string()));
  }
  let PaginationParams { limit, offset } = params;
  let users = state.get_users(limit, offset).await?;
  Ok((StatusCode::OK, Json(users)))
}
