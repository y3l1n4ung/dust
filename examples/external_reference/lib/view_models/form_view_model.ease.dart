// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'form_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _RegistrationFormViewModelAspect<T> {
  final T Function(RegistrationForm state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _RegistrationFormViewModelAspect(this.selector, this.value,
      [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class RegistrationFormViewModelProvider extends StatefulWidget {
  final Widget child;
  const RegistrationFormViewModelProvider({super.key, required this.child});

  @override
  State<RegistrationFormViewModelProvider> createState() =>
      _RegistrationFormViewModelProviderState();
}

class _RegistrationFormViewModelProviderState
    extends State<RegistrationFormViewModelProvider> {
  late final RegistrationFormViewModel _notifier = RegistrationFormViewModel();

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
    return _RegistrationFormViewModelInherited(
        notifier: _notifier, child: widget.child);
  }
}

class _RegistrationFormViewModelInherited
    extends InheritedModel<_RegistrationFormViewModelAspect> {
  final RegistrationFormViewModel notifier;

  const _RegistrationFormViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_RegistrationFormViewModelInherited oldWidget) =>
      true;

  @override
  bool updateShouldNotifyDependent(
    _RegistrationFormViewModelInherited oldWidget,
    Set<_RegistrationFormViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension RegistrationFormViewModelContext on BuildContext {
  RegistrationFormViewModel get registrationFormViewModel {
    final inherited =
        InheritedModel.inheritFrom<_RegistrationFormViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No RegistrationFormViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added RegistrationFormViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  RegistrationFormViewModel readRegistrationFormViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_RegistrationFormViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No RegistrationFormViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added RegistrationFormViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectRegistrationFormViewModel<T>(
    T Function(RegistrationForm state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_RegistrationFormViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No RegistrationFormViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added RegistrationFormViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_RegistrationFormViewModelInherited>(
      this,
      aspect:
          _RegistrationFormViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnRegistrationFormViewModel(
    void Function(RegistrationForm previous, RegistrationForm current)
        listener, {
    bool fireImmediately = false,
  }) {
    return readRegistrationFormViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
