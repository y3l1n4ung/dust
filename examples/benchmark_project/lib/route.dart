import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:dust_flutter/route.dart';

import 'route.g.dart';

export 'route.g.dart';
export 'package:dust_flutter/route.dart';

@AppRouter(initial: '/', notFound: '/404')
final class BenchmarkRouter extends $BenchmarkRouter {
  BenchmarkRouter({required this.refresh});

  final Listenable refresh;
}

final class BenchmarkGuard implements RouteGuard<AppRoutePath> {
  const BenchmarkGuard();

  @override
  AppRoutePath? canActivate(AppRoutePath route) => null;
}
