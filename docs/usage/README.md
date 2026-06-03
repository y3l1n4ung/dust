# Usage Guides

This section provides the canonical documentation for using Dust in your Flutter and Dart projects.

---

## Getting Started

If you are new to Dust, we recommend reading these guides in order:

1.  **[Data Classes (Derive)](./derive.md)**: Master `ToString`, `Eq`, and `CopyWith`.
2.  **[JSON Serialization (Serde)](./serde.md)**: High-performance encoding and decoding.
3.  **[HTTP Client](./http.md)**: Type-safe, Dio-backed API clients.
4.  **[State Management](./state.md)**: Boilerplate-free reactive ViewModels.
5.  **[Typed Routing](./routing.md)**: Safe Navigator 2.0 implementation.
6.  **[Dust DB](./db.md)**: SQLx-style sqlite3 query validation and row mapping.

---

## Package Installation

Depending on the features you need, add the following packages to your `pubspec.yaml`:

| Feature | Required Packages |
| :--- | :--- |
| **Basic Traits** | `dust_dart` |
| **JSON Support** | `dust_dart` |
| **Networking** | `dust_dart`, `dio` |
| **State** | `dust_flutter` |
| **Routing** | `dust_flutter` |
| **Database** | `dust_dart`, `dust_db_sqlite3` |

> [!TIP]
> `package:dust_dart/serde.dart` re-exports the core derive traits, so you do not need two imports when using JSON serialization.

---

## Quick Start

### 1. Install the CLI
```bash
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash
```

### 2. Configure Dependencies
Add the relevant versions to your `pubspec.yaml`:

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
