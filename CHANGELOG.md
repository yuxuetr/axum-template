# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [unreleased]

### Bug Fixes

- **(auth)** update middleware for axum v0.8 compatibility - ([856f551](https://github.com/yuxuetr/rust-template/commit/856f551a9e575cd7a04c321e6dc11f578db3b448)) - yuxuetr
- modify README.md - ([dcc4564](https://github.com/yuxuetr/rust-template/commit/dcc456444905feabef4b38512761bafc928de604)) - yuxuetr
- add test_data.sql and update CHANGELOG.md - ([c81f5ed](https://github.com/yuxuetr/rust-template/commit/c81f5ed7dc06956ea8eb2d031bc4ab997e459b6d)) - yuxuetr
- fix github action reqwest ssl error - ([f485425](https://github.com/yuxuetr/rust-template/commit/f4854258995885d933aa47eb4fddd7f62b63f3b0)) - yuxuetr
- github actions reqwest openssl again - ([d64ba8e](https://github.com/yuxuetr/rust-template/commit/d64ba8ef482d2ce2614c678634641346af4ec30e)) - yuxuetr
- github actions openssl error - ([80a8dba](https://github.com/yuxuetr/rust-template/commit/80a8dbad7ee7941ec1aa6711f3737236384c87fa)) - yuxuetr
- fix CI test reqwest openssl error - ([9e0cc3e](https://github.com/yuxuetr/rust-template/commit/9e0cc3eb4ad86c6c427a6e824aac93e218fe5aa4)) - yuxuetr
- github actions reqwest ssl error - ([0bed439](https://github.com/yuxuetr/rust-template/commit/0bed439e7175fa3ad6023937dfa0f1c6925e5337)) - yuxuetr
- github actions reqwest ssl error again - ([25546d9](https://github.com/yuxuetr/rust-template/commit/25546d9ece322b20384bb6792b86d7cdc938b5c5)) - yuxuetr

### Documentation

- update CHANGELOG.md and README.md - ([c715533](https://github.com/yuxuetr/rust-template/commit/c715533776fde8e4b062a986115cd69fb650bf42)) - yuxuetr
- update README.md with latest features and improvements - ([6eeaf29](https://github.com/yuxuetr/rust-template/commit/6eeaf299762cf033d6ab8da45b75f73e01680526)) - yuxuetr

### Features

- **(auth)** optimize JWT key caching to reduce file I/O operations - ([a07eb30](https://github.com/yuxuetr/rust-template/commit/a07eb30bff87723be743066ef1a0d45db3e0c7e8)) - yuxuetr
- **(error)** enhance error handling with traceability and logging - ([94ce81a](https://github.com/yuxuetr/rust-template/commit/94ce81a58033de56506365a4fe70e2fbff726ac9)) - yuxuetr
- **(health)** add comprehensive health check endpoints - ([e4a03aa](https://github.com/yuxuetr/rust-template/commit/e4a03aa3c17fabfe3ed8a7b798e4a195d5bf07e0)) - yuxuetr
- jwt_simple generate Ed25519 public & private pem key - ([61dce79](https://github.com/yuxuetr/rust-template/commit/61dce79478a30171fc7e29050287f39a5ebba5b1)) - yuxuetr
- support app config and state configuration - ([d18391c](https://github.com/yuxuetr/rust-template/commit/d18391c01ae1851808b9d0c6946701766679cc0e)) - yuxuetr
- support users model & handlers - ([b54999c](https://github.com/yuxuetr/rust-template/commit/b54999c941e81a67f8684c5b2eb3321119c66434)) - yuxuetr
- users module model ops and unit test - ([ceb05f5](https://github.com/yuxuetr/rust-template/commit/ceb05f545ef013ee8572418af542baf0545d9c19)) - yuxuetr
- support handlers integration for users module - ([203336f](https://github.com/yuxuetr/rust-template/commit/203336f9aed62b3b081ac191e9ad5948237b9498)) - yuxuetr
- support auth module and auth_middleware for user authentication - ([cd26cb5](https://github.com/yuxuetr/rust-template/commit/cd26cb50b3399bbbedf5dbe9304cd23ede85b39b)) - yuxuetr
- support rbac management user roles and permissions - ([7310279](https://github.com/yuxuetr/rust-template/commit/73102792d55cc5e560bd1739ee9dcd0f660e2894)) - yuxuetr
- support handlers input validator columns - ([5d763d8](https://github.com/yuxuetr/rust-template/commit/5d763d80add33acb821946be9731ac64614ce8e7)) - yuxuetr
- support console and file logging output - ([a52b446](https://github.com/yuxuetr/rust-template/commit/a52b4468fc1949318148a624cfd5d1aec4b5e6b2)) - yuxuetr

### Miscellaneous Chores

- **(axum)** update axum to latest version for better features - ([c41d19a](https://github.com/yuxuetr/rust-template/commit/c41d19acc67359e7048af34ff6a332f7d64cef6d)) - yuxuetr
- **(deps)** update dependency lock file after optimization changes - ([af44a99](https://github.com/yuxuetr/rust-template/commit/af44a99f141850d60517897e6107d36962ab9f4b)) - yuxuetr
- **(rust)** upgrade to Rust 2024 edition - ([420f843](https://github.com/yuxuetr/rust-template/commit/420f843c11a03b83e6deeebecf6627347563e976)) - yuxuetr
- **(sqlx)** upgrade to v0.8.6 to resolve future incompatibility warnings - ([40caf01](https://github.com/yuxuetr/rust-template/commit/40caf0187f492a7b46b1828838952e68fcd1f35d)) - yuxuetr

### Other

- update CHANGELOG.md - ([e396853](https://github.com/yuxuetr/rust-template/commit/e3968536989188ee0eba58f401d34d34d60e3761)) - yuxuetr
- update CHANGELOG.md - ([5762b4a](https://github.com/yuxuetr/rust-template/commit/5762b4aedffcd59baedcc15725a9601c3dc75c7f)) - yuxuetr

### Performance

- **(users)** optimize N+1 query problem in get_users function - ([4e8df77](https://github.com/yuxuetr/rust-template/commit/4e8df77509ff0a3d42929041b57fd5c7921911bc)) - yuxuetr

### Refactoring

- refactor util test in util_tests mod - ([5deebad](https://github.com/yuxuetr/rust-template/commit/5deebad1706f4e82f8e9edd49452206cebbb2710)) - yuxuetr

<!-- generated by git-cliff -->
