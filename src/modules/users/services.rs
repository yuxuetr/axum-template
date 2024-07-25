use crate::common::errors::AppError;
use crate::modules::users::dto::{CreateUser, UpdateUser};
use crate::modules::users::entity::User;
use crate::AppState;
use bcrypt::{hash, verify};
use chrono::Utc;

impl AppState {
  pub async fn create_user(&self, input: CreateUser) -> Result<User, AppError> {
    let hashed_password = hash(&input.password, 4).map_err(|_| AppError::InternalServerError)?;
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
      hash(password, 4).map_err(|_| AppError::InternalServerError)?
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

  pub async fn get_users(&self, limit: i64, offset: i64) -> Result<Vec<User>, AppError> {
    let users = sqlx::query_as(
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
    Ok(users)
  }

  pub async fn verify_user(&self, username: &str, password: &str) -> Result<User, AppError> {
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

    if verify(password, &user.password).map_err(|_| AppError::InternalServerError)? {
      Ok(user)
    } else {
      Err(AppError::Unauthorized("Invalid password".to_string()))
    }
  }
}
