// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'auth_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _AuthViewModelAspect<T> {
  final T Function(AuthStatus state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _AuthViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class AuthViewModelProvider extends StatefulWidget {
  final Widget child;
  const AuthViewModelProvider({super.key, required this.child});

  @override
  State<AuthViewModelProvider> createState() => _AuthViewModelProviderState();
}

class _AuthViewModelProviderState extends State<AuthViewModelProvider> {
  late final AuthViewModel _notifier = AuthViewModel();

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
    return _AuthViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _AuthViewModelInherited extends InheritedModel<_AuthViewModelAspect> {
  final AuthViewModel notifier;

  const _AuthViewModelInherited({required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_AuthViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _AuthViewModelInherited oldWidget,
    Set<_AuthViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension AuthViewModelContext on BuildContext {
  AuthViewModel get authViewModel {
    final inherited = InheritedModel.inheritFrom<_AuthViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No AuthViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added AuthViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  AuthViewModel readAuthViewModel() {
    final inherited = getInheritedWidgetOfExactType<_AuthViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No AuthViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added AuthViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectAuthViewModel<T>(
    T Function(AuthStatus state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited = getInheritedWidgetOfExactType<_AuthViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No AuthViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added AuthViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_AuthViewModelInherited>(
      this,
      aspect: _AuthViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnAuthViewModel(
    void Function(AuthStatus previous, AuthStatus current) listener, {
    bool fireImmediately = false,
  }) {
    return readAuthViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
