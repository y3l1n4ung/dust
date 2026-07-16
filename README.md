# 🌪️ Dust

**You focus on product. We focus on performance.**

[![CI](https://github.com/y3l1n4ung/dust/actions/workflows/ci.yml/badge.svg)](https://github.com/y3l1n4ung/dust/actions)
[![Release](https://img.shields.io/github/v/release/y3l1n4ung/dust?logo=github&color=blue)](https://github.com/y3l1n4ung/dust/releases)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

Dust is Rust-powered code generation for Dart and Flutter, built for human
developers and AI coding agents. It handles repetitive and complex code so you
can focus on your product.

## How Dust Works

1. Add Dust annotations to normal Dart or Flutter code.
2. Run `dust build` to generate the typed implementation.
3. Use the generated API from your application code.

## Installation

macOS and Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.ps1 | iex
```

The installers download the latest release, verify its SHA-256 checksum, and
place `dust` in `$HOME/.local/bin`. Add that directory to your `PATH`, then
verify the installation:

```bash
dust --version
```

## Quick Start

Dust has two main runtime packages:

- `dust_dart` provides data classes, JSON, validation, HTTP clients, and
  database annotations.
- `dust_flutter` provides Flutter routing, state management, and i18n.

Start with `dust_dart` from your application directory:

```bash
dart pub add dust_dart
```

Annotate a model:

```dart
import 'package:dust_dart/derive.dart';

part 'user.g.dart';

@Derive([ToString(), CopyWith()])
class User with _$User {
  const User(this.name);

  final String name;
}
```

Generate `user.g.dart`, then verify generated files are current:

```bash
dust build
dust check
```

Flutter apps can add `dust_flutter` too:

```bash
flutter pub add dust_flutter
```

See the [Flutter package guide](packages/dust_flutter/README.md) for routing,
state, and i18n setup.

## Features

| Feature | Status | Documentation |
| :--- | :--- | :--- |
| Data classes | Stable | [Derive guide](docs/usage/derive.md) |
| JSON serialization | Stable | [JSON guide](docs/usage/serde.md) |
| Validation | Stable | [Validation guide](docs/usage/validation.md) |
| HTTP clients | Stable | [HTTP guide](docs/usage/http.md) |
| Routing | Beta | [Routing guide](docs/usage/routing.md) |
| State management | Beta | [State guide](docs/usage/state.md) |
| i18n | Beta | [i18n guide](docs/usage/i18n.md) |
| Database | Beta | [Database guide](docs/usage/db.md) |
| Firebase | Planned | — |
| Supabase | Planned | — |

Stable features keep their documented authoring APIs compatible throughout
`0.1.x`. Beta features may still receive API refinements.

## Documentation

- [Usage guides](docs/usage/README.md)
- [Dart package](packages/dust_dart/README.md)
- [Flutter package](packages/dust_flutter/README.md)
- [Database runtime](packages/dust_db_sqlite3/README.md)
- [Contributor guide](CONTRIBUTING.md)
- [Architecture and internals](docs/developer.md)

## Examples

- [Product showcase](examples/product_showcase/README.md): Dart models, JSON,
  validation, HTTP clients, and row mapping.
- [Shopping app](examples/shopping_app/README.md): end-to-end Flutter routing,
  state, i18n, and database integration.
- [Benchmark project](examples/benchmark_project/README.md): scale and
  performance regression fixture.

## Support and Contributing

- [Open an issue](https://github.com/y3l1n4ung/dust/issues) for bugs and feature
  requests.
- Follow [CONTRIBUTING.md](CONTRIBUTING.md) to build and test Dust from source.
- Report vulnerabilities through [private security reporting](SECURITY.md).

## License

[MIT License](LICENSE).
