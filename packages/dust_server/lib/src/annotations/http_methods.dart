import 'verb.dart';

/// Declares a `GET` route.
final class GET extends Verb {
  /// Declares a `GET` route at [path].
  const GET(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'GET';
}

/// Declares a `POST` route.
final class POST extends Verb {
  /// Declares a `POST` route at [path].
  const POST(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'POST';
}

/// Declares a `PUT` route.
final class PUT extends Verb {
  /// Declares a `PUT` route at [path].
  const PUT(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'PUT';
}

/// Declares a `PATCH` route.
final class PATCH extends Verb {
  /// Declares a `PATCH` route at [path].
  const PATCH(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'PATCH';
}

/// Declares a `DELETE` route.
final class DELETE extends Verb {
  /// Declares a `DELETE` route at [path].
  const DELETE(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'DELETE';
}

/// Declares a `HEAD` route.
final class HEAD extends Verb {
  /// Declares a `HEAD` route at [path].
  const HEAD(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'HEAD';
}

/// Declares an `OPTIONS` route.
final class OPTIONS extends Verb {
  /// Declares an `OPTIONS` route at [path].
  const OPTIONS(
    super.path, {
    super.status,
    super.summary,
    super.description,
    super.tags,
    super.middleware,
    super.operationId,
    super.deprecated,
    super.hidden,
  });

  @override
  String get method => 'OPTIONS';
}
