use std::collections::{HashMap, HashSet};

use crate::AppState;
use crate::common::errors::AppError;
use crate::modules::users::dto::{CreateUser, IsWho, UpdateUser, User};
use crate::modules::users::entity::{
  Permission, Role, RoleName, UserInfo, UserPermissionRow, UserRoleRow, VecExtensions, permissions,
  role_permissions, roles, user_permissions, user_roles, users,
};

use chrono::Utc;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityLoaderTrait, EntityTrait, JoinType, PaginatorTrait,
  QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait,
  sea_query::OnConflict,
};

use super::dto::PaginatedUsers;
use crate::common::hash_password;

impl AppState {
  pub async fn create_user(&self, input: CreateUser) -> Result<User, AppError> {
    if self.is_user_exists_by_username(&input.username).await? {
      return Err(AppError::UserExisted(format!(
        "User: {} already exists",
        input.username
      )));
    }

    let hashed_password = hash_password(&input.password)?;

    let transaction = self
      .db
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let now = Utc::now();
    let new_user = users::ActiveModel {
      username: Set(input.username),
      password: Set(hashed_password),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    };

    let user_info = new_user
      .insert(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // default role User, ID: 1
    let new_user_role = user_roles::ActiveModel {
      user_id: Set(user_info.id),
      role_id: Set(1),
      created_at: Set(now),
      updated_at: Set(now),
    };
    new_user_role
      .insert(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let roles: Vec<Role> = roles::Entity::find_by_id(1)
      .all(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let permissions: Vec<Permission> = permissions::Entity::find()
      .join(
        JoinType::InnerJoin,
        permissions::Relation::RolePermissions.def(),
      )
      .join(
        JoinType::InnerJoin,
        role_permissions::Relation::Roles
          .def()
          .on_condition(|_left, _right| sea_orm::Condition::all()),
      )
      .join(JoinType::InnerJoin, roles::Relation::UserRoles.def())
      .filter(user_roles::Column::UserId.eq(user_info.id))
      .all(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // insert user permissions in user_permissions
    for permission in &permissions {
      let up = user_permissions::ActiveModel {
        user_id: Set(user_info.id),
        permission_id: Set(permission.id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      up.insert(&transaction)
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
    if !self.is_user_exists_by_id(user_id).await? {
      return Err(AppError::NotFound(format!(
        "User with id {} not found",
        user_id
      )));
    }

    let transaction = self
      .db
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user roles in user_roles
    user_roles::Entity::delete_many()
      .filter(user_roles::Column::UserId.eq(user_id))
      .exec(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user permissions in user_permissions
    user_permissions::Entity::delete_many()
      .filter(user_permissions::Column::UserId.eq(user_id))
      .exec(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user
    let result = users::Entity::delete_by_id(user_id)
      .exec(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    if result.rows_affected > 0 {
      Ok(())
    } else {
      Err(AppError::NotFound(format!(
        "User with id {} not found",
        user_id
      )))
    }
  }

  pub async fn update_user(&self, user_id: i32, input: UpdateUser) -> Result<User, AppError> {
    if !input.is_own_user && !input.is_admin && !input.is_moderator {
      return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    if !self.is_user_exists_by_id(user_id).await? {
      return Err(AppError::NotFound(format!(
        "User with id {} not found",
        user_id
      )));
    }

    let user = self.get_user_by_id(user_id).await?;

    // Admin: can update roles, permissions, and own info when updating self
    if input.is_admin {
      if input.is_own_user {
        let hashed_password = if let Some(password) = input.password {
          hash_password(&password)?
        } else {
          user.user_info.password.clone()
        };
        let mut active_user: users::ActiveModel = user.user_info.clone().into();
        active_user.username = Set(input.username.unwrap_or(user.user_info.username.clone()));
        active_user.password = Set(hashed_password);
        active_user.updated_at = Set(Utc::now());
        active_user
          .update(&self.db)
          .await
          .map_err(|err| AppError::DatabaseError(err.to_string()))?;
      }
      if let Some(roles) = input.roles {
        self.update_roles(roles.extract_ids(), user_id).await?;
      }
      if let Some(permissions) = input.permissions {
        self
          .update_permissions(permissions.extract_ids(), user_id)
          .await?;
      }
      return self.get_user_by_id(user_id).await;
    }

    // Moderator: can update user permissions only, not info or roles
    if input.is_moderator {
      if let Some(permissions) = input.permissions {
        self
          .update_permissions(permissions.extract_ids(), user_id)
          .await?;
      }
      return self.get_user_by_id(user_id).await;
    }

    // Own user: can update own info (username/password) only
    if input.is_own_user {
      let hashed_password = if let Some(password) = input.password {
        hash_password(&password)?
      } else {
        user.user_info.password.clone()
      };
      let mut active_user: users::ActiveModel = user.user_info.clone().into();
      active_user.username = Set(input.username.unwrap_or(user.user_info.username));
      active_user.password = Set(hashed_password);
      active_user.updated_at = Set(Utc::now());
      let updated_user_info = active_user
        .update(&self.db)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
      return self.get_user_obj_by_user_info(updated_user_info).await;
    }

    Ok(user)
  }

  pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
    let user_ex = users::Entity::load()
      .filter_by_id(user_id)
      .with(roles::Entity)
      .with(permissions::Entity)
      .one(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?
      .ok_or_else(|| AppError::NotFound(format!("User: {} not found", user_id)))?;

    Ok(Self::user_from_model_ex(user_ex))
  }

  pub async fn get_user_by_username(&self, username: &str) -> Result<User, AppError> {
    let user_ex = users::Entity::load()
      .filter(users::COLUMN.username.eq(username))
      .with(roles::Entity)
      .with(permissions::Entity)
      .one(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?
      .ok_or_else(|| AppError::NotFound(format!("User: {} not found", username)))?;

    Ok(Self::user_from_model_ex(user_ex))
  }

  pub async fn verify_user_by_username(&self, username: &str) -> Result<User, AppError> {
    self.get_user_by_username(username).await
  }

  pub async fn get_users(&self, limit: i64, offset: i64) -> Result<PaginatedUsers, AppError> {
    let limit_u64 = u64::try_from(limit).unwrap_or(10);
    let offset_u64 = u64::try_from(offset).unwrap_or(0);

    let users_info = users::Entity::find()
      .order_by_asc(users::Column::Id)
      .limit(limit_u64)
      .offset(offset_u64)
      .all(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let total_count_u64 = users::Entity::find()
      .count(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    let total_count = i64::try_from(total_count_u64).unwrap_or(0);

    if users_info.is_empty() {
      return Ok(PaginatedUsers {
        users: vec![],
        total_count,
      });
    }

    // Batch fetch all roles for these users
    let user_ids: Vec<i32> = users_info.iter().map(|u| u.id).collect();
    let user_roles_rows = roles::Entity::find()
      .select_only()
      .column_as(user_roles::Column::UserId, "user_id")
      .column(roles::Column::Id)
      .column(roles::Column::Name)
      .column(roles::Column::CreatedAt)
      .column(roles::Column::UpdatedAt)
      .join(JoinType::InnerJoin, roles::Relation::UserRoles.def())
      .filter(user_roles::Column::UserId.is_in(user_ids.clone()))
      .into_model::<UserRoleRow>()
      .all(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let user_roles_map: HashMap<i32, Vec<Role>> =
      user_roles_rows
        .into_iter()
        .fold(HashMap::new(), |mut map, row| {
          let role = Role {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
          };
          map.entry(row.user_id).or_default().push(role);
          map
        });

    // Batch fetch all permissions for these users
    let user_permissions_rows = permissions::Entity::find()
      .select_only()
      .column_as(user_permissions::Column::UserId, "user_id")
      .column(permissions::Column::Id)
      .column(permissions::Column::Name)
      .column(permissions::Column::CreatedAt)
      .column(permissions::Column::UpdatedAt)
      .join(
        JoinType::InnerJoin,
        permissions::Relation::UserPermissions.def(),
      )
      .filter(user_permissions::Column::UserId.is_in(user_ids))
      .into_model::<UserPermissionRow>()
      .all(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let user_permissions_map: HashMap<i32, Vec<Permission>> = user_permissions_rows
      .into_iter()
      .fold(HashMap::new(), |mut map, row| {
        let permission = Permission {
          id: row.id,
          name: row.name,
          created_at: row.created_at,
          updated_at: row.updated_at,
        };
        map.entry(row.user_id).or_default().push(permission);
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
            password: String::new(),
            ..user_info
          },
          roles,
          permissions,
        )
      })
      .collect();

    Ok(PaginatedUsers { users, total_count })
  }

  fn user_from_model_ex(user_ex: users::ModelEx) -> User {
    let roles: Vec<Role> = user_ex.roles.into_iter().map(Role::from).collect();
    let permissions: Vec<Permission> = user_ex
      .permissions
      .into_iter()
      .map(Permission::from)
      .collect();
    let user_info = UserInfo {
      id: user_ex.id,
      username: user_ex.username,
      password: user_ex.password,
      created_at: user_ex.created_at,
      updated_at: user_ex.updated_at,
    };
    User::new(user_info, roles, permissions)
  }

  async fn get_user_obj_by_user_info(&self, user_info: UserInfo) -> Result<User, AppError> {
    let roles = self.get_user_roles(user_info.id).await?;
    let permissions = self.get_user_permissions(user_info.id).await?;
    let user = User::new(user_info, roles, permissions);
    Ok(user)
  }

  pub async fn get_user_roles(&self, user_id: i32) -> Result<Vec<Role>, AppError> {
    let roles = roles::Entity::find()
      .join(JoinType::InnerJoin, roles::Relation::UserRoles.def())
      .filter(user_roles::Column::UserId.eq(user_id))
      .all(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(roles)
  }

  pub async fn get_user_permissions(&self, user_id: i32) -> Result<Vec<Permission>, AppError> {
    let permissions = permissions::Entity::find()
      .join(
        JoinType::InnerJoin,
        permissions::Relation::UserPermissions.def(),
      )
      .filter(user_permissions::Column::UserId.eq(user_id))
      .all(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(permissions)
  }

  pub async fn is_user_exists_by_id(&self, user_id: i32) -> Result<bool, AppError> {
    let count = users::Entity::find_by_id(user_id)
      .count(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(count > 0)
  }

  pub async fn is_user_exists_by_username(&self, username: &str) -> Result<bool, AppError> {
    let count = users::Entity::find()
      .filter(users::Column::Username.eq(username))
      .count(&self.db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(count > 0)
  }

  pub async fn update_permissions(
    &self,
    permission_ids: Vec<i32>,
    user_id: i32,
  ) -> Result<(), AppError> {
    let transaction = self
      .db
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // check permission_ids is valid in current user role's permissions
    let user_roles_list = user_roles::Entity::find()
      .filter(user_roles::Column::UserId.eq(user_id))
      .all(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let role_ids: Vec<i32> = user_roles_list.into_iter().map(|ur| ur.role_id).collect();
    let role_perms = if role_ids.is_empty() {
      vec![]
    } else {
      role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.is_in(role_ids))
        .all(&transaction)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?
    };

    let role_permissions: HashSet<i32> =
      role_perms.into_iter().map(|rp| rp.permission_id).collect();

    for permission_id in &permission_ids {
      if !role_permissions.contains(permission_id) {
        return Err(AppError::BadRequest(format!(
          "Permission: {} is not valid for user: {}",
          permission_id, user_id
        )));
      }
    }

    // compute need delete and insert permission ids
    let old_permissions: HashSet<i32> = user_permissions::Entity::find()
      .filter(user_permissions::Column::UserId.eq(user_id))
      .all(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?
      .into_iter()
      .map(|up| up.permission_id)
      .collect();

    let new_permissions: HashSet<i32> = permission_ids.iter().copied().collect();

    let permissions_to_delete: Vec<i32> = old_permissions
      .difference(&new_permissions)
      .copied()
      .collect();
    let permissions_to_insert: Vec<i32> = new_permissions
      .difference(&old_permissions)
      .copied()
      .collect();

    if !permissions_to_delete.is_empty() {
      user_permissions::Entity::delete_many()
        .filter(user_permissions::Column::UserId.eq(user_id))
        .filter(user_permissions::Column::PermissionId.is_in(permissions_to_delete))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }

    let now = Utc::now();
    for permission_id in permissions_to_insert {
      let model = user_permissions::ActiveModel {
        user_id: Set(user_id),
        permission_id: Set(permission_id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      user_permissions::Entity::insert(model)
        .on_conflict(
          OnConflict::columns([
            user_permissions::Column::UserId,
            user_permissions::Column::PermissionId,
          ])
          .update_column(user_permissions::Column::UpdatedAt)
          .to_owned(),
        )
        .exec(&transaction)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }

    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(())
  }

  async fn update_roles(&self, role_ids: Vec<i32>, user_id: i32) -> Result<(), AppError> {
    let transaction = self
      .db
      .begin()
      .await
      .map_err(|err| AppError::DatabaseError(format!("Failed to begin transaction: {}", err)))?;

    let old_permissions: HashSet<i32> = user_permissions::Entity::find()
      .filter(user_permissions::Column::UserId.eq(user_id))
      .all(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?
      .into_iter()
      .map(|up| up.permission_id)
      .collect();

    let current_role_ids: HashSet<i32> = user_roles::Entity::find()
      .filter(user_roles::Column::UserId.eq(user_id))
      .all(&transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?
      .into_iter()
      .map(|ur| ur.role_id)
      .collect();

    let new_role_ids: HashSet<i32> = role_ids.into_iter().collect();

    let roles_to_delete: Vec<i32> = current_role_ids
      .difference(&new_role_ids)
      .copied()
      .collect();
    let roles_to_insert: Vec<i32> = new_role_ids
      .difference(&current_role_ids)
      .copied()
      .collect();

    if !roles_to_delete.is_empty() {
      user_roles::Entity::delete_many()
        .filter(user_roles::Column::UserId.eq(user_id))
        .filter(user_roles::Column::RoleId.is_in(roles_to_delete))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::DatabaseError(format!("Failed to delete roles: {}", err)))?;
    }

    let now = Utc::now();
    for role_id in roles_to_insert {
      let model = user_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role_id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      user_roles::Entity::insert(model)
        .on_conflict(
          OnConflict::columns([user_roles::Column::UserId, user_roles::Column::RoleId])
            .update_column(user_roles::Column::UpdatedAt)
            .to_owned(),
        )
        .exec(&transaction)
        .await
        .map_err(|err| {
          AppError::DatabaseError(format!("Failed to insert role_id {}: {}", role_id, err))
        })?;
    }

    let new_role_ids_vec: Vec<i32> = new_role_ids.iter().copied().collect();
    let role_perms = if new_role_ids_vec.is_empty() {
      vec![]
    } else {
      role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.is_in(new_role_ids_vec))
        .all(&transaction)
        .await
        .map_err(|err| {
          AppError::DatabaseError(format!("Failed to fetch permissions for roles: {}", err))
        })?
    };

    let new_permissions: HashSet<i32> = role_perms.into_iter().map(|rp| rp.permission_id).collect();

    let permissions_to_delete: Vec<i32> = old_permissions
      .difference(&new_permissions)
      .copied()
      .collect();
    let permissions_to_insert: Vec<i32> = new_permissions
      .difference(&old_permissions)
      .copied()
      .collect();

    if !permissions_to_delete.is_empty() {
      user_permissions::Entity::delete_many()
        .filter(user_permissions::Column::UserId.eq(user_id))
        .filter(user_permissions::Column::PermissionId.is_in(permissions_to_delete))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::DatabaseError(format!("Failed to delete permissions: {}", err)))?;
    }

    for permission_id in permissions_to_insert {
      let model = user_permissions::ActiveModel {
        user_id: Set(user_id),
        permission_id: Set(permission_id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      user_permissions::Entity::insert(model)
        .on_conflict(
          OnConflict::columns([
            user_permissions::Column::UserId,
            user_permissions::Column::PermissionId,
          ])
          .update_column(user_permissions::Column::UpdatedAt)
          .to_owned(),
        )
        .exec(&transaction)
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
