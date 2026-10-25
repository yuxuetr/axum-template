use anyhow::Result;
use axum::Router;
use axum::middleware::from_fn_with_state;
use sea_orm::{Database, DatabaseConnection};
use std::ops::Deref;
use std::sync::Arc;

pub mod common;
pub mod modules;
pub use common::config::AppConfig;
pub use common::errors::AppError;
pub use modules::auth::{auth_middleware, auth_router};
pub use modules::health::health_router;
pub use modules::users::users_router;

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
  let router = Router::new()
    .merge(health_router(state.clone()))
    .nest("/users", users_router(state.clone()))
    .layer(from_fn_with_state(state.clone(), auth_middleware))
    .nest("/auth", auth_router(state.clone()));
  Ok(router)
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct AppStateInner {
  pub config: AppConfig,
  pub db: DatabaseConnection,
}

#[derive(Clone, Debug)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

impl AppState {
  pub fn new(config: AppConfig, db: DatabaseConnection) -> Self {
    Self {
      inner: Arc::new(AppStateInner { config, db }),
    }
  }

  pub async fn init_state() -> Result<AppState> {
    let config = AppConfig::from_file("app.yaml")?;
    let db = Database::connect(&config.database.db_url).await?;
    let state = AppState::new(config, db);
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
  use sea_orm::{ConnectionTrait, SqlxPostgresConnector, TransactionTrait};
  use sqlx_db_tester::TestPg;

  impl AppState {
    pub async fn init_test_state() -> Result<(TestPg, AppState), AppError> {
      let config = AppConfig::from_file("app.yaml").map_err(|_| AppError::InternalServerError)?;
      let pos = config
        .database
        .db_url
        .rfind('/')
        .ok_or(AppError::InternalServerError)?;
      let db_url = config.database.db_url[..pos].to_string();
      let tdb = TestPg::new(db_url, std::path::Path::new("./migrations"));
      let pool = tdb.get_pool().await;
      let db = SqlxPostgresConnector::from_sqlx_postgres_pool(pool);

      // test_data.sql start
      let sql = include_str!("../fixtures/test_data.sql").split(';');
      let ts = db
        .begin()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
      for s in sql {
        if s.trim().is_empty() {
          continue;
        }
        ts.execute_unprepared(s)
          .await
          .map_err(|e| AppError::DatabaseError(e.to_string()))?;
      }
      ts.commit()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
      // test_data.sql end

      let state = Self::new(config, db);
      Ok((tdb, state))
    }
  }
}
