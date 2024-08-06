use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub trait VecExtensions<T: AsRef<str> + Eq> {
  fn extract_ids(&self) -> Vec<i32>;
  fn extract_names(&self) -> Vec<String>;
  fn contains_name(&self, name: T) -> bool;
}

/// users table
#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
pub struct UserInfo {
  pub id: i32,
  pub username: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// roles table
#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
pub struct Role {
  pub id: i32,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// permissions table
#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
pub struct Permission {
  pub id: i32,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// role names
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum RoleName {
  Admin,
  Moderator,
  User,
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
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

impl Role {
  pub fn new(id: i32, role: &str) -> Self {
    Self {
      id,
      name: role.to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

impl Permission {
  pub fn new(id: i32, permission: &str) -> Self {
    Self {
      id,
      name: permission.to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
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
      .any(|name| RoleName::from_str(name).map_or(false, |rn| rn == role_name))
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
      .any(|name| PermissionName::from_str(name).map_or(false, |p_name| p_name == permission_name))
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
