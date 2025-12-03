use std::collections::HashSet;

use crate::common::errors::AppError;
use crate::modules::users::dto::{CreateUser, IsWho, UpdateUser, User};
use crate::modules::users::entity::{Permission, Role, RoleName, UserInfo, VecExtensions};
use crate::AppState;

use chrono::Utc;

use super::dto::PaginatedUsers;
use crate::common::hash_password;

impl AppState {
  pub async fn create_user(&self, input: CreateUser) -> Result<User, AppError> {
    match self.is_user_exists_by_username(&input.username).await? {
      true => {
        return Err(AppError::UserExisted(format!(
          "User: {} already exists",
          input.username
        )))
      }
      false => (),
    }

    let hashed_password = hash_password(&input.password)?;

    let mut transaction = self
      .pool
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let user_info = sqlx::query_as::<_, UserInfo>(
      r#"
      INSERT INTO users (username, password, created_at, updated_at)
      VALUES ($1, $2, $3, $4)
      RETURNING id, username, password, created_at, updated_at
      "#,
    )
    .bind(&input.username)
    .bind(hashed_password)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&mut *transaction)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let roles: Vec<Role> = sqlx::query_as(
      r#"
      WITH inserted AS (
          INSERT INTO user_roles (user_id, role_id, created_at, updated_at)
          VALUES ($1, 1, $2, $3) -- default role User, ID: 1
          RETURNING role_id
      )
      SELECT r.id, r.name, r.created_at, r.updated_at
      FROM roles r
      JOIN inserted i ON i.role_id = r.id
      "#,
    )
    .bind(user_info.id)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_all(&mut *transaction)
    .await?;

    let permissions: Vec<Permission> = sqlx::query_as(
      r#"
      SELECT p.id, p.name, p.created_at, p.updated_at
      FROM permissions p
      JOIN role_permissions rp ON rp.permission_id = p.id
      JOIN user_roles ur ON ur.role_id = rp.role_id
      WHERE ur.user_id = $1
      "#,
    )
    .bind(user_info.id)
    .fetch_all(&mut *transaction)
    .await?;

    // insert user permissions in user_permissions
    for permission in &permissions {
      sqlx::query(
        r#"
        INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        "#,
      )
      .bind(user_info.id)
      .bind(permission.id)
      .bind(Utc::now())
      .bind(Utc::now())
      .execute(&mut *transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }

    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let user = User::new(user_info, roles, permissions);

    Ok(user)
  }

