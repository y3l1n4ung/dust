// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'drawing_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _DrawingViewModelAspect<T> {
  final T Function(DrawingState state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _DrawingViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class DrawingViewModelProvider extends StatefulWidget {
  final Widget child;
  const DrawingViewModelProvider({super.key, required this.child});

  @override
  State<DrawingViewModelProvider> createState() =>
      _DrawingViewModelProviderState();
}

class _DrawingViewModelProviderState extends State<DrawingViewModelProvider> {
  late final DrawingViewModel _notifier = DrawingViewModel();

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
    return _DrawingViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _DrawingViewModelInherited
    extends InheritedModel<_DrawingViewModelAspect> {
  final DrawingViewModel notifier;

  const _DrawingViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_DrawingViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _DrawingViewModelInherited oldWidget,
    Set<_DrawingViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension DrawingViewModelContext on BuildContext {
  DrawingViewModel get drawingViewModel {
    final inherited =
        InheritedModel.inheritFrom<_DrawingViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No DrawingViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added DrawingViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  DrawingViewModel readDrawingViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_DrawingViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No DrawingViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added DrawingViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectDrawingViewModel<T>(
    T Function(DrawingState state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_DrawingViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No DrawingViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added DrawingViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_DrawingViewModelInherited>(
      this,
      aspect: _DrawingViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnDrawingViewModel(
    void Function(DrawingState previous, DrawingState current) listener, {
    bool fireImmediately = false,
  }) {
    return readDrawingViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
