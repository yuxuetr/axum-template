use axum_template::{get_router, AppState};

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;
use tracing_appender::rolling;
use tracing_subscriber::{
  fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
  let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into());
  let stdout_log = fmt::layer()
    .with_writer(std::io::stdout)
    .with_filter(env_filter);

  let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into());
  let file_appender = rolling::daily("logs", "app.log");
  let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
  let file_log = fmt::layer()
    .with_writer(non_blocking)
    .with_filter(env_filter);

  tracing_subscriber::registry()
    .with(stdout_log)
    .with(file_log)
    .init();

  let state = AppState::init_state().await?;
  let app = get_router(state.clone()).await?;

  let addr = format!("0.0.0.0:{}", &state.config.server.port);
  let listener = TcpListener::bind(&addr).await?;
  info!("Listening on: {}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}
