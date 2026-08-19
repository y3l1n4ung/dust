import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:server_app/server_app.dart';
import 'package:test/test.dart';

/// Drives the hand-emitted `orders.g.dart` over a real socket.
///
/// The models it decodes are generated for real: `NewOrder.validate()` and
/// `NewOrder.deserialize` come from `models.g.dart`, which `dust build`
/// produces from the annotations. Only the routing half is hand-written, and
/// this file is the spec it has to satisfy.

void main() {
  late ServerHandle server;
  late OrderStore store;

  setUp(() async {
    store = OrderStore([const Order(id: '1', item: 'shirt', quantity: 2)]);
    server = await serveRouter(
      Router()
        ..nest('/orders', orderRoutes())
        ..withState(store),
      InternetAddress.loopbackIPv4,
      0,
    );
  });

  tearDown(() => server.close(drain: const Duration(seconds: 1)));

  String origin() => 'http://${server.address.host}:${server.port}';

  Future<http.Response> send(
    String method,
    String path, {
    Object? body,
    String? token,
  }) {
    final uri = Uri.parse('${origin()}$path');
    final headers = {
      if (body != null) 'content-type': 'application/json',
      if (token != null) 'authorization': 'Bearer $token',
    };
    final encoded =
        body is String ? body : (body == null ? null : jsonEncode(body));

    return switch (method) {
      'GET' => http.get(uri, headers: headers),
      'POST' => http.post(uri, headers: headers, body: encoded),
      'DELETE' => http.delete(uri, headers: headers),
      _ => throw ArgumentError(method),
    };
  }

  group('reading', () {
    test('lists what the store holds', () async {
      final response = await send('GET', '/orders');

      expect(response.statusCode, 200);
      expect(jsonDecode(response.body), [
        {'id': '1', 'item': 'shirt', 'quantity': 2},
      ]);
    });

    test('an absent optional query is not a rejection', () async {
      expect((await send('GET', '/orders')).statusCode, 200);
    });

    test('an empty query value is not the same as absent', () async {
      // `?item=` is present and empty, so it filters to nothing rather than
      // falling back to the unfiltered list.
      expect(jsonDecode((await send('GET', '/orders?item=')).body), isEmpty);
    });

    test('a repeated query key takes one value rather than failing', () async {
      expect(
          (await send('GET', '/orders?item=shirt&item=hat')).statusCode, 200);
    });

    test('reads one order', () async {
      expect(
          jsonDecode((await send('GET', '/orders/1')).body)['item'], 'shirt');
    });

    test('an Err supplies its own status', () async {
      final response = await send('GET', '/orders/99');

      expect(response.statusCode, 404);
      expect(jsonDecode(response.body)['error'], 'no such order');
    });

    test('a percent-encoded id stays one segment', () async {
      // `%2F` is a slash in the value, not a separator, so this is a lookup
      // that misses rather than a route that does not exist.
      final response = await send('GET', '/orders/a%2Fb');

      expect(response.statusCode, 404);
      expect(jsonDecode(response.body)['error'], 'no such order');
    });
  });

  group('the generated validator', () {
    test('accepts a valid payload and answers 201', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat', 'quantity': 3},
        token: 'orders:write',
      );

      expect(response.statusCode, 201);
      expect(jsonDecode(response.body), {
        'id': '2',
        'item': 'hat',
        'quantity': 3,
      });
    });

    test('reports every broken rule at once', () async {
      // Both messages come from the annotations in models.dart, through the
      // validate() that dust build generated.
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': '', 'quantity': 99},
        token: 'orders:write',
      );

      expect(response.statusCode, 422);
      expect(jsonDecode(response.body)['fields'], {
        'item': ['is required'],
        'quantity': ['must be 1 to 10'],
      });
    });

    test('enforces the lower bound as well as the upper', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat', 'quantity': 0},
        token: 'orders:write',
      );

      expect(response.statusCode, 422);
      expect(jsonDecode(response.body)['fields'], {
        'quantity': ['must be 1 to 10'],
      });
    });

    test('accepts the boundaries', () async {
      for (final quantity in [1, 10]) {
        final response = await send(
          'POST',
          '/orders',
          body: {'item': 'hat', 'quantity': quantity},
          token: 'orders:write',
        );

        expect(response.statusCode, 201, reason: 'quantity $quantity');
      }
    });

    test('applies the generated default when a field is left out', () async {
      // `@SerDe(defaultValue: 1)` — the decoder supplies it, not the handler.
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat'},
        token: 'orders:write',
      );

      expect(jsonDecode(response.body)['quantity'], 1);
    });

    test('a missing required field is 422, not 500', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'quantity': 2},
        token: 'orders:write',
      );

      expect(response.statusCode, 422);
    });

    test('a wrong type is 422, not 500', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat', 'quantity': 'three'},
        token: 'orders:write',
      );

      expect(response.statusCode, 422);
    });

    test('a body that is not JSON is 400', () async {
      final response = await send(
        'POST',
        '/orders',
        body: 'not json',
        token: 'orders:write',
      );

      expect(response.statusCode, 400);
    });

    test('a JSON array where an object belongs is 422', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const [1, 2],
        token: 'orders:write',
      );

      expect(response.statusCode, 422);
    });
  });

  group('extractor order', () {
    test('a missing credential is 401 before the body is read', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat'},
      );

      expect(response.statusCode, 401);
      expect(store.orders, hasLength(1), reason: 'nothing was written');
    });

    test('the credential is checked before the payload is validated', () async {
      // Both are wrong. The 401 wins because @Extract is the first parameter.
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': '', 'quantity': 99},
      );

      expect(response.statusCode, 401);
    });

    test('a scoped extractor answers 403, not 401', () async {
      final response = await send(
        'DELETE',
        '/orders/1',
        token: 'orders:read',
      );

      expect(response.statusCode, 403);
      expect(jsonDecode(response.body)['error'], 'requires scope orders:write');
      expect(store.orders, hasLength(1));
    });
  });

  group('writing', () {
    test('a void handler answers 204 with no body', () async {
      final response = await send('DELETE', '/orders/1', token: 'orders:write');

      expect(response.statusCode, 204);
      expect(response.body, isEmpty);
      expect(store.orders, isEmpty);
    });

    test('deleting something absent is still 204', () async {
      // The handler returns void either way; it does not report a miss.
      expect(
        (await send('DELETE', '/orders/99', token: 'orders:write')).statusCode,
        204,
      );
    });

    test('a placed order is readable afterwards', () async {
      await send(
        'POST',
        '/orders',
        body: const {'item': 'hat', 'quantity': 2},
        token: 'orders:write',
      );

      expect(jsonDecode((await send('GET', '/orders/2')).body)['item'], 'hat');
    });
  });

  group('the module', () {
    test('describes every route it declared', () {
      final routes = (Router()..nest('/orders', orderRoutes())).describe();

      expect(
        routes.map((route) => '${route.method} ${route.path}').toList(),
        [
          'GET /orders',
          'GET /orders/{id}',
          'POST /orders',
          'DELETE /orders/{id}',
        ],
      );
    });

    test('answers 405 with Allow for a method it does not serve', () async {
      final response = await http.put(Uri.parse('${origin()}/orders/1'));

      expect(response.statusCode, 405);
      expect(response.headers['allow'], contains('DELETE'));
    });

    test('each call builds a fresh Router', () {
      expect(identical(orderRoutes(), orderRoutes()), isFalse);
    });

    test('can be passed where a RouterFactory is required', () async {
      final cluster = await serveCluster(
        orderRoutes,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 1,
      );

      expect(cluster.port, greaterThan(0));
      await cluster.close(drain: const Duration(seconds: 1));
    });
  });
}
