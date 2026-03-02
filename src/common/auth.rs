use crate::modules::users::User;
use crate::{AppConfig, AppError};

use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
  pub exp: usize,
  pub iat: usize,
  pub iss: String,
  pub aud: String,
  pub user: User,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let hashed_password = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();
  Ok(hashed_password)
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, AppError> {
  let argon2 = Argon2::default();
  let parsed_hash = PasswordHash::new(hashed_password)?;
  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();
  Ok(is_valid)
}

pub fn sign(user: impl Into<User>, config: &AppConfig) -> Result<String, AppError> {
  let user = user.into();
  let now = chrono::Utc::now().timestamp() as usize;
  let claims = JwtClaims {
    exp: now + config.auth.jwt_duration as usize,
    iat: now,
    iss: config.auth.jwt_iss.clone(),
    aud: config.auth.jwt_aud.clone(),
    user,
  };
  let header = Header::new(Algorithm::EdDSA);
  let token = encode(&header, &claims, &config.auth.encoding_key)?;
  Ok(token)
}

pub fn verify(token: &str, config: &AppConfig) -> Result<User, AppError> {
  let mut validation = Validation::new(Algorithm::EdDSA);
  validation.set_issuer(&[&config.auth.jwt_iss]);
  validation.set_audience(&[&config.auth.jwt_aud]);

  let token_data = decode::<JwtClaims>(token, &config.auth.decoding_key, &validation)?;
  Ok(token_data.claims.user)
}
