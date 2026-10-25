use chrono::{DateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod users {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[sea_orm::model]
  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "users")]
  pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    #[serde(skip)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(has_many)]
    pub user_roles: HasMany<super::user_roles::Entity>,
    #[sea_orm(has_many)]
    pub user_permissions: HasMany<super::user_permissions::Entity>,
    #[sea_orm(has_many, via = "user_roles")]
    pub roles: HasMany<super::roles::Entity>,
    #[sea_orm(has_many, via = "user_permissions")]
    pub permissions: HasMany<super::permissions::Entity>,
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod roles {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[sea_orm::model]
  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "roles")]
  pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(has_many)]
    pub user_roles: HasMany<super::user_roles::Entity>,
    #[sea_orm(has_many)]
    pub role_permissions: HasMany<super::role_permissions::Entity>,
    #[sea_orm(has_many, via = "user_roles")]
    pub users: HasMany<super::users::Entity>,
    #[sea_orm(has_many, via = "role_permissions")]
    pub permissions: HasMany<super::permissions::Entity>,
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod permissions {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[sea_orm::model]
  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "permissions")]
  pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(has_many)]
    pub role_permissions: HasMany<super::role_permissions::Entity>,
    #[sea_orm(has_many)]
    pub user_permissions: HasMany<super::user_permissions::Entity>,
    #[sea_orm(has_many, via = "role_permissions")]
    pub roles: HasMany<super::roles::Entity>,
    #[sea_orm(has_many, via = "user_permissions")]
    pub users: HasMany<super::users::Entity>,
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod user_roles {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[sea_orm::model]
  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "user_roles")]
  pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(belongs_to, from = "user_id", to = "id", on_delete = "Cascade")]
    pub user: BelongsTo<super::users::Entity>,
    #[sea_orm(belongs_to, from = "role_id", to = "id", on_delete = "Cascade")]
    pub role: BelongsTo<super::roles::Entity>,
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod role_permissions {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[sea_orm::model]
  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "role_permissions")]
  pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(belongs_to, from = "role_id", to = "id", on_delete = "Cascade")]
    pub role: BelongsTo<super::roles::Entity>,
    #[sea_orm(belongs_to, from = "permission_id", to = "id", on_delete = "Cascade")]
    pub permission: BelongsTo<super::permissions::Entity>,
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod user_permissions {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[sea_orm::model]
  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "user_permissions")]
  pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(belongs_to, from = "user_id", to = "id", on_delete = "Cascade")]
    pub user: BelongsTo<super::users::Entity>,
    #[sea_orm(belongs_to, from = "permission_id", to = "id", on_delete = "Cascade")]
    pub permission: BelongsTo<super::permissions::Entity>,
  }

  impl ActiveModelBehavior for ActiveModel {}
}

/// Helper row type for batch fetching roles with user_id
#[derive(Clone, Debug, FromQueryResult)]
pub struct UserRoleRow {
  pub user_id: i32,
  pub id: i32,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Helper row type for batch fetching permissions with user_id
#[derive(Clone, Debug, FromQueryResult)]
pub struct UserPermissionRow {
  pub user_id: i32,
  pub id: i32,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

pub trait VecExtensions<T: AsRef<str> + Eq> {
  fn extract_ids(&self) -> Vec<i32>;
  fn extract_names(&self) -> Vec<String>;
  fn contains_name(&self, name: T) -> bool;
}

/// users table
pub type UserInfo = users::Model;

/// roles table
pub type Role = roles::Model;

/// permissions table
pub type Permission = permissions::Model;

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
