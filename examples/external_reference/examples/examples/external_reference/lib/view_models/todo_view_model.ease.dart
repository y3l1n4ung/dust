// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'todo_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _TodoViewModelAspect<T> {
  final T Function(List<Todo> state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _TodoViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class TodoViewModelProvider extends StatefulWidget {
  final Widget child;
  const TodoViewModelProvider({super.key, required this.child});

  @override
  State<TodoViewModelProvider> createState() => _TodoViewModelProviderState();
}

class _TodoViewModelProviderState extends State<TodoViewModelProvider> {
  late final TodoViewModel _notifier = TodoViewModel();

  @override
  void initState() {
    super.initState();
    _notifier.addListener(_onStateChange);
  }

  @override
  void dispose() {
    _notifier.removeListener(_onStateChange);
    _notifier.dispose();
    super.dispose();
  }

  void _onStateChange() => setState(() {});

  @override
  Widget build(BuildContext context) {
    return _TodoViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _TodoViewModelInherited extends InheritedModel<_TodoViewModelAspect> {
  final TodoViewModel notifier;

  const _TodoViewModelInherited({required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_TodoViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _TodoViewModelInherited oldWidget,
    Set<_TodoViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension TodoViewModelContext on BuildContext {
  TodoViewModel get todoViewModel {
    final inherited = InheritedModel.inheritFrom<_TodoViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No TodoViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added TodoViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  TodoViewModel readTodoViewModel() {
    final inherited = getInheritedWidgetOfExactType<_TodoViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No TodoViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added TodoViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectTodoViewModel<T>(
    T Function(List<Todo> state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited = getInheritedWidgetOfExactType<_TodoViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No TodoViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added TodoViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_TodoViewModelInherited>(
      this,
      aspect: _TodoViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnTodoViewModel(
    void Function(List<Todo> previous, List<Todo> current) listener, {
    bool fireImmediately = false,
  }) {
    return readTodoViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
