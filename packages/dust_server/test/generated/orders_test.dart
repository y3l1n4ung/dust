import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

import 'orders.dart';

/// Drives the hand-emitted `orders.g.dart` stand-in over a real socket.
///
/// This is the spec the plugin has to satisfy. When generated output replaces
/// the second half of `orders.dart` and these still pass, the plugin is done.

void main() {
  late ServerHandle server;
  late OrderStore store;

  setUp(() async {
    store = OrderStore([const Order('1', 'shirt', 2)]);
    final app = Router()
      ..nest('/orders', orderRoutes())
      ..withState(store);

    server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
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
    final encoded = body == null ? null : jsonEncode(body);

    return switch (method) {
      'GET' => http.get(uri, headers: headers),
      'POST' => http.post(uri, headers: headers, body: encoded),
      'DELETE' => http.delete(uri, headers: headers),
      _ => throw ArgumentError(method),
    };
  }

  group('GET /orders', () {
    test('lists what the store holds', () async {
      final response = await send('GET', '/orders');

      expect(response.statusCode, 200);
      expect(jsonDecode(response.body), [
        {'id': '1', 'item': 'shirt', 'quantity': 2},
      ]);
    });

    test('an absent optional query is not a rejection', () async {
      // `@Query('item') String?` — nullable, so absent means null rather than
      // a 400, and the handler decides what that means.
      expect((await send('GET', '/orders')).statusCode, 200);
    });

    test('the query filters when it is there', () async {
      expect(jsonDecode((await send('GET', '/orders?item=shirt')).body),
          hasLength(1));
      expect(jsonDecode((await send('GET', '/orders?item=hat')).body), isEmpty);
    });
  });

  group('GET /orders/{id}', () {
    test('reads one order', () async {
      final response = await send('GET', '/orders/1');

      expect(response.statusCode, 200);
      expect(jsonDecode(response.body)['item'], 'shirt');
    });

    test('an Err supplies its own status', () async {
      // The plugin does not decide this is a 404. The returned Rejection does.
      final response = await send('GET', '/orders/99');

      expect(response.statusCode, 404);
      expect(jsonDecode(response.body)['error'], 'no such order');
    });
  });

  group('POST /orders', () {
    test('places an order and answers 201', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat', 'quantity': 3},
        token: 'orders:write',
      );

      expect(response.statusCode, 201);
      expect(jsonDecode(response.body)['id'], '2');
      expect(store.orders, hasLength(2));
    });

    test('a missing credential is 401, before the body is read', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat'},
      );

      expect(response.statusCode, 401);
      expect(store.orders, hasLength(1), reason: 'nothing was written');
    });

    test('a broken payload is 422 naming every field', () async {
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

    test('quantity defaults when it is left out', () async {
      final response = await send(
        'POST',
        '/orders',
        body: const {'item': 'hat'},
        token: 'orders:write',
      );

      expect(jsonDecode(response.body)['quantity'], 1);
    });
  });

  group('DELETE /orders/{id}', () {
    test('answers 204 with no body', () async {
      final response = await send('DELETE', '/orders/1', token: 'todos:write');

      expect(response.statusCode, 204);
      expect(response.body, isEmpty);
      expect(store.orders, isEmpty);
    });

    test('the scoped extractor refuses the wrong scope with 403', () async {
      // `@Extract(TodosWrite)` carries its configuration, so the scope check
      // happens in the extractor rather than in the handler.
      final response = await send('DELETE', '/orders/1', token: 'orders:read');

      expect(response.statusCode, 403);
      expect(store.orders, hasLength(1));
    });
  });

  group('the module is a function, not a getter', () {
    test('each call builds a fresh Router, so two apps do not share one',
        () async {
      // A Router is mutable and gets sealed when its handler is read. Two
      // servers in one isolate is every test file, so each needs its own.
      expect(identical(orderRoutes(), orderRoutes()), isFalse);
    });

    test('it can be passed where a RouterFactory is required', () async {
      // serveCluster takes Router Function(). A getter is not a value and
      // cannot be passed at all, so a clustered app could not use the module.
      final cluster = await serveCluster(
        orderRoutes,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 1,
      );

      expect(cluster.port, greaterThan(0));
      await cluster.close(drain: const Duration(seconds: 1));
    });

    test('configuring the result is visibly a mistake, not a silent one',
        () async {
      // With a getter, `orderRoutes.withState(x)` compiled, configured a
      // throwaway, and answered 500 at request time saying the state was never
      // attached. Written as a call, the same mistake reads as one.
      final configured = orderRoutes()..withState(OrderStore());

      // State on the module itself does reach its own routes.
      final server = await serveRouter(
        Router()..nest('/orders', configured),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final response = await http.get(
        Uri.parse('http://${server.address.host}:${server.port}/orders'),
      );

      expect(response.statusCode, 200);
    });
  });

  group('the route table', () {
    test('reports every route it declared, for tooling', () async {
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

    test('an unknown method on a known path is 405 with Allow', () async {
      final response = await http.put(Uri.parse('${origin()}/orders/1'));

      expect(response.statusCode, 405);
      expect(response.headers['allow'], contains('DELETE'));
    });
  });
}
