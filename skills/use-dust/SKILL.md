---
name: use-dust
description: Add, migrate, validate, or troubleshoot Dust code generation in Dart and Flutter application projects. Use for Dust setup, Dust annotations, generated .g.dart files, dust build/check/watch, data classes, JSON, validation, HTTP clients, Flutter routing/state/i18n, or Dust Database. Also covers the dust_server HTTP runtime, which is handwritten rather than generated. Do not use for developing Dust's Rust engine or for unrelated code-generation tools.
---

# Use Dust

Work from the Dart package or Flutter application root. Preserve handwritten
application APIs unless the user explicitly requests a migration.

## 1. Inspect the project

1. Read `pubspec.yaml`, `pubspec.lock`, `dust.yaml`, relevant Dart sources, and
   existing generated files before editing.
2. Determine whether the project is Dart-only or Flutter and which Dust feature
   the request needs.
3. Check the installed CLI and package versions with `dust --version`,
   `dust doctor`, and the dependency files. These commands are read-only
   diagnostics. Do not assume the latest release or silently change versions.
   If the CLI is absent, use the installation guide in the feature map and show
   the exact remote installer command before asking for authorization to run it.
4. Identify existing generators. Do not remove `build_runner`, builders, or
   their annotations until the requested migration has matching behavior and
   verified call sites.
5. Read [references/feature-map.md](references/feature-map.md) for the package,
   import, authoring contract, and command family for the feature.

If the task is not explicitly about Dust, do not migrate the project to Dust.
If the request targets Dust's Rust workspace or generator internals, state that
this skill does not apply and stop using it. Continue only under that
repository's contributor instructions, without carrying this skill's workflow
into the engine task.

## 2. Choose the smallest feature surface

- Use `dust_dart` for data classes, JSON, validation, HTTP clients, and Database
  annotations.
- Use `dust_flutter` only for Flutter routing, state, and i18n.
- Add `dust_db_sqlite3` only for native SQLite runtime access.
- Use `dust_server` for HTTP servers: a Dart runtime on `shelf` whose API is
  modelled on Rust's axum. It is a runtime, not a generator — routes, handlers,
  and extractors are handwritten, and `dust build` produces no server output.
  Do not add `part 'x.g.dart';` for it or promise generated routing.
- Treat routing, state, i18n, Database, and the server runtime as beta. Do not
  present Firebase, Supabase, PostgreSQL runtime support, or other planned
  features as available.
- Prefer focused imports such as `derive.dart`, `serde.dart`, `http.dart`,
  `route.dart`, `state.dart`, `i18n.dart`, or `db.dart`.

Before a migration that changes a public handwritten API, explain the intended
API change and confirm that it is within scope.

## 3. Implement the authoring contract

1. Add only the required runtime package with `dart pub add` for Dart projects
   or `flutter pub add` for Flutter projects.
2. Add the focused import, required annotation, generated mixin/base class, and
   `part 'name.g.dart';` or routing import required by the selected feature.
3. Keep source files and generated part names aligned.
4. Never hand-edit `.g.dart` files. Change handwritten source or configuration,
   then regenerate.
5. Preserve model fields, JSON keys, API paths, route names, translations,
   database schema, and existing behavior from inspected source. If the source
   is unavailable, ask for it or give a parameterized plan; do not invent an
   application contract.
6. Keep Database row models separate from database/DAO roots.
7. Keep one `route.dart` entrypoint for Flutter routing.
8. Use `watchXViewModel().value` only for rebuilding UI and
   `readXViewModel()` for callbacks, lifecycle code, and dependency factories.

For migrations from another generator, compare the exact feature in scope.
Do not claim Dust replaces every builder or remove unrelated generation tools.

## 4. Generate and verify

Run only the command family required by the selected feature:

- Normal generation: `dust build`, then `dust check`.
- Database: `dust build`, `dust db build`, `dust check`, then
  `dust check --db`.
- i18n: preview with `dust i18n scan`, write with `dust i18n build`, then run
  `dust i18n check`.
- Iteration: use `dust watch` only for normal generation.
- Server: no Dust command applies. Verify with `dart analyze` and `dart test`.

After generation, run `dart analyze` and `dart test` for Dart packages or
`flutter analyze` and `flutter test` for Flutter projects. Inspect the diff and
confirm generated outputs are deterministic and analyzer-safe.

Treat write boundaries explicitly:

- `dust build`, `dust db build`, and `dust i18n build` write files.
- `dust check`, `dust check --db`, `dust i18n scan`, and `dust i18n check` do
  not write generated application output.
- Do not execute a remote install script or broad clean command without the
  user's authorization.

## 5. Diagnose failures from evidence

- Stale or missing output: run the matching check command, verify the `part` or
  routing import, then rebuild the matching mode.
- Annotation not discovered: verify the focused import, fully supported target
  shape, package version, and source location.
- CLI/package mismatch: use `dust doctor` and the compatibility guide before
  changing versions.
- Database failure: separate row-derive failures from database/DAO validation;
  check migration paths, static SQL, metadata cache, and online/offline mode.
- i18n failure: check `dust.yaml`, literal key namespaces, locale ARB files,
  non-empty translations, placeholder metadata, and Flutter asset entries.
- Analyzer failure: fix handwritten or generated-contract inputs; do not hide
  warnings or rely on `dart format` to repair generator output.

Do not delete caches, generated files, translations, migrations, or user code
as a first troubleshooting step.

## Handoff

Report:

1. detected project type and versions;
2. selected Dust feature and required packages;
3. handwritten, configuration, dependency, and generated files changed;
4. commands run and their verified results;
5. anything pending, skipped, or blocked.

Distinguish verified state from recommendations. Do not claim generation,
analysis, or tests passed without command evidence.
