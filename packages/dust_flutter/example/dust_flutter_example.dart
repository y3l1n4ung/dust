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
