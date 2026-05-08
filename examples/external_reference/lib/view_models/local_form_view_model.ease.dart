// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'local_form_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _LocalFormViewModelAspect<T> {
  final T Function(LocalFormState state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _LocalFormViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class LocalFormViewModelProvider extends StatefulWidget {
  final Widget child;
  const LocalFormViewModelProvider({super.key, required this.child});

  @override
  State<LocalFormViewModelProvider> createState() =>
      _LocalFormViewModelProviderState();
}

class _LocalFormViewModelProviderState
    extends State<LocalFormViewModelProvider> {
  late final LocalFormViewModel _notifier = LocalFormViewModel();

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
    return _LocalFormViewModelInherited(
        notifier: _notifier, child: widget.child);
  }
}

class _LocalFormViewModelInherited
    extends InheritedModel<_LocalFormViewModelAspect> {
  final LocalFormViewModel notifier;

  const _LocalFormViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_LocalFormViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _LocalFormViewModelInherited oldWidget,
    Set<_LocalFormViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension LocalFormViewModelContext on BuildContext {
  LocalFormViewModel get localFormViewModel {
    final inherited =
        InheritedModel.inheritFrom<_LocalFormViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No LocalFormViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added LocalFormViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  LocalFormViewModel readLocalFormViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_LocalFormViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No LocalFormViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added LocalFormViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectLocalFormViewModel<T>(
    T Function(LocalFormState state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_LocalFormViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No LocalFormViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added LocalFormViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_LocalFormViewModelInherited>(
      this,
      aspect: _LocalFormViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnLocalFormViewModel(
    void Function(LocalFormState previous, LocalFormState current) listener, {
    bool fireImmediately = false,
  }) {
    return readLocalFormViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
