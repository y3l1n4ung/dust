# Changelog

All notable changes to `dust_flutter` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.4] - 2026-08-09

### Added

- Add `RouterBase.observers` for forwarding `NavigatorObserver` instances to
  generated routers.
- Add `RouterBase.onException` for unawaited generated navigation failures.

### Fixed

- Re-run redirects and guards when pop or platform page removal exposes a
  previous route.
- Avoid subscribing imperative router controller lookups to route changes.

## [0.1.3] - 2026-07-28

### Added

- Add router stack observer runtime support for generated routing.
- Add typed route results with `@AppRoute(result: Type)` and generated
  `Future<T?> push()` helpers.
- Add `runAction` for stale-safe async commands in sync ViewModels.

### Changed

- Make failed ViewModel initialization retry only through explicit `retryInit()`.
- Keep v0.1.3 routing, state, and i18n runtime APIs aligned with the Dust CLI.

### Fixed

- Unwrap deprecated `StateEffect` values before effect delivery.

## [0.1.2] - 2026-07-10

### Fixed

- Make generated route `push()` calls awaitable and complete them when the
  pushed route is popped, including optional pop results.
- Apply generated route `transition:` annotations at the page route boundary so
  custom and no-transition builders control the actual navigation animation.

## [0.1.1] - 2026-07-09

### Added

- Add opt-in `AppRouter` debug diagnostics for generated routing.
- Print generated route tables, named route paths, redirects, guards,
  navigation actions, and stack changes when diagnostics are enabled.

## [0.1.0] - 2026-05-08

### Added

- Initial public release of Dust Flutter annotations and runtime APIs.
- Includes Navigator 2.0 route annotations/runtime and ViewModel state
  annotations/runtime for generated Dust Flutter code.
- Includes the Flutter i18n runtime for Dust-generated localization workflows.
