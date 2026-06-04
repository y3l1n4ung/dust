import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' hide Route, Router;
import 'package:dust_flutter/route.dart' show Router, RouteGuard, RouteGuardResult;

import 'pages/benchmark_home_page.dart';
import 'pages/benchmark_not_found_page.dart';
import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

@Router(
  initial: BenchmarkHomePage,
  notFound: BenchmarkNotFoundPage,
  refreshListenable: 'refresh',
)
final class BenchmarkRouter extends $BenchmarkRouter {
  BenchmarkRouter({required this.refresh});

  final Listenable refresh;

  @override
  BenchmarkGuard createBenchmarkGuard() => const BenchmarkGuard();
}

final class BenchmarkGuard implements RouteGuard<AppRoutePath> {
  const BenchmarkGuard();

  @override
  Future<RouteGuardResult<AppRoutePath>> canActivate(RouteState state) async {
    return const RouteGuardResult.allow();
  }
}
