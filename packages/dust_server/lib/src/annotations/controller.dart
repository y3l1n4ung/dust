import 'package:meta/meta_meta.dart';

/// Marks a class as an HTTP controller.
///
/// The path is relative. A controller does not know where it is mounted, so a
/// version prefix is written once at the composition site and the same
/// controller can be mounted more than once.
///
/// ```dart
/// @Controller('/todos', tags: ['todos'])
/// class TodoController with _$TodoController {
///   TodoController(this._repo);
///
///   final TodoRepo _repo;
/// }
/// ```
@Target({TargetKind.classType})
final class Controller {
  /// Declares a controller rooted at [path].
  const Controller(
    this.path, {
    this.tags = const [],
    this.middleware = const [],
  });

  /// The path prefix shared by every handler in the class.
  final String path;

  /// Tags inherited by every handler, for documentation tooling.
  final List<String> tags;

  /// Middleware applied to every handler in the class.
  final List<Object> middleware;
}
