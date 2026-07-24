## 0.1.3

- Add router stack observer runtime support for generated routing.
- Unwrap deprecated `StateEffect` values before effect delivery.
- Keep v0.1.3 routing, state, and i18n runtime APIs aligned with the Dust CLI.

## 0.1.2

- Make generated route `push()` calls awaitable and complete them when the
  pushed route is popped, including optional pop results.
- Apply generated route `transition:` annotations at the page route boundary so
  custom and no-transition builders control the actual navigation animation.

## 0.1.1

- Add opt-in `AppRouter` debug diagnostics for generated routing.
- Print generated route tables, named route paths, redirects, guards,
  navigation actions, and stack changes when diagnostics are enabled.

## 0.1.0

- Initial public release of Dust Flutter annotations and runtime APIs.
- Includes Navigator 2.0 route annotations/runtime and ViewModel state
  annotations/runtime for generated Dust Flutter code.
- Includes the Flutter i18n runtime for Dust-generated localization workflows.
