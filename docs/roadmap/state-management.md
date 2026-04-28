# State Management Plan

## Goal

Generate lightweight, type-safe state management code from Dart annotations
without locking users into one ecosystem first.

## Package Shape

- Dart annotation package: `state_annotation`
- Rust plugin crate: `dust_plugin_state`
- Optional adapters: Flutter `ChangeNotifier`, Riverpod, Bloc.

## API Sketch

```dart
@StateStore()
class CounterState with _$CounterStateDust {
  final int value;
  final bool loading;

  const CounterState({required this.value, this.loading = false});
}

@StateActions(forState: CounterState)
abstract interface class CounterActions {
  CounterState increment(CounterState state);
  Future<CounterState> load(CounterState state);
}
```

Generated output:

```dart
final store = _$CounterStore(initialState: const CounterState(value: 0));
store.dispatch(const Increment());
```

## Generator Work

- Reuse `Eq()` and `CopyWith()` for immutable state models.
- Generate action classes from methods.
- Generate reducer dispatch with typed events.
- Generate async effect handling with explicit loading/error states.
- Keep core runtime plain Dart.
- Add optional Flutter adapters after core is stable.

## Tests

- Golden tests for store, actions, reducers, async effects.
- Dart runtime tests for dispatch order and error propagation.
- Flutter adapter tests only in Flutter example package.

## Done

- Core state store works without Flutter.
- Flutter adapter is thin and optional.
- State transitions are fully typed and analyzer-clean.
