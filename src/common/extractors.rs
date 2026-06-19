use crate::AppError;
use axum::{
  Form, Json,
  extract::{FromRequest, FromRequestParts, Path, Query, Request},
  http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedPath<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state).await?;
    value.validate()?;
    Ok(Self(value))
  }
}

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
    value.validate()?;
    Ok(Self(value))
  }
}

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Form(value) = Form::<T>::from_request(req, state).await?;
    value.validate()?;
    Ok(Self(value))
  }
}

impl<T, S> FromRequestParts<S> for ValidatedPath<T>
where
  T: DeserializeOwned + Send + Validate,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let Path(value) = Path::<T>::from_request_parts(parts, state).await?;
    value.validate()?;
    Ok(Self(value))
  }
}

#[cfg(test)]
mod tests {
  use super::{ValidatedForm, ValidatedJson, ValidatedPath, ValidatedQuery};
  use axum::{
    Json, Router,
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
  };
  use serde::Deserialize;
  use tokio::net::TcpListener;
  use validator::Validate;

  #[derive(Debug, Deserialize, Validate)]
  struct TestInput {
    #[validate(length(min = 3))]
    value: String,
  }

  #[derive(Debug, Deserialize, Validate)]
  struct TestPath {
    #[validate(range(min = 1))]
    id: i32,
  }

  async fn json_handler(ValidatedJson(input): ValidatedJson<TestInput>) -> impl IntoResponse {
    Json(input.value)
  }

  async fn query_handler(ValidatedQuery(input): ValidatedQuery<TestInput>) -> impl IntoResponse {
    Json(input.value)
  }

  async fn form_handler(ValidatedForm(input): ValidatedForm<TestInput>) -> impl IntoResponse {
    Json(input.value)
  }

  async fn path_handler(ValidatedPath(path): ValidatedPath<TestPath>) -> impl IntoResponse {
    Json(path.id)
  }

  async fn start_app(app: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
      axum::serve(listener, app).await.unwrap();
    });
    format!("http://{}", addr)
  }

  #[tokio::test]
  async fn validated_json_rejects_validation_errors() {
    let app = Router::new().route("/json", post(json_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .post(format!("{}/json", base_url))
      .json(&serde_json::json!({ "value": "ab" }))
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
  }

  #[tokio::test]
  async fn validated_json_rejects_malformed_json() {
    let app = Router::new().route("/json", post(json_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .post(format!("{}/json", base_url))
      .header(header::CONTENT_TYPE, "application/json")
      .body("{")
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn validated_query_rejects_validation_errors() {
    let app = Router::new().route("/query", get(query_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .get(format!("{}/query?value=ab", base_url))
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
  }

  #[tokio::test]
  async fn validated_query_rejects_malformed_query() {
    let app = Router::new().route("/query", get(query_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .get(format!("{}/query", base_url))
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn validated_form_rejects_validation_errors() {
    let app = Router::new().route("/form", post(form_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .post(format!("{}/form", base_url))
      .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
      .body("value=ab")
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
  }

  #[tokio::test]
  async fn validated_path_rejects_validation_errors() {
    let app = Router::new().route("/items/{id}", get(path_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .get(format!("{}/items/0", base_url))
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
  }

  #[tokio::test]
  async fn validated_path_rejects_malformed_path() {
    let app = Router::new().route("/items/{id}", get(path_handler));
    let base_url = start_app(app).await;

    let response = reqwest::Client::new()
      .get(format!("{}/items/not-a-number", base_url))
      .send()
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn validated_json_accepts_valid_input() {
    let request = Request::builder()
      .method("POST")
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(r#"{ "value": "abcd" }"#))
      .unwrap();

    let ValidatedJson(input) = ValidatedJson::<TestInput>::from_request(request, &())
      .await
      .unwrap();

    assert_eq!(input.value, "abcd");
  }
}