  pub async fn delete_user(&self, user_id: i32) -> Result<(), AppError> {
    match self.is_user_exists_by_id(user_id).await? {
      true => (),
      false => {
        return Err(AppError::NotFound(format!(
          "User with id {} not found",
          user_id
        )))
      }
    }
    let mut transaction = self
      .pool
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user roles in user_roles
    sqlx::query(
      r#"
      DELETE FROM user_roles
      WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user permissions in user_permissions
    sqlx::query(
      r#"
      DELETE FROM user_permissions
      WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user
    let result = sqlx::query(
      r#"
      DELETE FROM users
      WHERE id = $1
      RETURNING id, username, created_at, updated_at
      "#,
    )
    .bind(user_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // commit the transaction
    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    match result {
      Some(_) => Ok(()),
      None => Err(AppError::NotFound(format!(
        "User with id {} not found",
        user_id
      ))),
    }
  }

  pub async fn update_user(&self, user_id: i32, input: UpdateUser) -> Result<User, AppError> {
    if !input.is_own_user && !input.is_admin && !input.is_moderator {
      return Err(AppError::BadRequest("Permission denied".to_string()));
    }

    match self.is_user_exists_by_id(user_id).await? {
      true => (),
      false => {
        return Err(AppError::NotFound(format!(
          "User with id {} not found",
          user_id
        )))
      }
    }

    let user = self.get_user_by_id(user_id).await?;

    // only the user itself can update its user info
    if input.is_own_user {
      let hashed_password = if let Some(password) = input.password {
        hash_password(&password)?
      } else {
        user.user_info.password
      };
      let updated_user_info: UserInfo = sqlx::query_as(
        r#"
        UPDATE users
        SET username = $1, password = $2, updated_at = $3
        WHERE id = $4
        RETURNING id, username, created_at, updated_at
        "#,
      )
      .bind(input.username.unwrap_or(user.user_info.username))
      .bind(hashed_password)
      .bind(Utc::now())
      .bind(user_id)
      .fetch_one(&self.pool)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
      return self.get_user_obj_by_user_info(updated_user_info).await;
    }

    // modderator can update user permission but cannot update others user info and roles
    if input.is_moderator {
      if let Some(permissions) = input.permissions {
        self
          .update_permissions(permissions.extract_ids(), user_id)
          .await?;
      }
      return self.get_user_by_id(user_id).await;
    }

    // admin can update user roles, and permissions but cannot update others user info
    if input.is_admin {
      if let Some(roles) = input.roles {
        self.update_roles(roles.extract_ids(), user_id).await?;
      }
      if let Some(permissions) = input.permissions {
        self
          .update_permissions(permissions.extract_ids(), user_id)
          .await?;
      }
      return self.get_user_by_id(user_id).await;
    };

    Ok(user)
  }

  pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
    let user_info: UserInfo = sqlx::query_as(
      r#"
      SELECT id, username, password, created_at, updated_at
      FROM users
      WHERE id = $1
      "#,
    )
    .bind(user_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?
    .ok_or(AppError::NotFound(format!("User: {} not found", user_id)))?;

    let user = self.get_user_obj_by_user_info(user_info).await?;

    Ok(user)
  }

  pub async fn get_user_by_username(&self, username: &str) -> Result<User, AppError> {
    let user_info: UserInfo = sqlx::query_as(
      r#"
      SELECT id, username, created_at, updated_at
      FROM users
      WHERE username = $1
      "#,
    )
    .bind(username)
    .fetch_one(&self.pool)
    .await
    .map_err(|_| AppError::NotFound(format!("User: {} not found", username)))?;

    let user = self.get_user_obj_by_user_info(user_info).await?;
    Ok(user)
  }

  pub async fn verify_user_by_username(&self, username: &str) -> Result<User, AppError> {
    let user_info: UserInfo = sqlx::query_as(
      r#"
      SELECT id, username, password, created_at, updated_at
      FROM users
      WHERE username = $1
      "#,
    )
    .bind(username)
    .fetch_one(&self.pool)
    .await
    .map_err(|_| AppError::NotFound(format!("User: {} not found", username)))?;

    let user = self.get_user_obj_by_user_info(user_info).await?;
    Ok(user)
  }

  pub async fn get_users(&self, limit: i64, offset: i64) -> Result<PaginatedUsers, AppError> {
    // Optimized approach: fetch users with their basic info and then batch fetch roles/permissions
    let users_info = sqlx::query_as::<_, UserInfo>(
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

    if users_info.is_empty() {
      let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;

      return Ok(PaginatedUsers {
        users: vec![],
        total_count: total_count.0,
      });
    }

    // Batch fetch all roles for these users
    let user_ids: Vec<i32> = users_info.iter().map(|u| u.id).collect();
    let user_roles_map: std::collections::HashMap<i32, Vec<Role>> = sqlx::query!(
      r#"
      SELECT
        ur.user_id,
        r.id,
        r.name,
        r.created_at,
        r.updated_at
      FROM user_roles ur
      JOIN roles r ON ur.role_id = r.id
      WHERE ur.user_id = ANY($1)
      "#,
      &user_ids
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?
    .into_iter()
    .fold(std::collections::HashMap::new(), |mut map, row| {
      let role = Role {
        id: row.id,
        name: row.name,
        created_at: row.created_at,
        updated_at: row.updated_at,
      };
      map.entry(row.user_id).or_insert_with(Vec::new).push(role);
      map
    });

    // Batch fetch all permissions for these users
    let user_permissions_map: std::collections::HashMap<i32, Vec<Permission>> = sqlx::query!(
      r#"
      SELECT
        up.user_id,
        p.id,
        p.name,
        p.created_at,
        p.updated_at
      FROM user_permissions up
      JOIN permissions p ON up.permission_id = p.id
      WHERE up.user_id = ANY($1)
      "#,
      &user_ids
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?
    .into_iter()
    .fold(std::collections::HashMap::new(), |mut map, row| {
      let permission = Permission {
        id: row.id,
        name: row.name,
        created_at: row.created_at,
        updated_at: row.updated_at,
      };
      map
        .entry(row.user_id)
        .or_insert_with(Vec::new)
        .push(permission);
      map
    });

    // Construct User objects efficiently
    let users: Vec<User> = users_info
      .into_iter()
      .map(|user_info| {
        let roles = user_roles_map
          .get(&user_info.id)
          .cloned()
          .unwrap_or_default();
        let permissions = user_permissions_map
          .get(&user_info.id)
          .cloned()
          .unwrap_or_default();

        User::new(
          UserInfo {
            password: String::new(), // Not needed for listing
            ..user_info
          },
          roles,
          permissions,
        )
      })
      .collect();

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

  async fn get_user_obj_by_user_info(&self, user_info: UserInfo) -> Result<User, AppError> {
    let roles = self.get_user_roles(user_info.id).await?;
    let permissions = self.get_user_permissions(user_info.id).await?;
    let user = User::new(user_info, roles, permissions);
    Ok(user)
  }

  pub async fn get_user_roles(&self, user_id: i32) -> Result<Vec<Role>, AppError> {
    let roles = sqlx::query_as(
      r#"
      SELECT r.id, r.name, r.created_at, r.updated_at
      FROM roles r
      INNER JOIN user_roles ur ON r.id = ur.role_id
      WHERE ur.user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(roles)
  }

  pub async fn get_user_permissions(&self, user_id: i32) -> Result<Vec<Permission>, AppError> {
    let permissions = sqlx::query_as(
      r#"
      SELECT p.id, p.name, p.created_at, p.updated_at
      FROM permissions p
      INNER JOIN user_permissions up ON p.id = up.permission_id
      WHERE up.user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(permissions)
  }

  pub async fn is_user_exists_by_id(&self, user_id: i32) -> Result<bool, AppError> {
    let result = sqlx::query_scalar(
      r#"
      SELECT EXISTS (
        SELECT 1
        FROM users
        WHERE id = $1
      )
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(result)
  }

  pub async fn is_user_exists_by_username(&self, username: &str) -> Result<bool, AppError> {
    let result = sqlx::query_scalar(
      r#"
      SELECT EXISTS (
        SELECT 1
        FROM users
        WHERE username = $1
      )
      "#,
    )
    .bind(username)
    .fetch_one(&self.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(result)
  }

  pub async fn update_permissions(
    &self,
    permission_ids: Vec<i32>,
    user_id: i32,
  ) -> Result<(), AppError> {
    let mut transaction = self
      .pool
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // check permission_ids is valid in current user role's permissions
    let role_permissions: Vec<i32> = sqlx::query_scalar(
      r#"
      SELECT DISTINCT p.id
      FROM permissions p
      JOIN role_permissions rp ON rp.permission_id = p.id
      JOIN user_roles ur ON ur.role_id = rp.role_id
      WHERE ur.user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    for permission_id in &permission_ids {
      if !role_permissions.contains(permission_id) {
        return Err(AppError::BadRequest(format!(
          "Permission: {} is not valid for user: {}",
          permission_id, user_id
        )));
      }
    }

    // compute need delete and insert permission ids
    let old_permissions: HashSet<i32> = sqlx::query_scalar(
      r#"
      SELECT permission_id
      FROM user_permissions
      WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_all(&mut *transaction)
    .await?
    .into_iter()
    .collect();

    let new_permissions: HashSet<i32> = permission_ids.iter().cloned().collect();

    let permissions_to_delete: Vec<i32> = old_permissions
      .difference(&new_permissions)
      .cloned()
      .collect();
    let permissions_to_insert: Vec<i32> = new_permissions
      .difference(&old_permissions)
      .cloned()
      .collect();

    // delete user permissions in user_permissions by permissions_to_delete
    for permission_id in permissions_to_delete {
      sqlx::query(
        r#"
        DELETE FROM user_permissions
        WHERE user_id = $1
        AND permission_id = $2
        "#,
      )
      .bind(user_id)
      .bind(permission_id)
      .execute(&mut *transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }

    // insert user permissions in user_permissions by permissions_to_insert
    for permission_id in permissions_to_insert {
      sqlx::query(
        r#"
        INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, permission_id)
        DO UPDATE SET updated_at = EXCLUDED.updated_at
        "#,
      )
      .bind(user_id)
      .bind(permission_id)
      .bind(Utc::now())
      .bind(Utc::now())
      .execute(&mut *transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }

    // commit the transaction
    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(())
  }

  async fn update_roles(&self, role_ids: Vec<i32>, user_id: i32) -> Result<(), AppError> {
    let mut transaction = self
      .pool
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(format!("Failed to begin transaction: {}", err)))?;

    // get current user permissions
    let old_permissions: HashSet<i32> = sqlx::query_scalar(
      r#"
      SELECT permission_id
      FROM user_permissions
      WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_all(&mut *transaction)
    .await?
    .into_iter()
    .collect();

    // get current user role ids
    let current_role_ids: HashSet<i32> = sqlx::query_scalar(
      r#"
       SELECT role_id
       FROM user_roles
       WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_all(&mut *transaction)
    .await?
    .into_iter()
    .collect();

    // compute need delete and insert role ids
    let new_role_ids: HashSet<i32> = role_ids.into_iter().collect();

    let roles_to_delete: Vec<i32> = current_role_ids
      .difference(&new_role_ids)
      .cloned()
      .collect();
    let roles_to_insert: Vec<i32> = new_role_ids
      .difference(&current_role_ids)
      .cloned()
      .collect();

    // delete user roles in user_roles by roles_to_delete
    for role_id in roles_to_delete {
      sqlx::query(
        r#"
        DELETE FROM user_roles
        WHERE user_id = $1
        AND role_id = $2
        "#,
      )
      .bind(user_id)
      .bind(role_id)
      .execute(&mut *transaction)
      .await
      .map_err(|err| {
        AppError::DatabaseError(format!("Failed to delete role_id: {}, {}", role_id, err))
      })?;
    }

    for role_id in roles_to_insert {
      sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, role_id)
        DO UPDATE SET updated_at = EXCLUDED.updated_at
        "#,
      )
      .bind(user_id)
      .bind(role_id)
      .bind(Utc::now())
      .bind(Utc::now())
      .execute(&mut *transaction)
      .await
      .map_err(|err| {
        AppError::DatabaseError(format!("Failed to insert role_id {}: {}", role_id, err))
      })?;
    }

    // get all of new role's permissions
    let mut new_permissions = HashSet::<i32>::new();
    for role_id in &new_role_ids {
      let permissions: Vec<i32> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT permission_id
        FROM role_permissions
        WHERE role_id = $1
        "#,
      )
      .bind(role_id)
      .fetch_all(&mut *transaction)
      .await
      .map_err(|err| {
        AppError::DatabaseError(format!(
          "Failed to fetch permissions for role_id {}: {}",
          role_id, err
        ))
      })?;

      new_permissions.extend(permissions);
    }

    let permissions_to_delete: Vec<i32> = old_permissions
      .difference(&new_permissions)
      .cloned()
      .collect();
    let permissions_to_insert: Vec<i32> = new_permissions
      .difference(&old_permissions)
      .cloned()
      .collect();

    // delete user permissions in user_permissions by permissions_to_delete
    for permission_id in permissions_to_delete {
      sqlx::query(
        r#"
        DELETE FROM user_permissions
        WHERE user_id = $1
        AND permission_id = $2
        "#,
      )
      .bind(user_id)
      .bind(permission_id)
      .execute(&mut *transaction)
      .await
      .map_err(|err| {
        AppError::DatabaseError(format!(
          "Failed to delete permission_id {}: {}",
          permission_id, err
        ))
      })?;
    }

    for permission_id in permissions_to_insert {
      sqlx::query(
        r#"
        INSERT INTO user_permissions (user_id, permission_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, permission_id)
        DO UPDATE SET updated_at = EXCLUDED.updated_at
        "#,
      )
      .bind(user_id)
      .bind(permission_id)
      .bind(Utc::now())
      .bind(Utc::now())
      .execute(&mut *transaction)
      .await
      .map_err(|err| {
        AppError::DatabaseError(format!(
          "Failed to insert permission_id {}: {}",
          permission_id, err
        ))
      })?;
    }

    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(format!("Failed to commit transaction: {}", err)))?;
    Ok(())
  }

  pub async fn get_role_by_claim(&self, claims: &User, user_id: i32) -> Result<IsWho, AppError> {
    let is_own_user = claims.user_info.id == user_id;
    let is_moderator = claims.roles.contains_name(RoleName::Moderator);
    let is_admin = claims.roles.contains_name(RoleName::Admin);

    Ok(IsWho::new(is_own_user, is_moderator, is_admin))
  }
}
