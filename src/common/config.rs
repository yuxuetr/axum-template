use std::fs::read_to_string;
use std::sync::Arc;

use anyhow::Result;
use jwt_simple::prelude::*;
use serde::Deserialize;

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
  pub port: u16,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
  pub db_url: String,
}

#[allow(unused)]
#[derive(Clone)]
pub struct AuthConfig {
  pub secret_key_path: String,
  pub public_key_path: String,
  pub jwt_duration: u64,
  pub jwt_iss: String,
  pub jwt_aud: String,
  pub key_pair: Arc<Ed25519KeyPair>,
  pub public_key: Arc<Ed25519PublicKey>,
}

impl std::fmt::Debug for AuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AuthConfig")
      .field("secret_key_path", &self.secret_key_path)
      .field("public_key_path", &self.public_key_path)
      .field("jwt_duration", &self.jwt_duration)
      .field("jwt_iss", &self.jwt_iss)
      .field("jwt_aud", &self.jwt_aud)
      .field("key_pair", &"<hidden>")
      .field("public_key", &"<hidden>")
      .finish()
  }
}

impl AuthConfig {
  pub fn new(
    secret_key_path: String,
    public_key_path: String,
    jwt_duration: u64,
    jwt_iss: String,
    jwt_aud: String,
  ) -> Result<Self> {
    let secret_key_pem = read_to_string(&secret_key_path)?;
    let key_pair = Ed25519KeyPair::from_pem(&secret_key_pem)?;

    let public_key_pem = read_to_string(&public_key_path)?;
    let public_key = Ed25519PublicKey::from_pem(&public_key_pem)?;

    Ok(Self {
      secret_key_path,
      public_key_path,
      jwt_duration,
      jwt_iss,
      jwt_aud,
      key_pair: Arc::new(key_pair),
      public_key: Arc::new(public_key),
    })
  }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
struct AppConfigRaw {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub auth: AuthConfigRaw,
}

#[derive(Debug, Deserialize)]
struct AuthConfigRaw {
  pub secret_key: String,
  pub public_key: String,
  pub jwt_duration: u64,
  pub jwt_iss: String,
  pub jwt_aud: String,
}

#[allow(unused)]
impl AppConfig {
  pub fn from_file(file_path: &str) -> Result<Self> {
    let config_str = read_to_string(file_path)?;
    let config_raw: AppConfigRaw = serde_yaml::from_str(&config_str)?;

    let auth_config = AuthConfig::new(
      config_raw.auth.secret_key,
      config_raw.auth.public_key,
      config_raw.auth.jwt_duration,
      config_raw.auth.jwt_iss,
      config_raw.auth.jwt_aud,
    )?;

    Ok(Self {
      server: config_raw.server,
      database: config_raw.database,
      auth: auth_config,
    })
  }
}
