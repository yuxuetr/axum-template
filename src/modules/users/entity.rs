use chrono::{DateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod users {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

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
  }

  #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
  pub enum Relation {
    #[sea_orm(has_many = "super::user_roles::Entity")]
    UserRoles,
    #[sea_orm(has_many = "super::user_permissions::Entity")]
    UserPermissions,
  }

  impl Related<super::user_roles::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::UserRoles.def()
    }
  }

  impl Related<super::user_permissions::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::UserPermissions.def()
    }
  }

  impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
      super::user_roles::Relation::Role.def()
    }

    fn via() -> Option<RelationDef> {
      Some(super::user_roles::Relation::User.def().rev())
    }
  }

  impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
      super::user_permissions::Relation::Permission.def()
    }

    fn via() -> Option<RelationDef> {
      Some(super::user_permissions::Relation::User.def().rev())
    }
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod roles {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "roles")]
  pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
  }

  #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
  pub enum Relation {
    #[sea_orm(has_many = "super::user_roles::Entity")]
    UserRoles,
    #[sea_orm(has_many = "super::role_permissions::Entity")]
    RolePermissions,
  }

  impl Related<super::user_roles::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::UserRoles.def()
    }
  }

  impl Related<super::role_permissions::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::RolePermissions.def()
    }
  }

  impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
      super::user_roles::Relation::User.def()
    }

    fn via() -> Option<RelationDef> {
      Some(super::user_roles::Relation::Role.def().rev())
    }
  }

  impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
      super::role_permissions::Relation::Permission.def()
    }

    fn via() -> Option<RelationDef> {
      Some(super::role_permissions::Relation::Role.def().rev())
    }
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod permissions {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "permissions")]
  pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
  }

  #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
  pub enum Relation {
    #[sea_orm(has_many = "super::role_permissions::Entity")]
    RolePermissions,
    #[sea_orm(has_many = "super::user_permissions::Entity")]
    UserPermissions,
  }

  impl Related<super::role_permissions::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::RolePermissions.def()
    }
  }

  impl Related<super::user_permissions::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::UserPermissions.def()
    }
  }

  impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
      super::role_permissions::Relation::Role.def()
    }

    fn via() -> Option<RelationDef> {
      Some(super::role_permissions::Relation::Permission.def().rev())
    }
  }

  impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
      super::user_permissions::Relation::User.def()
    }

    fn via() -> Option<RelationDef> {
      Some(super::user_permissions::Relation::Permission.def().rev())
    }
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod user_roles {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "user_roles")]
  pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
  }

  #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
  pub enum Relation {
    #[sea_orm(
      belongs_to = "super::users::Entity",
      from = "Column::UserId",
      to = "super::users::Column::Id",
      on_delete = "Cascade"
    )]
    User,
    #[sea_orm(
      belongs_to = "super::roles::Entity",
      from = "Column::RoleId",
      to = "super::roles::Column::Id",
      on_delete = "Cascade"
    )]
    Role,
  }

  impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::User.def()
    }
  }

  impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::Role.def()
    }
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod role_permissions {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "role_permissions")]
  pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
  }

  #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
  pub enum Relation {
    #[sea_orm(
      belongs_to = "super::roles::Entity",
      from = "Column::RoleId",
      to = "super::roles::Column::Id",
      on_delete = "Cascade"
    )]
    Role,
    #[sea_orm(
      belongs_to = "super::permissions::Entity",
      from = "Column::PermissionId",
      to = "super::permissions::Column::Id",
      on_delete = "Cascade"
    )]
    Permission,
  }

  impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::Role.def()
    }
  }

  impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::Permission.def()
    }
  }

  impl ActiveModelBehavior for ActiveModel {}
}

pub mod user_permissions {
  use chrono::{DateTime, Utc};
  use sea_orm::entity::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
  #[sea_orm(table_name = "user_permissions")]
  pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
  }

  #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
  pub enum Relation {
    #[sea_orm(
      belongs_to = "super::users::Entity",
      from = "Column::UserId",
      to = "super::users::Column::Id",
      on_delete = "Cascade"
    )]
    User,
    #[sea_orm(
      belongs_to = "super::permissions::Entity",
      from = "Column::PermissionId",
      to = "super::permissions::Column::Id",
      on_delete = "Cascade"
    )]
    Permission,
  }

  impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::User.def()
    }
  }

  impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
      Relation::Permission.def()
    }
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
