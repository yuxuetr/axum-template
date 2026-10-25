# Rust Axum Web 应用通用化模版

本项目实践使用 Axum 搭建后台服务，并提供 **三个分支** 分别演示 Rust 生态主流的三种异步数据库访问方案（共享完全一致的 REST API、JWT + RBAC 权限体系与数据库表结构）：

- **`main` 分支（默认）**: 使用 **SQLx v0.8.6** + PostgreSQL（原生 SQL、轻量直观）
- **`seaorm` 分支**: 使用 **SeaORM v2.0** + PostgreSQL（功能完备的 ActiveRecord / 关系型异步 ORM，基于 `sqlx v0.9` 与 `#[sea_orm::model]` 新一代实体宏）
- **`toasty` 分支**: 使用 **Toasty v0.11** + PostgreSQL（Tokio 官方生态的声明式异步 ORM）

此外项目还包括:
- 采用类似 Nest.js 的项目组织结构，使得项目的每个模块文件职责单一、清晰易读
- 完整的认证授权系统 (JWT + RBAC)
- 高性能的数据库查询优化（批量关联查询消除 N+1 问题）
- 完善的错误处理和日志记录
- 健康检查端点支持 (`/health`、`/health/live`、`/health/ready`)
- 包含完整的单元测试和集成测试代码
- Github Actions CI

---

## 🗄️ 三大数据库分支对比与使用指南

你可以通过 `git checkout <branch>` 切换到对应分支查看或作为项目脚手架使用：

```bash
git checkout main    # SQLx 版本
git checkout seaorm  # SeaORM 版本
git checkout toasty  # Toasty 版本
```

| 维度 | `main` 分支 (SQLx) | `seaorm` 分支 (SeaORM) | `toasty` 分支 (Toasty) |
| :--- | :--- | :--- | :--- |
| **核心依赖** | `sqlx = "0.8.6"` | `sea-orm = "2.0"` (`sqlx 0.9`) | `toasty = "0.11.0"` + `jiff = "0.2"` |
| **编程范式** | 原生 SQL + 行映射 (`FromRow`) | 新一代实体宏 (`#[sea_orm::model]` + `ActiveModel`) | 声明式模型派生 (`#[derive(toasty::Model)]`) |
| **状态连接类型** | `sqlx::PgPool` | `sea_orm::DatabaseConnection` | `toasty::Db` |
| **时间类型** | `chrono::DateTime<Utc>` | `chrono::DateTime<Utc>` | `jiff::Timestamp` |
| **测试隔离方案** | `sqlx-db-tester::TestPg` | 内置 `test_util::TestPg` 临时库生命周期管理 | 内置 `test_util::TestPg` 临时库生命周期管理 |
| **适用场景** | 偏好手写 SQL、极致掌控复杂查询与 CTE | 复杂关系模型、动态条件构建、企业级 CRUD | 追求极简模型定义、强类型链式构建器与自动时间戳维护 |

### 1. `main` 分支：SQLx 使用方式

- **依赖配置 (`Cargo.toml`)**：
  ```toml
  sqlx = { version = "0.8.6", features = [
    "runtime-tokio",
    "tls-rustls",
    "postgres",
    "macros",
    "chrono",
    "uuid",
  ] }
  ```
- **连接池初始化 (`src/lib.rs`)**：
  ```rust
  let pool = PgPool::connect(&config.database.db_url).await?;
  let state = AppState::new(config, pool);
  ```
- **实体定义 (`src/modules/users/entity.rs`)**：
  直接为结构体派生 `sqlx::FromRow`：
  ```rust
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
  ```
- **查询与事务 (`src/modules/users/services.rs`)**：
  使用 `sqlx::query_as`、`sqlx::query_scalar` 执行参数化 SQL，通过 `self.pool.begin().await?` 开启事务：
  ```rust
  let mut transaction = self.pool.begin().await?;
  let user_info = sqlx::query_as::<_, UserInfo>(
    "INSERT INTO users (username, password, created_at, updated_at) VALUES ($1, $2, $3, $4) RETURNING *"
  )
  .bind(&input.username)
  .bind(hashed_password)
  .bind(Utc::now())
  .bind(Utc::now())
  .fetch_one(&mut *transaction)
  .await?;
  transaction.commit().await?;
  ```

### 2. `seaorm` 分支：SeaORM 使用方式

- **依赖配置 (`Cargo.toml`)**：
  ```toml
  sea-orm = { version = "2.0", features = [
    "sqlx-postgres",
    "runtime-tokio-rustls",
    "macros",
    "with-chrono",
  ] }
  ```
