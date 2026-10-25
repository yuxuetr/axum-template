use std::collections::{HashMap, HashSet};

use crate::AppState;
use crate::common::errors::AppError;
use crate::modules::users::dto::{CreateUser, IsWho, UpdateUser, User};
use crate::modules::users::entity::{
  Permission, Role, RoleName, RolePermission, UserInfo, UserPermission, UserRole, VecExtensions,
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

    let mut db = self.db.clone();
    let mut transaction = db
      .transaction()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let user_info = UserInfo::create()
      .username(&input.username)
      .password(hashed_password)
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // Default role User, ID: 1
    UserRole::create()
      .user_id(user_info.id)
      .role_id(1)
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let roles = Role::filter_by_id(1).exec(&mut transaction).await?;

    let role_perms = RolePermission::filter_by_role_id(1)
      .exec(&mut transaction)
      .await?;
    let perm_ids: Vec<i32> = role_perms.iter().map(|rp| rp.permission_id).collect();

    let permissions = if perm_ids.is_empty() {
      vec![]
    } else {
      Permission::filter(Permission::fields().id().in_list(perm_ids))
        .exec(&mut transaction)
        .await?
    };

    // insert user permissions in user_permissions
    for permission in &permissions {
      UserPermission::create()
        .user_id(user_info.id)
        .permission_id(permission.id)
        .exec(&mut transaction)
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
        )));
      }
    }
    let mut db = self.db.clone();
    let mut transaction = db
      .transaction()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user roles in user_roles
    UserRole::filter_by_user_id(user_id)
      .delete()
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // delete user permissions in user_permissions
    UserPermission::filter_by_user_id(user_id)
      .delete()
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // check and delete user
    let existing = UserInfo::filter_by_id(user_id)
      .first()
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    UserInfo::filter_by_id(user_id)
      .delete()
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // commit the transaction
    transaction
      .commit()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    match existing {
      Some(_) => Ok(()),
      None => Err(AppError::NotFound(format!(
        "User with id {} not found",
        user_id
      ))),
    }
  }

  pub async fn update_user(&self, user_id: i32, input: UpdateUser) -> Result<User, AppError> {
    if !input.is_own_user && !input.is_admin && !input.is_moderator {
      return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    match self.is_user_exists_by_id(user_id).await? {
      true => (),
      false => {
        return Err(AppError::NotFound(format!(
          "User with id {} not found",
          user_id
        )));
      }
    }

    let user = self.get_user_by_id(user_id).await?;

    // Admin: can update roles, permissions, and own info when updating self
    if input.is_admin {
      if input.is_own_user {
        let mut db = self.db.clone();
        let mut user_info = user.user_info.clone();
        let new_username = input.username.unwrap_or_else(|| user_info.username.clone());
        let hashed_password = if let Some(password) = input.password {
          hash_password(&password)?
        } else {
          user_info.password.clone()
        };
        user_info
          .update()
          .username(new_username)
          .password(hashed_password)
          .exec(&mut db)
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
      let mut db = self.db.clone();
      let mut updated_user_info = user.user_info;
      let new_username = input
        .username
        .unwrap_or_else(|| updated_user_info.username.clone());
      let hashed_password = if let Some(password) = input.password {
        hash_password(&password)?
      } else {
        updated_user_info.password.clone()
      };
      updated_user_info
        .update()
        .username(new_username)
        .password(hashed_password)
        .exec(&mut db)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
      return self.get_user_obj_by_user_info(updated_user_info).await;
    }

    Ok(user)
  }

  pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
    let mut db = self.db.clone();
    let user_info = UserInfo::filter_by_id(user_id)
      .first()
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?
      .ok_or(AppError::NotFound(format!("User: {} not found", user_id)))?;

    let user = self.get_user_obj_by_user_info(user_info).await?;

    Ok(user)
  }

  pub async fn get_user_by_username(&self, username: &str) -> Result<User, AppError> {
    let mut db = self.db.clone();
    let mut user_info = UserInfo::filter_by_username(username)
      .first()
      .exec(&mut db)
      .await
      .map_err(|_| AppError::NotFound(format!("User: {} not found", username)))?
      .ok_or_else(|| AppError::NotFound(format!("User: {} not found", username)))?;
    user_info.password = String::new();

    let user = self.get_user_obj_by_user_info(user_info).await?;
    Ok(user)
  }

  pub async fn verify_user_by_username(&self, username: &str) -> Result<User, AppError> {
    let mut db = self.db.clone();
    let user_info = UserInfo::filter_by_username(username)
      .first()
      .exec(&mut db)
      .await
      .map_err(|_| AppError::NotFound(format!("User: {} not found", username)))?
      .ok_or_else(|| AppError::NotFound(format!("User: {} not found", username)))?;

    let user = self.get_user_obj_by_user_info(user_info).await?;
    Ok(user)
  }

  pub async fn get_users(&self, limit: i64, offset: i64) -> Result<PaginatedUsers, AppError> {
    let mut db = self.db.clone();
    let limit_usize = usize::try_from(limit.max(0)).unwrap_or(0);
    let offset_usize = usize::try_from(offset.max(0)).unwrap_or(0);

    let users_info = UserInfo::all()
      .order_by(UserInfo::fields().id().asc())
      .limit(limit_usize)
      .offset(offset_usize)
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let total_count = UserInfo::all()
      .count()
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))? as i64;

    if users_info.is_empty() {
      return Ok(PaginatedUsers {
        users: vec![],
        total_count,
      });
    }

    let user_ids: Vec<i32> = users_info.iter().map(|u| u.id).collect();

    // Batch fetch all roles for these users
    let user_roles = UserRole::filter(UserRole::fields().user_id().in_list(user_ids.clone()))
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let role_ids: Vec<i32> = user_roles
      .iter()
      .map(|ur| ur.role_id)
      .collect::<HashSet<_>>()
      .into_iter()
      .collect();

    let roles_by_id: HashMap<i32, Role> = if role_ids.is_empty() {
      HashMap::new()
    } else {
      Role::filter(Role::fields().id().in_list(role_ids))
        .exec(&mut db)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?
        .into_iter()
        .map(|r| (r.id, r))
        .collect()
    };

    let user_roles_map: HashMap<i32, Vec<Role>> =
      user_roles.into_iter().fold(HashMap::new(), |mut map, ur| {
        if let Some(role) = roles_by_id.get(&ur.role_id) {
          map.entry(ur.user_id).or_default().push(role.clone());
        }
        map
      });

    // Batch fetch all permissions for these users
    let user_permissions =
      UserPermission::filter(UserPermission::fields().user_id().in_list(user_ids))
        .exec(&mut db)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let perm_ids: Vec<i32> = user_permissions
      .iter()
      .map(|up| up.permission_id)
      .collect::<HashSet<_>>()
      .into_iter()
      .collect();

    let perms_by_id: HashMap<i32, Permission> = if perm_ids.is_empty() {
      HashMap::new()
    } else {
      Permission::filter(Permission::fields().id().in_list(perm_ids))
        .exec(&mut db)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?
        .into_iter()
        .map(|p| (p.id, p))
        .collect()
    };

    let user_permissions_map: HashMap<i32, Vec<Permission>> =
      user_permissions
        .into_iter()
        .fold(HashMap::new(), |mut map, up| {
          if let Some(perm) = perms_by_id.get(&up.permission_id) {
            map.entry(up.user_id).or_default().push(perm.clone());
          }
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

    Ok(PaginatedUsers { users, total_count })
  }

  async fn get_user_obj_by_user_info(&self, user_info: UserInfo) -> Result<User, AppError> {
    let roles = self.get_user_roles(user_info.id).await?;
    let permissions = self.get_user_permissions(user_info.id).await?;
    let user = User::new(user_info, roles, permissions);
    Ok(user)
  }

  pub async fn get_user_roles(&self, user_id: i32) -> Result<Vec<Role>, AppError> {
    let mut db = self.db.clone();
    let user_roles = UserRole::filter_by_user_id(user_id)
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let role_ids: Vec<i32> = user_roles.into_iter().map(|ur| ur.role_id).collect();
    if role_ids.is_empty() {
      return Ok(vec![]);
    }

    let roles = Role::filter(Role::fields().id().in_list(role_ids))
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(roles)
  }

  pub async fn get_user_permissions(&self, user_id: i32) -> Result<Vec<Permission>, AppError> {
    let mut db = self.db.clone();
    let user_permissions = UserPermission::filter_by_user_id(user_id)
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    let perm_ids: Vec<i32> = user_permissions
      .into_iter()
      .map(|up| up.permission_id)
      .collect();
    if perm_ids.is_empty() {
      return Ok(vec![]);
    }

    let permissions = Permission::filter(Permission::fields().id().in_list(perm_ids))
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    Ok(permissions)
  }

  pub async fn is_user_exists_by_id(&self, user_id: i32) -> Result<bool, AppError> {
    let mut db = self.db.clone();
    let user = UserInfo::filter_by_id(user_id)
      .first()
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(user.is_some())
  }

  pub async fn is_user_exists_by_username(&self, username: &str) -> Result<bool, AppError> {
    let mut db = self.db.clone();
    let user = UserInfo::filter_by_username(username)
      .first()
      .exec(&mut db)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(user.is_some())
  }

  pub async fn update_permissions(
    &self,
    permission_ids: Vec<i32>,
    user_id: i32,
  ) -> Result<(), AppError> {
    let mut db = self.db.clone();
    let mut transaction = db
      .transaction()
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;

    // check permission_ids is valid in current user role's permissions
    let user_roles = UserRole::filter_by_user_id(user_id)
      .exec(&mut transaction)
      .await
      .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    let role_ids: Vec<i32> = user_roles.into_iter().map(|ur| ur.role_id).collect();

    let role_permissions: HashSet<i32> = if role_ids.is_empty() {
      HashSet::new()
    } else {
      RolePermission::filter(RolePermission::fields().role_id().in_list(role_ids))
        .exec(&mut transaction)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?
        .into_iter()
        .map(|rp| rp.permission_id)
        .collect()
    };

    for permission_id in &permission_ids {
      if !role_permissions.contains(permission_id) {
        return Err(AppError::BadRequest(format!(
          "Permission: {} is not valid for user: {}",
          permission_id, user_id
        )));
      }
    }

    // compute need delete and insert permission ids
    let old_permissions: HashSet<i32> = UserPermission::filter_by_user_id(user_id)
      .exec(&mut transaction)
      .await?
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

    // delete user permissions in user_permissions by permissions_to_delete
    for permission_id in permissions_to_delete {
      UserPermission::delete_by_user_id_and_permission_id(&mut transaction, user_id, permission_id)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    }

    // insert user permissions in user_permissions by permissions_to_insert
    for permission_id in permissions_to_insert {
      UserPermission::create()
        .user_id(user_id)
        .permission_id(permission_id)
        .exec(&mut transaction)
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
    let mut db = self.db.clone();
    let mut transaction = db
      .transaction()
      .await
      .map_err(|err| AppError::DatabaseError(format!("Failed to begin transaction: {}", err)))?;

    // get current user permissions
    let old_permissions: HashSet<i32> = UserPermission::filter_by_user_id(user_id)
      .exec(&mut transaction)
      .await?
      .into_iter()
      .map(|up| up.permission_id)
      .collect();

    // get current user role ids
    let current_role_ids: HashSet<i32> = UserRole::filter_by_user_id(user_id)
      .exec(&mut transaction)
      .await?
      .into_iter()
      .map(|ur| ur.role_id)
      .collect();

    // compute need delete and insert role ids
    let new_role_ids: HashSet<i32> = role_ids.into_iter().collect();

    let roles_to_delete: Vec<i32> = current_role_ids
      .difference(&new_role_ids)
      .copied()
      .collect();
    let roles_to_insert: Vec<i32> = new_role_ids
      .difference(&current_role_ids)
      .copied()
      .collect();

    // delete user roles in user_roles by roles_to_delete
    for role_id in roles_to_delete {
      UserRole::delete_by_user_id_and_role_id(&mut transaction, user_id, role_id)
        .await
        .map_err(|err| {
          AppError::DatabaseError(format!("Failed to delete role_id: {}, {}", role_id, err))
        })?;
    }

    for role_id in roles_to_insert {
      UserRole::create()
        .user_id(user_id)
        .role_id(role_id)
        .exec(&mut transaction)
        .await
        .map_err(|err| {
          AppError::DatabaseError(format!("Failed to insert role_id {}: {}", role_id, err))
        })?;
    }

    // get all of new role's permissions
    let mut new_permissions = HashSet::<i32>::new();
    for role_id in &new_role_ids {
      let permissions: Vec<i32> = RolePermission::filter_by_role_id(*role_id)
        .exec(&mut transaction)
        .await
        .map_err(|err| {
          AppError::DatabaseError(format!(
            "Failed to fetch permissions for role_id {}: {}",
            role_id, err
          ))
        })?
        .into_iter()
        .map(|rp| rp.permission_id)
        .collect();

      new_permissions.extend(permissions);
    }

    let permissions_to_delete: Vec<i32> = old_permissions
      .difference(&new_permissions)
      .copied()
      .collect();
    let permissions_to_insert: Vec<i32> = new_permissions
      .difference(&old_permissions)
      .copied()
      .collect();

    // delete user permissions in user_permissions by permissions_to_delete
    for permission_id in permissions_to_delete {
      UserPermission::delete_by_user_id_and_permission_id(&mut transaction, user_id, permission_id)
        .await
        .map_err(|err| {
          AppError::DatabaseError(format!(
            "Failed to delete permission_id {}: {}",
            permission_id, err
          ))
        })?;
    }

    for permission_id in permissions_to_insert {
      UserPermission::create()
        .user_id(user_id)
        .permission_id(permission_id)
        .exec(&mut transaction)
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
