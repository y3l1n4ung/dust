import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_dart/serde.dart';
import 'package:shelf/shelf.dart';

import 'error_reporting.dart';
import 'rejection.dart';

/// Encodes [value] as a JSON response.
///
/// Anything deriving `Serialize` is written through [Serializable.serialize],
/// including nested values and the elements of a list, so a handler answers
/// with the model it already has:
///
/// ```dart
/// return jsonResponse(repository.all());
/// ```
Response jsonResponse(Object? value, {int status = 200}) {
  return Response(
    status,
    body: jsonEncode(value, toEncodable: _encode),
    headers: const {'content-type': 'application/json'},
  );
}

/// Converts one value `dart:convert` does not encode on its own.
///
/// `serialize` is Dust's verb, so it wins: a type deriving `Serialize`
/// implements [Serializable] and is matched by interface.
///
/// `toJson` still runs for everything else — what `jsonEncode` would have done
/// without this hook, and what types from outside Dust implement. That call goes
/// through `dynamic`, which is Dart's ordinary dispatch and **not** reflection:
/// nothing here imports `dart:mirrors`, which would rule out AOT compilation and
/// defeat the point of generating code in the first place. A type with neither
/// is a `JsonUnsupportedObjectError` rather than a silent `null`.
Object? _encode(Object? value) {
  if (value is Serializable) return value.serialize();
  if (value is DateTime) return value.toIso8601String();
  if (value is Uri) return value.toString();

  try {
    return (value as dynamic).toJson();
  } on NoSuchMethodError {
    throw JsonUnsupportedObjectError(value);
  }
}

/// Answers with [body] as it arrives, without buffering it.
///
/// For anything too large to hold or too slow to wait for: a file download, a
/// generated export, a log tail. The bytes reach the client as the stream
/// produces them.
///
/// The adapter buffers a streamed body by default and flushes when the stream
/// ends, which for a stream that never ends means never. This opts out, the same
/// way `eventStream` does.
///
/// ```dart
/// Future<Response> download(Request request) async {
///   final file = File('report.csv').openRead();
///
///   return streamed(file, contentType: 'text/csv');
/// }
/// ```
///
/// No `content-length` is set, because the length is not known — the response is
/// chunked, which is what lets it start before the work finishes.
Response streamed(
  Stream<List<int>> body, {
  int status = 200,
  String contentType = 'application/octet-stream',
  Map<String, String> headers = const {},
}) {
  return Response(
    status,
    body: body,
    headers: {'content-type': contentType, ...headers},
    context: const {'shelf.io.buffer_output': false},
  );
}

/// Encodes [value] as a plain-text response.
Response textResponse(String value, {int status = 200}) {
  return Response(
    status,
    body: value,
    headers: const {'content-type': 'text/plain; charset=utf-8'},
  );
}

/// Encodes [value] as an octet-stream response.
Response bytesResponse(Uint8List value, {int status = 200}) {
  return Response(
    status,
    body: value,
    headers: const {'content-type': 'application/octet-stream'},
  );
}

/// Streams [value] as an octet-stream response.
Response streamResponse(Stream<List<int>> value, {int status = 200}) {
  return Response(
    status,
    body: value,
    headers: const {'content-type': 'application/octet-stream'},
  );
}

/// An empty 204 response.
Response noContent() => Response(204);

/// Runs [body], turning a thrown [Rejection] into its own response and
/// anything else into a 500 with no detail in it.
///
/// Generated handlers wrap the handwritten call in this, so a guard clause can
/// throw a rejection instead of threading a `Result` all the way back.
///
/// ```dart
/// return guard(() async {
///   final todo = await create(user, input);
///   return jsonResponse(todo.toJson(), status: 201);
/// });
/// ```
Future<Response> guard(Future<Response> Function() body) async {
  try {
    return _checkHeaders(await body());
  } on Rejection catch (rejection) {
    return rejection.intoResponse();
  } on HijackException {
    // Taking over the socket is how a WebSocket upgrade succeeds, and `shelf`
    // signals it by throwing. Catching it would answer 500 on every upgrade.
    rethrow;
  } on Object catch (error, stack) {
    ServerErrors.report(error, stack);
    return const Rejection.internal().intoResponse();
  }
}

/// Rejects a response carrying a header value `dart:io` will refuse to write.
///
/// A control character in a header value is how a response gets split, so
/// `HttpHeaders.set` throws a `FormatException` on one. That happens inside
/// `shelf_io`, after the handler has returned and outside every catch here, and
/// the result is worse than an error: the request never completes, the client
/// waits until it times out, and `onError` is never called, so nothing in the
/// log says which handler did it.
///
/// Catching it here turns a hung connection into a 500 and a reported error.
/// The response is not repaired — a handler that meant to send two headers has
/// a bug, and guessing which one it wanted would hide it.
Response _checkHeaders(Response response) {
  for (final entry in response.headers.entries) {
    if (_hasControlCharacter(entry.value)) {
      ServerErrors.report(
        FormatException(
          'header "${entry.key}" carries a control character, which cannot be '
          'written and would split the response',
          entry.value,
        ),
        StackTrace.current,
      );
      return const Rejection.internal().intoResponse();
    }
  }
  return response;
}

/// Whether [value] holds anything `dart:io` rejects in a header value.
///
/// Everything below `0x20` except tab, plus DEL. CR and LF are the dangerous
/// pair; the rest cannot be written either, so refusing them together keeps the
/// check honest rather than only guarding the exploit everyone knows.
bool _hasControlCharacter(String value) {
  for (final unit in value.codeUnits) {
    if (unit == 0x09) continue;
    if (unit < 0x20 || unit == 0x7f) return true;
  }
  return false;
}
