// Route guard interfaces are user extension contracts generated code targets.
// ignore_for_file: one_member_abstracts

part of 'routing_core.dart';

/// Common supertype for every route guard accepted by generated routers.
///
/// Generated guard lists are typed against the router's route base class, so a
/// class listed in `@AppRoute(guards: [...])` that implements neither
/// [RouteGuard] nor [AsyncRouteGuard] fails to compile instead of being
/// skipped at runtime.
abstract interface class RouteGuardBase<T extends Object> {}

/// Synchronous route-specific guard contract.
abstract interface class RouteGuard<T extends Object>
    implements RouteGuardBase<T> {
  /// Returns `null` to allow navigation, or a replacement route to redirect.
  T? canActivate(T route);
}

/// Asynchronous route-specific guard contract.
abstract interface class AsyncRouteGuard<T extends Object>
    implements RouteGuardBase<T> {
  /// Returns `null` to allow navigation, or a replacement route to redirect.
  Future<T?> canActivate(T route);
}

/// Runs route guards in generated declaration order.
final class RouteGuardChain<T extends Object> {
  /// Creates a guard chain.
  const RouteGuardChain(this.guards);

  /// Guards to run, in the order declared on the route annotation.
  final List<RouteGuardBase<T>> guards;

  /// Runs sync and async guards in the order generated for the route.
  Future<T?> canActivate(T route) => _runRouteGuards(route, guards);
}

/// Runs [guards] against [route] and returns the first redirect, or `null`.
///
/// [onGuardStart] is called before each guard runs, and [onGuardAllowed] after
/// a guard allows navigation, so diagnostics only report guards that actually
/// executed. A guard implementing neither [RouteGuard] nor [AsyncRouteGuard]
/// throws rather than being skipped, so a mistyped guard cannot silently
/// disable an access check.
Future<T?> _runRouteGuards<T extends Object>(
  T route,
  List<RouteGuardBase<T>> guards, {
  void Function(RouteGuardBase<T> guard)? onGuardStart,
  void Function(RouteGuardBase<T> guard)? onGuardAllowed,
  void Function(RouteGuardBase<T> guard, T redirect)? onGuardRedirect,
}) async {
  for (final guard in guards) {
    onGuardStart?.call(guard);
    final T? redirected;
    if (guard is RouteGuard<T>) {
      redirected = guard.canActivate(route);
    } else if (guard is AsyncRouteGuard<T>) {
      redirected = await guard.canActivate(route);
    } else {
      throw StateError(
        'Route guard ${guard.runtimeType} implements neither RouteGuard<$T> '
        'nor AsyncRouteGuard<$T>, so its access check cannot run. '
        'Implement one of them, and check the type argument matches the '
        "router's generated route base type.",
      );
    }

    if (redirected != null) {
      onGuardRedirect?.call(guard, redirected);
      return redirected;
    }
    onGuardAllowed?.call(guard);
  }

  return null;
}
