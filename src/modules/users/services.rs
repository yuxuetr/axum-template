use crate::common::errors::AppError;
use crate::modules::users::dto::{CreateUser, UpdateUser};
use crate::modules::users::entity::User;
use crate::AppState;

use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2,
};
use chrono::Utc;

use super::dto::PaginatedUsers;

impl AppState {
  pub async fn create_user(&self, input: CreateUser) -> Result<User, AppError> {
    let hashed_password = hash_password(&input.password)?;
    let user = sqlx::query_as(
      r#"
      INSERT INTO users (username, password, created_at, updated_at)
      VALUES ($1, $2, $3, $4)
      RETURNING id, username, created_at, updated_at
      "#,
    )
    .bind(input.username)
    .bind(hashed_password)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(user)
  }

  pub async fn delete_user(&self, user_id: i32) -> Result<(), AppError> {
    let result = sqlx::query(
      r#"
      DELETE FROM users WHERE id = $1
      RETURNING id, username, created_at, updated_at
      "#,
    )
    .bind(user_id)
    .execute(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(AppError::NotFound(format!(
        "User with id {} not found",
        user_id
      )));
    }
    Ok(())
  }

  pub async fn update_user(&self, user_id: i32, input: UpdateUser) -> Result<User, AppError> {
    let user = self.get_user_by_id(user_id).await?;

    let hashed_password = if let Some(password) = input.password {
      hash_password(&password)?
    } else {
      user.password
    };

    let user = sqlx::query_as(
      r#"
      UPDATE users
      SET username = $1, password = $2, updated_at = $3
      WHERE id = $4
      RETURNING id, username, created_at, updated_at
      "#,
    )
    .bind(input.username.unwrap_or(user.username))
    .bind(hashed_password)
    .bind(Utc::now())
    .bind(user_id)
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(user)
  }

  pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
    let user = sqlx::query_as(
      r#"
      SELECT id, username, password, created_at, updated_at
      FROM users
      WHERE id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(user)
  }

  pub async fn get_users(&self, limit: i64, offset: i64) -> Result<PaginatedUsers, AppError> {
    let users = sqlx::query_as::<_, User>(
      r#"
      SELECT id, username, created_at, updated_at
      FROM users
      ORDER BY id
      LIMIT $1
      OFFSET $2
      "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let total_count: (i64,) = sqlx::query_as(
      r#"
      SELECT COUNT(*)
      FROM users
      "#,
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(PaginatedUsers {
      users,
      total_count: total_count.0,
    })
  }

  pub async fn verify_user(&self, username: &str, password: &str) -> Result<bool, AppError> {
    let user: User = sqlx::query_as(
      r#"
      SELECT id, username, password, created_at, updated_at
      FROM users
      WHERE username = $1
      "#,
    )
    .bind(username)
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    if verify_password(password, &user.password)? {
      Ok(true)
    } else {
      Err(AppError::Unauthorized("Invalid password".to_string()))
    }
  }
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
