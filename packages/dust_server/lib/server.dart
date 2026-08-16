/// Runtime support for Dust-generated Dart HTTP servers.
///
/// One import carries the annotations a controller needs, the extractors the
/// generated code calls, the composition an application wires by hand, and the
/// `shelf` types those touch.
///
/// ```dart
/// import 'package:dust_server/server.dart';
///
/// @Controller('/todos', tags: ['todos'])
/// class TodoController with _$TodoController {
///   TodoController(this._repo);
///
///   final TodoRepo _repo;
///
///   @GET('/{id}')
///   Future<Result<Todo, NotFound>> get(
///     @Extract(BearerAuth) AuthUser user,
///     @Path() String id,
///   ) =>
///       _repo.find(user.id, id);
/// }
/// ```
///
/// The pieces are also importable one at a time, for files that need only part
/// of the surface:
///
/// | Library | Contents |
/// | :--- | :--- |
/// | `annotations.dart` | what the generator reads |
/// | `extraction.dart` | extractors and their interfaces |
/// | `response.dart` | rejections, encoders, `IntoResponse` |
/// | `router.dart` | the route tree and its composition |
/// | `layers.dart` | deadlines, request ids, access records |
/// | `serving.dart` | running and draining a server |
/// | `ws.dart` | WebSocket routes and sessions |
/// | `templating.dart` | rendering HTML from templates |
library;

export 'annotations.dart';
export 'extraction.dart';
export 'layers.dart';
export 'response.dart';
export 'router.dart';
export 'serving.dart';
export 'templating.dart';
export 'ws.dart';
