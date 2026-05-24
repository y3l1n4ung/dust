# State Management

Dust provides state management generated from annotations. It uses explicit typed arguments for dependencies and optimizes rebuilds using aspect-based tracking.

---

## Installation

Add the state management package to your `pubspec.yaml`:

```yaml
dependencies:
  dust_state: ^0.1.0
```

---

## Basic Example

### 1. Define the ViewModel
Annotate your class with `@ViewModel` and extend the generated base class.

```dart
import 'package:dust_state/dust_state.dart';

part 'counter_view_model.g.dart';

@ViewModel(state: CounterState)
class CounterViewModel extends $CounterViewModel {
  CounterViewModel(super.args);

  void increment() {
    emit(state.copyWith(count: state.count + 1));
  }
}

class CounterState {
  const CounterState({this.count = 0});
  final int count;
}
```

### 2. Provide the Scope
```dart
CounterViewModelScope(
  create: (context, args) => CounterViewModel(args),
  child: const CounterPage(),
)
```

### 3. Consume in UI
```dart
class CounterPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    // Rebuilds only when state.count changes
    final count = context.watchCounterViewModel().count;

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
| `initial` | `Object` | Optional: The initial state value. Defaults to `const State()`. |

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
  
  // Access dependencies via 'args' or generated getters
  void load() => args.repository.fetch();
}
```

---

## Key Concepts

### Aspect-Based Rebuilding
Dust uses `InheritedModel` for granular rebuilds. When you access a field via `context.watchViewModel().field`, the widget registers a dependency **only** on that specific property. 

> [!TIP]
> This automatic optimization removes the need for `Selector` or `buildWhen` logic found in other frameworks.

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

---

## Migration Guide

**Coming from `Bloc` or `Provider`?**

| Feature | `flutter_bloc` | `provider` | Dust |
| :--- | :--- | :--- | :--- |
| Core Logic | `Bloc` / `Cubit` | `ChangeNotifier` | `ViewModel` |
| Scoping | `BlocProvider` | `Provider` | `ViewModelScope` |
| UI Binding | `BlocBuilder` | `context.watch` | `context.watchViewModel()` |
| Rebuild Optimization | `buildWhen` | `Selector` | **Automatic (Aspect-based)** |

---

## Generation Output

Dust generates a base class (`$ClassName`) that handles the state stream and disposal.

```dart
// counter_view_model.g.dart (Simplified)
abstract class $CounterViewModel extends ViewModelBase<CounterState, EmptyArgs> {
  $CounterViewModel(super.args) : super(initialState: const CounterState());
}

extension CounterViewModelX on BuildContext {
  CounterViewModel watchCounterViewModel() => DustProvider.watch<CounterViewModel>(this);
  CounterViewModel readCounterViewModel() => DustProvider.read<CounterViewModel>(this);
}
```

---

## Best Practices

> [!WARNING]
> **Anti-Patterns to Avoid:**
> - **Don't** call async actions directly in `build()`.
> - **Don't** use `watch()` inside callbacks (use `read()` instead).
> - **Don't** mutate state lists or maps in place (always use `copyWith`).
