import 'package:dust_flutter/route.dart';

import 'features/auth/models/auth_state.dart';
import 'features/auth/models/user.dart';
import 'features/auth/view_models/auth_view_model.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

/// Shopping router model for the shopping app example.
@AppRouter(initial: '/', notFound: '/404')
final class ShoppingRouter extends $ShoppingRouter {
  /// Creates a [ShoppingRouter].
  ShoppingRouter({required this.auth});

  /// Auth.
  final AuthViewModel auth;

  @override
  AppRoutePath? redirect(AppRoutePath route) {
    final status = auth.state.status;
    final isAuthenticated = auth.state.isAuthenticated;
    final isAuthRoute = route is LoginRoute || route is RegisterRoute;

    if (status == AuthStatus.loading || status == AuthStatus.initial) {
      return null;
    }

    if (!isAuthenticated && route.requiresAuth) {
      return LoginRoute(redirectPath: route.location);
    }

    if (isAuthenticated && isAuthRoute) {
      final redirectPath = switch (route) {
        LoginRoute(:final redirectPath) => redirectPath,
        RegisterRoute(:final redirectPath) => redirectPath,
        _ => null,
      };
      return _safeRedirect(redirectPath) ?? const ProductsRoute();
    }

    return null;
  }

  AppRoutePath? _safeRedirect(String? redirectPath) {
    if (redirectPath == null || redirectPath.isEmpty) return null;
    final uri = Uri.tryParse(redirectPath);
    if (uri == null || uri.host.isNotEmpty) return null;
    final route = parseAppRoute(uri);
    if (route is NotFoundRoute) return null;
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
final class StaffGuard implements RouteGuard<AppRoutePath> {
  /// Creates a [StaffGuard].
  const StaffGuard(this.auth);

  /// Auth.
  final AuthViewModel auth;

  @override
  AppRoutePath? canActivate(AppRoutePath route) {
    if (!auth.state.isAuthenticated) {
      return LoginRoute(redirectPath: route.location);
    }
    return _hasAccess(ShoppingAccessLevel.staff) ? null : const ProductsRoute();
  }

  bool _hasAccess(ShoppingAccessLevel minimum) =>
      shoppingAccessLevel(auth.state.user).index >= minimum.index;
}

/// Admin guard model for the shopping app example.
final class AdminGuard implements RouteGuard<AppRoutePath> {
  /// Creates an [AdminGuard].
  const AdminGuard(this.auth);

  /// Auth.
  final AuthViewModel auth;

  @override
  AppRoutePath? canActivate(AppRoutePath route) {
    if (!auth.state.isAuthenticated) {
      return LoginRoute(redirectPath: route.location);
    }
    return _hasAccess(ShoppingAccessLevel.admin) ? null : const ProductsRoute();
  }

  bool _hasAccess(ShoppingAccessLevel minimum) =>
      shoppingAccessLevel(auth.state.user).index >= minimum.index;
}
