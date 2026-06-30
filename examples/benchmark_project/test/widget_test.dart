import 'package:dust_benchmark_project/app/benchmark_app.dart';
import 'package:dust_benchmark_project/route.dart';
import 'package:dust_benchmark_project/state/benchmark_state.dart';
import 'package:dust_benchmark_project/state/benchmark_view_model.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('benchmark state serde and copyWith cover generated derive output', () {
    const state = BenchmarkState(
      mode: BenchmarkMode.cold,
      activeFeature: 'route',
      buildsRun: 2,
    );

    final decoded = BenchmarkState.fromJson(state.toJson());
    final copied = state.copyWith(activeFeature: 'state');

    expect(decoded, equals(state));
    expect(copied.activeFeature, 'state');
    expect(copied.buildsRun, 2);
  });

  test('generated typed routes parse and format benchmark paths', () {
    final route = parseAppRoute(
      Uri.parse('/models/42?tab=serde&archived=true'),
    );

    expect(route, isA<ModelDetailRoute>());
    expect((route as ModelDetailRoute).id, 42);
    expect(route.tab, 'serde');
    expect(route.archived, isTrue);
    expect(route.location, '/models/42?tab=serde&archived=true');
  });

  test('benchmark guard allows generated guarded routes', () {
    final result = const BenchmarkGuard().canActivate(const HomeRoute());

    expect(result, isNull);
  });

  test('generated benchmark route guards are scoped and runnable', () async {
    final refresh = ValueNotifier<int>(0);
    addTearDown(refresh.dispose);
    final router = BenchmarkRouter(refresh: refresh);

    final homeGuards = routeGuards(const HomeRoute(), router);
    expect(homeGuards, hasLength(1));
    expect(homeGuards.single, isA<BenchmarkGuard>());
    expect(
      await RouteGuardChain<AppRoutePath>(
        homeGuards,
      ).canActivate(const HomeRoute()),
      isNull,
    );

    expect(routeGuards(const NotFoundRoute(), router), isEmpty);
    expect(routeGuards(const ModelDetailRoute(id: 42), router), isEmpty);
  });

  testWidgets('benchmark app renders generated route and state APIs', (
    tester,
  ) async {
    await tester.pumpWidget(const BenchmarkApp());
    await tester.pumpAndSettle();

    expect(find.text('Dust benchmark: derive'), findsOneWidget);
    expect(find.text('Generated files: 5000'), findsOneWidget);

    await tester.tap(find.widgetWithText(ListTile, 'serde'));
    await tester.pumpAndSettle();

    expect(find.text('Model 5'), findsOneWidget);
    expect(find.text('Tab: serde'), findsOneWidget);
    expect(find.text('Archived: false'), findsOneWidget);
  });

  testWidgets('benchmark scope exposes typed read without subscribing', (
    tester,
  ) async {
    late BenchmarkViewModel viewModel;
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: BenchmarkViewModelScope(
          args: (_) => const BenchmarkViewModelArgs(),
          create: (_, args) => viewModel = BenchmarkViewModel(args),
          child: Builder(
            builder: (context) {
              return Text(context.readBenchmarkViewModel().state.activeFeature);
            },
          ),
        ),
      ),
    );

    await tester.pump();
    expect(find.text('derive'), findsOneWidget);

    viewModel.selectFeature('http');
    await tester.pump();
    expect(find.text('derive'), findsOneWidget);
  });
}
