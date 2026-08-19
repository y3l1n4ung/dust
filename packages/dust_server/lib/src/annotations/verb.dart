import 'package:meta/meta_meta.dart';

/// The shared shape of every verb annotation.
///
/// Anything that would otherwise need a second annotation on the method is an
/// argument here instead, so a handler carries exactly one.
///
/// ```dart
/// @POST('/', status: 201, summary: 'Create a todo')
/// Future<Todo> create(@Extract(TodosWrite) AuthUser user, @Body() CreateTodo input);
/// ```
@Target({TargetKind.method, TargetKind.function})
abstract base class Verb {
  /// Declares one route.
  const Verb(
    this.path, {
    this.status,
    this.summary,
    this.description,
    this.tags = const [],
    this.middleware = const [],
    this.operationId,
    this.deprecated = false,
    this.hidden = false,
  });

  /// The route path, relative to the controller.
  final String path;

  /// The success status code. Derived from the return type when omitted.
  final int? status;

  /// A one-line summary, for documentation tooling.
  final String? summary;

  /// A longer description, for documentation tooling.
  final String? description;

  /// Tags for this operation, appended to the controller's.
  final List<String> tags;

  /// Middleware applied to this handler, appended to the controller's.
  final List<Object> middleware;

  /// A stable id for this route. Derived from the method name when omitted.
  final String? operationId;

  /// Whether the operation is marked deprecated.
  final bool deprecated;

  /// Whether the route is hidden from generated documentation.
  final bool hidden;

  /// The HTTP method this annotation declares.
  String get method;
}
