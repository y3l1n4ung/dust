# Usage Guides

Use these guides after completing the [root Quick Start](../../README.md#quick-start).
Each guide shows the handwritten API first and links to a working example.

## Feature Guides

| Task | Guide | Package |
| :--- | :--- | :--- |
| Generate data-class helpers | [Derive](./derive.md) | `dust_dart` |
| Encode and decode JSON | [Serde](./serde.md) | `dust_dart` |
| Validate models and Flutter forms | [Validation](./validation.md) | `dust_dart` |
| Generate Dio clients | [HTTP](./http.md) | `dust_dart` |
| Build typed ViewModels | [State](./state.md) | `dust_flutter` |
| Configure Navigator 2.0 routing | [Routing](./routing.md) | `dust_flutter` |
| Maintain ARB translations | [Internationalization](./i18n.md) | `dust_flutter` |
| Validate SQLite queries | [Database](./db.md) | `dust_dart`, `dust_db_sqlite3` |

Install the CLI from the [installation guide](../../README.md#installation).
Each feature guide contains its exact package command and setup requirements.

## CLI Overview

| Command | Behavior |
| :--- | :--- |
| `dust build` | Writes normal generated Dart outputs. |
| `dust check` | Checks normal generated outputs without writing. |
| `dust watch` | Runs an initial build, then rebuilds after source changes. |
| `dust clean` | Removes Dust-owned generated outputs and cache state. |
| `dust doctor` | Reports workspace and plugin readiness. |
| `dust upgrade` | Checks, verifies, and upgrades the installed CLI binary. |
| `dust db build` | Validates static SQLite queries and writes database/DAO output. |
| `dust check --db` | Checks database/DAO output and SQL without writing. |
| `dust i18n scan` | Reports statically discoverable translation calls. |
| `dust i18n build` | Reconciles ARB files and refreshes the Flutter bootstrap. |
| `dust i18n check` | Validates ARB files and localization setup without writing. |

Run `dust --help` or `dust <command> --help` for the current options.

`dust upgrade --check` reads release metadata only. `dust upgrade --dry-run`
downloads and verifies the selected release without replacing the installed
binary.

> [!NOTE]
> Normal `dust build` does not generate `@SqlxDatabase` or `@SqlxDao` output.
> Database projects run `dust build` for row derives and `dust db build` for
> database and DAO generation.

## Typical Workflow

For normal Dart and Flutter generation:

```bash
dust build
dust check
```

For Database projects:

```bash
dust build
dust db build
dust check
dust check --db
```

For translation maintenance:

```bash
dust i18n scan
dust i18n build
dust i18n check
```

## Runnable Examples

- [Product showcase](../../examples/product_showcase): Dart derives, JSON,
  validation, and HTTP generation.
- [Shopping app](../../examples/shopping_app): Flutter state, routing, i18n,
  HTTP, and Database integration.
- [Benchmark project](../../examples/benchmark_project): large generated input
  used for build and cache measurements.

Contributor checkout and Rust development commands live in
[CONTRIBUTING.md](../../CONTRIBUTING.md).
