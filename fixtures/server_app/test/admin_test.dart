import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:server_app/server_app.dart';
import 'package:test/test.dart';

/// A guard applied to a whole module, rather than named on every handler.
///
/// `routeLayer(fromExtractor(...))` runs the extractor once per request and
/// `Extension<T>` reads the value back — axum's `from_extractor` and
/// `Extension`. Extracted once for every route, and impossible to forget on a
/// route added later.

void main() {
  late ServerHandle server;
  late OrderStore store;

  setUp(() async {
    store = OrderStore([const Order(id: '1', item: 'shirt', quantity: 2)]);
    server = await serveRouter(
      Router()
        ..nest('/admin', adminRoutes())
        ..withState(store),
      InternetAddress.loopbackIPv4,
      0,
    );
  });

  tearDown(() => server.close(drain: const Duration(seconds: 1)));

  String origin() => 'http://${server.address.host}:${server.port}';

  Future<http.Response> fetch(String path, {String? token}) => http.get(
        Uri.parse('${origin()}$path'),
        headers: {if (token != null) 'authorization': 'Bearer $token'},
      );

  group('the guard covers every route', () {
    test('a credential gets through', () async {
      final response = await fetch('/admin/orders', token: 'orders:write');

      expect(response.statusCode, 200);
      expect(jsonDecode(response.body), hasLength(1));
    });

    test('no credential is refused on every route', () async {
      for (final path in ['/admin/orders', '/admin/whoami']) {
        expect((await fetch(path)).statusCode, 401, reason: path);
      }
    });

    test('the wrong scope is refused on every route', () async {
      for (final path in ['/admin/orders', '/admin/whoami']) {
        final response = await fetch(path, token: 'orders:read');

        expect(response.statusCode, 403, reason: path);
        expect(
          jsonDecode(response.body)['error'],
          'requires scope orders:write',
        );
      }
    });

    test('a refused request never reaches a handler', () async {
      await http.delete(
        Uri.parse('${origin()}/admin/orders'),
        headers: const {'authorization': 'Bearer orders:read'},
      );

      expect(store.orders, hasLength(1), reason: 'clearOrders did not run');
    });
  });

  group('the value the guard produced', () {
    test('reaches the handler without a second extraction', () async {
      final response = await fetch('/admin/whoami', token: 'orders:write');

      expect(jsonDecode(response.body), {
        'id': 'u-1',
        'scopes': ['orders:write'],
      });
    });

    test('is the same for every handler in the module', () async {
      // One extraction serves all three routes.
      expect((await fetch('/admin/orders', token: 'orders:write')).statusCode,
          200);
      expect((await fetch('/admin/whoami', token: 'orders:write')).statusCode,
          200);
    });
  });

  group('routeLayer, not layer', () {
    test('an unknown path under the prefix is 404, not 401', () async {
      // A guard that answered 401 here would tell an unauthenticated caller
      // nothing useful and hide a typo in the route table from whoever wrote it.
      expect((await fetch('/admin/nothing')).statusCode, 404);
    });

    test('a known path with an unknown method is 405, not 401', () async {
      final response = await http.post(Uri.parse('${origin()}/admin/whoami'));

      expect(response.statusCode, 405);
    });
  });

  group('when the guard is missing', () {
    test('the cached read is a 500, not a 401', () async {
      // Mounting the handlers without the guard is a wiring mistake. Answering
      // 401 would send whoever is debugging it after credentials that were
      // never the problem.
      final unguarded = await serveRouter(
        Router()
          ..route('/whoami', get(_exposedWhoAmI))
          ..withState(store),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => unguarded.close(drain: const Duration(seconds: 1)));

      final response = await http.get(
        Uri.parse('http://${unguarded.address.host}:${unguarded.port}/whoami'),
      );

      expect(response.statusCode, 500);
    });
  });
}

/// Stands in for the generated handler, which is private to its own library.
Future<Response> _exposedWhoAmI(Request request) async {
  final caller = await const Extension<Caller>().extract(request);
  if (caller case Err(:final error)) return error.intoResponse();

  return jsonResponse({'id': (caller as Ok<Caller, Rejection>).value.id});
}
