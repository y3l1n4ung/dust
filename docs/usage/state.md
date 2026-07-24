# State Management

Dust generates typed Flutter ViewModel scopes, selectors, listeners, and async
state helpers from `@ViewModel`.

## Add the Packages

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Flutter runtime and Dart derives used by the examples:

```bash
flutter pub add dust_flutter dust_dart
```

## Quick Start

Define an immutable state and a ViewModel that extends its generated base:

```dart
import 'package:dust_dart/derive.dart';
import 'package:dust_flutter/state.dart';

part 'counter_view_model.g.dart';

@Derive([Eq(), CopyWith()])
class CounterState with _$CounterState {
  const CounterState({this.count = 0});

  final int count;
}

@ViewModel(state: CounterState)
class CounterViewModel extends $CounterViewModel {
  CounterViewModel(super.args);

  void increment() => emit(state.copyWith(count: state.count + 1));
}
```

Generate the typed Flutter surface:

```bash
dust build
```

Provide the ViewModel above the widgets that use it:

```dart
CounterViewModelScope(
  args: (_) => const ViewModelArgs(),
  create: (_, args) => CounterViewModel(args),
  child: const CounterPage(),
)
```

Read state during `build` and call commands without subscribing:

```dart
class CounterPage extends StatelessWidget {
  const CounterPage({super.key});

  @override
  Widget build(BuildContext context) {
    final count = context.watchCounterViewModel().value.count;

    return TextButton(
      onPressed: context.readCounterViewModel().increment,
      child: Text('Count: $count'),
    );
  }
}
```

> [!IMPORTANT]
> Use `watchXViewModel().value` only where UI should rebuild. Use
> `readXViewModel()` in callbacks, lifecycle methods, and dependency factories.

## State and Dependencies

Keep UI-changing values in the state type. Put repositories, clients, storage,
and observers in a typed `ViewModelArgs` subclass:

```dart
final class ProfileViewModelArgs extends ViewModelArgs {
  const ProfileViewModelArgs({required this.repository, super.observer});

  final ProfileRepository repository;
}

@ViewModel(state: ProfileState, args: ProfileViewModelArgs)
class ProfileViewModel extends $ProfileViewModel {
  ProfileViewModel(super.args);

  Future<void> save() => args.repository.save(state.profile);
}
```

For synchronous state, Dust uses `const StateType()` as the initial value when
possible. Supply `initial:` for enums, imported state values, or constructors
that need arguments:

```dart
@ViewModel(state: AppTab, initial: AppTab.home)
class AppViewModel extends $AppViewModel {
  AppViewModel(super.args);
}
```

## Scope Lifecycle

The default generated scope owns the ViewModel, calls `init()`, and disposes it.
Override `onInit()` for one-time synchronous or asynchronous setup:

```dart
@override
Future<void> onInit() => loadProfile();
```

Use `identity` when a dependency change must replace the owned ViewModel:

```dart
ProfileViewModelScope(
  identity: (_) => repository,
  args: (_) => ProfileViewModelArgs(repository: repository),
  create: (_, args) => ProfileViewModel(args),
  child: const ProfilePage(),
)
```

> [!NOTE]
> Rebuilding a scope with a new `args` closure does not by itself replace its
> ViewModel. Change `identity` or the widget key when dependencies change.

Use `.value` for an externally owned ViewModel. The scope initializes and
listens to it but does not dispose it:

```dart
ProfileViewModelScope.value(
  value: profileViewModel,
  child: const ProfilePage(),
)
```

Group scopes without manual nesting:

```dart
ViewModelScopes(
  scopes: [
    (child) => AppViewModelScope(
      args: (_) => AppViewModelArgs(repository: repository),
      create: (_, args) => AppViewModel(args),
      child: child,
    ),
    (child) => ProfileViewModelScope(
      args: (context) => ProfileViewModelArgs(
        repository: context.readAppViewModel().args.repository,
      ),
      create: (_, args) => ProfileViewModel(args),
      child: child,
    ),
  ],
  child: const App(),
)
```

Scopes are nested in list order; the first entry is outermost.

## Selectors

