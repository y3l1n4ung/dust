import 'package:dust_flutter/i18n.dart';
import 'package:dust_flutter/route.dart';
import 'package:dust_flutter/state.dart';
import 'package:flutter/widgets.dart';

/// State type used by a generated ViewModel.
final class CounterState {
  /// Creates counter state.
  const CounterState({this.count = 0});

  /// Current counter value.
  final int count;
}

/// Source class annotated for Dust state generation.
///
/// In an app, add `part 'counter_view_model.g.dart';`, run `dust build`, and
/// extend the generated `$CounterViewModel` base shown in the state guide.
@ViewModel(state: CounterState)
final class CounterViewModelSource {
  /// Creates the annotated source class.
  const CounterViewModelSource();
}

/// A page annotated for typed route generation.
@AppRoute('/products/:id', name: 'product')
final class ProductPage extends StatelessWidget {
  /// Creates a product detail page.
  const ProductPage({required this.id, this.tab, super.key});

  /// Product id parsed from the path segment.
  final int id;

  /// Optional query parameter.
  final String? tab;

  @override
  Widget build(BuildContext context) {
    return Text('Product $id ${tab ?? ''}');
  }
}

/// Search route demonstrating nullable and default-valued query parameters.
@AppRoute('/products', name: 'productSearch', guards: [])
final class ProductSearchPage extends StatelessWidget {
  /// Creates a product search page.
  const ProductSearchPage({
    this.query,
    this.page = 1,
    this.showArchived = false,
    super.key,
  });

  /// Search text encoded as a nullable query parameter.
  final String? query;

  /// Current page encoded as a default-valued query parameter.
  final int page;

  /// Whether archived products are shown.
  final bool showArchived;

  @override
  Widget build(BuildContext context) {
    return Text('Search ${query ?? ''} page $page archived $showArchived');
  }
}

/// Fullscreen picker route that returns a selected product id.
@AppRoute(
  '/product-picker',
  name: 'productPicker',
  result: int,
  guards: [],
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
final class ProductPickerPage extends StatelessWidget {
  /// Creates the demo product picker page.
  const ProductPickerPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Pick product');
  }
}

/// Shared shell widget used by the shell route examples below.
class DemoShell extends StatelessWidget {
  /// Creates a demo shell.
  const DemoShell({required this.child, super.key});

  /// Page rendered inside the shell body.
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('Demo shell'),
        Expanded(child: child),
      ],
    );
  }
}

/// Home route wrapped by [DemoShell].
@AppRoute('/', name: 'home', shell: DemoShell, guards: [])
final class HomePage extends StatelessWidget {
  /// Creates the demo home page.
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Home');
  }
}

/// Settings route inherits [DemoShell] from the root shell route.
@AppRoute('/settings', name: 'settings', guards: [])
final class SettingsPage extends StatelessWidget {
  /// Creates the demo settings page.
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Settings');
  }
}

/// Child route that also inherits [DemoShell] from [HomePage].
@AppRoute('/orders', name: 'orders')
final class OrdersPage extends StatelessWidget {
  /// Creates the demo orders page.
  const OrdersPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Orders');
  }
}

/// Shell used when a child subtree needs a more specific layout.
class ReportsShell extends StatelessWidget {
  /// Creates a reports shell.
  const ReportsShell({required this.child, super.key});

  /// Page rendered inside the reports shell.
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('Reports shell'),
        Expanded(child: child),
      ],
    );
  }
}

/// Child route overriding the inherited shell with [ReportsShell].
@AppRoute('/reports', name: 'reports', shell: ReportsShell)
final class ReportsPage extends StatelessWidget {
  /// Creates the demo reports page.
  const ReportsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Reports');
  }
}

/// Renders an async lifecycle state produced by generated async ViewModels.
Widget renderProducts(AsyncState<List<String>> state) {
  return switch (state) {
    AsyncInitial<List<String>>() ||
    AsyncLoading<List<String>>() =>
      const Text('Loading'),
    AsyncData<List<String>>(:final data) => Text('Loaded ${data.length}'),
    AsyncFailure<List<String>>(:final error, :final previousData) => Text(
        'Error: $error. Previous count: ${previousData?.length ?? 0}',
      ),
  };
}

/// Creates an in-memory i18n controller for tests or demos.
I18nController createDemoI18n() {
  return I18nController(
    config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
    bundles: const [
      I18nBundle(
        locale: 'en',
        namespace: 'home',
        messages: {'title': 'Home'},
      ),
    ],
  );
}
