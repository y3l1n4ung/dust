part of 'routing_core.dart';

final class _RouteStackStore<T extends Object> {
  _RouteStackStore({
    required T initialRoute,
    required String? initialBranch,
  }) {
    _activeBranch = initialBranch;
    _activeEntries.add(_RouteEntry(initialRoute));
    _storeActiveEntries();
    rebuildPageKeyMap();
  }

  List<_RouteEntry<T>> _activeEntries = <_RouteEntry<T>>[];
  List<_RouteEntry<T>> _rootEntries = <_RouteEntry<T>>[];
  final Map<String, List<_RouteEntry<T>>> _branchEntries =
      <String, List<_RouteEntry<T>>>{};
  final Map<Key, int> _keyToStackIndex = <Key, int>{};
  String? _activeBranch;

  List<_RouteEntry<T>> get entries => _activeEntries;

  String? get activeBranch => _activeBranch;

  RouteStack<T> routes() =>
      List<T>.unmodifiable(_activeEntries.map((entry) => entry.route));

  List<LocalKey> pageKeys() =>
      _activeEntries.map((entry) => entry.key).toList(growable: false);

  T currentRouteOr(T fallback) =>
      _activeEntries.isEmpty ? fallback : _activeEntries.last.route;

  int? stackIndexForKey(Key? key) => key == null ? null : _keyToStackIndex[key];

  _RouteEntry<T> removeAt(int index) => _activeEntries.removeAt(index);

  bool activateBranch(String? branch) {
    if (branch == _activeBranch) return false;
    _storeActiveEntries();
    _activeBranch = branch;
    _activeEntries = branch == null
        ? _rootEntries
        : _branchEntries.putIfAbsent(branch, () => <_RouteEntry<T>>[]);
    return true;
  }

  bool activeRootMatches(T route, RouteLocation<T> routeLocation) {
    return _activeEntries.isNotEmpty &&
        routeLocation(_activeEntries.first.route) == routeLocation(route);
  }

  Iterable<_RouteEntry<T>> replaceWithOne(_RouteEntry<T> entry) {
    final previous = List<_RouteEntry<T>>.of(_activeEntries);
    _activeEntries
      ..clear()
      ..add(entry);
    return previous;
  }

  void push(_RouteEntry<T> entry) {
    _activeEntries.add(entry);
  }

  ({_RouteEntry<T> committed, _RouteEntry<T>? replaced}) replaceTop(
    T route,
    RouteLocation<T> routeLocation,
  ) {
    if (_activeEntries.isEmpty) {
      final committed = _RouteEntry(route);
      _activeEntries.add(committed);
      return (committed: committed, replaced: null);
    }

    final previous = _activeEntries.last;
    final sameLocation = routeLocation(previous.route) == routeLocation(route);
    final committed =
        sameLocation ? previous.withRoute(route) : _RouteEntry(route);
    _activeEntries[_activeEntries.length - 1] = committed;
    return (committed: committed, replaced: sameLocation ? null : previous);
  }

  Iterable<_RouteEntry<T>> replaceWithRoutes(RouteStack<T> routes) {
    final previous = List<_RouteEntry<T>>.of(_activeEntries);
    _activeEntries
      ..clear()
      ..addAll(routes.map(_RouteEntry<T>.new));
    return previous;
  }

  void finishMutation() {
    _storeActiveEntries();
    rebuildPageKeyMap();
  }

  void rebuildPageKeyMap() {
    _keyToStackIndex
      ..clear()
      ..addEntries(
        _activeEntries.indexed.map((entry) {
          final (index, routeEntry) = entry;
          return MapEntry<Key, int>(routeEntry.key, index);
        }),
      );
  }

  Iterable<_RouteEntry<T>> allEntries() sync* {
    final seen = <LocalKey>{};
    for (final entry in _rootEntries) {
      if (seen.add(entry.key)) yield entry;
    }
    for (final branch in _branchEntries.values) {
      for (final entry in branch) {
        if (seen.add(entry.key)) yield entry;
      }
    }
  }

  void _storeActiveEntries() {
    final branch = _activeBranch;
    if (branch == null) {
      _rootEntries = _activeEntries;
    } else {
      _branchEntries[branch] = _activeEntries;
    }
  }
}

final class _RouteEntry<T extends Object> {
  _RouteEntry(T route, {bool tracksPop = false})
      : this._(
          route,
          UniqueKey(),
          tracksPop ? Completer<Object?>() : null,
        );

  _RouteEntry._(this.route, this.key, this._popped);

  final T route;
  final LocalKey key;
  final Completer<Object?>? _popped;

  Future<Object?> get popped => _popped?.future ?? Future<Object?>.value();

  void complete(Object? result) {
    final popped = _popped;
    if (popped == null || popped.isCompleted) return;
    popped.complete(result);
  }

  _RouteEntry<T> withRoute(T route) => _RouteEntry<T>._(route, key, _popped);
}
