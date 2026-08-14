part of 'routing_core.dart';

extension _GeneratedRouterDelegateNavigation<T extends Object>
    on GeneratedRouterDelegate<T> {
  Future<_RouteEntry<T>?> _applyRoute(
    T requested,
    NavigationMode mode, [
    int guardRedirects = 0,
    List<String>? guardRedirectChain,
  ]) async {
    if (_disposed) return null;
    final epoch = ++_navigationEpoch;
    var candidate = requested;
    final guardChain = guardRedirectChain ?? [_debugRoute(requested)];
    _log('${_debugMode(mode)} ${_debugRoute(requested)}');
    _logRouteDetails('route', requested);

    if (guardRedirects >= maxRedirects) {
      throw StateError(
        'Router guard redirects hit the redirect cap ($maxRedirects) '
        'navigating to "${config.routeLocation(requested)}". '
        'Guard redirect chain: ${guardChain.join(' -> ')}. '
        'Check guards that return one of these routes repeatedly.',
      );
    }

    var redirects = 0;
    final redirectChain = <String>[_debugRoute(candidate)];
    for (; redirects < maxRedirects; redirects += 1) {
      final redirected = config.router.redirect(candidate);
      if (redirected == null ||
          config.routeLocation(redirected) == config.routeLocation(candidate)) {
        break;
      }
      _log(
        'redirecting ${_debugRoute(candidate)} => '
        '${_debugRoute(redirected)}',
      );
      _logRouteDetails('redirect target', redirected);
      candidate = redirected;
      redirectChain.add(_debugRoute(candidate));
    }
    if (redirects >= maxRedirects) {
      throw StateError(
        'Router hit the redirect cap ($maxRedirects) navigating to '
        '"${config.routeLocation(requested)}". '
        'Redirect chain: ${redirectChain.join(' -> ')}. '
        'Check redirect() for a cycle or return null to allow navigation.',
      );
    }
    assert(
      redirects < maxRedirects,
      'Router hit the redirect cap ($maxRedirects) navigating to '
      '"${config.routeLocation(requested)}". Check for a redirect cycle.',
    );

    if (epoch != _navigationEpoch) return null;

    if (mode == NavigationMode.restore) {
      final restored = config.restoreStack?.call(candidate);
      final routes =
          restored == null || restored.isEmpty ? <T>[candidate] : restored;
      for (final route in routes) {
        final guardFuture = _runGuards(route);
        final redirected = guardFuture == null ? null : await guardFuture;
        if (epoch != _navigationEpoch) return null;
        if (redirected != null) {
          _log(
            'guard redirect ${_debugRoute(route)} => '
            '${_debugRoute(redirected)}',
          );
          _logRouteDetails('guard target', redirected);
          return _applyRoute(
            redirected,
            mode,
            guardRedirects + 1,
            [...guardChain, _debugRoute(redirected)],
          );
        }
      }
      return _commitRoute(candidate, mode, routes);
    }

    final guardFuture = _runGuards(candidate);
    final redirected = guardFuture == null ? null : await guardFuture;
    if (epoch != _navigationEpoch) return null;
    if (redirected != null) {
      _log(
        'guard redirect ${_debugRoute(candidate)} => '
        '${_debugRoute(redirected)}',
      );
      _logRouteDetails('guard target', redirected);
      return _applyRoute(
        redirected,
        mode,
        guardRedirects + 1,
        [...guardChain, _debugRoute(redirected)],
      );
    }
    return _commitRoute(candidate, mode);
  }

  Future<T?>? _runGuards(T route) {
    final guards = config.resolveGuards(route);
    if (guards.isEmpty) return null;
    _log('guards ${guards.length} for ${_debugRoute(route)}');
    return _runGuardChain(route, guards);
  }

  Future<T?> _runGuardChain(T route, List<RouteGuardBase<T>> guards) {
    return _runRouteGuards(
      route,
      guards,
      onGuardStart: (guard) {
        _log('guard ${guard.runtimeType} for ${_debugRoute(route)}');
      },
      onGuardAllowed: (guard) {
        _log('guard ${guard.runtimeType} allow ${_debugRoute(route)}');
      },
      onGuardRedirect: (guard, redirected) {
        _log(
          'guard ${guard.runtimeType} redirect '
          '${_debugRoute(route)} => ${_debugRoute(redirected)}',
        );
      },
    );
  }
}
