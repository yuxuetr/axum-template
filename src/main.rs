use axum_template::AppState;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
  let _state = AppState::init_state().await?;
  Ok(())
}
