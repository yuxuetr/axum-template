use super::{CreateUser, PaginationParams, UpdateUser};
use crate::common::errors::AppError;
use crate::AppState;

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
  Json,
};

pub async fn create_user_handler(
  State(state): State<AppState>,
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = state.create_user(payload).await?;
  Ok((StatusCode::CREATED, Json(user)))
}

pub async fn delete_user_handler(
  State(state): State<AppState>,
  Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
  state.delete_user(user_id).await?;
  Ok(StatusCode::OK)
}

pub async fn update_user_handler(
  State(state): State<AppState>,
  Path(user_id): Path<i32>,
  Json(payload): Json<UpdateUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = state.update_user(user_id, payload).await?;
  Ok((StatusCode::OK, Json(user)))
}

pub async fn get_user_handler(
  State(state): State<AppState>,
  Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
  let user = state.get_user_by_id(user_id).await?;
  Ok((StatusCode::OK, Json(user)))
}

pub async fn get_users_handler(
  State(state): State<AppState>,
  Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
  let PaginationParams { limit, offset } = params;
  let users = state.get_users(limit, offset).await?;
  Ok((StatusCode::OK, Json(users)))
}
