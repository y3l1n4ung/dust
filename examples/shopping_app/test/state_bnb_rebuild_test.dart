import 'package:dust_flutter/state.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/view_models/app_view_model.dart';

final class CountingStateObserver implements StateObserver {
  final changes = <({Object viewModel, Object? previous, Object? next})>[];
  final effects = <({Object viewModel, Object effect})>[];

  @override
  void onChanged(Object viewModel, Object? previous, Object? next) {
    changes.add((viewModel: viewModel, previous: previous, next: next));
  }

  @override
  void onEffect(Object viewModel, Object effect) {
    effects.add((viewModel: viewModel, effect: effect));
  }
}

void main() {
  testWidgets('same-tab bnb taps do not emit or rebuild selected body', (
    tester,
  ) async {
    final observer = CountingStateObserver();
    var selectedBodyBuilds = 0;

    await tester.pumpWidget(
      MaterialApp(
        home: BnbViewModelScope(
          args: (_) => BnbViewModelArgs(observer: observer),
          create: (_, args) => BnbViewModel(args),
          child: _BnbNoopShell(
            onSelectedBodyBuild: () {
              selectedBodyBuilds += 1;
            },
          ),
        ),
      ),
    );

    expect(find.text('selected:home'), findsOneWidget);
    expect(selectedBodyBuilds, 1);
    expect(observer.changes, isEmpty);

    await tester.tap(find.byKey(const ValueKey<String>('bnb-home')));
    await tester.pump();

    expect(find.text('selected:home'), findsOneWidget);
    expect(selectedBodyBuilds, 1);
    expect(observer.changes, isEmpty);

    await tester.tap(find.byKey(const ValueKey<String>('bnb-cart')));
    await tester.pump();

    expect(find.text('selected:cart'), findsOneWidget);
    expect(selectedBodyBuilds, 2);
    expect(observer.changes, hasLength(1));

    await tester.tap(find.byKey(const ValueKey<String>('bnb-cart')));
    await tester.pump();

    expect(find.text('selected:cart'), findsOneWidget);
    expect(selectedBodyBuilds, 2);
    expect(observer.changes, hasLength(1));
    expect(observer.effects, isEmpty);
  });
}

final class _BnbNoopShell extends StatelessWidget {
  const _BnbNoopShell({required this.onSelectedBodyBuild});

  final VoidCallback onSelectedBodyBuild;

  @override
  Widget build(BuildContext context) {
    final tab = context.watchBnbViewModel().value.currentTab;
    return Scaffold(
      body: BnbViewModelSelector<BnbTab>(
        selector: (state) => state.currentTab,
        builder: (context, selectedTab, child) {
          onSelectedBodyBuild();
          return Center(child: Text('selected:${selectedTab.name}'));
        },
      ),
      bottomNavigationBar: BottomNavigationBar(
        type: BottomNavigationBarType.fixed,
        currentIndex: tab.index,
        onTap: context.readBnbViewModel().selectIndex,
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.home, key: ValueKey<String>('bnb-home')),
            label: 'Home',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.storefront, key: ValueKey<String>('bnb-products')),
            label: 'Products',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.shopping_cart, key: ValueKey<String>('bnb-cart')),
            label: 'Cart',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.receipt_long, key: ValueKey<String>('bnb-orders')),
            label: 'Orders',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.person, key: ValueKey<String>('bnb-profile')),
            label: 'Profile',
          ),
        ],
      ),
    );
  }
}
