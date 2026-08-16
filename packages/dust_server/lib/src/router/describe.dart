import 'flatten.dart';
import 'mounted_route.dart';
import 'router_base.dart';

/// Route introspection, kept off the router's core so that serving requests
/// and describing them stay separable.
extension RouterDescribe on Router {
  /// Every route below this router, with the path it serves.
  ///
  /// Each entry carries the metadata of every router it was mounted through,
  /// outermost first, so a description attached at any level is recoverable.
  ///
  /// ```dart
  /// for (final route in app.describe()) {
  ///   print('${route.method} ${route.path}');
  /// }
  /// ```
  List<MountedRoute> describe() {
    return [
      for (final route in flattenRoutes(this))
        MountedRoute(
          method: route.method,
          path: route.path,
          metadata: route.metadata,
        ),
    ];
  }
}
