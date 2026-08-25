# Documentation

The root [README](../README.md) explains what Dust does and provides the first
runnable example. Use this directory for feature setup, API details, examples,
and contributor documentation.

## Start Here

| Goal | Documentation |
| :--- | :--- |
| Install Dust and generate the first model | [README Quick Start](../README.md#quick-start) |
| Check CLI and package versions | [Compatibility guide](./usage/compatibility.md) |
| Choose and configure a feature | [Usage guides](./usage/README.md) |
| Explore a Dart example | [Product showcase](../examples/product_showcase/README.md) |
| Explore a Flutter application | [Shopping app](../examples/shopping_app/README.md) |
| Contribute to Dust | [Contributing guide](../CONTRIBUTING.md) |

## Feature Guides

### Dart

- [Data-class derives](./usage/derive.md)
- [JSON serialization](./usage/serde.md)
- [Model and form validation](./usage/validation.md)
- [Dio HTTP clients](./usage/http.md)

### Flutter

- [State management](./usage/state.md)
- [Typed routing](./usage/routing.md)
- [Internationalization](./usage/i18n.md)

### Database

- [SQLite query validation and generated DAOs](./usage/db.md)

### Server

- [dust_server guides](./dust_server/README.md): routing, extraction, responses,
  middleware, serving, WebSockets, and testing

  A runtime, not a generator. Server code is handwritten today; `dust build`
  produces nothing for it.

## Package References

- [`dust_dart`](../packages/dust_dart/README.md): Dart annotations and runtime
  APIs.
- [`dust_flutter`](../packages/dust_flutter/README.md): Flutter state, routing,
  validation, and i18n APIs.
- [`dust_db_sqlite3`](../packages/dust_db_sqlite3/README.md): native SQLite
  runtime for generated Database code.
- [`dust_server`](../packages/dust_server/README.md): HTTP server runtime on
  `shelf`, shaped after axum. Beta, and not yet generated from annotations.

## Examples

| Example | Covers |
| :--- | :--- |
| [Product showcase](../examples/product_showcase/README.md) | Derives, JSON, validation, and HTTP generation. |
| [Shopping app](../examples/shopping_app/README.md) | Flutter state, routing, i18n, HTTP, and Database integration. |
| [Benchmark project](../examples/benchmark_project/README.md) | Large generated input for build and cache measurements. |

## Contributor Documentation

- [Contributing](../CONTRIBUTING.md)
- [Developer guide](./developer.md)
- [Plugin guide](./plugin-guide.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
- [Security policy](../SECURITY.md)
- [Roadmap and milestones](https://github.com/y3l1n4ung/dust/milestones)
