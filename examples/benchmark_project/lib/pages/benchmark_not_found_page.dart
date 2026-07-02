import 'package:flutter/material.dart';

import '../route.dart';

/// Benchmark not found page.
@AppRoute('/404', name: 'notFound', guards: [])
class BenchmarkNotFoundPage extends StatelessWidget {
  /// Creates a [BenchmarkNotFoundPage].
  const BenchmarkNotFoundPage({this.path = '', super.key});

  /// Path.
  final String path;

  @override
  Widget build(BuildContext context) {
    return Scaffold(body: Center(child: Text('No benchmark route for $path')));
  }
}
