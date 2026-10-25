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
pub mod test_util {
  use super::*;
  use sea_orm::{ConnectionTrait, DbBackend, Statement};
  use uuid::Uuid;

  pub struct TestPg {
    admin_url: String,
    dbname: String,
  }

  impl Drop for TestPg {
    fn drop(&mut self) {
      let admin_url = self.admin_url.clone();
      let dbname = self.dbname.clone();
      let _ = std::thread::spawn(move || {
        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
          .enable_all()
          .build()
        {
          rt.block_on(async move {
            if let Ok(admin_db) = Database::connect(&admin_url).await {
              let _ = admin_db
                .execute_raw(Statement::from_sql_and_values(
                  DbBackend::Postgres,
                  "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = $1 AND pid <> pg_backend_pid()",
                  [dbname.clone().into()],
                ))
                .await;
              let _ = admin_db
                .execute_unprepared(&format!("DROP DATABASE IF EXISTS \"{}\"", dbname))
                .await;
            }
          });
        }
      })
      .join();
    }
  }

  async fn exec_sql_script(db: &DatabaseConnection, script: &str) -> Result<(), AppError> {
    for raw_stmt in script.split(';') {
      let cleaned: String = raw_stmt
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
      let stmt = cleaned.trim();
      if stmt.is_empty() {
        continue;
      }
      db.execute_unprepared(stmt)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }
    Ok(())
  }

  impl AppState {
    pub async fn init_test_state() -> Result<(TestPg, AppState), AppError> {
      let config = AppConfig::from_file("app.yaml").map_err(|_| AppError::InternalServerError)?;
      let admin_url = config.database.db_url.clone();
      let pos = config
        .database
        .db_url
        .rfind('/')
        .ok_or(AppError::InternalServerError)?;
      let server_url = &config.database.db_url[..pos];
      let dbname = format!("axum_test_{}", Uuid::new_v4().simple());

      let admin_db = Database::connect(&admin_url)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
      admin_db
        .execute_unprepared(&format!("CREATE DATABASE \"{}\"", dbname))
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
      let _ = admin_db.close().await;

      let tdb = TestPg {
        admin_url,
        dbname: dbname.clone(),
      };

      let test_db_url = format!("{}/{}", server_url, dbname);
      let db = Database::connect(&test_db_url)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;

      let migrations = [
        include_str!("../migrations/20240725011950_create_users_table.sql"),
        include_str!("../migrations/20240731130501_rbac.sql"),
        include_str!("../migrations/20240805123718_initial_admin_user.sql"),
        include_str!("../migrations/20240806000001_username_unique.sql"),
        include_str!("../migrations/20240806000002_fix_view_reports_typo.sql"),
        include_str!("../fixtures/test_data.sql"),
      ];
      for script in migrations {
        exec_sql_script(&db, script).await?;
      }

      let state = Self::new(config, db);
      Ok((tdb, state))
    }
  }
}
