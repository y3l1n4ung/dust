# 🌪️ Dust

**Rust-powered, blazingly fast, full-stack code generation for Dart and Flutter.**

[![CI](https://github.com/y3l1n4ung/dust/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/y3l1n4ung/dust/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/y3l1n4ung/dust/graph/badge.svg)](https://app.codecov.io/gh/y3l1n4ung/dust)
[![Quality Gate](https://sonarcloud.io/api/project_badges/measure?project=y3l1n4ung_dust&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=y3l1n4ung_dust)
[![OpenSSF Scorecard](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.scorecard.dev%2Fprojects%2Fgithub.com%2Fy3l1n4ung%2Fdust&query=%24.score&label=OpenSSF%20Scorecard&color=informational)](https://scorecard.dev/viewer/?uri=github.com/y3l1n4ung/dust)
[![Release](https://img.shields.io/github/v/release/y3l1n4ung/dust?logo=github&color=blue)](https://github.com/y3l1n4ung/dust/releases)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

Dust is beginner- and AI-agent-friendly, with designs inspired by the Rust
ecosystem, including Rust's derive system, Serde, Axum, and SQLx.

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

Dust packages have their own pub versions. When changing the CLI or packages,
check the [compatibility guide](docs/usage/compatibility.md).

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

## Agent Skill

Install the [use-dust Agent Skill](skills/use-dust/SKILL.md):

```bash
npx skills add y3l1n4ung/dust --skill use-dust
```

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
| HTTP servers | Beta, runtime only | [Server guide](docs/dust_server/README.md) |
| Firebase | Planned | — |
| Supabase | Planned | — |

Stable features keep their documented authoring APIs compatible throughout
`0.1.x`. Beta features may still receive API refinements.

`dust_server` is a runtime, not a generator. Server code is written by hand
today; `dust build` produces nothing for it.

## Documentation

- [Usage guides](docs/usage/README.md)
- [CLI and package compatibility](docs/usage/compatibility.md)
- [Dart package](packages/dust_dart/README.md)
- [Flutter package](packages/dust_flutter/README.md)
- [Database runtime](packages/dust_db_sqlite3/README.md)
- [Server runtime](packages/dust_server/README.md)
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
