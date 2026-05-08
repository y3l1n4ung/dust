// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'pagination_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _PaginationViewModelAspect<T> {
  final T Function(PaginationStatus state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _PaginationViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class PaginationViewModelProvider extends StatefulWidget {
  final Widget child;
  const PaginationViewModelProvider({super.key, required this.child});

  @override
  State<PaginationViewModelProvider> createState() =>
      _PaginationViewModelProviderState();
}

class _PaginationViewModelProviderState
    extends State<PaginationViewModelProvider> {
  late final PaginationViewModel _notifier = PaginationViewModel();

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
    return _PaginationViewModelInherited(
        notifier: _notifier, child: widget.child);
  }
}

class _PaginationViewModelInherited
    extends InheritedModel<_PaginationViewModelAspect> {
  final PaginationViewModel notifier;

  const _PaginationViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_PaginationViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _PaginationViewModelInherited oldWidget,
    Set<_PaginationViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension PaginationViewModelContext on BuildContext {
  PaginationViewModel get paginationViewModel {
    final inherited =
        InheritedModel.inheritFrom<_PaginationViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No PaginationViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added PaginationViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  PaginationViewModel readPaginationViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_PaginationViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No PaginationViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added PaginationViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectPaginationViewModel<T>(
    T Function(PaginationStatus state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_PaginationViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No PaginationViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added PaginationViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_PaginationViewModelInherited>(
      this,
      aspect: _PaginationViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnPaginationViewModel(
    void Function(PaginationStatus previous, PaginationStatus current)
        listener, {
    bool fireImmediately = false,
  }) {
    return readPaginationViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
