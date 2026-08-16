import 'package:shelf/shelf.dart';

/// One route in a router.
final class Route {
  /// Declares a route serving [method] at [path].
  const Route(this.method, this.path, this.handler) : isMount = false;

  /// Declares a subtree handled by [handler], rooted at [path].
  ///
  /// The handler sees paths relative to [path], the way `shelf_router.mount`
  /// and axum's `nest_service` both work, so a handler written for the root of
  /// its own world keeps working when mounted deeper.
  const Route.mount(this.path, this.handler)
      : method = anyMethod,
        isMount = true;

  /// The method a route matches when it serves every method.
  static const anyMethod = '*';

  /// The uppercase HTTP method, or [anyMethod].
  final String method;

  /// The path, relative to the enclosing router.
  final String path;

  /// The handler to run.
  final Handler handler;

  /// Whether [handler] owns everything below [path].
  final bool isMount;
}
