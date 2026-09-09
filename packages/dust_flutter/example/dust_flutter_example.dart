import 'package:dust_flutter/i18n.dart';
import 'package:dust_flutter/route.dart';
import 'package:dust_flutter/state.dart';
import 'package:flutter/widgets.dart';

part 'dust_flutter_example_pages.dart';

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

/// Public invite route for signed-out magic links.
@AppRoute('/invite/:code', name: 'invite', guards: [])
final class InvitePage extends StatelessWidget {
  /// Creates an invite page.
  const InvitePage({required this.code, this.team, super.key});

  /// Invite token from the URL path.
  final String code;

  /// Optional team hint from the query string.
  final String? team;

  @override
  Widget build(BuildContext context) {
    return Text('Invite $code ${team ?? ''}');
  }
}

/// Organization-scoped detail route with path and query parameters.
@AppRoute('/orgs/:orgId/projects/:projectId', name: 'orgProject')
final class OrgProjectPage extends StatelessWidget {
  /// Creates an organization project page.
  const OrgProjectPage({
    required this.orgId,
    required this.projectId,
    this.tab,
    super.key,
  });

  /// Organization or workspace id from the path.
  final String orgId;

  /// Project id from the path.
  final int projectId;

  /// Optional selected tab from the query string.
  final String? tab;

  @override
  Widget build(BuildContext context) {
    return Text('Org $orgId project $projectId ${tab ?? ''}');
  }
}
