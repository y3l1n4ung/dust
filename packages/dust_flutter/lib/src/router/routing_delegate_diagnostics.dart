part of 'routing_core.dart';

extension _GeneratedRouterDelegateDiagnostics<T extends Object>
    on GeneratedRouterDelegate<T> {
  void _log(String message) {
    if (!config.router.debugLogDiagnostics) return;
    debugPrint('AppRouter: $message');
  }

  void _logRouteDetails(String label, T route) {
    if (!config.router.debugLogDiagnostics) return;
    final info = config.debugInfo?.call(route);
    final branch = info?.branch ?? config.routeBranch(route);
    _log(
      '$label ${_debugRoute(route)} '
      'name=${info?.name ?? '-'} '
      'shell=${info?.shell ?? '-'} '
      'branch=${branch ?? '-'}',
    );
  }

  void _logKnownRoutes() {
    if (!config.router.debugLogDiagnostics || config.debugRoutes.isEmpty) {
      return;
    }
    final routes = _debugRoutes(config.debugRoutes).toList(growable: false);
    if (routes.isEmpty) return;

    _log(
      'Full paths for routes:\n'
      '${routes.map((route) => '           => ${route.path}').join('\n')}',
    );

    final namedRoutes =
        routes.where((route) => route.name != null).toList(growable: false);
    if (namedRoutes.isEmpty) return;

    _log(
      'known full paths for route names:\n'
      '${namedRoutes.map((route) {
        return '           ${route.name} => ${route.path}';
      }).join('\n')}',
    );
  }

  Iterable<({String path, String? name})> _debugRoutes(
    List<GeneratedRoute> routes, [
    String parent = '',
  ]) sync* {
    for (final route in routes) {
      final path = _joinDebugPath(parent, route.path);
      if (route.page != null) {
        yield (path: path, name: route.name);
      }
      yield* _debugRoutes(route.routes, path);
    }
  }

  String _joinDebugPath(String parent, String path) {
    if (path.startsWith('/')) return path;
    if (parent.isEmpty || parent == '/') return '/$path';
    return '$parent/$path';
  }

  String _debugMode(NavigationMode mode) {
    return switch (mode) {
      NavigationMode.go => 'go',
      NavigationMode.push => 'push',
      NavigationMode.replace => 'replace',
      NavigationMode.restore => 'restoring',
    };
  }

  String _debugRoute(T route) {
    try {
      return config.routeLocation(route);
    } catch (_) {
      return route.toString();
    }
  }

  String _debugStack() {
    return '[${_history.entries.map((entry) => _debugRoute(entry.route)).join(', ')}]';
  }
}
