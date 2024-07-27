use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("not found: {0}")]
  NotFound(String),

  #[error("unauthorized: {0}")]
  Unauthorized(String),

  #[error("forbidden: {0}")]
  Forbidden(String),

  #[error("bad request: {0}")]
  BadRequest(String),

  #[error("internal server error")]
  InternalServerError,

  #[error("database error: {0}")]
  DatabaseError(String),

  #[error("validation error: {0}")]
  ValidationError(String),

  #[error("password hash error: {0}")]
  PasswordHashError(#[from] argon2::password_hash::Error),

  #[error("sql error: {0}")]
  SqlxError(#[from] sqlx::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorOutput {
  pub error: String,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let status = match &self {
      Self::NotFound(_) => StatusCode::NOT_FOUND,
      Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
      Self::Forbidden(_) => StatusCode::FORBIDDEN,
      Self::BadRequest(_) => StatusCode::BAD_REQUEST,
      Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
      Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      Self::SqlxError(_) => StatusCode::UNPROCESSABLE_ENTITY,
    };

    (status, Json(ErrorOutput::new(self.to_string()))).into_response()
  }
}

impl ErrorOutput {
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      error: error.into(),
    }
  }
}