- **连接初始化 (`src/lib.rs`)**：
  ```rust
  let db = sea_orm::Database::connect(&config.database.db_url).await?;
  let state = AppState::new(config, db);
  ```
- **实体与关系定义 (`src/modules/users/entity.rs`)**：
  采用 SeaORM 2.0 全新的 `#[sea_orm::model]` 属性宏，直接在 `Model` 结构体中通过 `HasMany<E>`、`BelongsTo<E>` 声明一对多与多对多关联（如 `users <-> user_roles <-> roles`），自动推导生成 `Relation` 枚举与 `Related<R>` Trait 实现，无需手写样板代码：
  ```rust
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
  ```
- **查询与事务 (`src/modules/users/services.rs`)**：
  使用 SeaORM 2.0 全新的智能实体加载器 `Entity::load().with(...)` 一步加载多对多关联实体（自动使用 DataLoader 消除 N+1 查询），结合类型安全列 `COLUMN` 与 `ActiveModel` + `TransactionTrait` 处理写入与事务：
  ```rust
  // 1. SeaORM 2.0 Smart Entity Loader 一步加载用户及其关联角色与权限
  let user_ex = users::Entity::load()
    .filter_by_id(user_id)
    .with(roles::Entity)
    .with(permissions::Entity)
    .one(&self.db)
    .await?;

  // 2. ActiveModel + 事务写入
  let txn = self.db.begin().await?;
  let user_info = users::ActiveModel {
    username: Set(input.username.clone()),
    password: Set(hashed_password),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&txn)
  .await?;
  txn.commit().await?;
  ```

### 3. `toasty` 分支：Toasty 使用方式

- **依赖配置 (`Cargo.toml`)**：
  ```toml
  toasty = { version = "0.11.0", features = ["postgresql", "jiff", "serde"] }
  jiff = { version = "0.2", features = ["serde"] }
  ```
- **数据库构建与模型注册 (`src/lib.rs`)**：
  通过 `toasty::models!(crate::*)` 自动扫描并注册 crate 内所有 `#[derive(toasty::Model)]` 模型：
  ```rust
  let db = toasty::Db::builder()
    .models(toasty::models!(crate::*))
    .connect(&config.database.db_url)
    .await?;
  let state = AppState::new(config, db);
  ```
- **声明式模型定义 (`src/modules/users/entity.rs`)**：
  使用 `#[derive(toasty::Model)]` 标注业务结构体，`#[key]`、`#[unique]`、`#[index]` 会自动生成对应的 `filter_by_*` / `delete_by_*` 方法，`#[auto]` 在 `created_at` 与 `updated_at` 上会自动维护创建与更新时间戳：
  ```rust
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
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
  }
  ```
- **链式查询构建器与事务 (`src/modules/users/services.rs`)**：
  使用生成的 `Model::create()`、`instance.update()`、`Model::filter_by_*()`、`in_list`、`count()` 以及 `db.transaction()`：
  ```rust
  let mut db = self.db.clone();
  let mut transaction = db.transaction().await?;

  let user_info = UserInfo::create()
    .username(&input.username)
    .password(hashed_password)
    .exec(&mut transaction)
    .await?;

  let roles = Role::filter(Role::fields().id().in_list(role_ids))
    .exec(&mut transaction)
    .await?;

  transaction.commit().await?;
  ```
  对于健康探活或自定义 SQL，也可直接使用 `toasty::sql::query("SELECT 1").exec(&mut db).await` 或 `toasty::sql::statement(...)`。

---

## 快速开始

### 环境要求
- Rust 1.85+ (Rust 2024 Edition)
- PostgreSQL 12+
- Docker (可选)

### 本地开发
```bash
# 1. 克隆项目并选择分支 (main / seaorm / toasty)
git clone https://github.com/yuxuetr/axum-template.git
cd axum-template
git checkout main # 或 git checkout seaorm / git checkout toasty

# 2. 创建数据库
createdb axum_template

# 3. 运行数据库迁移
sqlx migrate run

# 4. 启动服务
cargo run
```

服务将在 `http://localhost:3000` 启动。

### 运行测试
三个分支均支持通过 `DATABASE_URL` 环境变量指定测试所连接的 PostgreSQL 实例，测试框架会自动创建隔离的临时数据库并在测试结束后自动清理：

