/// The routing feature, gathered in one place.
///
/// Internal machinery stays out: the matcher, the flattener, the composer, and
/// the mutable half of a router are imported directly by the files that need
/// them, so nothing here leaks past the package boundary.
library;

export 'describe.dart';
export 'method_router.dart';
export 'middleware.dart';
export 'mounted_route.dart';
export 'path_segment.dart';
export 'paths.dart';
export 'route.dart';
export 'router_base.dart';
export 'service.dart';
