use anyhow::Result;
use sqlx::PgPool;

mod modules;
use modules::common::config::AppConfig;

#[allow(unused)]
pub struct AppState {
  config: AppConfig,
  pool: PgPool,
}

impl AppState {
  pub fn new(config: AppConfig, pool: PgPool) -> Self {
    Self { config, pool }
  }

  pub async fn init_state() -> Result<AppState> {
    let config = AppConfig::from_file("app.yaml");
    println!("{:?}", &config);
    let pool = PgPool::connect(&config.database.db_url).await?;
    let state = AppState::new(config, pool);
    Ok(state)
  }
}
