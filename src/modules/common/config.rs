use std::fs::read_to_string;

use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  port: u16,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
  pub db_url: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct AuthConfig {
  pub secret_key: String,
  pub public_key: String,
  pub jwt_duration: u64,
  pub jwt_iss: String,
  pub jwt_aud: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub auth: AuthConfig,
}

#[allow(unused)]
impl AppConfig {
  pub fn from_file(file_path: &str) -> Self {
    let config_str = read_to_string(file_path).expect("Unable to read config file");
    serde_yaml::from_str(&config_str).expect("Unable to parse config file")
  }
}
