import 'package:dust_flutter/route.dart';

import 'features/auth/models/auth_state.dart';
import 'features/auth/models/user.dart';
import 'features/auth/view_models/auth_view_model.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

@AppRouter(initial: '/', notFound: '/404')
final class ShoppingRouter extends $ShoppingRouter {
  ShoppingRouter({required this.auth});

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

enum ShoppingAccessLevel { guest, customer, staff, admin }

// Demo access levels. Production apps should use server-issued claims.
ShoppingAccessLevel shoppingAccessLevel(User? user) {
  final username = user?.username.toLowerCase();
  return switch (username) {
    null => ShoppingAccessLevel.guest,
    'admin' => ShoppingAccessLevel.admin,
    'manager' || 'staff' => ShoppingAccessLevel.staff,
    _ => ShoppingAccessLevel.customer,
  };
}

final class StaffGuard implements RouteGuard<AppRoutePath> {
  const StaffGuard(this.auth);

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

final class AdminGuard implements RouteGuard<AppRoutePath> {
  const AdminGuard(this.auth);

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
