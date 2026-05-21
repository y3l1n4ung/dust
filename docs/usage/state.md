# State Management Guide

Dust state lives in `dust_state`: Flutter-native state management generated from annotations. Dependencies are explicit typed args, not string maps.

## Args

```dart
final class TaskBoardArgs extends ViewModelArgs {
  const TaskBoardArgs({
    required this.repository,
    super.observer,
  });

  final PrototypeRepository repository;
}
```

`ViewModelArgs` carries shared runtime dependencies such as `observer`. App dependencies are normal final fields.

## ViewModel

```dart
@ViewModel(state: TaskBoardState, args: TaskBoardArgs)
final class TaskBoardViewModel extends $TaskBoardViewModel {
  TaskBoardViewModel(super.args);

  @override
  Future<void> onInit() async {
    await refresh(showLoading: true);
  }

  Future<void> refresh({bool showLoading = false}) async {
    if (showLoading) emit(state.copyWith(isLoading: true));
    final todos = await repository.fetchTodos(userId: 1, limit: 20);
    emit(state.copyWith(todos: todos, isLoading: false));
  }
}
```

Generated base:

```dart
abstract class $TaskBoardViewModel
    extends ViewModelBase<TaskBoardState, TaskBoardArgs> {
  $TaskBoardViewModel(super.args)
      : super(initialState: const TaskBoardState());

  PrototypeRepository get repository => args.repository;
}
```


## Initial State

For local state classes with a default `const State()` constructor, omit `initial`:

```dart
@ViewModel(state: TaskBoardState, args: TaskBoardArgs)
final class TaskBoardViewModel extends $TaskBoardViewModel {
  TaskBoardViewModel(super.args);
}
```

For enum, imported, or non-default states, pass an explicit constant initial value:

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

## Scope

```dart
TaskBoardViewModelScope(
  args: (context) => TaskBoardArgs(
    repository: context.repository,
    observer: context.stateObserver,
  ),
  create: (context, args) => TaskBoardViewModel(args),
  child: const TasksPage(),
)
```

Use `.value` when another owner controls disposal:

```dart
TaskBoardViewModelScope.value(
  value: existingViewModel,
  child: const TasksPage(),
)
```

## Watch

```dart
Widget build(BuildContext context) {
  final board = context.watchTaskBoardViewModel();
  return Text('${board.pendingCount} pending');
}
```

`watch` returns a generated smart proxy. Accessed fields register `InheritedModel` aspects, so unrelated state changes do not rebuild the widget.

## Read

```dart
FilledButton(
  onPressed: () => context.readTaskBoardViewModel().refresh(),
  child: const Text('Refresh'),
)
```

`read` never subscribes. Use it in callbacks and effects.

## Effects

```dart
TaskBoardViewModelListener(
  listener: (context, effect) {
    if (effect case TaskSaved(:final title)) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Saved $title')),
      );
    }
  },
  child: const TasksPage(),
)
```

ViewModels emit effects with `emitEffect(effect)`. Effects do not mutate state.

## Observer

```dart
RepositoryScope(
  repository: repository,
  observer: const LoggingStateObserver(),
  child: const PrototypeApp(),
)
```

Observers receive state transitions and emitted effects. Use `SilentStateObserver` in tests.

## Anti-Patterns

- Do not call async actions from `build`.
- Do not use `watch` in callbacks.
- Do not mutate state lists/maps in place.
- Do not put navigation/snackbars directly in `build`.
- Do not use broad `value` when a narrow proxy getter is enough.
- Do not pass dependencies through `Map<String, Type>` injection.

## Prototype

See `examples/state_management_prototype`. Files ending in `.g.dart` are manually written prototype output and define the full Rust generator target.
