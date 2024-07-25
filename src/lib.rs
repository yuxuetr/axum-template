use anyhow::Result;
use axum::Router;
use sqlx::PgPool;
use std::ops::Deref;
use std::sync::Arc;

mod common;
mod modules;
pub use common::config::AppConfig;
pub use common::errors::AppError;
pub use modules::users::users_router;

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
  let router = Router::new().nest("/users", users_router(state.clone()));
  Ok(router)
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct AppStateInner {
  pub config: AppConfig,
  pub pool: PgPool,
}

#[derive(Clone, Debug)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

impl AppState {
  pub fn new(config: AppConfig, pool: PgPool) -> Self {
    Self {
      inner: Arc::new(AppStateInner { config, pool }),
    }
  }

  pub async fn init_state() -> Result<AppState> {
    let config = AppConfig::from_file("app.yaml");
    let pool = PgPool::connect(&config.database.db_url).await?;
    let state = AppState::new(config, pool);
    Ok(state)
  }
}

impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
