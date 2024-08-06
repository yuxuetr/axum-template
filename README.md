# Rust Axum Web 应用通用化模版

本项目实践使用 Axum 搭建后台服务，包括:

- 构建 API
- 使用 Sqlx 和 Postgres 数据库提供数据库服务
- 采用类似 Nest.js 的项目组织结构，使得项目的文本文件负责单独的职责使得代码更信息易读
- 包含单元测试和集成测试代码
- Github Actions CI

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

## 博客地址

- [https://yuxuetr.com/blog/2024/08/06/axum-template-01](https://yuxuetr.com/blog/2024/08/06/axum-template-01)
