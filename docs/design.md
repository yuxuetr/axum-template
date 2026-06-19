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
