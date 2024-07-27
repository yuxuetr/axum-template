use anyhow::Result;
use axum::Router;
use sqlx::PgPool;
use std::ops::Deref;
use std::sync::Arc;

mod common;
mod modules;
pub use common::config::AppConfig;
pub use common::errors::AppError;
pub use modules::users::*;

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

/// This module contains utility functions for testing.
#[allow(unused)]
#[cfg(test)]
mod test_util {
  use super::*;
  use sqlx_db_tester::TestPg;

  impl AppState {
    pub async fn init_test_state() -> Result<(TestPg, AppState), AppError> {
      let config = AppConfig::from_file("app.yaml");
      let pos = config.database.db_url.rfind('/').unwrap();
      let db_url = config.database.db_url[..pos].to_string();
      let tdb = TestPg::new(db_url, std::path::Path::new("./migrations"));
      let pool = tdb.get_pool().await;
      let state = Self {
        inner: Arc::new(AppStateInner { config, pool }),
      };
      Ok((tdb, state))
    }
  }
}