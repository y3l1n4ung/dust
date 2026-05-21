# State Management Prototype

This prototype is the generator contract for Dust state management. App files are normal Flutter code; `*.g.dart` files are manually written output that `crates/dust_state_plugin` must generate later.

## Contract

- Package target: `dust_state`
- Rust target: `crates/dust_state_plugin`
- Runtime primitives: `ValueNotifier`, `InheritedModel`, `BuildContext` extensions
- No Riverpod, Bloc, Provider, or signals dependency

## Generated Shape

For each `@ViewModel(state: StateType, args: ArgsType)`, generation must emit:

- user-owned `XxxViewModelArgs extends ViewModelArgs` support
- `$XxxViewModel` hidden base class
- `XxxViewModelScope`
- smart proxy returned by `context.watchXxxViewModel()`
- `context.readXxxViewModel()`
- `XxxViewModelListener`

## Behavior To Preserve

- `watch` subscribes by accessed aspect only.
- `read` never subscribes.
- `listener` handles side effects without rebuilding.
- scope resolves dependencies, creates once, initializes once, and disposes once.
- observer sees state changes and effects.
- async init is scheduled outside build.

## Verification

```bash
flutter analyze
flutter test
flutter build web
```

## Current Manual Fixtures

- `features/tasks/view_models/task_board_view_model.g.dart`
- `features/session/view_models/session_view_model.g.dart`
- `features/theme/view_models/theme_view_model.g.dart`
- `features/shell/view_models/shell_view_model.g.dart`
- `features/posts/view_models/post_detail_view_model.g.dart`

These files should become golden sources for Rust generator tests.
