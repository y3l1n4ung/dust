# Changelog

All notable changes to Dust are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [v0.2.0] - 2026-09-13

### Added

- **fp**: `Option<T>` gains the Rust operations it was missing, so it is no
  longer the thin half of `fp.dart`:
  - construction and null interop: `Option.fromNullable`, `toNullable`,
    `toIterable`
  - queries: `contains`, `isSomeAnd`, `isNoneOr`, `unwrap`, `expect`
  - transforms: `mapOr`, `mapOrElse`, `inspect`, `filter`
  - choosing: `and`, `or`, `orElse`, `xor`
  - combining and `Result` interop: `zip`, `zipWith`, `unzip`, `okOr`,
    `okOrElse`, `flatten`, `transpose`

  A present `null` stays present throughout: `Some<T?>(null)` is `Ok(null)`
  from `okOr`, and only `toNullable` collapses it. Lazy forms (`mapOrElse`,
  `orElse`, `okOrElse`) never call a callback whose branch does not apply.
  `unwrap` and `expect` throw `StateError` on `None`.

  JSON and database support for `Option` fields is not part of this: it waits
  on deciding what `None()` and `Some(null)` mean on the wire (#88, #541).

- **Database**: described column types and nullability are checked against the
  row class and reported as warnings. A `TEXT` column read into an `int` is
  named, and so is a nullable column read into a non-nullable field.

  The accepted type pairs are documented per dialect and deliberately
  permissive: anything the table does not cover is accepted rather than
  reported, so a converter type or an enum read through `tryFrom` costs nothing.
  SQLite's rows are wide because it has affinity rather than types.

  Nullability is checked for PostgreSQL only. SQLite describes a `PRIMARY KEY`
  column as nullable, which warned about five correct queries in
  `fixtures/server_app`; the column-alias overrides work on either dialect.

- **Database**: SQLx's column-alias overrides. `SELECT x as "total?"` marks a
  column nullable and `as "total!"` marks it not null, whatever the database
  inferred — which a `LEFT JOIN` otherwise gets wrong, since it makes a
  `NOT NULL` column nullable in its result. The marker is part of the alias, so
  it arrives as part of the column name and is removed before the column is
  matched to a row field; a row class spells the column `total` and the
  generated decoder reads `total`.

- **Database**: `DatabaseClient.migrate()` applies the migrations a database was
  generated with. SQLite applies them while opening and returns `Ok`;
  PostgreSQL is reached over a network and cannot, so it applies them here. One
  startup path serves both, and no application needs to know which.

- **Database**: `fixtures/postgres_app` and a CI job that runs it against a
  `postgres:16` service — validating from the committed query cache with no
  server first, then building against a real one and checking the cache is
  current.

- **Database**: PostgreSQL generates and runs. `@SqlxDatabase(type:
  SqlxDatabaseType.postgres)` emits a facade over `PgPool` instead of reporting
  that Postgres is reserved, and the facade's signature follows the database —
  `connect(String url)` where SQLite has `open(String path)`.

  SQL is checked against the schema, the same as SQLite: `dust db build` reads
  `DUST_DATABASE_URL` and describes every static query. Migrations are applied
  into a scratch schema inside a transaction that is rolled back, so validating
  leaves the database as it found it and works against one that already holds
  the schema. `--offline` validates from the committed cache with no server, and
  a cache written for another driver is refused.

- **Database**: what the engine knows about each database now lives in one
  place. Picking a runtime type, deciding whether SQL can be validated, and
  naming the escape hatch were three separate `match` arms, so adding a database
  meant finding all of them and missing one meant generated code naming a type
  from the wrong driver. A dialect is a value; adding MySQL is one more of them.

- **Database**: `dust_db_postgres`, the PostgreSQL runtime, wrapping
  `package:postgres` the way `dust_db_sqlite3` wraps `package:sqlite3`. The
  query text does not change between dialects — PostgreSQL reads `$1` natively
  where SQLite rewrites it — and a Dart `List` binds as a PostgreSQL array, so
  `= ANY($1)` needs no encoding. Nested transactions are savepoints this package
  issues itself, since `package:postgres` exposes no savepoint API.

  Generation does not target it yet: `@SqlxDatabase(type:
  SqlxDatabaseType.postgres)` still reports that Postgres is reserved. The
  runtime lands first so the plugin has something to emit against.

- **Database**: `UnsafeSql` on the database facade, for the administrative SQL
  validation cannot reach. `AppDatabase.unsafe` gives `fetch`, `fetchAs<T>` with
  an explicit mapper, and `execute`. It is reachable from the facade and not
  from an executor, so a request handler cannot get to unchecked SQL — unlike
  `raw`, where `db as Executor` always succeeded.
- **Database**: SQLite binds a `List` argument as JSON text, so a set
  membership test is one placeholder over constant SQL —
  `WHERE id IN (SELECT value FROM json_each(?))` with `[ids]`. SQLite has no
  array type and `package:sqlite3` will not bind a nested list, so building
  `IN (?, ?, ?)` by hand was the most common reason to reach for unchecked SQL;
  the form above is describable, so build-time validation covers it. A
  `Uint8List` is still bound as a BLOB.


- **Database**: each use of the `unsafe` escape hatch warns, so unchecked SQL
  reads as a deliberate line in a diff. Silence one call with a
  `dust:allow-unsafe-sql` comment on it or on the line above; two lines away
  does not count, so one marker can never cover a file. Unchecked SQL is not
  described and never enters the committed query cache — its text may be
  dynamic, so no build could reproduce the entry.

### Changed

- **Database**: a call-site query that fails validation is reported by the
  function containing it — `OrdersService.cancelStale` rather than a bare
  `queryExecute`, which said nothing in a file holding several queries. A query
  inside a closure names the function around it, and one outside any function
  keeps the helper name.

- **Database**: placeholder rewriting moved from generated code into the driver.
  `.g.dart` now carries the SQL verbatim, `$n` and all, and binds arguments in
  declaration order; `dust_db_sqlite3` rewrites to `?` at bind time. Only DAO
  methods were ever rewritten, so the same `$1` meant one thing in a `@Query`
  and another in an inline `queryAs`, and a repeated `$1` expanded on one path
  and not the other. Which placeholder form the database receives is the
  driver's business — Postgres takes `$n` unchanged — so generated code could
  not pick one without being wrong for the other dialect.

- **Database**: the pool vocabulary follows SQLx. `DatabaseExecutor` becomes
  `Executor`, `DatabaseConnection` becomes `Connection`, and
  `DatabaseTransaction` becomes `Transaction`; `Pool` is unchanged. Dust's names
  diverged for no reason and in one place inverted SQLx's, since `Executor` had
  been taken by a different type. Rename call sites; the shapes are identical.
- **Database**: adding a database is a `Dialect` entry rather than a search for
  every `match` on the driver. Analysis, both annotation parsers and the column
  type checker read the dialect registry, so a driver is named, spelled and
  aliased in exactly one place.
- Repository: hand-written source files stay under 300 lines, checked in the
  lint gate and in CI. A publishable package is named once, in the root pubspec
  workspace, and the lint, format and test targets are derived from it; Sonar is
  checked for a coverage report per package, which is what silently failed
  before.



- **Database**: inline query terminals return `Result<T, SqlxError>` rather than
  throwing. Generated `@SqlxDao` methods already did; the inline path unwrapped
  the same executor call into a `StateError` that discarded the typed error, so
  the two ways of running a query disagreed about what a failure is. Affects
  `QueryAs.fetch*With`, `QueryScalar.fetchOne`/`fetchOptional`, `QueryRaw.fetch`,
  `QueryExecute.execute`, and the generated `extension $TypeQuery` terminals.
  Migration in
  [`packages/dust_dart/CHANGELOG.md`](packages/dust_dart/CHANGELOG.md).

- **Database**: the offline query metadata cache moved from
  `.dart_tool/dust/db_query_cache_v2/` to `.dust_sql/` at the package root, and
  is now a committed build input rather than a build artifact. `.dart_tool/` is
  gitignored, so nothing could validate from the cache on a clean checkout —
  which is the only way CI can validate a Postgres project, since `describe`
  there needs a live server. `dust clean` leaves `.dust_sql/` in place. Cache
  format version 3; regenerate with an online `dust db build` and commit the
  result.
- **Database**: each cache entry records the driver it was described against.
  The same SQL describes differently per dialect, so a cache written for one
  driver is now rejected against another with an error naming both, rather than
  reported as a missing entry.

### Fixed

- **Database**: `@Query` SQL renders to a Dart literal that compiles and carries
  the SQL that was written. The text was wrapped in `r'''...'''` whenever it did
  not already contain the delimiter, which breaks four ways.

  Two refuse to compile: SQL ending on a quote merges with the closing
  delimiter, and SQL containing `'''` closes the literal early. Any query
  filtering on a string constant hits the first, and `dust db build` reported
  success before the generated file failed to parse.

  Two are worse, because they compile. SQL ending on two quotes loses one to the
  delimiter, and SQL containing a carriage return loses it to the source reader.
  Both produce a query that runs against the database carrying SQL nobody wrote.

  All four now take the escaped form. A quote anywhere but the end still stays
  raw, and so do `$n` placeholders, backslashes, tabs and newlines, which is
  what keeps generated SQL readable.
- **SerDe**: an enum declared in one library now round-trips through a class in
  another. The enum's helpers are private to its own library, so a class
  elsewhere fell through to `status.toJson()` and `Status.fromJson(...)`,
  neither of which an enum has: the generated code did not compile, and where
  it was reached at runtime a value the encoder wrote could not be read back.
  Those use sites now go through the enum's public `$StatusSerializer` and
  `$StatusDeserializer`, including nullable fields, collections and
  import-prefixed types.
- **Database**: `dust_db_postgres` is recognised as a workspace runtime
  package. It was in the CLI's compatibility contract but not in the import
  table workspace discovery scans, so `dust doctor` reported it unused against
  a project that imports it, and a build resolving an incompatible version was
  accepted rather than refused. A test now holds the dialect registry, workspace
  discovery and the contract to each other.

### Removed

- **Database**: the `SqlxDriver` typedef, and the `Connection` and `Transaction`
  marker types that aliased what are now the real names.
- **Database**: `queryRaw`, `QueryRaw`, and the `raw` channel on executors, with
  the `Executor` interface that carried it. `Executor` was `DatabaseExecutor`
  plus `raw`, and `Pool`, `Connection` and `Transaction` all implemented it, so
  `db as Executor` always succeeded — a fence that stopped nobody. Unchecked SQL
  is now `unsafe` on the database facade, which an executor cannot reach.

  A DAO method returning `List<Row>` is reported at build time instead of
  generating an unchecked fetch, and `queryRaw` is no longer parsed.

## [v0.1.4] - 2026-09-03

> [!IMPORTANT]
> **Breaking, and `^0.1.3` upgrades into it.** `dust_dart` 0.1.4 removes
> `RowMapperRegistry`, `registerRowMapper`, and the `QueryAs` instance
> terminals `fetchOne`, `fetchOptional` and `fetchAll`. A pubspec asking for
> `dust_dart: ^0.1.3` resolves to 0.1.4, so an app that pins nothing gets the
> removal without asking for it. Run `dust build` to regenerate; call sites of
> `queryAs<T>` do not change. The two moves that need an edit are written out
> under Migrating in
> [`packages/dust_dart/CHANGELOG.md`](packages/dust_dart/CHANGELOG.md).
>
> `dust_server` is also breaking, twice, but it is pre-1.0 and its ranges are
> prerelease — see the Server entries below.

### Added

- **Database**: row mapping has an interface and generated query terminals.
  `@Derive([FromRow()])` now emits a private `_$TypeFromRow(Row row)`, a public
  `$TypeRowDeserializer` implementing the new `RowDeserializer<T>` in
  `dust_dart`, and `extension $TypeQuery on QueryAs<Type>` carrying `fetchOne`,
  `fetchOptional`, and `fetchAll`. Dart resolves an extension member from the
  static type of the receiver, so `queryAs<Order>(sql, args).fetchOne(db)` is
  picked at compile time and a row type with no `FromRow` has no terminal at
  all — the call does not compile.
- **Database**: `dust db build` rejects a `queryAs<T>` whose `T` has no row
  mapping anywhere in the package, naming the type and the call site.
- **Database**: `QueryAs.fetchOneWith`, `fetchOptionalWith`, and `fetchAllWith`
  take the row mapping as an argument, for a row type Dust does not generate.
- **Server**: `dust_server`, a Dart HTTP runtime on shelf, is published for the
  first time — `0.1.0-beta.1` through `0.1.0-beta.3` all fall in this release.
  Generated handlers target it directly: `dust build` emits `Router.module`,
  `Route`, the `Rejection` type, `QueryExtractable`, `StateExtractable`, and
  `intoResponse()`, so it now carries a row in the CLI compatibility contract
  like every other runtime package.
- **Server**: `DisposableLayer`, `Service`, and a typed serve address, so a
  layer holding a connection pool is disposed once when the server stops
  rather than per request.
- **Server**: a dead isolate is reported instead of silently absorbed, and the
  diagnostic says why it cannot be restarted.
- **Server**: response bodies stream — a body too large to encode is streamed
  rather than failing, a multipart body is streamed rather than buffered, and
  background work is drained alongside requests at shutdown.
- **Server**: an extractor runs as a layer, under axum's names.
- **Server**: `TestClient` testing framework, exported via
  `package:dust_server/testing.dart`. Three modes: handler (in-process, no
  socket), serve (real HTTP on port 0), and origin (connect to an existing
  server). Cascade API with `TestRequest` setters and `TestResponse` named
  status assertions (`assertOk`, `assertCreated`, `assertConflict`, etc.).
- **Routing**: `@AppRoute(..., branch: 'name')` for stateful branch stacks that
  preserve independent tab history without adding another annotation.
- **Routing**: expanded router diagnostics to include route names, effective
  shells, branches, guard decisions, redirect targets, and committed stacks.
- **Routing**: runtime hooks for `NavigatorObserver` integration and asynchronous
  navigation error handling.
- **Routing**: expanded route edge-case coverage across shell validation,
  generated shell emission, Dart parser/controller behavior, guard chains, and
  stack lifecycle.
- **Routing**: practical routing use-case recipes and Flutter package examples
  for search filters, shell inheritance, shell overrides, and result routes.
- **Routing**: research-backed routing recipes for invite links,
  organization-scoped detail pages, typed multi-step setup flows, and Flutter
  web path URLs.
- **Routing**: `auth` column in `dust route table` so route inspection shows
  whether each route is `public` or `protected` instead of leaving auth state
  implicit in the `guards` column.

### Removed

- **Database**: `RowMapperRegistry` and `registerRowMapper`, along with the
  `registerRowMapper` initializer generated row files used to emit. A
  process-wide `Map<Type, RowMapper>` filled by top-level initializers made a
  missing row mapping a runtime `SqlxError.decode` that depended on whether the
  part file had been imported anywhere in the isolate. Generated DAOs never
  used it, and with generated terminals nothing else does either.

### Changed

- **Database**: generated row output follows the naming the other derives use.
  The public `extension TypeFromRow on Type` with its `static fromRow` is
  replaced by the private `_$TypeFromRow` function — mirroring serde's
  `_$TypeSerialize` — and the public `$TypeRowDeserializer` witness, mirroring
  `$TypeSerializer`. Generated DAOs decode through the witness. Call sites of
  `queryAs<T>` are unchanged.
- **Server**: `serve`, `serveIsolates`, and the surrounding names follow axum,
  which is the shape the rest of Dust's server design is written against. This
  landed at `0.1.0-beta.2` with no deprecated aliases; a `0.1.0-beta.1` app is
  renamed by hand. `packages/dust_server/CHANGELOG.md` lists each move.
- **Server**: `dust_dart` runtime constraint raised to `^0.1.4`, alongside
  `dust_db_sqlite3`. Both were asking for `^0.1.3`, which resolves to either
  side of the `dust_dart` removals above.
- Simplified generated route names. The router name still scopes the generated
  base route type and helpers, so `ShopRouter` generates `ShopRoute`,
  `parseShopRoute`, and `$ShopRouter`, while concrete route classes use their
  route names directly, such as `CheckoutRoute`. Route pages, annotations, and
  navigation call sites are unchanged; code that names generated types directly
  must be renamed.
- Restored generated router base classes to Dust's `$ClassName` convention, so
  handwritten routers extend generated bases such as `$ShopRouter`.
- Split the routing usage docs into a shorter main guide plus focused deep-link,
  shell/branch, and guard references.
- Split the router delegate internals into parser, stack, and diagnostics
  helpers while keeping the generated routing API unchanged.
- Pruned generated router imports so route files keep page imports and directly
  referenced shell, guard, transition, and result-type imports without leaking
  unrelated page-library dependencies.
- Moved repeated generated route URL, URI extras, bool parsing, shell check,
  and no-transition helpers into the `dust_flutter` runtime.
- Revalidated exposed routes after pop and page removal so guards and redirects
  stay current when session state changes.
- Raised the supported `dust_flutter` runtime floor to `0.1.4`.

### Fixed

- **Parser**: a comment between an annotation and the declaration it applies to
  no longer drops the declaration. Dart treats such a comment as trivia and
  keeps the metadata attached — the analyzer reports
  `override_on_non_overriding_member` for an `@override` separated from its
  method by a doc comment — but the parser returned the comment where it
  expected the declaration and skipped the member entirely. A `@SqlxDao` method
  documented this way generated nothing, and the failure surfaced as Dart
  complaining about a missing override rather than as a diagnostic. All three
  comment forms were affected, not only doc comments.
- **Database**: `DUST_DATABASE_URL` is used only when its scheme names the
  project's own driver. One workspace can hold projects on both drivers while
  the variable names one database; a SQLite project handed a PostgreSQL URL used
  to try to open it and report whatever the other driver's URL parser disliked
  (`unknown query parameter \`sslmode\``). It now falls back to its in-memory
  schema, and a PostgreSQL project handed a SQLite URL says which scheme it
  needs.
- **Database**: the query metadata cache is one file per library rather than one
  per package. Libraries are validated in parallel worker threads, so a shared
  path meant several threads read-modify-writing the same file at once: across
  15 clean builds of a three-library fixture the file was invalid JSON six times
  and held 4 or 8 of its 12 entries the rest.
- **Database**: a `@Sqlx(flatten: true)` field whose row class is declared in
  another library is accepted. The target was resolved against the file being
  validated, so the normal layout was rejected for a class that derives it.
- **Database**: a package that declares the same row class name in two libraries
  says so, instead of picking one.
- **Database**: SQL validation now resolves the schema and the row classes across
  the whole package instead of within one file.
- **Database**: library discovery reads annotation names nested in an argument
  list, so `@Derive([FromRow()])` is found by every plugin that owns a name
  inside the brackets.
- **Database**: a focused build no longer rewrites the generated output of a
  library whose code belongs to a plugin that is not running in that mode.
- **Database**: the SQL placeholder scanner understands comments and
  dollar-quoted bodies.
- **Database**: the diagnostic for a placeholder count mismatch names what
  happened and prints the SQL the database parsed.
- **Database**: cached libraries are invalidated when workspace analysis changes.
- **Server**: a factory that threw inside an isolate hung startup forever
  instead of reporting the failure.
- **Server**: a shared layer is disposed once rather than per isolate, and a
  timeout now says that it does not cover a streaming response.
- **Server**: a failing event stream is reported instead of swallowed, and no
  request id is echoed back from the client.
- **Server**: a bad response header hung the connection, and numeric header
  values were trimmed.
- **Server**: an event stream is flushed rather than buffered, and any response
  whose length is not known is flushed too.
- **Server**: a WebSocket upgrade is treated as the success it is, in tracing,
  the access log, and the layers around it.
- **Server**: a bare `String` return is sent as text, and a 401 offers every
  scheme it accepts.
- **Server**: `TestClient` header assertions answer the same way in handler
  mode and over real HTTP, the cookie jar keeps every `set-cookie` a response
  sends rather than the first, and a binary response body arrives as bytes
  rather than being decoded as UTF-8 on the way in and losing what does not
  survive it.
- Dismissed dialogs, modal sheets, and other imperatively pushed routes on
  system back instead of popping the generated page underneath them. Back now
  also respects `PopScope` on a generated page, which the previous pop path
  ignored entirely.
- Failed route guards loudly instead of silently skipping them. Generated guard
  lists are now typed as `List<RouteGuardBase<ShopRoute>>`, so a class in
  `guards:` that implements neither `RouteGuard` nor `AsyncRouteGuard` is an
  analyzer error, and the router throws rather than allowing navigation if one
  reaches the runtime. Router diagnostics no longer log `allow` for a guard that
  never ran.
- Rejected invalid route helper identifiers, `pop` helper conflicts, and
  generated route class collisions before emitting analyzer-broken Dart.
- Rejected generated route classes that would collide with existing Dart
  classes or router support classes when using unprefixed route names.
- Reported a concrete fix when a local route shell widget cannot be generated
  as `Shell(child: page)`.

## [v0.1.3] - 2026-07-28

This release hardens Dust for real Dart and Flutter projects.

### Added

- CLI/runtime compatibility checks for supported Dust package versions.
- Source-first JSON `serialize` and `deserialize` APIs with `toJson` and
  `fromJson` ecosystem mirrors.
- `Validatable` for generated validation APIs.
- Typed route results with generated `Future<T?> push()` helpers.
- Router stack observer support.
- `runAction` for stale-safe async ViewModel commands.
- Explicit i18n source-locale sync and opt-in iOS locale metadata sync.
- SQLite connect options and safer DB transaction support.
- Database runtime contracts and typed query helper support.
- `dust upgrade` support.

### Changed

- Preserved existing translated ARB values during i18n generation.
- Made failed ViewModel initialization retry only through explicit
  `retryInit()`.
- Hardened Dust DB migrations, mapper errors, and execution API naming.
- Automated codegen tool fingerprints for safer cache invalidation.

### Fixed

- Deep-link and restored-stack routing edge cases.
- Deprecated `StateEffect` values are unwrapped before effect delivery.

## [v0.1.2] - 2026-07-10

### Fixed

- Generated `RouteAction.push()` now returns a `Future<R?>` that completes when
  the pushed route is popped, including pop results.
- Route `transition:` annotations now apply at the actual page route boundary.
- Generated no-transition routes now use zero-duration transitions.

## [v0.1.1] - 2026-07-09

### Added

- Router debug diagnostics for generated router troubleshooting.

### Changed

- Released CLI binary reports `0.1.1`.
- Validation codegen keeps Dart model validation separate from Flutter form
  validators.
- Package README and pub.dev-facing documentation were polished.
- Apple Silicon release assets were aligned.

## [v0.1.0] - 2026-05-08

### Added

- Polish derive APIs and output.
- Strengthen serde generation.
- Add enum serde support.
- Add shared plugin analysis pipeline.
- Upgrade CLI clap surface.
- Attach source context to diagnostics.

### Changed

- Implement plugin-driven annotation discovery.
- Split serde emit module.
- Split serde writer module.
- Split driver build module.
- Split driver watch module.
- Split resolver module.
- Split driver lower module.
- Split tree-sitter parser module.
- Split derive copywith module.
- Split driver integration tests.
- Split derive plugin tests.
- Split serde class tests.
- Split remaining oversized tests.
- Split driver batch module.
- Migrate CLI parser to clap.
- Trim build pipeline allocations.
- Split batch build orchestration.
- Split build processing stages.
- Dedupe driver outcome handling.
- Share driver bootstrap context.
- Unify diagnostic rendering and lowering parsing.

### Fixed

- Dust package discovery error on workspace.

<!-- generated by git-cliff -->
