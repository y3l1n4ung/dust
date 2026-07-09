# 🌪️ Dust

**You focus on product. We focus on performance.**

[![CI](https://github.com/y3l1n4ung/dust/actions/workflows/ci.yml/badge.svg)](https://github.com/y3l1n4ung/dust/actions)
[![Release](https://img.shields.io/github/v/release/y3l1n4ung/dust?logo=github&color=blue)](https://github.com/y3l1n4ung/dust/releases)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

Dust is a high-performance alternative to `build_runner`. It offers built-in
support for data classes, validation, JSON serialization, HTTP clients, routing,
state management, and database codegen.

## Our Promise

- Stable public APIs for features marked stable.
- Performance and quality improvements should change generated code, the Rust
  engine, and runtime internals first.
- Features marked beta may still receive API refinements before stabilization.
- Your handwritten product code should stay focused on product logic.

> [!IMPORTANT]
> **Tired of waiting for your code to generate?** Dust is built to handle the large Flutter projects with near-instant rebuilds.

---

## ✨ Why Dust?

- 🚀 **Performance:** Written in Rust. Generates thousands of files in seconds. 
- 🎯 **Product Focus:** We handle code generation so you focus only on product.
- 🧱 **Stable By Design:** Public APIs are designed to stay stable; improvements
  should land in generated code and internals first.
- 🧩 **All-in-One:** Data classes, JSON, validation, HTTP clients, routing,
  state, and DB codegen in one unified tool.
- 🔄 **Incremental:** Intelligent watch mode only rebuilds the specific files you edited.
- 🛡️ **Type Safe:** Advanced validation catches errors before you even run your app.

---

## 🏗️ Supported Features

| Feature | Stability | Description | Documentation |
| :--- | :--- | :--- | :--- |
| **Data Classes** | Stable public API. API will not change. | `ToString`, `Eq`, `HashCode`, and `CopyWith` generation. | [Read Guide →](docs/usage/derive.md) |
| **JSON Serialization** | Stable public API. API will not change. | Blazing fast JSON encode/decode with support for renames and custom codecs. | [Read Guide →](docs/usage/serde.md) |
| **Validation** | Stable public API. API will not change. | Dart model validation plus Flutter-only form validators from typed field rules. | [Read Guide →](docs/usage/validation.md) |
| **HTTP Client** | Stable public API. API will not change. | Type-safe, Dio-backed API client generation from annotations. | [Read Guide →](docs/usage/http.md) |
| **Routing** | Beta. API may still be refined. | Boilerplate-free Navigator 2.0 routing with typed parameters. | [Read Guide →](docs/usage/routing.md) |
| **State Management** | Beta. API may still be refined. | Lightweight, high-performance state containers with action generation. | [Read Guide →](docs/usage/state.md) |
| **Database** | Beta. API may still be refined. | SQLx-style sqlite3 query validation, DAOs, and row mapping. | [Read Guide →](docs/usage/db.md) |
| **Firebase** | Coming soon. | Typed Firebase integration and generated data access helpers. | _(Coming Soon)_ |
| **Supabase** | Coming soon. | Typed Supabase integration and generated data access helpers. | _(Coming Soon)_ |
| **i18n** | Beta. API may still be refined. | Flutter i18n runtime, ARB assets, static scanning, generated bootstrap, and translation checks. | [Read Guide →](docs/usage/i18n.md) |

---

## 🚀 Quick Start

This path clones Dust, fetches the example dependencies, runs generation, and
verifies the generated files are current. It requires Git, Rust stable, and the
Dart/Flutter SDK on your `PATH`.

```bash
git clone https://github.com/y3l1n4ung/dust.git
cd dust
```

```bash
cd examples/product_showcase
dart pub get
cd ../..
```

```bash
cargo run -p dust_cli -- build --root examples/product_showcase
cargo run -p dust_cli -- check --root examples/product_showcase
```

The final command should report the showcase is clean:

```text
check  scanned: 25  clean: 25  stale: 0
```

### Use Dust in your project

Add annotations to your Dart source:

```dart
import 'package:dust_dart/dust_dart.dart';

part 'user.g.dart';

@Derive([ToString(), CopyWith(), Serialize()])
class User with _$User {
  final String name;
  const User(this.name);
}
```

When using an installed CLI in your own package, run `dust build` from that
package root. From a source checkout, use the same pattern with
`cargo run -p dust_cli -- build --root path-to-your-package`.

---

## 🛠️ Commands

| Command | Description |
| :--- | :--- |
| `dust build` | Run a full project generation. |
| `dust watch` | High-performance file watcher for instant rebuilds. |
| `dust check` | CI mode: Verifies if generated files are up to date. |
| `dust clean` | Clears all generated files and persistent caches. |

---

## 🤝 Contributing

Dust is open-source and we welcome all contributors!

- **Found a bug?** [Open an issue](https://github.com/y3l1n4ung/dust/issues)
- **Security reports:** [Use private vulnerability reporting](SECURITY.md)
- **Rust/Dart Setup:** [See CONTRIBUTING.md](CONTRIBUTING.md)
- **Architecture:** [Read the Developer Guide](docs/developer.md)

---

## 📜 License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 [Ye Lin Aung](https://github.com/y3l1n4ung).
