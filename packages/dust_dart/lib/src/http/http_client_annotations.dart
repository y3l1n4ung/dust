import 'package:meta/meta.dart';

/// Defines the thread context for JSON parsing.
enum DustParseThread {
  /// Decoding happens on the thread that initiated the request.
  main,

  /// Decoding is offloaded to a background isolate using Isolate.run.
  isolate,
}

/// Defines the runtime target for generated HTTP clients.
enum DustHttpTarget {
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
  final DustHttpTarget target;

  /// The global default strategy for JSON decoding.
  final DustParseThread parseThread;

  /// Static headers applied to every request from this client.
  final Map<String, String> headers;

  const HttpClient({
    this.baseUrl,
    this.target = DustHttpTarget.dart,
    this.parseThread = DustParseThread.main,
    this.headers = const {},
  });
}

/// Instructs Dust to generate a test suite (`.test.g.dart`) for this API.
@immutable
class GenerateTest {
  const GenerateTest();
}

/// Base class for HTTP method annotations.
@immutable
abstract class HttpMethod {
  final String path;
  const HttpMethod(this.path);
}

class GET extends HttpMethod {
  const GET(super.path);
}

class POST extends HttpMethod {
  const POST(super.path);
}

class PUT extends HttpMethod {
  const PUT(super.path);
}

class PATCH extends HttpMethod {
  const PATCH(super.path);
}

class DELETE extends HttpMethod {
  const DELETE(super.path);
}

class HEAD extends HttpMethod {
  const HEAD(super.path);
}

class OPTIONS extends HttpMethod {
  const OPTIONS(super.path);
}

/// Maps a parameter to a `{name}` placeholder in the URL path.
@immutable
class Path {
  final String? name;
  const Path([this.name]);
}

/// Adds a query parameter to the URL.
@immutable
class Query {
  final String name;
  const Query(this.name);
}

/// Maps a [Map<String, dynamic>] parameter to multiple query parameters.
@immutable
class Queries {
  const Queries();
}

/// Maps a parameter to a request header.
@immutable
class Header {
  final String name;
  const Header(this.name);
}

/// Method-level annotation for static headers.
@immutable
class Headers {
  final Map<String, String> values;
  const Headers(this.values);
}

/// Maps a [Map<String, String>] parameter to multiple request headers.
@immutable
class HeaderMap {
  const HeaderMap();
}

/// Encodes the parameter as the request body.
@immutable
class Body {
  const Body();
}

/// Adds a field to `application/x-www-form-urlencoded`.
/// Requires [@FormUrlEncoded] on the method.
@immutable
class Field {
  final String name;
  const Field(this.name);
}

/// Adds a part to `multipart/form-data`.
/// Requires [@MultiPart] on the method.
@immutable
class Part {
  final String name;
  const Part(this.name);
}

/// Maps a parameter to `RequestOptions.extra`.
@immutable
class Extra {
  final String key;
  const Extra(this.key);
}

/// Sets content-type to `application/x-www-form-urlencoded`.
@immutable
class FormUrlEncoded {
  const FormUrlEncoded();
}

/// Sets content-type to `multipart/form-data`.
@immutable
class MultiPart {
  const MultiPart();
}

/// Overrides the class-level parsing strategy for a single method.
@immutable
class HttpParse {
  final DustParseThread thread;
  const HttpParse({required this.thread});
}
