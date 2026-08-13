import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

void main() {
  test('route.dart exposes the shopping cart navigation contract', () {
    final routes = <ShoppingRoutePath>[
      const ShoppingCartRoute(),
      const ShoppingCheckoutRoute(),
      const ShoppingOrderConfirmationRoute(orderId: 'ORDER 1/2'),
    ];

    expect(
      routes.map(shoppingRouteLocation).toList(),
      const [
        '/cart',
        '/checkout',
        '/order-confirmation/ORDER%201%2F2',
      ],
    );

    expect(
      parseShoppingRoute(Uri.parse('/cart')).runtimeType,
      ShoppingCartRoute,
    );
    expect(
      parseShoppingRoute(Uri.parse('/checkout')).runtimeType,
      ShoppingCheckoutRoute,
    );

    final confirmation = parseShoppingRoute(
      Uri.parse('/order-confirmation/ORDER%201%2F2'),
    );
    expect(confirmation.runtimeType, ShoppingOrderConfirmationRoute);
    expect(
      (confirmation as ShoppingOrderConfirmationRoute).orderId,
      'ORDER 1/2',
    );
  });

  test('route.dart is the only handwritten owner of the generated barrel', () {
    expect(
      _generatedBarrelReferences(),
      const [
        (
          file: 'lib/route.dart',
          directive: 'import',
          uri: 'route/routes.g.dart',
        ),
        (
          file: 'lib/route.dart',
          directive: 'export',
          uri: 'route/routes.g.dart',
        ),
      ],
    );
  });
}

List<({String directive, String file, String uri})>
    _generatedBarrelReferences() {
  final root = Directory.current;
  final files = ['lib', 'test']
      .map((path) => Directory('${root.path}/$path'))
      .where((directory) => directory.existsSync())
      .expand((directory) => directory.listSync(recursive: true))
      .whereType<File>()
      .where((file) => file.path.endsWith('.dart'))
      .where((file) => !_isGenerated(file))
      .toList()
    ..sort((a, b) => a.path.compareTo(b.path));

  final references = <({String directive, String file, String uri})>[];
  final directive = RegExp(r"^\s*(import|export)\s+'([^']+)';");
  for (final file in files) {
    final relativePath = file.path.substring(root.path.length + 1);
    final lines = file.readAsLinesSync();
    for (final line in lines) {
      final match = directive.firstMatch(line);
      if (match == null) continue;

      final uri = match.group(2)!;
      if (uri != 'route/routes.g.dart') continue;

      references.add((
        file: relativePath,
        directive: match.group(1)!,
        uri: uri,
      ));
    }
  }
  return references;
}

bool _isGenerated(File file) => file.path.endsWith('.g.dart');
