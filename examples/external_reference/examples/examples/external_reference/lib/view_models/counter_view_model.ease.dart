// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'counter_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _CounterViewModelAspect<T> {
  final T Function(int state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _CounterViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class CounterViewModelProvider extends StatefulWidget {
  final Widget child;
  const CounterViewModelProvider({super.key, required this.child});

  @override
  State<CounterViewModelProvider> createState() =>
      _CounterViewModelProviderState();
}

class _CounterViewModelProviderState extends State<CounterViewModelProvider> {
  late final CounterViewModel _notifier = CounterViewModel();

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
    return _CounterViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _CounterViewModelInherited
    extends InheritedModel<_CounterViewModelAspect> {
  final CounterViewModel notifier;

  const _CounterViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_CounterViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _CounterViewModelInherited oldWidget,
    Set<_CounterViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension CounterViewModelContext on BuildContext {
  CounterViewModel get counterViewModel {
    final inherited =
        InheritedModel.inheritFrom<_CounterViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No CounterViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added CounterViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  CounterViewModel readCounterViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_CounterViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No CounterViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added CounterViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectCounterViewModel<T>(
    T Function(int state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_CounterViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No CounterViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added CounterViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_CounterViewModelInherited>(
      this,
      aspect: _CounterViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnCounterViewModel(
    void Function(int previous, int current) listener, {
    bool fireImmediately = false,
  }) {
    return readCounterViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
