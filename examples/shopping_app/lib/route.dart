import 'package:dust_flutter/route.dart';

import 'features/auth/models/auth_state.dart';
import 'features/auth/models/user.dart';
import 'features/auth/view_models/auth_view_model.dart';
import 'route/routes.g.dart';

export 'package:dust_flutter/route.dart';
export 'route/routes.g.dart';

/// Shopping router model for the shopping app example.
@AppRouter(initial: '/', notFound: '/404')
final class ShoppingRouter extends ShoppingRouterBase {
  /// Creates a [ShoppingRouter].
  ShoppingRouter({required this.auth});

  /// Auth.
  final AuthViewModel auth;

  @override
  ShoppingRoutePath? redirect(ShoppingRoutePath route) {
    final status = auth.state.status;
    final isAuthenticated = auth.state.isAuthenticated;
    final isAuthRoute =
        route is ShoppingLoginRoute || route is ShoppingRegisterRoute;

    if (status == AuthStatus.loading || status == AuthStatus.initial) {
      return null;
    }

    if (!isAuthenticated && route.requiresAuth) {
      return ShoppingLoginRoute(redirectPath: route.location);
    }

    if (isAuthenticated && isAuthRoute) {
      final redirectPath = switch (route) {
        ShoppingLoginRoute(:final redirectPath) => redirectPath,
        ShoppingRegisterRoute(:final redirectPath) => redirectPath,
        _ => null,
      };
      return _safeRedirect(redirectPath) ?? const ShoppingProductsRoute();
    }

    return null;
  }

  ShoppingRoutePath? _safeRedirect(String? redirectPath) {
    if (redirectPath == null || redirectPath.isEmpty) return null;
    final uri = Uri.tryParse(redirectPath);
    if (uri == null || uri.host.isNotEmpty) return null;
    final route = parseShoppingRoute(uri);
    if (route is ShoppingNotFoundRoute) return null;
    return route;
  }
}

/// Shopping access level values for the shopping app example.
enum ShoppingAccessLevel {
  /// Guest shopping access level.
  guest,

  /// Customer shopping access level.
  customer,

  /// Staff shopping access level.
  staff,

  /// Admin shopping access level.
  admin,
}

// Demo access levels. Production apps should use server-issued claims.
/// Shopping access level shopping access level.
ShoppingAccessLevel shoppingAccessLevel(User? user) {
  final username = user?.username.toLowerCase();
  return switch (username) {
    null => ShoppingAccessLevel.guest,
    'admin' => ShoppingAccessLevel.admin,
    'manager' || 'staff' => ShoppingAccessLevel.staff,
    _ => ShoppingAccessLevel.customer,
  };
}

/// Staff guard model for the shopping app example.
final class StaffGuard implements RouteGuard<ShoppingRoutePath> {
  /// Creates a [StaffGuard].
  const StaffGuard(this.auth);

  /// Auth.
  final AuthViewModel auth;

  @override
  ShoppingRoutePath? canActivate(ShoppingRoutePath route) {
    if (!auth.state.isAuthenticated) {
      return ShoppingLoginRoute(redirectPath: route.location);
    }
    return _hasAccess(ShoppingAccessLevel.staff)
        ? null
        : const ShoppingProductsRoute();
  }

  bool _hasAccess(ShoppingAccessLevel minimum) =>
      shoppingAccessLevel(auth.state.user).index >= minimum.index;
}

/// Admin guard model for the shopping app example.
final class AdminGuard implements RouteGuard<ShoppingRoutePath> {
  /// Creates an [AdminGuard].
  const AdminGuard(this.auth);

  /// Auth.
  final AuthViewModel auth;

  @override
  ShoppingRoutePath? canActivate(ShoppingRoutePath route) {
    if (!auth.state.isAuthenticated) {
      return ShoppingLoginRoute(redirectPath: route.location);
    }
    return _hasAccess(ShoppingAccessLevel.admin)
        ? null
        : const ShoppingProductsRoute();
  }

  bool _hasAccess(ShoppingAccessLevel minimum) =>
      shoppingAccessLevel(auth.state.user).index >= minimum.index;
}
