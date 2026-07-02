import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:dust_flutter/route.dart';

import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

/// Benchmark router model for the benchmark example.
@AppRouter(initial: '/', notFound: '/404')
final class BenchmarkRouter extends $BenchmarkRouter {
  /// Creates a [BenchmarkRouter].
  BenchmarkRouter({required this.refresh});

  /// Refresh.
  final Listenable refresh;
}

/// Benchmark guard model for the benchmark example.
final class BenchmarkGuard implements RouteGuard<AppRoutePath> {
  /// Creates a [BenchmarkGuard].
  const BenchmarkGuard();

  @override
  AppRoutePath? canActivate(AppRoutePath route) => null;
}
