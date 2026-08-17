import 'package:dust_server/server.dart';

export '../support.dart';

/// A handler that answers with [label], so tests can name which route ran.
Handler label(String label) => (request) async => textResponse(label);

/// The route table most router cases are built on.
const sampleRoutes = <String>[
  '/',
  '/a',
  '/a/{id}',
  '/a/{id}/b',
  '/a/{id}/b/{other}',
  '/x/y/z',
];

/// A router serving [sampleRoutes], each answering with its own path.
Router sampleRouter() {
  final router = Router();
  for (final route in sampleRoutes) {
    router.route(route, get(label(route)));
  }
  return router;
}

/// Rewrites `{id}` into the `<id>` form `shelf_router` expects.
String toShelfPath(String path) => path.replaceAllMapped(
      RegExp(r'\{([^/}]+)\}'),
      (match) => '<${match.group(1)}>',
    );

/// A three-route module, the way a generated controller exposes one.
Router todosModule({String prefix = '/todos', Object? metadata}) {
  return Router.module(
    prefix: prefix,
    metadata: metadata,
    routes: [
      Route('GET', '/', label('list')),
      Route('GET', '/{id}', label('get')),
      Route('POST', '/', label('create')),
    ],
  );
}

/// Middleware that appends [name] to [log] on the way in.
Middleware tagged(String name, List<String> log) {
  return (Handler inner) {
    return (Request request) async {
      log.add(name);
      return inner(request);
    };
  };
}

/// A const-expressible layer, as an annotation would carry it.
final class HeaderLayer implements Layer {
  /// Sets [name] to [value] on the way out.
  const HeaderLayer(this.name, this.value);

  /// The header name.
  final String name;

  /// The header value.
  final String value;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final response = await inner(request);
        return response.change(headers: {name: value});
      };
    };
  }
}
