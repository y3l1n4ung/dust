import 'dart:async';

import 'package:flutter/material.dart' hide Route, Router;

import 'app_state.dart';
import 'pages/dashboard_page.dart';
import 'pages/not_found_page.dart';
import 'route_annotations.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_router/dust_router.dart';

@Router(initial: DashboardPage, notFound: NotFoundPage)
final class AppRouter extends $AppRouter {
  AppRouter({required this.session});

  final AppSession session;

  @override
  Listenable get refreshListenable => session;

  @override
  AdminGuard createAdminGuard() => AdminGuard(session);

  @override
  BillingGuard createBillingGuard() => BillingGuard(session);

  @override
  AppRoutePath? redirect(RouteState state) {
    final isLoggedIn = session.isLoggedIn;
    final goingToLogin = state.route is LoginRoute;

    if (!isLoggedIn && state.route.requiresAuth) {
      return LoginRoute(from: state.location);
    }

    if (isLoggedIn && goingToLogin) {
      return _safeFrom((state.route as LoginRoute).from) ??
          const DashboardRoute();
    }

    return null;
  }

  AppRoutePath? _safeFrom(String? from) {
    if (from == null || from.isEmpty) {
      return null;
    }
    final uri = Uri.tryParse(from);
    if (uri == null || uri.host.isNotEmpty) {
      return null;
    }
    final route = parseAppRoute(uri);
    if (route is NotFoundRoute) {
      return null;
    }
    return route;
  }
}

final class AdminGuard implements RouteGuard<AppRoutePath> {
  const AdminGuard(this.session);

  final AppSession session;

  @override
  Future<RouteGuardResult<AppRoutePath>> canActivate(RouteState state) async {
    if (!session.isAdmin) {
      return RouteGuardResult<AppRoutePath>.redirect(const ForbiddenRoute());
    }
    return RouteGuardResult<AppRoutePath>.allow();
  }
}

final class BillingGuard implements RouteGuard<AppRoutePath> {
  const BillingGuard(this.session);

  final AppSession session;

  @override
  Future<RouteGuardResult<AppRoutePath>> canActivate(RouteState state) async {
    if (!session.billingEnabled) {
      return RouteGuardResult<AppRoutePath>.redirect(const ForbiddenRoute());
    }
    return RouteGuardResult<AppRoutePath>.allow();
  }
}
