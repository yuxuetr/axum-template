use crate::modules::users::User;
use crate::{AppConfig, AppError};

use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2,
};
use jwt_simple::prelude::*;
use std::fs::read_to_string;

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

pub async fn sign(user: impl Into<User>) -> Result<String, AppError> {
  let config = AppConfig::from_file("app.yaml");
  let secret_key_pem = read_to_string(config.clone().auth.secret_key)?;
  let key_pair = Ed25519KeyPair::from_pem(&secret_key_pem)?;

  let user = user.into();
  let claims =
    Claims::with_custom_claims::<User>(user, Duration::from_secs(config.clone().auth.jwt_duration));

  let claims = claims
    .with_issuer(config.clone().auth.jwt_iss)
    .with_audience(config.clone().auth.jwt_aud);

  let token = key_pair.sign(claims)?;
  Ok(token)
}

pub async fn verify(token: &str) -> Result<User, AppError> {
  let config = AppConfig::from_file("app.yaml");
  let public_key_pem = read_to_string(config.clone().auth.public_key)?;
  let public_key = Ed25519PublicKey::from_pem(&public_key_pem)?;

  let options = VerificationOptions {
    allowed_issuers: Some(HashSet::from_strings(&[config.clone().auth.jwt_iss])),
    allowed_audiences: Some(HashSet::from_strings(&[config.clone().auth.jwt_aud])),
    ..Default::default()
  };

  let claims = public_key.verify_token::<User>(token, Some(options))?;
  Ok(claims.custom)
}
