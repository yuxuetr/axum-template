use crate::{common::auth::verify, AppState};
use axum::{
  body::Body,
  extract::{FromRequestParts, State},
  http::{Request, StatusCode},
  middleware::Next,
  response::IntoResponse,
};
use axum_extra::{
  headers::{authorization::Bearer, Authorization},
  TypedHeader,
};
use tracing::warn;

pub async fn auth_middleware(
  State(state): State<AppState>,
  req: Request<Body>,
  next: Next,
) -> impl IntoResponse {
  let (mut parts, body) = req.into_parts();
  let req = match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await
  {
    Ok(TypedHeader(Authorization(bearer))) => {
      let token = bearer.token();
      match verify(token).await {
        Ok(user) => {
          let mut req = Request::from_parts(parts, body);
          let user = match state.get_user_by_username(&user.user_info.username).await {
            Ok(user) => user,
            Err(e) => {
              let msg = format!("user not exists or removed: {:?}", e);
              warn!(msg);
              return (StatusCode::FORBIDDEN, msg).into_response();
            }
          };
          req.extensions_mut().insert(user);
          req
        }
        Err(e) => {
          let msg = format!("verify token failed: {:?}", e);
          warn!(msg);
          return (StatusCode::FORBIDDEN, msg).into_response();
        }
      }
    }
    Err(e) => {
      let msg = format!("parse Authorization header failed: {}", e);
      warn!(msg);
      return (StatusCode::UNAUTHORIZED, msg).into_response();
    }
  };
  next.run(req).await
}
