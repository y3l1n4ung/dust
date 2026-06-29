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

## Getting Started

If you are new to Dust, we recommend reading these guides in order:

1.  **[Data Classes (Derive)](./derive.md)**: Master `ToString`, `Eq`, and `CopyWith`.
2.  **[JSON Serialization (Serde)](./serde.md)**: High-performance encoding and decoding.
3.  **[HTTP Client](./http.md)**: Type-safe, Dio-backed API clients.
4.  **[Validation](./validation.md)**: Generated model and Flutter form validation.
5.  **[State Management](./state.md)**: Boilerplate-free reactive ViewModels.
6.  **[Typed Routing](./routing.md)**: Safe Navigator 2.0 implementation.
7.  **[i18n](./i18n.md)**: Runtime translations, ARB assets, and namespaced keys.
8.  **[Dust DB](./db.md)**: SQLx-style sqlite3 query validation and row mapping.

---

## Package Installation

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

> [!TIP]
> Use `package:dust_dart/dust_dart.dart` for starter examples or mixed Dust features. Feature guides may use narrower imports for focused examples.

---

## Quick Start

### 1. Install the CLI
```bash
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash
```

### 2. Configure Dependencies
Add the relevant packages to your `pubspec.yaml`:

```yaml
dependencies:
  dust_dart: ^0.1.0
  dio: ^5.0.0
```

### 3. Generate Code
```bash
dart pub get
dust build
```

> [!NOTE]
> Dust generates code into `.g.dart` files. Ensure you have the corresponding `part` directive in your source files.

---

## Learning from Examples

The guides in this directory reference real-world implementations found in the [Product Showcase Example](../../examples/product_showcase). This example includes automated tests and provides a "Golden Standard" for Dust usage.

If you have the repository cloned, you can build the showcase manually:
```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```
