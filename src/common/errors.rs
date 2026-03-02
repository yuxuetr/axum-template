use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use validator::ValidationErrors;

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

  #[error("password error: {0}")]
  PasswordError(String),

  #[error("sql error: {0}")]
  SqlxError(#[from] sqlx::Error),

  #[error("jwt error: {0}")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("io error: {0}")]
  IOError(#[from] std::io::Error),

  #[error("user existed: {0}")]
  UserExisted(String),
}

impl From<ValidationErrors> for AppError {
  fn from(errors: ValidationErrors) -> Self {
    let errors = errors
      .field_errors()
      .iter()
      .flat_map(|(_, errors)| {
        errors.iter().map(|error| {
          if let Some(message) = &error.message {
            message.clone().into_owned()
          } else {
            "Invalid value".to_string()
          }
        })
      })
      .collect::<Vec<String>>()
      .join(", ");
    AppError::ValidationError(errors)
  }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorOutput {
  pub error: String,
  pub error_id: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let error_id = Uuid::new_v4().to_string();
    let error_message = self.to_string();

    // Log the error with tracing for debugging
    tracing::error!(
      error_id = %error_id,
      error = %error_message,
      "Application error occurred"
    );

    let (status, client_message) = match &self {
      Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
      Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
      Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
      Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
      Self::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
      Self::PasswordError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
      Self::UserExisted(msg) => (StatusCode::CONFLICT, msg.clone()),
      Self::JwtError(_) => (
        StatusCode::UNAUTHORIZED,
        "invalid or expired token".to_string(),
      ),
      // Internal errors: never leak details to client
      Self::InternalServerError
      | Self::DatabaseError(_)
      | Self::SqlxError(_)
      | Self::PasswordHashError(_)
      | Self::IOError(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error".to_string(),
      ),
    };

    (status, Json(ErrorOutput::with_id(client_message, error_id))).into_response()
  }
}

impl ErrorOutput {
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      error: error.into(),
      error_id: Uuid::new_v4().to_string(),
      timestamp: chrono::Utc::now(),
    }
  }

  pub fn with_id(error: impl Into<String>, error_id: String) -> Self {
    Self {
      error: error.into(),
      error_id,
      timestamp: chrono::Utc::now(),
    }
  }
}
