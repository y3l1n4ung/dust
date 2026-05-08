// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'search_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _SearchViewModelAspect<T> {
  final T Function(SearchStatus state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _SearchViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class SearchViewModelProvider extends StatefulWidget {
  final Widget child;
  const SearchViewModelProvider({super.key, required this.child});

  @override
  State<SearchViewModelProvider> createState() =>
      _SearchViewModelProviderState();
}

class _SearchViewModelProviderState extends State<SearchViewModelProvider> {
  late final SearchViewModel _notifier = SearchViewModel();

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
    return _SearchViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _SearchViewModelInherited extends InheritedModel<_SearchViewModelAspect> {
  final SearchViewModel notifier;

  const _SearchViewModelInherited(
      {required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_SearchViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _SearchViewModelInherited oldWidget,
    Set<_SearchViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension SearchViewModelContext on BuildContext {
  SearchViewModel get searchViewModel {
    final inherited =
        InheritedModel.inheritFrom<_SearchViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No SearchViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added SearchViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  SearchViewModel readSearchViewModel() {
    final inherited =
        getInheritedWidgetOfExactType<_SearchViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No SearchViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added SearchViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectSearchViewModel<T>(
    T Function(SearchStatus state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited =
        getInheritedWidgetOfExactType<_SearchViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No SearchViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added SearchViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_SearchViewModelInherited>(
      this,
      aspect: _SearchViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnSearchViewModel(
    void Function(SearchStatus previous, SearchStatus current) listener, {
    bool fireImmediately = false,
  }) {
    return readSearchViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
