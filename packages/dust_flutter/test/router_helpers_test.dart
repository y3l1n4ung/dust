import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('generated route helpers', () {
    test('preserve unknown query parameters and fragment', () {
      final route = Object();
      final uri = Uri.parse('/products?tab=reviews&utm=mail&utm=app#details');

      final preserved = withGeneratedRouteUriExtras(
        route,
        uri,
        const {'tab'},
      );

      expect(identical(preserved, route), isTrue);
      expect(
        generatedRoutePath(
          ['products'],
          queryParameters: {'tab': 'reviews'},
          uriExtras: generatedRouteUriExtrasOf(route),
        ),
        '/products?tab=reviews&utm=mail&utm=app#details',
      );
    });

    test('does not store route extras when URI has no unknown values', () {
      final route = Object();

      withGeneratedRouteUriExtras(
        route,
        Uri.parse('/products?tab=reviews'),
        const {'tab'},
      );

      expect(generatedRouteUriExtrasOf(route), isNull);
    });

    test('build generated paths with encoded segments and fragment-only extras',
        () {
      final route = Object();
      withGeneratedRouteUriExtras(route, Uri.parse('/#top'), const <String>{});

      expect(
        generatedRoutePath(
          ['orders', 'ORDER 1/2'],
          uriExtras: generatedRouteUriExtrasOf(route),
        ),
        '/orders/ORDER%201%2F2#top',
      );
      expect(generatedRoutePath(const []), '/');
    });

    test('parse generated route bool URL values', () {
      expect(generatedRouteParseBool('true'), isTrue);
      expect(generatedRouteParseBool('1'), isTrue);
      expect(generatedRouteParseBool('false'), isFalse);
      expect(generatedRouteParseBool('0'), isFalse);
      expect(generatedRouteParseBool(''), isNull);
      expect(generatedRouteParseBool(null), isNull);
      expect(generatedRouteParseBool('yes'), isNull);
    });

    test('parse generated route enum URL values', () {
      expect(
        generatedRouteParseEnum(_RouteTab.values, 'reviews'),
        _RouteTab.reviews,
      );
      expect(generatedRouteParseEnum(_RouteTab.values, 'missing'), isNull);
      expect(generatedRouteParseEnum(_RouteTab.values, ''), isNull);
      expect(generatedRouteParseEnum(_RouteTab.values, null), isNull);
    });

    test('parse generated repeated int query values', () {
      expect(generatedRouteParseIntList(['1', '2', '3']), [1, 2, 3]);
      expect(generatedRouteParseIntList(['1', 'bad']), isNull);
      expect(generatedRouteParseIntList(null), isNull);
    });

    test('compare generated list route defaults by value', () {
      expect(generatedRouteListEquals(['sale'], ['sale']), isTrue);
      expect(generatedRouteListEquals(['sale'], ['new']), isFalse);
      expect(generatedRouteListEquals(['sale'], ['sale', 'new']), isFalse);
    });

    test('detect shell metadata mismatches', () {
      const routes = [
        GeneratedRoute('/', page: _HomePage, shell: _Shell),
        GeneratedRoute('plain', page: _PlainPage),
      ];

      expect(
        generatedRouteShellsMatch(
          routes,
          const {_HomePage: _Shell, _PlainPage: null},
        ),
        isTrue,
      );
      expect(
        generatedRouteShellsMatch(routes, const {_HomePage: null}),
        isFalse,
      );
    });

    test('exposes route metadata names branches and result types', () {
      const route = GeneratedRoute(
        '/picker',
        page: _PlainPage,
        name: 'picker',
        branch: 'mainTabs',
        resultType: 'bool',
      );
      const defaultRoute = GeneratedRoute('/home');
      const debugInfo = RouteDebugInfo(
        name: 'picker',
        branch: 'mainTabs',
        resultType: 'bool',
      );

      expect(route.name, 'picker');
      expect(route.branch, 'mainTabs');
      expect(route.resultType, 'bool');
      expect(defaultRoute.resultType, 'void');
      expect(debugInfo.resultType, 'bool');
      expect(const RouteDebugInfo().resultType, 'void');
    });

    test('runtime no-transition builder has zero durations', () {
      const builder = GeneratedNoTransitionBuilder();

      expect(builder.transitionDuration, Duration.zero);
      expect(builder.reverseTransitionDuration, Duration.zero);
    });
  });
}

enum _RouteTab { overview, reviews }

final class _HomePage extends StatelessWidget {
  const _HomePage();

  @override
  Widget build(BuildContext context) => const SizedBox.shrink();
}

final class _PlainPage extends StatelessWidget {
  const _PlainPage();

  @override
  Widget build(BuildContext context) => const SizedBox.shrink();
}

final class _Shell extends StatelessWidget {
  const _Shell({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) => child;
}
