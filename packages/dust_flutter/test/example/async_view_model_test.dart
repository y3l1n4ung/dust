import 'dart:async';

import 'package:dust_dart/fp.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../example/async_view_model.dart';

/// A repository whose answers the test hands out one at a time.
final class _ScriptedRepository implements CatalogRepository {
  final pending = <Completer<Result<List<String>, CatalogError>>>[];

  @override
  Future<Result<List<String>, CatalogError>> products() {
    final completer = Completer<Result<List<String>, CatalogError>>();
    pending.add(completer);
    return completer.future;
  }

  void answer(Result<List<String>, CatalogError> result) {
    pending.removeAt(0).complete(result);
  }
}

void main() {
  late _ScriptedRepository repository;
  late CatalogViewModel viewModel;

  Future<void> pumpPage(WidgetTester tester) async {
    repository = _ScriptedRepository();
    viewModel = CatalogViewModel(CatalogArgs(repository: repository));
    await tester.pumpWidget(
      MaterialApp(home: CatalogPage(viewModel: viewModel)),
    );
  }

  testWidgets('shows a spinner, then the products', (tester) async {
    await pumpPage(tester);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    repository.answer(const Ok(['Tea', 'Coffee']));
    await tester.pump();

    expect(find.text('Tea'), findsOneWidget);
    expect(find.text('Coffee'), findsOneWidget);
  });

  testWidgets('an offline failure keeps its type and can be retried',
      (tester) async {
    await pumpPage(tester);
    repository.answer(const Err(CatalogOffline()));
    await tester.pump();

    expect(viewModel.state.error, isA<CatalogOffline>());
    expect(find.text('You are offline.'), findsOneWidget);

    await tester.tap(find.text('Retry'));
    repository.answer(const Ok(['Tea']));
    await tester.pump();

    expect(find.text('Tea'), findsOneWidget);
  });

  testWidgets('a forbidden failure offers no retry', (tester) async {
    await pumpPage(tester);
    repository.answer(const Err(CatalogForbidden()));
    await tester.pump();

    expect(viewModel.state.error, isA<CatalogForbidden>());
    expect(find.text('You cannot see this catalog.'), findsOneWidget);
    expect(find.text('Retry'), findsNothing);
  });

  testWidgets('a refresh keeps the products on screen', (tester) async {
    await pumpPage(tester);
    repository.answer(const Ok(['Tea']));
    await tester.pump();

    await tester.tap(find.byTooltip('Refresh'));
    await tester.pump();

    expect(find.text('Tea'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsOneWidget);

    repository.answer(const Ok(['Tea', 'Cocoa']));
    await tester.pump();

    expect(find.text('Cocoa'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsNothing);
  });
}
