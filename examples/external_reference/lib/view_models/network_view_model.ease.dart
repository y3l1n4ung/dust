// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'network_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _NetworkViewModelAspect<T> {
  final T Function(NetworkStatus state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _NetworkViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class NetworkViewModelProvider extends StatefulWidget {
  final Widget child;
  const NetworkViewModelProvider({super.key, required this.child});

  @override
  State<NetworkViewModelProvider> createState() =>
      _NetworkViewModelProviderState();
}

class _NetworkViewModelProviderState extends State<NetworkViewModelProvider> {
  late final NetworkViewModel _notifier = NetworkViewModel();

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
    return _NetworkViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _NetworkViewModelInherited
    extends InheritedModel<_NetworkViewModelAspect> {
  final NetworkViewModel notifier;

  const _NetworkViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_NetworkViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _NetworkViewModelInherited oldWidget,
    Set<_NetworkViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension NetworkViewModelContext on BuildContext {
  NetworkViewModel get networkViewModel {
    final inherited =
        InheritedModel.inheritFrom<_NetworkViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No NetworkViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added NetworkViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  NetworkViewModel readNetworkViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_NetworkViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No NetworkViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added NetworkViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectNetworkViewModel<T>(
    T Function(NetworkStatus state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_NetworkViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No NetworkViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added NetworkViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_NetworkViewModelInherited>(
      this,
      aspect: _NetworkViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnNetworkViewModel(
    void Function(NetworkStatus previous, NetworkStatus current) listener, {
    bool fireImmediately = false,
  }) {
    return readNetworkViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
