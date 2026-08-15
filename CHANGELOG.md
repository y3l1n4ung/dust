# Changelog

All notable changes to Dust are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [v0.1.4] - 2026-08-09

### Added

- Added `@AppRoute(..., branch: 'name')` for stateful branch stacks that
  preserve independent tab history without adding another annotation.
- Expanded router diagnostics to include route names, effective shells,
  branches, guard decisions, redirect targets, and committed stacks.
- Router runtime hooks for `NavigatorObserver` integration and asynchronous
  navigation error handling.
- Expanded route edge-case coverage across shell validation, generated shell
  emission, Dart parser/controller behavior, guard chains, and stack lifecycle.
- Added practical routing use-case recipes and Flutter package examples for
  search filters, shell inheritance, shell overrides, and result routes.
- Added research-backed routing recipes for invite links, organization-scoped
  detail pages, typed multi-step setup flows, and Flutter web path URLs.
- Added an `auth` column to `dust route table` so route inspection shows whether
  each route is `public` or `protected` instead of leaving auth state implicit
  in the `guards` column.

### Changed

- Scoped every generated router symbol to the handwritten router class. The
  router name drops its `Router` suffix and the remaining stem prefixes the
  route path base class, route classes, parser function, navigator, and route
  action class, so `ShopRouter` now generates `ShopRoutePath`, `parseShopRoute`,
  and `ShopCheckoutRoute` instead of `AppRoutePath`, `parseAppRoute`, and
  `CheckoutRoute`. Route pages, annotations, and navigation call sites are
  unchanged; code that names generated types directly must be renamed.
- Restored generated router base classes to Dust's `$ClassName` convention, so
  handwritten routers extend generated bases such as `$ShopRouter`.
- Split the routing usage docs into a shorter main guide plus focused deep-link
  and shell/guard references.
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

- Dismissed dialogs, modal sheets, and other imperatively pushed routes on
  system back instead of popping the generated page underneath them. Back now
  also respects `PopScope` on a generated page, which the previous pop path
  ignored entirely.
- Failed route guards loudly instead of silently skipping them. Generated guard
  lists are now typed as `List<RouteGuardBase<RoutePath>>`, so a class in
  `guards:` that implements neither `RouteGuard` nor `AsyncRouteGuard` is an
  analyzer error, and the router throws rather than allowing navigation if one
  reaches the runtime. Router diagnostics no longer log `allow` for a guard that
  never ran.
- Rejected invalid route helper identifiers, `pop` helper conflicts, and
  generated route class collisions before emitting analyzer-broken Dart.
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
