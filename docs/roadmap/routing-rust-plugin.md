# Routing Rust Plugin Implementation Plan

## Goal

Implement `dust_route_plugin` so Dust can generate the routing prototype from
normal Dart annotations:

- collect `@Router` and `@Route` across the Dart workspace
- validate route metadata and constructor parameters
- emit standalone `route.g.dart` from the router root library
- import Navigator 2.0 runtime from `dust_flutter`
- keep generated navigation route-name-first: `context.routes.project(...).go()`

## Current Implementation Status

- [x] Crate skeleton exists and is registered in the Rust workspace.
- [x] Driver registers the route plugin.
- [x] Codegen fingerprint includes route plugin source files.
- [x] Plugin claims `@Router`, `@Route`, and
      `GeneratedRoute`.
- [x] Workspace analysis collects `dust_route.routes.v1` and
      `dust_route.routers.v1` JSON facts.
- [x] Validation rejects relative paths, duplicate paths/names, missing path
      constructor params, nullable/defaulted path params, and complex route
      params.
- [x] Emit standalone primary `route.g.dart` for route pages available in the
      router root library.
- [x] Reuse runtime from `dust_flutter`; no local `routing_core.dart`
      output.
- [x] Preserve constructor default value source in IR and support query
      defaults.
- [x] Extend workspace analysis with source/import metadata and support
      cross-file page imports.
- [ ] Add driver-level output/cache tests.

## Crate Shape

Add a new Rust crate:

```text
crates/dust_route_plugin/
  Cargo.toml
  src/
    lib.rs
    plugin.rs
    constants.rs
    model.rs
    parse.rs
    analysis.rs
    validate.rs
    emit/
      mod.rs
      core.rs
      generated.rs
```

Responsibilities:

- `plugin.rs`: implement `DustPlugin`, claim routing annotations, and connect
  collect, validate, and emit phases.
- `constants.rs`: annotation names, workspace-analysis keys, generated file
  names, and supported transition/page-type literals.
- `model.rs`: typed route records, router root records, parameter records,
  guard records, shell records, and generated route tree records.
- `parse.rs`: convert parsed Dart surfaces into routing records.
- `analysis.rs`: serialize and deserialize workspace route facts.
- `validate.rs`: produce diagnostics before emission.
- `emit/generated.rs`: generate `route.g.dart`.
- `emit/`: generate `route.g.dart`.

## Workspace Analysis

Use versioned string-set facts first, because the current plugin API already
supports them without broad core changes:

- `dust_route.routes.v1`
- `dust_route.routers.v1`

Each entry is compact JSON containing:

- package name and source path
- import URI for generated Dart imports
- annotated class name
- route path and route name
- constructor parameter names, types, defaults, nullability, and required flags
- shell type reference
- guard type references
- page type, transition, fullscreen dialog, and maintain-state metadata
- source span for diagnostics

## Validation Rules

Fail generation with diagnostics when:

- no `@Router` root exists for a routing output
- more than one router root targets the same generated files
- initial or not-found page does not point to an annotated route page
- route path is not absolute
- route path normalization produces duplicates
- route names collide after explicit or derived names are resolved
- a `:pathParam` has no matching constructor parameter
- a path parameter is nullable, optional, or has an unsupported type
- a query parameter has an unsupported type
- a default value cannot be represented in generated Dart
- a guard or shell type cannot be imported from the generated file
- a transition expression cannot be emitted as valid Dart from `route.g.dart`

Supported route parameter types are URL primitives only:

- `String`
- `int`
- `double`
- `bool`
- nullable variants
- constructor defaults for query parameters

Do not add object, list, map, record, function, or `dynamic` route-param
support. Complex values must be loaded from app state or repositories after the
page is reconstructed from the URL.

## Emission Rules

Only the router root library emits routing output:

- `route.g.dart`

`route.g.dart` must:

- be a standalone Dart library, not a `part`
- import `route.dart`
- import `package:dust_flutter/route.dart`
- import all discovered pages, shells, and guard type owners with stable aliases
- generate `$AppRouter`
- generate the sealed route path type and one route data class per route
- generate route metadata records
- generate URL parse and restore helpers
- generate guard resolution
- generate page builders and transition metadata
- generate `BuildContext.routes`
- generate one route factory method per route name
- return `RouteNavigation` with shared `go`, `push`, and `replace` actions

URL encode/decode emission must use Dart `Uri` only:

- generate one private `_routePath(segments, queryParameters: ...)` helper
- `_routePath` builds absolute locations with
  `Uri(pathSegments: segments, queryParameters: ...)`, then prefixes `/`
- route classes call `_routePath(...)`; do not emit raw empty path segments in
  every route class
- read paths from `uri.pathSegments`
- read queries from `uri.queryParameters`
- use `int.tryParse`, `double.tryParse`, and generated bool parsing for typed
  primitive decoding
- return the configured not-found route when path params fail to decode
- do not emit JSON parsing, `dynamic` parsing, or object codecs for route params

`dust_flutter` runtime must:

- use Flutter Navigator 2.0 only
- provide `DustRouterBase`, `DustRouterConfig`, `DustRouterController`,
  `DustRouteState`, `RouteGuard`, and `RouteGuardResult`
- implement `RouteInformationParser` and `RouterDelegate`
- manage browser URL sync, push, replace, pop, and back/forward
- run router redirect before route guards
- cancel stale async navigation with an epoch counter
- expose the current controller through `BuildContext`

## Tests

Add Rust tests under `crates/dust_route_plugin/tests/route_plugin_tests/`:

- parse router root annotation
- parse page route annotation
- parse route constructor params
- collect route facts from multiple Dart files
- deserialize workspace route facts
- validate duplicate route names
- validate duplicate normalized paths
- validate missing path constructor param
- validate unsupported path/query param type
- validate missing initial route
- validate missing not-found route
- generate stable imports for pages, shells, and guards
- snapshot generated `route.g.dart`
- validate generated code imports package runtime
- verify route-name-first navigation generation

Add driver-level tests only after plugin unit tests pass:

- route output is written only by the router root
- route output cache invalidates when an annotated page changes
- non-routing generators are not affected

## Implementation Order

1. Add crate skeleton and register it in the Rust workspace with `publish = false`.
2. Implement route models and JSON workspace-analysis records.
3. Implement annotation parsing for `@Router` and `@Route`.
4. Add validation with focused diagnostics.
5. Emit minimal `route.g.dart` for two simple routes.
6. Expand emission to params, shells, guards, transitions, and not-found routes.
7. Register the plugin in the driver.
8. Add driver cache/output tests.
9. Compare generated output against `examples/routing_prototype` and remove
    manual generated drift.
