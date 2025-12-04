# Rust Axum Web 应用通用化模版

本项目实践使用 Axum 搭建后台服务，包括:

- 构建 API
- 使用 Sqlx v0.8.6 和 Postgres 数据库提供数据库服务
- 采用类似 Nest.js 的项目组织结构，使得项目的文本文件负责单独的职责使得代码更信息易读
- 完整的认证授权系统 (JWT + RBAC)
- 高性能的数据库查询优化
- 完善的错误处理和日志记录
- 健康检查端点支持
- 包含单元测试和集成测试代码
- Github Actions CI

## 快速开始

### 环境要求
- Rust 1.70+
- PostgreSQL 12+
- Docker (可选)

### 本地开发
```bash
# 1. 克隆项目
git clone https://github.com/yuxuetr/rust-template.git
cd rust-template

# 2. 创建数据库
createdb axum_template

# 3. 运行数据库迁移
sqlx migrate run

# 4. 启动服务
cargo run
```

服务将在 `http://localhost:3000` 启动。

### Docker 部署
```bash
# 构建镜像
docker build -t axum-template .

# 运行容器
docker run -p 3000:3000 -e DATABASE_URL="postgresql://user:password@localhost/dbname" axum-template
```

## 基础开发环境搭建

可以参考我的 Rust 项目模版，[地址](https://github.com/yuxuetr/rust-template)

## 基于类似 Nest.js 的项目组织方式

```shell
├── docs                # 放置文档
├── fixtures            # 放置一些必要的文件，比如公私钥，测试SQL
├── migrations          # Sqlx的目录
├── rest_client         # VS Code REST Client的测试API文件
├── src                 # 源代码
│   ├── common          # 放置公共模块，比如加解密，签名与验证，error模块，config模块等
│   └── modules         # 业务模块
│       ├── auth        # 注册认证模块，模块中包含: `handlers`,`services`,`dto`,`tests`,`middleware`等
│       ├── health      # 健康检查模块，提供应用和数据库状态监控
│       └── users       # 用户管理模块，模块中包含: `handlers`,`services`,`dto`,`tests`, `entity`等
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

1. 如果测试过多或者复杂可能要给测试进行排序，防止并发导致错误，尤其在继承测试中，可以使用`serial_test`
2. 为了便于测试时测试创建数据库，测试完删除数据库，可以使用`sqlx-db-tester`，并且在单独初始化全局状态时，为测试也初始化一个，连接测试数据库
3. 为了方便数据库操作，在全局状态里包含**全局配置**和**数据库连接**，并且可以将所有的`services.rs`关于数据库的操作都实现在全局状态下，便于操作
4. 集成测试如果使用`reqwest`作为请求的客户端的话，在代码提交 Github Actions 时，
   会由于 reqwest 默认依赖 OpenSSL 导致容器崩溃，可以参考我的`Cargo.toml`关于，
   我们需要在`OpenSSL`与`boring`SSL 之间二选一，由于`jwt-simple`也依赖`boring`，
   所以要禁用掉默认的`reqwest`依赖的`OpenSSL`
5. 我将生成公钥私钥的内容放在了`build.rs`,这样只要执行`cargo build`或者`cargo run`之类的构建操作，
   就会自动生成证书文件在`fixtures`目录下，具体逻辑可以查看`build.rs`文件
6. 关于测试，Rust 中可以将单独文件当做一个`mod`,但是 Rust 不会识别`integration_tests.rs`为一个测试模块，但是可以识别`tests.rs`，
   所以我将单元测试与集成测试都写在模块的`tests.rs`中并且使用`util_tests`和`integration_tests`模块分别包裹，
   这样测试日志可以明确看清楚是属于什么测试，并且按照模块写测试，便于出错后排查
7. 使用`pre-commit`严格执行各类工具检查，使代码更加规范化，`cargo-deny`也会优化代码，让代码更合理
8. Token 的签名算法使用`Ed25519`，这样便于将公钥与私钥分开，签名依赖私钥，验证签名依赖公钥，这样如果想将验证签名作为一个独立的服务很容易也更安全

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
- **JWT认证**: 使用Ed25519算法，公私钥分离设计
- **RBAC权限控制**: 基于角色和权限的访问控制
- **中间件保护**: 自动Token验证和用户信息注入

### ⚡ 性能优化
- **JWT密钥缓存**: 避免重复文件读取，提升认证性能
- **数据库查询优化**: 解决N+1查询问题，使用批量查询
- **连接池管理**: 高效的数据库连接复用

### 🛡️ 错误处理与监控
- **结构化错误响应**: 包含错误ID、时间戳的标准化错误格式
- **错误追踪**: UUID错误标识符，便于日志分析和问题定位
- **健康监控**: 全面的应用和数据库健康状态检查

### 🧪 测试体系
- **单元测试**: 覆盖核心业务逻辑
- **集成测试**: 端到端API测试
- **并发测试**: 使用`serial_test`确保测试独立性

## 最新技术栈

- **Rust**: 最新稳定版，兼容Rust 2024
- **Axum**: v0.8.7 - 高性能异步Web框架
- **SQLx**: v0.8.6 - 编译时安全的SQL工具包
- **PostgreSQL**: 可靠的关系型数据库
- **JWT-Simple**: 轻量级JWT处理库
- **Tracing**: 结构化日志记录

## 博客地址

- [https://yuxuetr.com/blog/2024/08/06/axum-template-01](https://yuxuetr.com/blog/2024/08/06/axum-template-01)

## Changelog

详细的项目变更记录请查看 [CHANGELOG.md](./CHANGELOG.md)

### 最近更新

#### [unreleased] - 2024-12-04

##### 🐛 Bug Fixes
- **(security)** 修复 idna 安全漏洞 (RUSTSEC-2024-0421) 和测试失败问题
- 修复集成测试中 HTTP 客户端代理配置导致的 502 错误
- 替换未维护的 proc-macro-error 依赖为 proc-macro-error2
- 更新许可证配置以支持新的依赖许可证

##### 📦 Dependencies
- 升级 validator 从 0.18.1 到 0.20.0
- 自动更新相关依赖包以解决安全和兼容性问题

##### ✅ Testing
- 所有 15 个测试通过 (9 个工具测试 + 6 个集成测试)
- 通过所有 cargo-deny 安全检查
