// coverage:ignore-file

import 'package:meta/meta.dart';

/// Defines the thread context for JSON parsing.
enum HttpParseThread {
  /// Decoding happens on the thread that initiated the request.
  main,

  /// Decoding is offloaded to a background isolate.
  ///
  /// Dart-targeted clients use `Isolate.run`; Flutter-targeted clients use
  /// Flutter's `compute` helper.
  isolate,
}

/// Defines the runtime target for generated HTTP clients.
enum HttpTarget {
  /// Generate runtime behavior that works in pure Dart environments.
  dart,

  /// Generate runtime behavior intended for Flutter applications.
  flutter,
}

/// Annotation for a class to generate a Dio-backed HTTP client.
@immutable
class HttpClient {
  /// The base URL for all methods in this client.
  final String? baseUrl;

  /// The runtime target the generated client should optimize for.
  final HttpTarget target;

  /// The global default strategy for JSON decoding.
  final HttpParseThread parseThread;

  /// Whether Dust should generate a companion request-mapping test file.
  final bool generateTest;

  /// Static headers applied to every request from this client.
  final Map<String, String> headers;

  /// Creates one HTTP client generation annotation.
  const HttpClient({
    this.baseUrl,
    this.target = HttpTarget.dart,
    this.parseThread = HttpParseThread.main,
    this.headers = const {},
    this.generateTest = false,
  });
}

/// Base class for HTTP method annotations.
@immutable
abstract class HttpMethod {
  /// Method path template relative to the client base URL.
  final String path;

  /// Creates one HTTP method annotation.
  const HttpMethod(this.path);
}

/// Marks a method as an HTTP GET endpoint.
class GET extends HttpMethod {
  /// Creates one GET endpoint annotation.
  const GET(super.path);
}

/// Marks a method as an HTTP POST endpoint.
class POST extends HttpMethod {
  /// Creates one POST endpoint annotation.
  const POST(super.path);
}

/// Marks a method as an HTTP PUT endpoint.
class PUT extends HttpMethod {
  /// Creates one PUT endpoint annotation.
  const PUT(super.path);
}

/// Marks a method as an HTTP PATCH endpoint.
class PATCH extends HttpMethod {
  /// Creates one PATCH endpoint annotation.
  const PATCH(super.path);
}

/// Marks a method as an HTTP DELETE endpoint.
class DELETE extends HttpMethod {
  /// Creates one DELETE endpoint annotation.
  const DELETE(super.path);
}

/// Marks a method as an HTTP HEAD endpoint.
class HEAD extends HttpMethod {
  /// Creates one HEAD endpoint annotation.
  const HEAD(super.path);
}

/// Marks a method as an HTTP OPTIONS endpoint.
class OPTIONS extends HttpMethod {
  /// Creates one OPTIONS endpoint annotation.
  const OPTIONS(super.path);
}

/// Maps a parameter to a `{name}` placeholder in the URL path.
@immutable
class Path {
  /// Placeholder name. Defaults to the annotated parameter name when omitted.
  final String? name;

  /// Creates one path parameter annotation.
  const Path([this.name]);
}

/// Adds a query parameter to the URL.
@immutable
class Query {
  /// Query-string key.
  final String name;

  /// Creates one query parameter annotation.
  const Query(this.name);
}

/// Maps a [Map<String, dynamic>] parameter to multiple query parameters.
@immutable
class Queries {
  /// Creates one query map annotation.
  const Queries();
}

/// Maps a parameter to a request header.
@immutable
class Header {
  /// Header name.
  final String name;

  /// Creates one header parameter annotation.
  const Header(this.name);
}

/// Method-level annotation for static headers.
@immutable
class Headers {
  /// Static header values.
  final Map<String, String> values;

  /// Creates one static headers annotation.
  const Headers(this.values);
}

/// Maps a [Map<String, String>] parameter to multiple request headers.
@immutable
class HeaderMap {
  /// Creates one header map annotation.
  const HeaderMap();
}

/// Encodes the parameter as the request body.
@immutable
class Body {
  /// Creates one body parameter annotation.
  const Body();
}

/// Adds a field to `application/x-www-form-urlencoded`.
/// Requires [@FormUrlEncoded] on the method.
@immutable
class Field {
  /// Form field name.
  final String name;

  /// Creates one form field annotation.
  const Field(this.name);
}

/// Adds a part to `multipart/form-data`.
/// Requires [@MultiPart] on the method.
@immutable
class Part {
  /// Multipart field name.
  final String name;

  /// Creates one multipart part annotation.
  const Part(this.name);
}

/// Maps a parameter to `RequestOptions.extra`.
@immutable
class Extra {
  /// Request extra key.
  final String key;

  /// Creates one request extra annotation.
  const Extra(this.key);
}

/// Sets content-type to `application/x-www-form-urlencoded`.
@immutable
class FormUrlEncoded {
  /// Creates one form-url-encoded request annotation.
  const FormUrlEncoded();
}

/// Sets content-type to `multipart/form-data`.
@immutable
class MultiPart {
  /// Creates one multipart request annotation.
  const MultiPart();
}

/// Overrides the class-level parsing strategy for a single method.
@immutable
class HttpParse {
  /// Method-level JSON parse thread override.
  final HttpParseThread thread;

  /// Creates one HTTP parse override annotation.
  const HttpParse({required this.thread});
}
