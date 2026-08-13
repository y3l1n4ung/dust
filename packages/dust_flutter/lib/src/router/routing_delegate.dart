part of 'routing_core.dart';

/// Internal navigation action mode.
enum NavigationMode {
  /// Replace the current stack with the target route.
  go,

  /// Add the target route to the top of the stack.
  push,

  /// Replace the current top route with the target route.
  replace,

  /// Restore a route from platform route information.
  restore,
}

/// Navigator 2.0 delegate used by generated routers.
final class GeneratedRouterDelegate<T extends Object> extends RouterDelegate<T>
    with ChangeNotifier, PopNavigatorRouterDelegateMixin<T> {
  /// Creates a router delegate from generated [config].
  GeneratedRouterDelegate(this.config) {
    _controller = RouterController<T>._(this);
    _history = _RouteStackStore<T>(
      initialRoute: config.initialRoute,
      initialBranch: config.routeBranch(config.initialRoute),
    );
    _logKnownRoutes();
    _log('setting initial route ${_debugRoute(config.initialRoute)}');
    config.router.refreshListenable?.addListener(_scheduleRefresh);
    _scheduleRefresh();
  }

  /// Generated router runtime configuration.
  final RouterRuntimeConfig<T> config;

  /// Mutable route stack used by `Navigator.pages`.
  RouteStack<T> get stack => _history.routes();

  /// Stack entry keys exposed for router runtime tests.
  @visibleForTesting
  List<LocalKey> get debugPageKeys => _history.pageKeys();

  /// Waits for the latest route refresh scheduled by
  /// [RouterBase.refreshListenable].
  @visibleForTesting
  Future<void> debugWaitForScheduledRefresh() =>
      _scheduledRefresh ?? Future<void>.value();

  late final _RouteStackStore<T> _history;
  late final RouterController<T> _controller;
  bool _refreshScheduled = false;
  Future<void>? _scheduledRefresh;
  int _navigationEpoch = 0;
  bool _disposed = false;

  /// Last route in the stack, or the configured initial route.
  T get currentRoute => _history.currentRouteOr(config.initialRoute);

  @override
  final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();

  @override
  T get currentConfiguration => currentRoute;

  @override
  Widget build(BuildContext context) {
    return RouterScope(
      controller: _controller,
      notifier: this,
      child: Navigator(
        key: navigatorKey,
        observers: config.router.observers,
        pages: [
          for (final entry in _history.entries)
            config.buildPage(entry.route, entry.key, (didPop, result) {
              if (didPop) entry.complete(result);
            }),
        ],
        onDidRemovePage: (page) {
          if (_disposed) return;
          final key = page.key;
          final index = _history.stackIndexForKey(key);
          if (index != null && index >= 0 && index < _history.entries.length) {
            final previous = _history.routes();
            final removed = _history.removeAt(index);
            removed.complete(null);
            _log('remove ${_debugRoute(removed.route)}');
            _finishStackCommit(previous);
            _revalidateExposedTop();
          }
        },
      ),
    );
  }

  @override
  Future<void> setNewRoutePath(T configuration) async {
    if (_disposed) return;
    await _applyRoute(configuration, NavigationMode.restore);
  }

  @override
  Future<bool> popRoute() async {
    return _popTop(null);
  }

  /// Navigates to [route], replacing the current stack.
  void go(T route) =>
      _unawaitedNavigation(_applyRoute(route, NavigationMode.go));

  /// Pushes [route] on top of the current stack.
  Future<R?> push<R>(T route) async {
    final entry = await _applyRoute(route, NavigationMode.push);
    if (entry == null) return null;
    return entry.popped.then((result) => result as R?);
  }

  /// Replaces the current top route with [route].
  void replace(T route) =>
      _unawaitedNavigation(_applyRoute(route, NavigationMode.replace));

  /// Pops the top route and completes its push future with [result].
  bool pop<R>([R? result]) => _popTop(result);

  @override
  void dispose() {
    if (_disposed) return;
    _disposed = true;
    _navigationEpoch += 1;
    config.router.refreshListenable?.removeListener(_scheduleRefresh);
    _completeEntries(_history.allEntries());
    super.dispose();
  }

  void _scheduleRefresh() {
    if (_disposed || _refreshScheduled) return;
    _refreshScheduled = true;
    _log('refreshing ${_debugRoute(currentRoute)}');
    final refresh = Future<void>.microtask(() async {
      _refreshScheduled = false;
      if (_disposed) return;
      await _applyRoute(currentRoute, NavigationMode.replace);
    });
    _scheduledRefresh = refresh;
    _unawaitedNavigation(refresh);
  }

  void _finishStackCommit(RouteStack<T> previous) {
    _history.finishMutation();
    _log('stack ${_debugStack()}');
    final next = _history.routes();
    notifyListeners();
    if (_routeLocationsChanged(previous, next)) {
      config.router.didChangeRouteStack(previous, next);
    }
  }

  bool _routeLocationsChanged(RouteStack<T> previous, RouteStack<T> next) {
    if (previous.length != next.length) return true;
    for (var index = 0; index < previous.length; index += 1) {
      if (config.routeLocation(previous[index]) !=
          config.routeLocation(next[index])) {
        return true;
      }
    }
    return false;
  }
}
