import 'dart:typed_data';

import 'package:dust_dart/derive.dart';
import 'package:shelf/shelf.dart';

import 'authorization.dart';
import 'context.dart';
import 'cookie.dart';
import 'extractable.dart';
import 'form.dart';
import 'multipart.dart';
import 'multipart_stream.dart';
import 'require.dart';
import 'shortcut.dart' as extractors;

/// Reading values straight off the request.
///
/// An extractor is a value so a generator can name one, compose it, and check
/// it. Written by hand that indirection is noise — the handler knows what it
/// wants and it wants it from this request:
///
/// ```dart
/// Future<Result<Todo, Rejection>> readTodo(Request request) async {
///   final user = await request.extract<AuthUser>(const BearerAuth());
///   final id = await request.path<String>('id');
///   final repository = await request.state<TodoRepository>();
///
///   final todo = repository.find(id);
///   return todo == null || !user.mayActOn(todo.owner)
///       ? const Err(_missing)
///       : Ok(todo);
/// }
/// ```
///
/// Every method throws the rejection an extractor produced rather than
/// returning a `Result`, so the first failure ends the handler and the verb
/// builder turns it into the response.
///
/// That makes this the wrong tool **inside** a custom extractor. An `extract`
/// has to hand its failure back as `Err`, because a combinator that tries
/// several extractors in turn reads the `Result` to decide whether to try the
/// next one; a throw sails past it and ends the request. Use the extractor
/// classes there:
///
/// ```dart
/// // inside FromRequestParts.extract
/// switch (await const BearerTokenExtractable().extract(request)) {
///   case Err(:final error):
///     return Err(error);
///   case Ok(value: final token):
///     ...
/// }
/// ```
///
/// In a handler, throwing is what you want; in an extractor, returning is.
extension RequestExtraction on Request {
  /// Runs [extractor] against this request.
  ///
  /// The escape hatch for anything without a method here, which is every
  /// custom extractor. Unlike the rest, what it produces is not written at the
  /// call site, so **annotate the variable** — otherwise the reader has to open
  /// the extractor to learn what they are holding:
  ///
  /// ```dart
  /// final user = await request.extract<AuthUser>(const TodosWrite());
  /// ```
  ///
  /// Leave it unassigned only when the extractor is a pure guard and its value
  /// is genuinely unwanted:
  ///
  /// ```dart
  /// await request.extract(const TodosWrite());
  /// ```
  ///
  /// An authenticating extractor that returns a caller is rarely a pure guard:
  /// checking that someone is signed in without checking *who* is half a
  /// check.
  Future<T> extract<T>(FromRequestParts<T> extractor) =>
      extractor.require(this);

  /// The `{name}` path segment, coerced to [T].
  ///
  /// The type argument is a coercion, not a cast: `path<int>('id')` on
  /// `/todos/abc` rejects with 400 before the handler sees anything.
  Future<T> path<T>(String name) => extractors.path<T>(name).require(this);

  /// One query value, coerced to [T].
  ///
  /// A nullable [T] makes it optional; a non-nullable one rejects with 400
  /// when the key is absent.
  Future<T> query<T>(String name) => extractors.query<T>(name).require(this);

  /// Every value of a repeated query key, coerced to [T].
  Future<List<T>> queryList<T>(String name) =>
      extractors.queryList<T>(name).require(this);

  /// The application state of type [T] attached with `withState`.
  ///
  /// Missing state is a 500: the handler cannot run, and the fault is a
  /// composition mistake rather than anything the client did.
  Future<T> state<T extends Object>() => extractors.state<T>().require(this);

  /// One header, or `null` when it is absent.
  Future<String?> header(String name) => extractors.header(name).require(this);

  /// The `Host` header this request was addressed to.
  ///
  /// A client sends it, so compare it against names you serve rather than
  /// building a URL out of it.
  Future<String> host() => extractors.host().require(this);

  /// Every header at once, keyed by lower-case name.
  Future<Map<String, String>> headerMap() => extractors.headers().require(this);

  /// Every query pair at once.
  Future<Map<String, String>> queries() => extractors.queries().require(this);

  /// The raw, undecoded query string, or `null` when there is none.
  Future<String?> rawQuery() => extractors.rawQuery().require(this);

  /// One cookie, coerced to [T]. A nullable [T] makes it optional.
  Future<T> cookie<T>(String name) => extractors.cookie<T>(name).require(this);

  /// Every cookie at once.
  Future<CookieJar> cookies() => extractors.cookies().require(this);

  /// The token from an `Authorization: Bearer` header.
  ///
  /// Rejects with 401 and the `Bearer` challenge when the header is missing or
  /// uses another scheme, so a custom extractor starts from the token.
  Future<String> bearerToken() => extractors.bearerToken().require(this);

  /// The decoded username and password of an `Authorization: Basic` header.
  Future<BasicCredentials> basicCredentials({String realm = 'restricted'}) =>
      extractors.basicCredentials(realm: realm).require(this);

  /// An API key from a header, falling back to the query string.
  Future<String> apiKey({
    String header = 'x-api-key',
    String query = 'api_key',
    bool allowQuery = true,
  }) =>
      extractors
          .apiKey(header: header, query: query, allowQuery: allowQuery)
          .require(this);

  /// A session identifier from a cookie, as 401 rather than 400 when absent.
  Future<String> sessionId({String name = 'session'}) =>
      extractors.sessionId(name: name).require(this);

  /// The JSON object body, decoded with [deserialize].
  Future<T> body<T>(T Function(Map<String, Object?> json) deserialize) =>
      extractors.body<T>(deserialize).require(this);

  /// The JSON array body, each element decoded with [deserialize].
  Future<List<T>> bodyList<T>(
    T Function(Map<String, Object?> json) deserialize,
  ) =>
      extractors.bodyList<T>(deserialize).require(this);

  /// The JSON object body, decoded and then checked against its constraints.
  ///
  /// The one call a create endpoint needs: a body of the wrong shape and a
  /// body that breaks a `@Validate` both answer 422, and only the second names
  /// the fields.
  Future<T> validBody<T extends Validatable>(
    T Function(Map<String, Object?> json) deserialize,
  ) =>
      extractors.valid(extractors.body<T>(deserialize)).require(this);

  /// The body as UTF-8 text.
  Future<String> textBody() => extractors.textBody().require(this);

  /// The body as raw bytes.
  Future<Uint8List> rawBody() => extractors.rawBody().require(this);

  /// The body as an unread stream, for a handler that owns its own limit.
  Future<Stream<List<int>>> bodyStream() =>
      extractors.bodyStream().require(this);

  /// The decoded fields of an `application/x-www-form-urlencoded` body.
  Future<FormMap> form() => extractors.form().require(this);

  /// The parts of a `multipart/form-data` body.
  Future<MultipartForm> multipart() => extractors.multipart().require(this);

  /// Hands the multipart body over a part at a time, without buffering it.
  Future<StreamedMultipart> multipartStream({int limit = 64 * 1024 * 1024}) =>
      extractors.multipartStream(limit: limit).require(this);

  /// The connection's peer address and port.
  Future<PeerInfo> peer() => extractors.peer().require(this);
}
