import 'package:meta/meta_meta.dart';

/// Declares that a library's top-level handlers form a route module.
///
/// This is the function-style counterpart to `@Controller`: it gives the module
/// a prefix without requiring a class to hang it on. Without it, a library of
/// annotated functions still generates a group, but with no prefix of its own.
///
/// ```dart
/// @Routes('/notes')
/// library notes;
///
/// import 'package:dust_server/server.dart';
///
/// part 'notes.g.dart';
///
/// @GET('/{id}')
/// Future<Note> read(@Path() String id, @State() NoteRepo repo) => repo.find(id);
/// ```
@Target({TargetKind.library})
final class Routes {
  /// Declares a route module rooted at [path].
  const Routes(
    this.path, {
    this.name,
    this.tags = const [],
    this.middleware = const [],
  });

  /// The path prefix shared by every handler in the library.
  final String path;

  /// The name of the generated getter.
  ///
  /// Defaults to the file's basename in camel case plus `Routes`, so
  /// `notes.dart` generates `noteRoutes`. Set this when two modules would
  /// otherwise collide in one import scope.
  final String? name;

  /// Tags inherited by every handler, for documentation tooling.
  final List<String> tags;

  /// Middleware applied to every handler in the library.
  final List<Object> middleware;
}
