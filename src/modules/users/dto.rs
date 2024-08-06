use super::{Permission, PermissionName, Role, RoleName, UserInfo, VecExtensions};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Input Dto
/// user create input dto
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateUser {
  #[validate(length(
    min = 3,
    max = 50,
    message = "username length must be between 3 and 50 characters"
  ))]
  pub username: String,
  #[validate(length(
    min = 6,
    max = 50,
    message = "password length must be between 8 and 50 characters"
  ))]
  pub password: String,
}

/// user update input dto with is_who
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateUser {
  pub username: Option<String>,
  pub password: Option<String>,
  pub roles: Option<Vec<RoleIn>>,
  pub permissions: Option<Vec<PermissionIn>>,
  pub is_own_user: bool,
  pub is_moderator: bool,
  pub is_admin: bool,
}

/// user update input dto
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct UpdateUserOptions {
  #[validate(length(
    min = 3,
    max = 50,
    message = "username length must be between 3 and 50 characters"
  ))]
  pub username: Option<String>,
  #[validate(length(
    min = 6,
    max = 50,
    message = "password length must be between 8 and 50 characters"
  ))]
  pub password: Option<String>,
  #[validate(nested)]
  pub roles: Option<Vec<RoleIn>>,
  #[validate(nested)]
  pub permissions: Option<Vec<PermissionIn>>,
}

/// current user role input dto
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IsWho {
  pub is_own_user: bool,
  pub is_moderator: bool,
  pub is_admin: bool,
}

/// update role input dto
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct RoleIn {
  #[validate(range(min = 1, max = 4))]
  pub id: i32,
  #[validate(length(min = 3, max = 50))]
  pub name: String,
}

/// update permission input dto
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct PermissionIn {
  #[validate(range(min = 1, max = 12))]
  pub id: i32,
  #[validate(length(min = 3, max = 50))]
  pub name: String,
}

impl IsWho {
  pub fn new(is_own_user: bool, is_moderator: bool, is_admin: bool) -> Self {
    Self {
      is_own_user,
      is_moderator,
      is_admin,
    }
  }
}

impl UpdateUser {
  pub fn new(input: UpdateUserOptions, is_who: IsWho) -> Self {
    Self {
      username: input.username,
      password: input.password,
      roles: input.roles,
      permissions: input.permissions,
      is_own_user: is_who.is_own_user,
      is_moderator: is_who.is_moderator,
      is_admin: is_who.is_admin,
    }
  }
}

/// implement `VecExtensions` trait for `Vec<RoleIn>``
impl VecExtensions<RoleName> for Vec<RoleIn> {
  fn extract_ids(&self) -> Vec<i32> {
    self.iter().map(|role| role.id).collect()
  }

  fn extract_names(&self) -> Vec<String> {
    self.iter().map(|role| role.name.clone()).collect()
  }

  fn contains_name(&self, role_name: RoleName) -> bool {
    self
      .extract_names()
      .iter()
      .any(|name| RoleName::from_str(name).map_or(false, |rn| rn == role_name))
  }
}

/// implement `VecExtensions` trait for `Vec<PermissionIn>`
impl VecExtensions<PermissionName> for Vec<PermissionIn> {
  fn extract_ids(&self) -> Vec<i32> {
    self.iter().map(|role| role.id).collect()
  }

  fn extract_names(&self) -> Vec<String> {
    self.iter().map(|role| role.clone().name).collect()
  }

  fn contains_name(&self, permission_name: PermissionName) -> bool {
    self
      .extract_names()
      .iter()
      .any(|name| PermissionName::from_str(name).map_or(false, |p_name| p_name == permission_name))
  }
}

/// user pagination input dto
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PaginationParams {
  #[validate(range(min = 1, max = 100))]
  pub limit: i64,
  #[validate(range(min = 0))]
  pub offset: i64,
}

impl Default for PaginationParams {
  fn default() -> Self {
    Self {
      limit: 10,
      offset: 0,
    }
  }
}
/// Output Dto
/// user output dto
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
  pub user_info: UserInfo,
  pub roles: Vec<Role>,
  pub permissions: Vec<Permission>,
}

impl User {
  pub fn new(user_info: UserInfo, roles: Vec<Role>, permissions: Vec<Permission>) -> Self {
    Self {
      user_info,
      roles,
      permissions,
    }
  }
}

/// paginated users output dto
#[derive(Debug, Deserialize, Serialize)]
pub struct PaginatedUsers {
  pub users: Vec<User>,
  pub total_count: i64,
}
