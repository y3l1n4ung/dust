# State Management Hardening Notes

This page records the current Dust state-management design and the hardening
rules that protect it. User-facing usage lives in
[usage/state.md](./usage/state.md).

## Current Shape

Dust has one annotation:

```dart
@ViewModel(state: CounterState)
@ViewModel(state: HomePageData, mode: ViewModelMode.async)
```

Sync ViewModels extend generated `ViewModelBase<State, Args>` support. Async
ViewModels extend generated `AsyncViewModelBase<Data, Args>` support and expose
`AsyncState<Data>` to the widget tree.

## Design Rules

- Dependencies go through typed `args`.
- UI state goes through `state`, `value`, or generated selector widgets.
- Generated proxies expose `.value` only.
- Do not generate mirror getters for `state.*` or `args.*`.
- Do not expose closure-backed `watch().select(...)`.
- Use generated selector widgets for selected-value rebuilds.
- Use generated listeners for one-shot effects; listener children must not
  rebuild for effects.
- Use scope `identity` when an owned ViewModel must be recreated for route id,
  user id, tenant, repository, or workspace changes.
- `invalidateSelf()` resets sync state to the generated initial state and async
  state to `AsyncInitial<T>`.

## Async Contract

Async ViewModels implement one method:

```dart
@override
Future<HomePageData> loadData();
```

The generated/runtime API owns lifecycle state:

- `load()` starts fresh loading without preserving visible data.
- `refresh()` preserves previous visible data while loading.
- `retry()` aliases `refresh()`.
- `invalidateSelf()` clears pending work and visible data.
- stale async results are ignored by action tokens.
- nullable loaded data is tracked with `hasData` and `hasPreviousData`, not by
  testing `data != null`.

The public lifecycle states are:

```dart
AsyncInitial<T>
AsyncLoading<T>
AsyncData<T>
AsyncFailure<T>
```

`AsyncLoading<T>` and `AsyncFailure<T>` can carry previous data. There is no
separate public `refreshing` state class; refresh is `AsyncLoading` with
`hasPreviousData == true`.

## Generated UI Helpers

Every ViewModel gets:

- generated base class
- generated scope
- generated `watch...().value` and `read...()` context helpers
- generated selector widget
- generated listener widget

Async ViewModels also get a generated builder:

```dart
HomeViewModelBuilder(
  loading: (_) => const CircularProgressIndicator(),
  data: (context, data) => HomeContent(data: data),
  error: (context, error, previousData) => HomeErrorView(
    error: error,
    previousData: previousData,
  ),
)
```

Raw `switch` on `AsyncState<T>` stays available for UIs that need exact
lifecycle control.

## Fixtures And Gates

The state-management hardening stack is protected by:

- runtime async tests in `packages/dust_flutter/test/async_state_test.dart`
- sync invalidation tests in `packages/dust_flutter/test/view_model_test.dart`
- generated selector/listener widget tests in
  `examples/shopping_app/test/state_selector_test.dart`
- generated scope lifecycle tests in
  `examples/shopping_app/test/state_scope_lifecycle_test.dart`
- generated async builder widget tests in
  `examples/shopping_app/test/app_view_model_test.dart`
- exact generated async output snapshot in
  `crates/dust_driver/tests/driver_tests/state_outputs/snapshots/async_profile_view_model.dart.snapshot`
- driver output tests in
  `crates/dust_driver/tests/driver_tests/state_outputs.rs`
- plugin emission and validation tests in `crates/dust_state_plugin/tests`

Minimum local validation for state changes:

```sh
cargo fmt --all -- --check
cargo test -p dust_state_plugin --test state_plugin_tests
cargo test -p dust_driver --test driver_tests state_outputs::build_writes_async_state_output_for_view_model_library -- --exact
(cd packages/dust_flutter && flutter test test/async_state_test.dart test/view_model_test.dart)
(cd examples/shopping_app && flutter analyze --no-pub)
(cd examples/shopping_app && flutter test test/state_selector_test.dart test/state_scope_lifecycle_test.dart test/app_view_model_test.dart)
target/debug/dust check --root examples/shopping_app --fail-fast
target/debug/dust check --root examples/shopping_app --db --fail-fast
```
