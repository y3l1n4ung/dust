# State Management Plan

## Goal

Generate Flutter-native, type-safe ViewModel code from Dart annotations without Riverpod, Bloc, Provider, or signals. Dust state uses normal Flutter primitives: `ValueNotifier`, generated scopes, `InheritedModel`, and `BuildContext` extensions.

## Package Shape

- Flutter package: `dust_flutter`
- Rust crate: `crates/dust_state_plugin`
- Prototype contract: `examples/state_management_prototype`

`dust_flutter` owns state annotations and the small runtime.

## Public API

```dart
final class TaskBoardArgs extends ViewModelArgs {
  const TaskBoardArgs({
    required this.repository,
    super.observer,
  });

  final PrototypeRepository repository;
}

@ViewModel(state: TaskBoardState, args: TaskBoardArgs)
final class TaskBoardViewModel extends $TaskBoardViewModel {
  TaskBoardViewModel(super.args);

  @override
  Future<void> onInit() async {
    await refresh(showLoading: true);
  }

  Future<void> refresh({bool showLoading = false}) async {
    if (showLoading) emit(state.copyWith(isLoading: true));
    final todos = await args.repository.fetchTodos(userId: 1, limit: 20);
    emit(state.copyWith(todos: todos, isLoading: false));
  }
}
```

Enum/imported states use explicit constant initial values:

```dart
@ViewModel(
  state: ShellTab,
  args: ShellViewModelArgs,
  initial: ShellTab.dashboard,
)
final class ShellViewModel extends $ShellViewModel {
  ShellViewModel(super.args);
}
```

Generated output owns:

- `$TaskBoardViewModel` typed base extending `ViewModelBase<TaskBoardState, TaskBoardArgs>`
- no generated dependency getters; app code reads dependencies through `args.repository`, `args.storage`, and other typed args fields
- `TaskBoardViewModelScope`
- `TaskBoardViewModelListener`
- smart proxy returned by `context.watchTaskBoardViewModel()`
- `context.readTaskBoardViewModel()` extension

## Runtime Contract

- `ViewModelBase` owns `state`, `emit`, `emitEffect`, idempotent `init`, observer hooks, and disposal safety.
- `ViewModelArgs` is the only dependency injection path; no string map injection.
- `watch` returns a smart proxy and registers only accessed aspects.
- `read` returns the raw ViewModel and never registers dependencies.
- listeners consume effect streams without rebuilding UI.
- scopes create, initialize, and dispose owned ViewModels exactly once.
- `.value` scopes provide externally owned ViewModels and never dispose them.

## Current Implementation Status

Implemented now:

- `packages/dust_flutter` state runtime and annotation package.
- manual prototype `.g.dart` files migrated to typed args + runtime base.
- `crates/dust_state_plugin` parses, validates, and emits base/scope/proxy/listener/context-extension output.
- generated aspect metadata is emitted from local and workspace state fields.
- driver test verifies state output emission.

Still open:
- golden snapshots for generated state files.
- driver check/watch/clean coverage for state outputs.
- stale async cancellation helpers for racing actions.

## Verified

- `flutter analyze packages/dust_flutter`
- `flutter test packages/dust_flutter`
- `flutter analyze` in `examples/state_management_prototype`
- `flutter test` in `examples/state_management_prototype`
- `flutter build web` in `examples/state_management_prototype`
- `cargo test -p dust_state_plugin`
- `cargo test -p dust_driver --test driver_tests state_outputs`
