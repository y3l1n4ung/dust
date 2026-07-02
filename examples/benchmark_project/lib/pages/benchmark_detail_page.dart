import 'package:flutter/material.dart';

import '../route.dart';
import 'benchmark_shell.dart';

/// Benchmark detail page.
@AppRoute('/models/:id', name: 'modelDetail', shell: BenchmarkShell)
class BenchmarkDetailPage extends StatelessWidget {
  /// Creates a [BenchmarkDetailPage].
  const BenchmarkDetailPage({
    required this.id,
    this.tab,
    this.archived,
    super.key,
  });

  /// Unique identifier.
  final int id;

  /// Tab.
  final String? tab;

  /// Archived.
  final bool? archived;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('Model $id'),
          Text('Tab: ${tab ?? 'overview'}'),
          Text('Archived: ${archived ?? false}'),
          const SizedBox(height: 16),
          FilledButton(
            onPressed: () => context.navigator.home().replace(),
            child: const Text('Back to benchmark'),
          ),
        ],
      ),
    );
  }
}
