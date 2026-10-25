use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub trait VecExtensions<T: AsRef<str> + Eq> {
  fn extract_ids(&self) -> Vec<i32>;
  fn extract_names(&self) -> Vec<String>;
  fn contains_name(&self, name: T) -> bool;
}

/// users table
#[derive(Clone, Debug, Deserialize, Serialize, toasty::Model)]
#[table = "users"]
pub struct UserInfo {
  #[key]
  #[auto]
  pub id: i32,
  #[unique]
  pub username: String,
  #[serde(skip)]
  #[default(String::new())]
  pub password: String,
  #[auto]
  pub created_at: Timestamp,
  #[auto]
  pub updated_at: Timestamp,
}

/// roles table
#[derive(Clone, Debug, Deserialize, Serialize, toasty::Model)]
#[table = "roles"]
pub struct Role {
  #[key]
  #[auto]
  pub id: i32,
  #[unique]
  pub name: String,
  #[auto]
  pub created_at: Timestamp,
  #[auto]
  pub updated_at: Timestamp,
}

/// permissions table
#[derive(Clone, Debug, Deserialize, Serialize, toasty::Model)]
#[table = "permissions"]
pub struct Permission {
  #[key]
  #[auto]
  pub id: i32,
  #[unique]
  pub name: String,
  #[auto]
  pub created_at: Timestamp,
  #[auto]
  pub updated_at: Timestamp,
}

/// user_roles table
#[derive(Clone, Debug, Deserialize, Serialize, toasty::Model)]
#[table = "user_roles"]
#[key(user_id, role_id)]
pub struct UserRole {
  #[index]
  pub user_id: i32,
  #[index]
  pub role_id: i32,
  #[auto]
  pub created_at: Timestamp,
  #[auto]
  pub updated_at: Timestamp,
}

/// role_permissions table
#[derive(Clone, Debug, Deserialize, Serialize, toasty::Model)]
#[table = "role_permissions"]
#[key(role_id, permission_id)]
pub struct RolePermission {
  #[index]
  pub role_id: i32,
  #[index]
  pub permission_id: i32,
  #[auto]
  pub created_at: Timestamp,
  #[auto]
  pub updated_at: Timestamp,
}

/// user_permissions table
#[derive(Clone, Debug, Deserialize, Serialize, toasty::Model)]
#[table = "user_permissions"]
#[key(user_id, permission_id)]
pub struct UserPermission {
  #[index]
  pub user_id: i32,
  #[index]
  pub permission_id: i32,
  #[auto]
  pub created_at: Timestamp,
  #[auto]
  pub updated_at: Timestamp,
}

/// role names
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum RoleName {
  Admin,
  Moderator,
  User,
}

impl FromStr for RoleName {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Admin" => Ok(RoleName::Admin),
      "Moderator" => Ok(RoleName::Moderator),
      "User" => Ok(RoleName::User),
      _ => Ok(RoleName::User), // Default to User for unknown roles
    }
  }
}

/// permission names
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PermissionName {
  Read,
  Write,
  Delete,
  ManagePermissions,
  ManageUsers,
  ManageRoles,
  ViewReports,
  EditSettings,
  UpdateUserInfo,
  UpdateUserRoles,
  UpdateUserPermissions,
}

/// new methods for the entities
impl UserInfo {
  pub fn new(username: &str, password: &str) -> Self {
    Self {
      id: 0,
      username: username.to_string(),
      password: password.to_string(),
      created_at: Timestamp::now(),
      updated_at: Timestamp::now(),
    }
  }
}

impl Role {
  pub fn new(id: i32, role: &str) -> Self {
    Self {
      id,
      name: role.to_string(),
      created_at: Timestamp::now(),
      updated_at: Timestamp::now(),
    }
  }
}

impl Permission {
  pub fn new(id: i32, permission: &str) -> Self {
    Self {
      id,
      name: permission.to_string(),
      created_at: Timestamp::now(),
      updated_at: Timestamp::now(),
    }
  }
}

impl RoleName {
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(name: &str) -> Option<Self> {
    match name {
      "Admin" => Some(RoleName::Admin),
      "Moderator" => Some(RoleName::Moderator),
      "User" => Some(RoleName::User),
      _ => None,
    }
  }
}

impl AsRef<str> for RoleName {
  fn as_ref(&self) -> &str {
    match *self {
      RoleName::Admin => "Admin",
      RoleName::Moderator => "Moderator",
      RoleName::User => "User",
    }
  }
}

impl VecExtensions<RoleName> for Vec<Role> {
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
      .any(|name| RoleName::from_str(name).is_some_and(|rn| rn == role_name))
  }
}

impl PermissionName {
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(name: &str) -> Option<Self> {
    match name {
      "READ" => Some(PermissionName::Read),
      "WRITE" => Some(PermissionName::Write),
      "DELETE" => Some(PermissionName::Delete),
      "MANAGE_PERMISSIONS" => Some(PermissionName::ManagePermissions),
      "MANAGE_USERS" => Some(PermissionName::ManageUsers),
      "MANAGE_ROLES" => Some(PermissionName::ManageRoles),
      "VIEW_REPORTS" => Some(PermissionName::ViewReports),
      "EDIT_SETTINGS" => Some(PermissionName::EditSettings),
      "UPDATE_USER_INFO" => Some(PermissionName::UpdateUserInfo),
      "UPDATE_USER_ROLES" => Some(PermissionName::UpdateUserRoles),
      "UPDATE_USER_PERMISSIONS" => Some(PermissionName::UpdateUserPermissions),
      _ => None,
    }
  }
}

impl VecExtensions<PermissionName> for Vec<Permission> {
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
      .any(|name| PermissionName::from_str(name).is_some_and(|p_name| p_name == permission_name))
  }
}

impl AsRef<str> for PermissionName {
  fn as_ref(&self) -> &str {
    match *self {
      PermissionName::Read => "READ",
      PermissionName::Write => "WRITE",
      PermissionName::Delete => "DELETE",
      PermissionName::ManagePermissions => "MANAGE_PERMISSIONS",
      PermissionName::ManageUsers => "MANAGE_USERS",
      PermissionName::ManageRoles => "MANAGE_ROLES",
      PermissionName::ViewReports => "VIEW_REPORTS",
      PermissionName::EditSettings => "EDIT_SETTINGS",
      PermissionName::UpdateUserInfo => "UPDATE_USER_INFO",
      PermissionName::UpdateUserRoles => "UPDATE_USER_ROLES",
      PermissionName::UpdateUserPermissions => "UPDATE_USER_PERMISSIONS",
    }
  }
}
