# Dust Feature Map

Use the project's installed package version as the source of truth. These links
describe the current public `0.1.x` authoring surface:

| Feature | Packages | Focused import | Guide |
| :--- | :--- | :--- | :--- |
| Data classes | `dust_dart` | `package:dust_dart/derive.dart` | [Derive](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/derive.md) |
| JSON | `dust_dart` | `package:dust_dart/serde.dart` | [Serde](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/serde.md) |
| Validation | `dust_dart` | `package:dust_dart/derive.dart` | [Validation](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/validation.md) |
| HTTP clients | `dust_dart` | `package:dust_dart/http.dart` | [HTTP](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/http.md) |
| Routing | `dust_flutter` | `package:dust_flutter/route.dart` | [Routing](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/routing.md) |
| State | `dust_flutter`, often `dust_dart` | `package:dust_flutter/state.dart` | [State](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/state.md) |
| i18n | `dust_flutter` | `package:dust_flutter/i18n.dart` | [i18n](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/i18n.md) |
| Database | `dust_dart`, `dust_db_sqlite3` | `package:dust_dart/db.dart` | [Database](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/db.md) |
| HTTP servers *(no generation)* | `dust_server` | `package:dust_server/server.dart` | [Server](https://github.com/y3l1n4ung/dust/blob/main/docs/dust_server/README.md) |

`dust_server` is a Dart runtime on `shelf` whose API is modelled on Rust's
axum. It is not built on axum, and no Rust dependency is involved.

Install the CLI from the [Dust installation guide](https://github.com/y3l1n4ung/dust#installation).
Review remote installer commands before execution. Check CLI/package alignment
with the [compatibility guide](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/compatibility.md).

## Authoring contracts

- **Data classes:** add `part 'model.g.dart';`, apply
  `@Derive([ToString(), Eq(), CopyWith()])`, and mix `_$Model` into the class.
- **JSON:** derive `Serialize()` and/or `Deserialize()`. Use `@SerDe` for
  options. Deserialization needs a forwarding
  `factory Model.fromJson(Map<String, Object?> json) => _$ModelFromJson(json)`.
- **Validation:** derive `Validate()` and add typed `@Validate(...)` field
  rules. Field annotations alone do not generate validation.
- **HTTP:** define an `abstract interface class` annotated with `@HttpClient`
  and a redirecting factory to `_$Client`. Annotate one HTTP verb per method
  and map parameters with `@Path`, `@Query`, `@Body`, and related annotations.
- **Routing:** create one `lib/route.dart` entrypoint that imports/exports
  `route.g.dart`, extend the generated `$RootRouter`, and annotate route widgets with
  `@AppRoute`.
- **State:** annotate a ViewModel with `@ViewModel`, extend its generated base,
  keep UI values in state, and put dependencies in typed args.
- **i18n:** configure locales in `dust.yaml`, use literal namespaced keys,
  declare every locale asset directory, and wrap the app with generated
  `AppI18n`.
- **Database:** keep row models and database roots in separate libraries. Use
  `@Derive([FromRow()])`, `@SqlxDatabase`, `@SqlxDao`, raw static `@Query` SQL,
  and migrations. SQLite is native-only; it does not support web.
- **HTTP servers:** `dust_server` is a Dart HTTP runtime on `shelf`, with an
  API modelled on Rust's axum — `Router` with `route`, `nest`, `merge`,
  `mount`, `layer`, `routeLayer`, `withState`, and `fallback`;
  `FromRequestParts` and `FromRequest` extractors; `IntoResponse` and
  `Rejection`. Nothing is generated, so compose the router by hand and serve
  it. Native-only; no web support.

## Command routing

| Task | Write command | Read-only verification |
| :--- | :--- | :--- |
| Normal `.g.dart` generation | `dust build` | `dust check` |
| Normal incremental generation | `dust watch` | `dust check` |
| Database row mapping | `dust build` | `dust check` |
| Database/DAO/SQL validation | `dust db build` | `dust check --db` |
| Offline Database validation | `dust db build --offline` | `dust check --db --offline` |
| Translation inventory | — | `dust i18n scan` |
| ARB/bootstrap reconciliation | `dust i18n build` | `dust i18n check` |

An offline Database command requires compatible query metadata written by a
successful online database build. Normal `dust build` does not generate
`@SqlxDatabase`, `@SqlxDao`, or `@Query` output.

No Dust command generates server code.

Firebase, Supabase, PostgreSQL runtime support, and ORM/query-builder behavior
are not current Dust features. Do not generate examples that imply otherwise.
