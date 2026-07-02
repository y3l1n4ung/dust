import 'package:flutter/material.dart';

import '../state/benchmark_view_model.dart';

/// Benchmark shell.
class BenchmarkShell extends StatelessWidget {
  /// Creates a [BenchmarkShell].
  const BenchmarkShell({required this.child, super.key});

  /// Child.
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final activeFeature = context.watchBenchmarkViewModel().value.activeFeature;
    return Scaffold(
      appBar: AppBar(title: Text('Dust benchmark: $activeFeature')),
      body: child,
    );
  }
}
