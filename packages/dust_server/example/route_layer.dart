import 'dart:io';

import 'package:dust_server/server.dart';

/// A layer that runs only for routes that matched.
///
/// `layer` wraps **everything**, including the router's own 404. `routeLayer`
/// wraps only a route that was found. For a guard, that difference is the whole
/// point:
///
/// | Request | with `layer` | with `routeLayer` |
/// | :--- | :--- | :--- |
/// | `/admin/orders`, no token | 401 | 401 |
/// | `/admin/typo`, no token | **401** | **404** |
///
/// The second row is what you want. A guard over the whole router answers 401
/// for paths that do not exist, so a typo in your own route table looks like an
/// auth problem — and you spend an afternoon on the credential instead of the
/// spelling.
///
/// It also does not leak: an unauthenticated caller learns `/admin/typo` is not
/// a route, which they could have learned by asking any other unmatched path.
///
/// Run it with `dart run example/route_layer.dart`:
///
/// ```bash
/// curl -si localhost:8080/admin/orders                        # 401
/// curl -s  localhost:8080/admin/orders -H 'authorization: Bearer staff'
/// curl -si localhost:8080/admin/typo                          # 404, not 401
/// curl -s  localhost:8080/health                              # untouched
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  final admin = Router()
    ..routeLayer(requireStaff)
    ..route('/orders', get(listOrders));

  return Router()
    ..nest('/admin', admin)
    ..route('/health', get(health));
}

/// `GET /admin/orders`
List<String> listOrders(Request request) => const ['order-1'];

/// `GET /health` — outside the guard, so no credential is asked for.
Map<String, Object?> health(Request request) => const {'status': 'ok'};

/// Refuses anything without the staff token.
Handler requireStaff(Handler inner) {
  return (Request request) async {
    switch (await const BearerTokenExtractable().extract(request)) {
      case Err(:final error):
        return error.intoResponse();
      case Ok(value: final token):
        if (token != 'staff') {
          return const Rejection.forbidden('not a staff token').intoResponse();
        }
        return inner(request);
    }
  };
}
