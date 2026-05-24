# Routing Feature Checklist

This checklist tracks Dust routing from both sides: Rust generator/plugin and
Flutter runtime/prototype. A routing item is not done until Rust generation and
Flutter behavior are both covered.

## Status Legend

- `[x]` implemented and covered by tests.
- `[~]` partially implemented or covered by only one side.
- `[ ]` not implemented yet.

## Public API

- [x] `@Router` annotation lives on the router entrypoint in `lib/route.dart`.
- [x] `@Route` annotation lives on normal page/widget classes in `lib/**`.
- [x] App code imports only `route.dart` for router usage.
- [x] `route.g.dart` is standalone generated Dart, not a `part` file.
- [x] Generated code imports page, shell, guard, and package runtime owners.
- [x] `dust_router` owns annotations plus Navigator 2.0 runtime.
- [x] Generated navigation is route-name-first: `context.routes.project(...).go()`.
- [x] Navigation actions support `go`, `push`, `replace`, and `pop`.
- [x] Public docs describe router, route, guards, shells, params, transitions, and app setup.
- [x] Public Dartdoc complete for annotation/runtime public API surface.

## Route Discovery And Validation

- [x] Discover `@Router` and `@Route` across workspace files.
- [x] Collect versioned route facts: `dust_route.routes.v1`.
- [x] Collect versioned router facts: `dust_route.routers.v1`.
- [x] Preserve route source path and generated import URI.
- [x] Preserve page-library imports needed by shells and guards.
- [x] Reject relative route paths.
- [x] Reject duplicate normalized paths.
- [x] Reject duplicate names.
- [x] Reject missing path constructor params.
- [x] Reject nullable/defaulted path params.
- [x] Reject unsupported path/query param types.
- [x] Reject query defaults when default source is unavailable.
- [x] Support query defaults when default source is preserved.
- [x] Ignore `Key? key` and `super.key` route params.
- [x] Validate missing initial route page with driver-level fixture.
- [x] Validate missing not-found route page with driver-level fixture.
- [x] Validate unresolved shell/guard imports with driver-level fixture.
- [x] Validate multiple `@Router` roots target separate outputs correctly.

## URL And Params

- [x] Use Dart `Uri` for path and query building.
- [x] Avoid raw empty path segment pattern in generated route classes.
- [x] Support absolute root route `/`.
- [x] Support path params from `:param` segments.
- [x] Support query params from non-path constructor params.
- [x] Support `String`, `int`, `double`, `bool`, nullable variants.
- [x] Support default query params and omit defaults from generated URLs.
- [x] Parse `int` with `int.tryParse`.
- [x] Parse `double` with `double.tryParse`.
- [x] Parse `bool` from `true`, `false`, `1`, `0`.
- [x] Redirect malformed path params to not-found route.
- [x] Preserve unmatched not-found URI without double encoding.
- [x] Avoid double-decoding string path params from Dart `Uri.pathSegments`.
- [x] Reject custom objects, lists, maps, records, functions, and `dynamic`.
- [x] Add Rust test for bool query parsing emission.
- [x] Add Flutter test for bool query compact links.
- [x] Add Flutter test for malformed `int` and `bool` params; no double route exists yet.
- [x] Add Flutter test for encoded path and query values with spaces/slashes.

## Route Graph And Nesting

- [x] Generate nested metadata from path segments.
- [x] Support synthetic group nodes without page class.
- [x] Keep root `/` as own route, not parent for every route.
- [x] Generate readable nested `GeneratedRoute` tree.
- [x] Keep parse/navigation typed and flat even when metadata is nested.
- [x] Cover long nested example in prototype: `/billing/invoices/:invoiceId`.
- [x] Cover child route example: `/projects/:projectId/settings`.
- [x] Add Rust snapshot/fixture for 4+ level nested route tree.
- [x] Add Flutter test for back/forward across nested shell pages.
- [x] Add metadata traversal helper tests in annotation package.

## Shells

- [x] Route-level `shell: AppShell` metadata is parsed.
- [x] Generated page builder wraps child with the route's explicit shell.
- [x] Generated metadata includes shell type.
- [x] Prototype covers app shell pages.
- [x] Generate inherited shell wrapping from nearest parent route shell.
- [ ] Support true nested shell page stacks instead of one flat leaf page.
- [x] Support shell inheritance from parent route policy.
- [x] Add Flutter test that shell remains stable across child navigation.
- [x] Add Rust test for shell import from separate file.

## Guards And Redirects

- [x] Router-level redirect runs before route guards.
- [x] Route guards are declared with `guards: [GuardType]`.
- [x] Explicit public route uses `guards: []`.
- [x] Generated router declares guard factory hooks.
- [x] App router can override guard factories for dependency injection.
- [x] `RouteGuardChain` runs guards in declaration order.
- [x] Guards may allow, block, or redirect with typed route.
- [x] Runtime cancels stale async navigation with epoch counter.
- [x] Prototype covers auth redirect.
- [x] Prototype covers admin guard redirect.
- [x] Prototype covers feature/billing guard redirect.
- [x] Prototype covers public checkout route.
- [x] Add Flutter test for async stale guard cancellation.
- [x] Add Flutter test for guard block/no-route-change behavior.
- [x] Add Rust test for multiple guard order emission.

## Transitions And Pages

