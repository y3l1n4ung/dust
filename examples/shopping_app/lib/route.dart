import 'package:dust_flutter/route.dart' show AppRouter;

import 'features/auth/models/auth_state.dart';
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
