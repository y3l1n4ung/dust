import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../example/stale_actions.dart';

void main() {
  late Map<String, Completer<List<String>>> pending;
  late SearchViewModel viewModel;

  Future<void> pumpPage(WidgetTester tester) async {
    pending = {};
    viewModel = SearchViewModel(
      SearchArgs(search: (query) => (pending[query] = Completer()).future),
    );
    await tester.pumpWidget(
      MaterialApp(home: SearchPage(viewModel: viewModel)),
    );
  }

  testWidgets('a slow answer for an older query is dropped', (tester) async {
    await pumpPage(tester);

    await tester.enterText(find.byType(TextField), 'c');
    await tester.enterText(find.byType(TextField), 'co');

    pending['co']!.complete(['Cocoa', 'Coffee', 'Cola']);
    await tester.pump();
    pending['c']!.complete(['Cocoa', 'Coffee', 'Cola', 'Cider']);
    await tester.pump();

    expect(viewModel.state.query, 'co');
    expect(find.text('Cider'), findsNothing);
    expect(find.text('Cola'), findsOneWidget);
  });

  testWidgets('an empty search shows a snackbar once', (tester) async {
    await pumpPage(tester);

    await tester.enterText(find.byType(TextField), 'x');
    pending['x']!.complete(const []);
    await tester.pump();
    await tester.pump();

    expect(find.text('Nothing starts with "x".'), findsOneWidget);
  });

  test('the sample search finds names by prefix, even for an empty query',
      () async {
    expect(await slowerForShorterQueries('co'), ['Cocoa', 'Coffee', 'Cola']);
    expect(await slowerForShorterQueries(''), hasLength(4));
  });
}
