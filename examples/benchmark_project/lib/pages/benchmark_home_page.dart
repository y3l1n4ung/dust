import 'package:flutter/material.dart' hide Route;

import '../route.dart';
import '../state/benchmark_state.dart';
import '../state/benchmark_view_model.dart';
import 'benchmark_shell.dart';

const benchmarkFeatures = ['derive', 'serde', 'http', 'route', 'state'];

@Route(
  '/',
  name: 'home',
  shell: BenchmarkShell,
  guards: [BenchmarkGuard],
  transition: FadeUpwardsPageTransitionsBuilder(),
)
class BenchmarkHomePage extends StatelessWidget {
  const BenchmarkHomePage({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watchBenchmarkViewModel().value;
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Text('Generated files: 5000'),
        Text('Builds run: ${state.buildsRun}'),
        const SizedBox(height: 16),
        for (final feature in benchmarkFeatures)
          ListTile(
            title: Text(feature),
            selected: feature == state.activeFeature,
            onTap: () {
              context.readBenchmarkViewModel().selectFeature(feature);
              context
                  .routes
                  .modelDetail(
                    id: feature.length,
                    tab: feature,
                    archived: false,
                  )
                  .push();
            },
          ),
        const SizedBox(height: 16),
        FilledButton(
          onPressed: () => context
              .readBenchmarkViewModel()
              .recordBuild(BenchmarkMode.invalidated),
          child: const Text('Record invalidated build'),
        ),
      ],
    );
  }
}