Watching `.value` rebuilds for every state change. Use the generated selector
when a widget depends on one value:

```dart
ProfileViewModelSelector<ProfileStatus>(
  selector: (state) => state.status,
  builder: (context, status, child) {
    return ProfileStatusBadge(status: status);
  },
)
```

Selectors compare with `==` by default. Pass `equals` for custom comparison and
`child` for a subtree that should not be rebuilt.

## Effects

Use effects for one-shot UI work such as snackbars or navigation:

```dart
void saveCompleted() => emitEffect(const ProfileSaved());
```

Listen with the generated widget:

```dart
ProfileViewModelListener(
  listener: (context, effect) {
    if (effect is ProfileSaved) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Saved')),
      );
    }
  },
  child: const ProfilePage(),
)
```

Effects do not change state, and the listener does not rebuild its child when
an effect arrives.

## Async ViewModels

Async mode treats the annotated `state` type as loaded data and wraps it in
`AsyncState<T>`:

```dart
@ViewModel(
  state: HomePageData,
  args: HomeViewModelArgs,
  mode: ViewModelMode.async,
)
class HomeViewModel extends $HomeViewModel {
  HomeViewModel(super.args);

  @override
  Future<HomePageData> loadData() => args.repository.loadHomePage();
}
```

The scope automatically calls `load()` through `onInit()`.

| API | Behavior |
| :--- | :--- |
| `load()` | Loads fresh data without preserving visible data. |
| `refresh()` | Reloads while preserving visible data when available. |
| `retry()` | Calls `refresh()`. |
| `data` | Returns currently visible data when available. |
| `visibleData` | Returns current or preserved previous data. |
| `invalidateSelf()` | Cancels stale work, clears visible data, and starts a fresh `load()`. |

The generated builder handles the common lifecycle:

```dart
HomeViewModelBuilder(
  loading: (context) => const CircularProgressIndicator(),
  data: (context, data) => HomeContent(data: data),
  error: (context, error, previousData) => HomeErrorView(
    error: error,
    previousData: previousData,
    onRetry: context.readHomeViewModel().retry,
  ),
)
```

During refresh, the builder continues to use the `data` callback with the
preserved value. Read `AsyncState<T>` directly when the UI must distinguish
`AsyncInitial`, `AsyncLoading`, `AsyncData`, and `AsyncFailure`.

## Stale Async Actions

For asynchronous commands on a synchronous ViewModel, action tokens prevent an
older request from overwriting newer state:

```dart
static const _loadProducts = 'load-products';

Future<void> loadProducts() async {
  final token = beginAction(_loadProducts);
  final products = await args.repository.loadProducts();
  if (!isCurrentAction(token)) return;
  emit(state.copyWith(products: products));
}
```

Starting the same action key supersedes its previous token. Use
`cancelAction(key)` to invalidate one action. `invalidateSelf()` invalidates all
pending action tokens. Sync ViewModels restore the generated initial state. Async
ViewModels start a fresh load without preserving previous data.

## Configuration

| Option | Behavior |
| :--- | :--- |
| `state` | Required sync state type or async loaded-data type. |
| `args` | Optional `ViewModelArgs` subtype; defaults to `ViewModelArgs`. |
| `initial` | Optional sync initial expression; not allowed in async mode. |
| `mode` | `ViewModelMode.sync` or `ViewModelMode.async`. |

> [!TIP]
> Generated watch proxies expose only `.value`. Read dependencies through
> `viewModel.args` and state through `state` or the generated watch/selector
> APIs; Dust does not generate mirror getters for either.

## Examples

- [Flutter package example](../../packages/dust_flutter/example/dust_flutter_example.dart)
- [Shopping app ViewModels](../../examples/shopping_app/lib/core/view_models/app_view_model.dart)
- [Shopping app scope composition](../../examples/shopping_app/lib/main.dart)
- [Async action tokens](../../examples/shopping_app/lib/features/products/view_models/products_view_model.dart)
- [Selector and listener tests](../../examples/shopping_app/test/state_selector_test.dart)
- [Scope lifecycle tests](../../examples/shopping_app/test/state_scope_realworld_lifecycle_test.dart)