```bash
# 使用 cargo-nextest 运行全部单元与集成测试
DATABASE_URL="postgres://postgres:postgres@localhost:5432/postgres" cargo nextest run --all-features

# 或使用标准 cargo test
DATABASE_URL="postgres://postgres:postgres@localhost:5432/postgres" cargo test --all-features
```

### Docker 部署
```bash
# 构建镜像
docker build -t axum-template .

# 运行容器
docker run -p 3000:3000 -e DATABASE_URL="postgresql://user:password@localhost/dbname" axum-template
```

## 基础开发环境搭建

可以参考我的 Rust 项目模版，[地址](https://github.com/yuxuetr/axum-template)

## 基于类似 Nest.js 的项目组织方式

```shell
├── docs                # 放置文档
├── fixtures            # 放置一些必要的文件，比如公私钥，测试SQL
├── migrations          # 数据库迁移 SQL 目录
├── rest_client         # VS Code REST Client的测试API文件
├── src                 # 源代码
│   ├── common          # 放置公共模块，比如加解密，签名与验证，error模块，config模块等
│   └── modules         # 业务模块
│       ├── auth        # 注册认证模块，模块中包含: `handlers`,`services`,`dto`,`tests`,`middleware`等
│       ├── health      # 健康检查模块，提供应用和数据库状态监控
│       └── users       # 用户管理模块，模块中包含: `handlers`,`services`,`dto`,`tests`, `entity`等
```

### **业务模块**文件的作用:

- `mod.rs`: 模块的导入导出，**创建当前模块路由器**
- `handlers.rs`: 处理所有的用户请求
- `services.rs`: 专门处理实际的逻辑以及与数据库交互
- `entity.rs`: 定义与数据库表对应的结构，以及便于结构操作的内容
- `dto.rs`: 做数据转换，比如定义请求和响应的结构体便于序列化和反序列化，减少参数传递
- `tests.rs`: 做单元测试(`util_tests`)和集成测试(`integration_tests`)
- `middleware.rs`: 适合模块自身的中间件
- `src/lib.rs`: 定义总的**路由器**,加载配置文件，初始化全局状态，全局配置，全局错误等
- `src/main.rs`: 应用入口

## 一点经验

1. 如果测试过多或者复杂可能要给测试进行排序，防止并发导致错误，尤其在集成测试中，可以使用`serial_test`
2. 为了便于测试时创建独立数据库、测试完自动删除数据库，`main` 分支使用 `sqlx-db-tester`，`seaorm` 与 `toasty` 分支在 `test_util::TestPg` 中内置了基于各自 ORM 连接的临时数据库创建、迁移与自动销毁机制
3. 为了方便数据库操作，在全局状态里包含**全局配置**和**数据库连接**，并且可以将所有的`services.rs`关于数据库的操作都实现在全局状态下，便于操作
4. 集成测试如果使用`reqwest`作为请求的客户端的话，在代码提交 Github Actions 时，
   会由于 reqwest 默认依赖 OpenSSL 导致容器崩溃，可以参考我的`Cargo.toml`配置，
   禁用掉默认的`reqwest`依赖的`OpenSSL`，使用`rustls-tls` feature 替代
5. 我将生成公钥私钥的内容放在了`build.rs`,这样只要执行`cargo build`或者`cargo run`之类的构建操作，
   就会自动生成证书文件在`fixtures`目录下（已加入`.gitignore`），具体逻辑可以查看`build.rs`文件
6. 关于测试，Rust 中可以将单独文件当做一个`mod`,但是 Rust 不会识别`integration_tests.rs`为一个测试模块，但是可以识别`tests.rs`，
   所以我将单元测试与集成测试都写在模块的`tests.rs`中并且使用`util_tests`和`integration_tests`模块分别包裹，
   这样测试日志可以明确看清楚是属于什么测试，并且按照模块写测试，便于出错后排查
7. 使用`pre-commit`严格执行各类工具检查，使代码更加规范化，`cargo-deny`也会优化代码，让代码更合理
8. Token 的签名算法使用`Ed25519`（通过`jsonwebtoken`纯 Rust 实现，无需 cmake/BoringSSL 等 C 依赖），
   公钥与私钥分离，签名依赖私钥，验证签名依赖公钥，方便将验证签名作为独立服务部署
9. JWT Claims 仅携带 `sub`(user_id)，不存储完整用户信息，每次鉴权从数据库获取最新权限，确保权限变更实时生效
10. 业务代码中不使用 `unwrap`/`expect`，所有错误均显式处理；内部错误（数据库、IO 等）对客户端返回统一的 `internal server error`，不泄露内部细节

## API 端点

### 认证模块 (`/auth`)
- `POST /auth/signup` - 用户注册
- `POST /auth/signin` - 用户登录

### 用户管理模块 (`/users`)
- `GET /users` - 获取用户列表 (支持分页)
- `GET /users/:id` - 获取用户详情
- `PATCH /users/:id` - 更新用户信息
- `DELETE /users/:id` - 删除用户

### 健康检查模块
- `GET /health` - 基础健康检查 (返回应用状态、版本、运行时间)
- `GET /health/ready` - 就绪检查 (包含数据库连接状态和响应时间)
- `GET /health/live` - 存活检查 (Kubernetes liveness probe)

## 核心特性

### 🔐 认证授权系统
- **JWT认证**: 使用 Ed25519 算法（纯 Rust 实现），公私钥分离设计
- **RBAC权限控制**: 基于角色和权限的访问控制（Admin/Moderator/User 三级角色）
- **中间件保护**: 自动 Token 验证和用户权限实时查询
- **错误脱敏**: 内部错误不泄露给客户端，JWT 错误统一返回 401

### ⚡ 性能优化
- **JWT密钥缓存**: 启动时加载密钥到内存，避免重复文件读取
- **数据库查询优化**: 解决 N+1 查询问题，使用批量查询
- **连接池管理**: 高效的数据库连接复用
- **动态测试端口**: 集成测试使用 `127.0.0.1:0` 避免端口冲突

### 🛡️ 错误处理与监控
- **结构化错误响应**: 包含错误ID、时间戳的标准化错误格式
- **错误追踪**: UUID错误标识符，便于日志分析和问题定位
- **健康监控**: 全面的应用和数据库健康状态检查

### ✅ 请求参数自动校验
- **通用校验提取器**: 提供 `ValidatedJson<T>`、`ValidatedQuery<T>`、`ValidatedForm<T>`、`ValidatedPath<T>`
- **DTO 声明校验规则**: 请求 DTO 只需要实现 `Deserialize` 和 `Validate`
- **Handler 保持业务纯净**: 使用 validated extractor 后，不需要在每个 handler 中手动调用 `input.validate()?`
- **统一错误语义**: 反序列化或提取失败返回 `400 Bad Request`，DTO 校验失败返回 `422 Unprocessable Entity`

示例:

```rust
use crate::common::{ValidatedJson, ValidatedQuery};

pub async fn signup_handler(
  State(state): State<AppState>,
  ValidatedJson(payload): ValidatedJson<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = state.create_user(payload).await?;
  Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_users_handler(
  State(state): State<AppState>,
  ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
  let PaginationParams { limit, offset } = params;
  let users = state.get_users(limit, offset).await?;
  Ok((StatusCode::OK, Json(users)))
}
```

对于需要校验的业务输入，优先使用 `Validated*` 提取器；不需要校验的基础参数仍可继续使用 Axum 原生提取器。文件上传等 multipart 场景通常包含文件大小、MIME、流式读取等业务约束，建议按具体业务单独处理。

### 🧪 测试体系
- **单元测试**: 覆盖核心业务逻辑、RBAC 角色/权限事务同步与异常分支
- **集成测试**: 端到端 HTTP API 测试（认证、用户管理、健康检查）
- **并发测试**: 使用`serial_test`与独立临时数据库确保测试独立性

## 最新技术栈

- **Rust**: 最新稳定版，兼容 Rust 2024 Edition
- **Axum**: v0.8 - 高性能异步 Web 框架
- **数据库访问层（多分支可选）**:
  - **SQLx**: v0.8.6 (`main` 分支) - 编译时安全的异步 SQL 工具包
  - **SeaORM**: v2.0 (`seaorm` 分支) - 功能丰富的异步关系型 ORM
  - **Toasty**: v0.11 (`toasty` 分支) - Tokio 生态声明式异步 ORM
- **PostgreSQL**: 可靠的关系型数据库
- **jsonwebtoken**: 基于 Ed25519 的 JWT 签名/验证（纯 Rust 实现，无 C 依赖）
- **Argon2**: 密码哈希
- **Tracing**: 结构化日志记录

## 博客地址

- [https://yuxuetr.com/blog/2024/08/06/axum-template-01](https://yuxuetr.com/blog/2024/08/06/axum-template-01)

## Changelog

详细的项目变更记录请查看 [CHANGELOG.md](./CHANGELOG.md)
