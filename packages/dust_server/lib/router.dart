/// Routing: the request tree, how it is composed, and how it is inspected.
///
/// Import this when an application only wires routes and needs nothing else,
/// for example a composition file that assembles generated modules.
///
/// ```dart
/// import 'package:dust_server/router.dart';
///
/// final app = Router()
///   ..route('/health', get(health))
///   ..nest('/api/v1', v1)
///   ..layer(Cors())
///   ..withState(repo);
/// ```
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'package:shelf/shelf.dart' show Handler, Middleware, Request, Response;

export 'src/router/router.dart';
