# State Management

Dust generates typed Flutter ViewModel glue from `@ViewModel`.
Dependencies go through `args`; UI state goes through `state`, `value`, or
generated selector widgets.

## Installation

```yaml
dependencies:
  dust_dart: ^0.1.0
  dust_flutter: ^0.1.0
```

## Sync ViewModel

```dart
import 'package:dust_dart/derive.dart';
import 'package:dust_flutter/state.dart';

part 'counter_view_model.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class CounterState with _$CounterState {
  const CounterState({this.count = 0});

  final int count;
}

@ViewModel(state: CounterState)
class CounterViewModel extends $CounterViewModel {
  CounterViewModel(super.args);

  void increment() {
    emit(state.copyWith(count: state.count + 1));
  }

  void reset() {
    invalidateSelf();
  }
}
```

Provide the generated scope:

```dart
CounterViewModelScope(
  args: (_) => const ViewModelArgs(),
  create: (_, args) => CounterViewModel(args),
  child: const CounterPage(),
)
```

Group multiple generated scopes with `ViewModelScopes`:

```dart
ViewModelScopes(
  scopes: [
    (child) => CounterViewModelScope(
      args: (_) => const ViewModelArgs(),
      create: (_, args) => CounterViewModel(args),
      child: child,
    ),
    (child) => ProfileViewModelScope(
      args: (_) => ProfileViewModelArgs(repository: repository),
      create: (_, args) => ProfileViewModel(args),
      child: child,
    ),
  ],
  child: const App(),
)
```

Scopes are nested in list order. The first scope is the outermost scope.

Use the generated context helpers:

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

## Args

Use `ViewModelArgs` for repositories, services, HTTP clients, sockets, storage,
and observers. Do not generate mirror getters for dependencies.

```dart
final class ProfileArgs extends ViewModelArgs {
  const ProfileArgs({required this.repository, super.observer});

  final ProfileRepository repository;
}

@ViewModel(state: ProfileState, args: ProfileArgs)
class ProfileViewModel extends $ProfileViewModel {
  ProfileViewModel(super.args);

  Future<void> save() {
    return args.repository.save(state.profile);
  }
}
```

## Selectors

`context.watchProfileViewModel().value` rebuilds for the whole state. For
fine-grained rebuilds, use the generated selector widget.

```dart
ProfileViewModelSelector<ProfileStatus>(
  selector: (state) => state.status,
  builder: (context, status, child) {
    return ProfileStatusBadge(status: status);
  },
)
```

Selector widgets are covered by rebuild-count tests in the shopping app: they
do not rebuild when unrelated state fields change.

## Effects

Use the generated listener for one-shot effects. Listeners do not rebuild their
child when effects arrive.

```dart
ProfileViewModelListener(
  listener: (context, effect) {
    if (effect is ShowProfileSaved) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Saved')),
      );
    }
  },
  child: const ProfilePage(),
)
```

Emit effects from the ViewModel:

```dart
void notifySaved() {
  emitEffect(const ShowProfileSaved());
}
```

## Async ViewModel

Async ViewModels use the same annotation with `mode: ViewModelMode.async`.
The annotated `state` type is the loaded data type; the generated base wraps
it in `AsyncState<T>`.

```dart
@Derive([ToString(), Eq()])
class HomePageData with _$HomePageData {
  const HomePageData({
    required this.featuredProducts,
    required this.categories,
  });

  final List<Product> featuredProducts;
  final List<String> categories;
}

final class HomeViewModelArgs extends ViewModelArgs {
  const HomeViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(
  state: HomePageData,
  args: HomeViewModelArgs,
  mode: ViewModelMode.async,
)
class HomeViewModel extends $HomeViewModel {
  HomeViewModel(super.args);

  @override
  Future<HomePageData> loadData() async {
    final products = await args.repository.getProductsPage(limit: 6);
    final categories = await args.repository.getCategories();
    return HomePageData(
      featuredProducts: products,
      categories: categories,
    );
  }
}
```

Async ViewModels get:

- `load()`: clears visible data and loads fresh data.
- `refresh()`: loads fresh data while preserving previous visible data.
- `retry()`: aliases `refresh()`.
- `invalidateSelf()`: clears pending work and resets to `AsyncInitial<T>`.
- `data`: current visible data, if present.
- `visibleData`: current or previous data, if present.

Use the generated async builder for the common loading/data/error UI:

```dart
HomeViewModelBuilder(
  loading: (_) => const CircularProgressIndicator(),
  data: (context, data) => HomeContent(data: data),
  error: (context, error, previousData) {
    return HomeErrorView(
      error: error,
      previousData: previousData,
      onRetry: context.readHomeViewModel().retry,
    );
  },
)
```

Use a raw switch only when the UI needs every lifecycle detail:

```dart
return switch (context.watchHomeViewModel().value) {
  AsyncData<HomePageData>(data: final data) => HomeContent(data: data),
  AsyncLoading<HomePageData>(
    hasPreviousData: true,
    previousData: final previousData,
  ) =>
    HomeContent(data: previousData as HomePageData, refreshing: true),
  AsyncFailure<HomePageData>(
    error: final error,
    previousData: final previousData,
  ) =>
    HomeErrorView(error: error, previousData: previousData),
  _ => const CircularProgressIndicator(),
};
```

## Generated API

Dust generates one support block per ViewModel:

- `$CounterViewModel` / `$HomeViewModel` base class
- `ViewModelScopes`
- `CounterViewModelScope` / `HomeViewModelScope`
- `context.watchCounterViewModel().value`
- `context.readCounterViewModel()`
- `CounterViewModelSelector<R>`
- `CounterViewModelListener`
- async-only `HomeViewModelBuilder`

Generated proxies expose `.value` only. They do not mirror `state.*`,
`args.*`, or closure-backed `watch().select(...)`.

## Configuration Reference

| Property | Type | Description |
| :--- | :--- | :--- |
| `state` | `Type` | Required. Sync state type or async loaded data type. |
| `args` | `Type` | Optional `ViewModelArgs` subtype. Defaults to `ViewModelArgs`. |
| `initial` | Expression | Optional sync initial state. Not allowed with async mode. |
| `mode` | `ViewModelMode` | `ViewModelMode.sync` or `ViewModelMode.async`. Defaults to sync. |

## Rules

> [!WARNING]
> - Do not mutate state collections in place. Use immutable snapshots and `copyWith`.
> - Do not call async actions directly from `build()`.
> - Do not use `watch` inside callbacks; use `read`.
> - Put repositories, HTTP clients, sockets, and storage in `args`.
> - Put UI-changing data in state or loaded async data.
