# Changelog

All notable changes to Dust are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
