# Design Notes

## Validated request extractors

The template provides reusable Axum extractors that combine request extraction
with `validator::Validate` execution:

- `ValidatedJson<T>` for JSON request bodies.
- `ValidatedQuery<T>` for URL query strings.
- `ValidatedForm<T>` for URL encoded form bodies.
- `ValidatedPath<T>` for structured path parameters.

Each extractor is generic over any DTO that implements Serde deserialization and
`Validate`. This keeps validation rules on the DTO while moving validation
execution to the HTTP boundary.

Handlers should use the validated extractor whenever user input requires DTO
validation. Raw Axum extractors are still appropriate for values that do not
need validation or for custom extraction flows such as multipart uploads.

Extraction and deserialization failures return `400 Bad Request`. DTO validation
failures return `422 Unprocessable Entity` through `AppError::ValidationError`.

## Multi-branch database layer (`main` / `seaorm` / `toasty`)

The repository provides three git branches implementing the exact same HTTP
routes, DTOs, JWT + RBAC rules, and PostgreSQL schema with three different Rust
database libraries:

1. **`main` (`sqlx`)**:
   - Stores `sqlx::PgPool` in `AppStateInner`.
   - Maps database rows using `#[derive(sqlx::FromRow)]` in `src/modules/users/entity.rs`.
   - Executes parameterized SQL queries and transactions in `src/modules/users/services.rs`.
2. **`seaorm` (`sea-orm`)**:
   - Stores `sea_orm::DatabaseConnection` in `AppStateInner`.
   - Defines `DeriveEntityModel` entities, `Relation` enums, and `Related` implementations for `users`, `roles`, `permissions`, `user_roles`, `role_permissions`, and `user_permissions`.
   - Uses `EntityTrait`, `QueryFilter`, `PaginatorTrait`, `ActiveModel`, and `TransactionTrait` in `src/modules/users/services.rs`.
3. **`toasty` (`toasty`)**:
   - Stores `toasty::Db` in `AppStateInner`, registered via `toasty::models!(crate::*)`.
   - Defines declarative `#[derive(toasty::Model)]` models with `#[key]`, `#[auto]`, `#[unique]`, and `#[index]` attributes in `src/modules/users/entity.rs`.
   - Uses Toasty's generated query builders (`create`, `update`, `filter`, `in_list`, `count`, `order_by`, `limit`, `offset`) and `db.transaction()` in `src/modules/users/services.rs`, plus `toasty::sql` for raw SQL readiness checks and test database setup.
