// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'side_effect_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _SideEffectViewModelAspect<T> {
  final T Function(SideEffectState state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _SideEffectViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class SideEffectViewModelProvider extends StatefulWidget {
  final Widget child;
  const SideEffectViewModelProvider({super.key, required this.child});

  @override
  State<SideEffectViewModelProvider> createState() =>
      _SideEffectViewModelProviderState();
}

class _SideEffectViewModelProviderState
    extends State<SideEffectViewModelProvider> {
  late final SideEffectViewModel _notifier = SideEffectViewModel();

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
    return _SideEffectViewModelInherited(
        notifier: _notifier, child: widget.child);
  }
}

class _SideEffectViewModelInherited
    extends InheritedModel<_SideEffectViewModelAspect> {
  final SideEffectViewModel notifier;

  const _SideEffectViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_SideEffectViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _SideEffectViewModelInherited oldWidget,
    Set<_SideEffectViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension SideEffectViewModelContext on BuildContext {
  SideEffectViewModel get sideEffectViewModel {
    final inherited =
        InheritedModel.inheritFrom<_SideEffectViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No SideEffectViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added SideEffectViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  SideEffectViewModel readSideEffectViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_SideEffectViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No SideEffectViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added SideEffectViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectSideEffectViewModel<T>(
    T Function(SideEffectState state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_SideEffectViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No SideEffectViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added SideEffectViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_SideEffectViewModelInherited>(
      this,
      aspect: _SideEffectViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnSideEffectViewModel(
    void Function(SideEffectState previous, SideEffectState current) listener, {
    bool fireImmediately = false,
  }) {
    return readSideEffectViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