- [x] Use Flutter `MaterialPage` for generated pages.
- [x] Support per-route `PageTransitionsBuilder`.
- [x] Support app-wide default transitions through `PageTransitionsTheme`.
- [x] Support `BottomToTopPageTransitionsBuilder` for fullscreen dialog routes.
- [x] Support generated private no-transition builder for generated output.
- [x] Remove Dust-owned `PageType` and `RouteTransitions` registry.
- [x] Support `fullscreenDialog`.
- [x] Support `maintainState`.
- [x] Prototype covers modal/custom checkout.
- [x] Add Flutter behavior smoke for configured transition builders.
- [x] Rust page-type emission tests obsolete; Dust now emits MaterialPage only.

## Runtime Behavior

- [x] Runtime exposes `DustRouterBase`.
- [x] Runtime exposes `DustRouterController`.
- [x] Runtime exposes `DustRouteState`.
- [x] Runtime exposes `RouteGuard` and `RouteGuardResult`.
- [x] Runtime exposes `RouteGuardChain`.
- [x] Runtime exposes `GeneratedRoute`.
- [x] Runtime uses Flutter Navigator 2.0 only.
- [x] Runtime builds `RouterConfig` for `MaterialApp.router`.
- [x] Runtime owns `RouteInformationParser`.
- [x] Runtime owns `RouterDelegate`.
- [x] Runtime syncs browser URLs.
- [x] Runtime supports `go`, `push`, `replace`, `pop`.
- [x] Runtime supports reactive `refreshListenable`.
- [x] Add annotation package tests for controller stack operations.
- [x] Add annotation package tests for refresh-triggered redirect.
- [x] Add annotation package tests for browser parser/restore roundtrip.
- [x] Add annotation package tests for stale async guard cancellation.

## Generated Code Quality

- [x] Generated `route.g.dart` has generated-code header.
- [x] Generated code is formatted directly by the Rust emitter.
- [x] Generation does not shell out to `dart format`.
- [x] Generated output is deterministic and stable.
- [x] Generated code has small public API surface.
- [x] Generated imports are stable and deduplicated.
- [x] Generated code does not expose former Rust/internal APIs.
- [x] Add snapshot fixture for full generated production route file.
- [x] Add snapshot coverage for generated prototype formatting.
- [x] Add generated-code size/readability check for large route count.

## Driver And Cache

- [x] Driver registers `dust_route_plugin`.
- [x] Route annotations are partless configs; page files need no `part`.
- [x] Route-only page libraries do not emit fake `.g.dart` outputs.
- [x] Router root emits only `route.g.dart`.
- [x] Cache stores route workspace analysis facts.
- [x] Cached router root rebuilds when an annotated route page changes.
- [x] Non-routing generators still work with route plugin registered.
- [x] Add driver test for `dust check` stale route output.
- [x] Add driver test for `dust clean` removing `route.g.dart` only.
- [x] Add watch test for route page change rebuilding router output.
- [x] Add pub workspace/member test for route generation.

## Rust Test Matrix

- [x] Plugin registration claims route annotations.
- [x] Parse `@Router` initial/notFound.
- [x] Parse `@Route` metadata.
- [x] Workspace analysis collects route/router facts.
- [x] Validation accepts URL primitives.
- [x] Validation rejects invalid params and duplicate paths/names.
- [x] Emission imports cross-file pages.
- [x] Emission supports query defaults.
- [x] Emission uses package runtime.
- [x] Driver writes route output from router root only.
- [x] Driver refreshes route output when page annotation changes.
- [x] Snapshot full generated output.
- [x] Driver `check` route stale behavior.
- [x] Driver `watch` route rebuild behavior.
- [x] Driver `clean` route output behavior.
- [x] Pub workspace route generation.

## Flutter Test Matrix

- [x] `packages/dust_router`: annotation and runtime metadata tests.
- [x] Prototype `flutter analyze` passes.
- [x] Prototype `flutter test` passes.
- [x] Prototype `flutter build web` passes.
- [x] Auth redirect fixture.
- [x] Admin guard fixture.
- [x] Billing guard fixture.
- [x] Public modal route fixture.
- [x] Deep link parser fixture.
- [x] Constructor param URL restore fixture.
- [ ] Browser back/forward fixture.
- [ ] Browser reload fixture for deep links on web server fallback.
- [x] Refresh-listenable redirect fixture.
- [x] Async stale guard fixture.
- [x] Encoded URL values fixture.
- [x] All configured transition builders fixture.
- [x] Large route count smoke fixture.

## Release Gate

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p dust_route_plugin`
- [x] `cargo test -p dust_driver`
- [x] `cargo clippy -p dust_driver -p dust_route_plugin --all-targets -- -D warnings`
- [x] `cargo run -p dust_cli -- build --root examples/routing_prototype --fail-fast`
- [x] Route emitter snapshot tests cover generated formatting without running a
  Dart formatter.
- [x] `cd packages/dust_router && flutter test`
- [x] `cd examples/routing_prototype && flutter analyze`
- [x] `cd examples/routing_prototype && flutter test`
- [x] `cd examples/routing_prototype && flutter build web`
- [x] Manual browser smoke: `/`, `/invite/abc`, `/projects/42?tab=activity`, `/billing/invoices/inv_001`, `/checkout/team?annual=true`, unknown path.
