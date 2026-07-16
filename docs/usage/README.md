# Usage Guides

You focus on product. We focus on performance.

This section provides the canonical documentation for using Dust in your
Flutter and Dart projects.

## Our Promise

- Stable authoring APIs for features marked stable.
- Generated code can improve without forcing handwritten product-code churn.
- Features marked beta may still receive API refinements before stabilization.
- Performance is part of the product contract, not a best-effort optimization.

---

## Guide Order

Start with the verified [root README quick start](../../README.md#quick-start),
then use these task guides as needed:

1.  **[Generate data classes](./derive.md)**: Add `ToString`, `Eq`, and `CopyWith`.
2.  **[Generate JSON serialization](./serde.md)**: Encode and decode typed models.
3.  **[Generate HTTP clients](./http.md)**: Build type-safe Dio-backed clients.
4.  **[Add validation](./validation.md)**: Generate Dart model validation and Flutter-only form validators.
5.  **[Build ViewModels](./state.md)**: Manage sync and async Flutter state.
6.  **[Configure typed routing](./routing.md)**: Use Navigator 2.0 with typed routes.
7.  **[Add i18n](./i18n.md)**: Generate ARB bootstrap and runtime lookup helpers.
8.  **[Validate SQLite queries](./db.md)**: Use SQLx-style sqlite3 validation and row mapping.

---

## Package Installation

Install the `dust` CLI from the [root installation guide](../../README.md#installation)
before running `dust build`, `dust check`, or feature-specific commands.

Depending on the features you need, add the following packages to your `pubspec.yaml`:

| Feature | Required Packages |
| :--- | :--- |
| **Basic Traits** | `dust_dart` |
| **Validation** | `dust_dart` |
| **JSON Support** | `dust_dart` |
| **Networking** | `dust_dart`, `dio` |
| **State** | `dust_flutter` |
| **Routing** | `dust_flutter` |
| **i18n** | `dust_flutter`, `flutter_localizations` |
| **Database** | `dust_dart`, `dust_db_sqlite3` |

For the current published package versions and package-specific setup, see the
[dust_dart README](../../packages/dust_dart/README.md),
[dust_flutter README](../../packages/dust_flutter/README.md), and
[dust_db_sqlite3 README](../../packages/dust_db_sqlite3/README.md).

> [!TIP]
> Use `package:dust_dart/dust_dart.dart` for starter examples or mixed Dust features. Feature guides may use narrower imports for focused examples.

---

## Learning from Examples

The guides in this directory reference real-world implementations found in the [Product Showcase Example](../../examples/product_showcase). This example includes automated tests and provides a "Golden Standard" for Dust usage.

If you have the repository cloned, you can build the showcase manually:
```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```
