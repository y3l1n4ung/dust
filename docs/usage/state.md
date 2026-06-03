# State Management

Dust provides state management generated from annotations. It uses explicit typed arguments for dependencies and provides convenient scoping and access via BuildContext.

---

## Installation

Add the state management package to your `pubspec.yaml`:

```yaml
dependencies:
  dust_flutter: ^0.1.0
```

---

## Basic Example

### 1. Define the ViewModel
Annotate your class with `@ViewModel` and extend the generated base class.

```dart
import 'package:dust_flutter/state.dart';

part 'counter_view_model.g.dart';

@ViewModel(state: CounterState)
class CounterViewModel extends $CounterViewModel {
  CounterViewModel(super.args);

  void increment() {
    // Access state via the 'state' property
    emit(state.copyWith(count: state.count + 1));
  }
}

class CounterState {
  const CounterState({this.count = 0});
  final int count;

  CounterState copyWith({int? count}) => CounterState(count: count ?? this.count);
}
```

### 2. Provide the Scope
```dart
CounterViewModelScope(
  args: (context) => const EmptyArgs(),
  create: (context, args) => CounterViewModel(args),
  child: const CounterPage(),
)
```

### 3. Consume in UI
```dart
class CounterPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    // Access state via .value
    final count = context.watchCounterViewModel().value.count;

    return Scaffold(
      body: Center(child: Text('Count: $count')),
      floatingActionButton: FloatingActionButton(
        onPressed: () => context.readCounterViewModel().increment(),
        child: const Icon(Icons.add),
      ),
    );
  }
}
```

---

## Configuration Reference

### `@ViewModel` Options

| Property | Type | Description |
| :--- | :--- | :--- |
| `state` | `Type` | **Required.** The class used for the ViewModel's state. |
| `args` | `Type` | Optional: A custom class (extending `ViewModelArgs`) for dependency injection. |
| `initial` | `Object` | Optional: The initial state source code. Defaults to `const State()`. |

---

## Dependency Injection (`ViewModelArgs`)

For complex dependencies (Repositories, Services), use a custom `args` class.

```dart
final class ProfileArgs extends ViewModelArgs {
  const ProfileArgs({required this.repository});
  final ProfileRepository repository;
}

@ViewModel(state: ProfileState, args: ProfileArgs)
class ProfileViewModel extends $ProfileViewModel {
  ProfileViewModel(super.args);

  // Access dependencies via the 'args' property
  void load() => args.repository.fetch();
}
```

---

## Key Concepts

### Side Effects (`ViewModelListener`)
Use `ViewModelListener` to handle one-time events like navigation or showing snackbars.

```dart
CounterViewModelListener(
  listener: (context, effect) {
    if (effect is ShowCelebration) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Yay!')));
    }
  },
  child: const CounterPage(),
)
```

### Context Extensions
Dust generates extensions on `BuildContext` for easy access:

*   `context.watchClassName()`: Rebuilds the widget when state changes. Returns a proxy to access `.value`.
*   `context.readClassName()`: Returns the ViewModel instance. Does not trigger rebuilds.
*   `context.className`: Shorthand for `context.watchClassName().value`.

---

## Migration Guide

**Coming from `Bloc` or `Provider`?**

| Feature | `flutter_bloc` | `provider` | Dust |
| :--- | :--- | :--- | :--- |
| Core Logic | `Bloc` / `Cubit` | `ChangeNotifier` | `ViewModel` |
| Scoping | `BlocProvider` | `Provider` | `ViewModelScope` |
| UI Binding | `BlocBuilder` | `context.watch` | `context.watchViewModel().value` |
| State Access | `state` | `this` | `state` |
| Dependencies | `repository` | `this.repo` | `args.repo` |

---

## Generation Output

Dust generates a base class (`$ClassName`) that handles the state stream and disposal.

```dart
// counter_view_model.g.dart (Simplified)
abstract class $CounterViewModel extends ViewModelBase<CounterState, EmptyArgs> {
  $CounterViewModel(super.args) : super(initialState: const CounterState());
}
```

---

## Best Practices

> [!WARNING]
> **Anti-Patterns to Avoid:**
> - **Don't** call async actions directly in `build()`.
> - **Don't** use `watch()` inside callbacks (use `read()` instead).
> - **Don't** mutate state lists or maps in place (always use `copyWith`).
