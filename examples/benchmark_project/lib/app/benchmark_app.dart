import 'package:flutter/material.dart';

import '../route.dart';
import '../state/benchmark_view_model.dart';

class BenchmarkApp extends StatelessWidget {
  const BenchmarkApp({super.key});

  @override
  Widget build(BuildContext context) {
    return BenchmarkViewModelScope(
      args: (_) => const BenchmarkViewModelArgs(),
      create: (_, args) => BenchmarkViewModel(args),
      child: Builder(
        builder: (context) {
          final viewModel = context.readBenchmarkViewModel();
          return MaterialApp.router(
            title: 'Dust Benchmark Project',
            routerConfig: BenchmarkRouter(refresh: viewModel).config,
          );
        },
      ),
    );
  }
}
