import 'package:meta/meta_meta.dart';

/// Binds a handler parameter to a custom extractor.
///
/// The argument is the extractor type, not an instance. Dust emits
/// `const MyExtractor().extract(request)` and casts the result to the
/// parameter's type, so the analyzer catches an extractor that produces
/// something else.
///
/// ```dart
/// Future<Todo> create(@Extract(TodosWrite) AuthUser user, @Body() CreateTodo input);
/// ```
@Target({TargetKind.parameter})
final class Extract {
  /// Binds the parameter to [extractor].
  const Extract(this.extractor);

  /// The extractor type, which must implement `FromRequestParts<T>` and expose a
  /// zero-argument `const` constructor.
  final Type extractor;
}

/// Binds a parameter to a router-captured path segment.
@Target({TargetKind.parameter})
final class Path {
  /// Reads the `{name}` segment, defaulting to the parameter name.
  const Path([this.name]);

  /// The path parameter name.
  final String? name;
}

/// Binds a parameter to one query value.
@Target({TargetKind.parameter})
final class Query {
  /// Reads the query value named [name].
  const Query(this.name);

  /// The query key.
  final String name;
}

/// Binds a parameter to the whole query map.
@Target({TargetKind.parameter})
final class Queries {
  /// Reads every query pair.
  const Queries();
}

/// Binds a parameter to the raw, undecoded query string.
@Target({TargetKind.parameter})
final class RawQuery {
  /// Reads the query component of the request URI.
  const RawQuery();
}

/// Binds a parameter to one header.
@Target({TargetKind.parameter})
final class Header {
  /// Reads the header named [name].
  const Header(this.name);

  /// The header name, matched case-insensitively.
  final String name;
}

/// Binds a parameter to the whole header map.
@Target({TargetKind.parameter})
final class HeaderMap {
  /// Reads every header.
  const HeaderMap();
}

/// Binds a parameter to the typed request body.
///
/// The media type is JSON unless the handler carries [FormUrlEncoded].
@Target({TargetKind.parameter})
final class Body {
  /// Reads and decodes the body.
  const Body();
}

/// Binds a parameter to one form field.
@Target({TargetKind.parameter})
final class Field {
  /// Reads the form field named [name].
  const Field(this.name);

  /// The field name.
  final String name;
}

/// Binds a parameter to one multipart part.
@Target({TargetKind.parameter})
final class Part {
  /// Reads the multipart part named [name].
  const Part(this.name);

  /// The part name.
  final String name;
}

/// Binds a parameter to the undecoded body.
///
/// The parameter type selects the form: `Uint8List` for bytes, `String` for
/// UTF-8 text, `Stream<List<int>>` for an unread stream.
@Target({TargetKind.parameter})
final class RawBody {
  /// Reads the body without decoding it.
  const RawBody();
}

/// Binds a parameter to application state attached with `withState`.
///
/// The parameter's type selects the value, so nothing names a key on either
/// side. This is how a library of top-level handlers gets what a controller
/// class holds in a field.
///
/// ```dart
/// @GET('/{id}')
/// Future<Note> read(@Path() String id, @State() NoteRepo repo) => repo.find(id);
/// ```
@Target({TargetKind.parameter})
final class State {
  /// Reads the state matching the parameter's type.
  const State();
}

/// Binds a parameter to the connection's peer information.
@Target({TargetKind.parameter})
final class Peer {
  /// Reads the connection information.
  const Peer();
}

/// Declares that the handler's body is `application/x-www-form-urlencoded`.
@Target({TargetKind.method, TargetKind.function})
final class FormUrlEncoded {
  /// Selects the form media type.
  const FormUrlEncoded();
}

/// Declares that the handler's body is `multipart/form-data`.
@Target({TargetKind.method, TargetKind.function})
final class MultiPart {
  /// Selects the multipart media type.
  const MultiPart();
}
