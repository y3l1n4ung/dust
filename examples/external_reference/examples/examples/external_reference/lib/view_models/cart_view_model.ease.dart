// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'cart_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _CartViewModelAspect<T> {
  final T Function(CartStatus state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _CartViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class CartViewModelProvider extends StatefulWidget {
  final Widget child;
  const CartViewModelProvider({super.key, required this.child});

  @override
  State<CartViewModelProvider> createState() => _CartViewModelProviderState();
}

class _CartViewModelProviderState extends State<CartViewModelProvider> {
  late final CartViewModel _notifier = CartViewModel();

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
    return _CartViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _CartViewModelInherited extends InheritedModel<_CartViewModelAspect> {
  final CartViewModel notifier;

  const _CartViewModelInherited({required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_CartViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _CartViewModelInherited oldWidget,
    Set<_CartViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension CartViewModelContext on BuildContext {
  CartViewModel get cartViewModel {
    final inherited = InheritedModel.inheritFrom<_CartViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No CartViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added CartViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  CartViewModel readCartViewModel() {
    final inherited = getInheritedWidgetOfExactType<_CartViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No CartViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added CartViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectCartViewModel<T>(
    T Function(CartStatus state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited = getInheritedWidgetOfExactType<_CartViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No CartViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added CartViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_CartViewModelInherited>(
      this,
      aspect: _CartViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnCartViewModel(
    void Function(CartStatus previous, CartStatus current) listener, {
    bool fireImmediately = false,
  }) {
    return readCartViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
