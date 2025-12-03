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
          Ok(user) => {
            let user = match state.get_user_by_username(&user.user_info.username).await {
              Ok(user) => user,
              Err(e) => {
                let msg = format!("user not exists or removed: {:?}", e);
                warn!(msg);
                return (StatusCode::FORBIDDEN, msg).into_response();
              }
            };
            let mut req = req;
            req.extensions_mut().insert(user);
            next.run(req).await
          }
          Err(e) => {
            let msg = format!("verify token failed: {:?}", e);
            warn!(msg);
            (StatusCode::FORBIDDEN, msg).into_response()
          }
        }
      } else {
        let msg = "Invalid Authorization header format".to_string();
        warn!(msg);
        (StatusCode::UNAUTHORIZED, msg).into_response()
      }
    }
    None => {
      let msg = "Missing Authorization header".to_string();
      warn!(msg);
      (StatusCode::UNAUTHORIZED, msg).into_response()
    }
  }
}
