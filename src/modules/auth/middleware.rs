use crate::{AppState, common::auth::verify};
use axum::{
  body::Body,
  extract::State,
  http::{Request, StatusCode},
  middleware::Next,
  response::IntoResponse,
};
use tracing::warn;

pub async fn auth_middleware(
  State(state): State<AppState>,
  req: Request<Body>,
  next: Next,
) -> impl IntoResponse {
  let headers = req.headers();
  let auth_header = headers
    .get("authorization")
    .and_then(|header| header.to_str().ok());

  match auth_header {
    Some(auth_str) => {
      if let Some(bearer_str) = auth_str.strip_prefix("Bearer ") {
        match verify(bearer_str, &state.config) {
          Ok(user_id) => {
            let user = match state.get_user_by_id(user_id).await {
              Ok(user) => user,
              Err(e) => {
                warn!(user_id, error = ?e, "user not exists or removed");
                return (StatusCode::FORBIDDEN, "user not exists or removed").into_response();
              }
            };
            let mut req = req;
            req.extensions_mut().insert(user);
            next.run(req).await
          }
          Err(e) => {
            warn!(error = ?e, "verify token failed");
            (StatusCode::UNAUTHORIZED, "invalid or expired token").into_response()
          }
        }
      } else {
        warn!("Invalid Authorization header format");
        (
          StatusCode::UNAUTHORIZED,
          "Invalid Authorization header format",
        )
          .into_response()
      }
    }
    None => {
      warn!("Missing Authorization header");
      (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response()
    }
  }
}
