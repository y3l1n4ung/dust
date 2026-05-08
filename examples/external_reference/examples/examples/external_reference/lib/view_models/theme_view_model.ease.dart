// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'theme_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _ThemeViewModelAspect<T> {
  final T Function(AppTheme state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _ThemeViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class ThemeViewModelProvider extends StatefulWidget {
  final Widget child;
  const ThemeViewModelProvider({super.key, required this.child});

  @override
  State<ThemeViewModelProvider> createState() => _ThemeViewModelProviderState();
}

class _ThemeViewModelProviderState extends State<ThemeViewModelProvider> {
  late final ThemeViewModel _notifier = ThemeViewModel();

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
    return _ThemeViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _ThemeViewModelInherited extends InheritedModel<_ThemeViewModelAspect> {
  final ThemeViewModel notifier;

  const _ThemeViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_ThemeViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _ThemeViewModelInherited oldWidget,
    Set<_ThemeViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension ThemeViewModelContext on BuildContext {
  ThemeViewModel get themeViewModel {
    final inherited =
        InheritedModel.inheritFrom<_ThemeViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No ThemeViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added ThemeViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  ThemeViewModel readThemeViewModel() {
    final inherited = getInheritedWidgetOfExactType<_ThemeViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No ThemeViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added ThemeViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectThemeViewModel<T>(
    T Function(AppTheme state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited = getInheritedWidgetOfExactType<_ThemeViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No ThemeViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added ThemeViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_ThemeViewModelInherited>(
      this,
      aspect: _ThemeViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnThemeViewModel(
    void Function(AppTheme previous, AppTheme current) listener, {
    bool fireImmediately = false,
  }) {
    return readThemeViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
