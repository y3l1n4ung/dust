part of 'routing_core.dart';

extension _GeneratedRouterDelegateMutation<T extends Object>
    on GeneratedRouterDelegate<T> {
  _RouteEntry<T> _commitRoute(
    T route,
    NavigationMode mode, [
    RouteStack<T>? restored,
  ]) {
    final previous = _history.routes();
    final targetBranch = config.routeBranch(route);
    final previousBranch = _history.activeBranch;
    final switchedBranch = _history.activateBranch(targetBranch);
    if (switchedBranch) {
      _log('branch ${previousBranch ?? '-'} => ${targetBranch ?? '-'}');
    }
    late final _RouteEntry<T> committed;
    switch (mode) {
      case NavigationMode.go:
        if (switchedBranch &&
            targetBranch != null &&
            _history.activeRootMatches(route, config.routeLocation)) {
          committed = _history.entries.last;
          break;
        }
        committed = _RouteEntry(route);
        _completeEntries(_history.replaceWithOne(committed));
      case NavigationMode.push:
        committed = _RouteEntry(route, tracksPop: true);
        _history.push(committed);
      case NavigationMode.replace:
        final replacement = _history.replaceTop(route, config.routeLocation);
        committed = replacement.committed;
        replacement.replaced?.complete(null);
      case NavigationMode.restore:
        final routes = restored ?? <T>[route];
        _completeEntries(_history.replaceWithRoutes(routes));
        committed = _history.entries.last;
    }
    _finishStackCommit(previous);
    return committed;
  }

  void _completeEntries(Iterable<_RouteEntry<T>> entries) {
    for (final entry in entries) {
      entry.complete(null);
    }
  }

  bool _popTop(Object? result) {
    if (_disposed) return false;
    if (_history.entries.length <= 1) return false;
    final previous = _history.routes();
    final removed = _history.entries.removeLast();
    removed.complete(result);
    _log('pop ${_debugRoute(removed.route)}');
    _finishStackCommit(previous);
    _revalidateExposedTop();
    return true;
  }

  void _revalidateExposedTop() {
    if (_disposed || _history.entries.isEmpty) return;
    _unawaitedNavigation(_applyRoute(currentRoute, NavigationMode.replace));
  }

  void _unawaitedNavigation(Future<void> future) {
    unawaited(
      future.catchError((Object error, StackTrace stackTrace) {
        config.router.onException(error, stackTrace);
      }),
    );
  }
}
