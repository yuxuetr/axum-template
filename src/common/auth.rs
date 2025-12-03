use crate::modules::users::User;
use crate::{AppConfig, AppError};

use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jwt_simple::prelude::*;
use std::collections::HashSet;

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
  let claims =
    Claims::with_custom_claims::<User>(user, Duration::from_secs(config.auth.jwt_duration));

  let claims = claims
    .with_issuer(&config.auth.jwt_iss)
    .with_audience(&config.auth.jwt_aud);

  let token = config.auth.key_pair.sign(claims)?;
  Ok(token)
}

pub fn verify(token: &str, config: &AppConfig) -> Result<User, AppError> {
  let options = VerificationOptions {
    allowed_issuers: Some(HashSet::from_strings(&[&config.auth.jwt_iss])),
    allowed_audiences: Some(HashSet::from_strings(&[&config.auth.jwt_aud])),
    ..Default::default()
  };

  let claims = config
    .auth
    .public_key
    .verify_token::<User>(token, Some(options))?;
  Ok(claims.custom)
}
