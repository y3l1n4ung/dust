part of 'routing_core.dart';

/// Synchronous route-specific guard contract.
abstract interface class RouteGuard<T extends Object> {
  /// Returns `null` to allow navigation, or a replacement route to redirect.
  T? canActivate(T route);
}

/// Asynchronous route-specific guard contract.
abstract interface class AsyncRouteGuard<T extends Object> {
  /// Returns `null` to allow navigation, or a replacement route to redirect.
  Future<T?> canActivate(T route);
}

/// Runs route guards in deterministic sync-then-async order.
final class RouteGuardChain<T extends Object> {
  /// Creates a guard chain.
  const RouteGuardChain(this.guards);

  /// Guards to run. Generated code guarantees each entry is a supported guard.
  final List<Object> guards;

  /// Runs sync guards first, then async guards, preserving declaration order.
  Future<T?> canActivate(T route) async {
    for (final guard in guards) {
      if (guard is RouteGuard<T>) {
        final redirected = guard.canActivate(route);
        if (redirected != null) return redirected;
      }
    }

    for (final guard in guards) {
      if (guard is AsyncRouteGuard<T>) {
        final redirected = await guard.canActivate(route);
        if (redirected != null) return redirected;
      }
    }

    return null;
  }
}
