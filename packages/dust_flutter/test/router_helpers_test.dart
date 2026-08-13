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

    test('parse generated route bool URL values', () {
      expect(generatedRouteParseBool('true'), isTrue);
      expect(generatedRouteParseBool('1'), isTrue);
      expect(generatedRouteParseBool('false'), isFalse);
      expect(generatedRouteParseBool('0'), isFalse);
      expect(generatedRouteParseBool(''), isNull);
      expect(generatedRouteParseBool(null), isNull);
      expect(generatedRouteParseBool('yes'), isNull);
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

    test('runtime no-transition builder has zero durations', () {
      const builder = GeneratedNoTransitionBuilder();

      expect(builder.transitionDuration, Duration.zero);
      expect(builder.reverseTransitionDuration, Duration.zero);
    });
  });
}

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
