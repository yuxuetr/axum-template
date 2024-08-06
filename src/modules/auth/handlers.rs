use crate::modules::users::CreateUser;
use crate::{AppError, AppState};
use axum::{
  extract::{Json, State},
  http::StatusCode,
  response::IntoResponse,
};
use tracing::info;
use validator::Validate;

use super::TokenRequest;

pub async fn signup_handler(
  State(state): State<AppState>,
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  payload.validate()?;
  info!("Auth Handler::create user: input: {:?}", payload);
  let user = state.create_user(payload).await?;
  Ok((StatusCode::CREATED, Json(user)))
}

pub async fn signin_handler(
  State(state): State<AppState>,
  Json(payload): Json<TokenRequest>,
) -> Result<impl IntoResponse, AppError> {
  payload.validate()?;
  info!("Auth Handler::get token: username: {:?}", payload.username);
  let token = state
    .get_token(&payload.username, &payload.password)
    .await?;
  Ok((StatusCode::OK, Json(token)))
}
