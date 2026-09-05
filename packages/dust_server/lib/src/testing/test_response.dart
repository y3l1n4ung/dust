import 'dart:convert';

import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

/// The response from a `TestClient` request.
///
/// ```dart
/// response
///     ..assertCreated()
///     ..assertHeader('content-type', 'application/json')
///     ..assertJsonContains({'item': 'shirt'});
/// ```
final class TestResponse {
  /// @nodoc
  ///
  /// Header names are lowercased here rather than at each call site. Shelf
  /// hands back a case-insensitive map whose keys keep the case the handler
  /// wrote, and copying that into a plain map would take the case with it
  /// while losing the insensitive lookup — so `Content-Type` from a handler
  /// would miss `assertHeader('content-type', ...)` that the same assertion
  /// finds over real HTTP, where `dart:io` has already lowercased.
  @internal
  factory TestResponse(
    int statusCode,
    Map<String, List<String>> headers,
    String body,
  ) {
    final all = <String, List<String>>{};
    for (final entry in headers.entries) {
      all[entry.key.toLowerCase()] = List<String>.unmodifiable(entry.value);
    }
    return TestResponse._(
      statusCode,
      Map.unmodifiable(all),
      Map.unmodifiable(<String, String>{
        for (final entry in all.entries) entry.key: entry.value.join(', '),
      }),
      body,
    );
  }

  TestResponse._(this.statusCode, this.headersAll, this.headers, this.body);

  /// @nodoc
  @internal
  static Future<TestResponse> fromShelf(Response response) async {
    final body = await response.readAsString();
    return TestResponse(response.statusCode, response.headersAll, body);
  }

  /// HTTP status code.
  final int statusCode;

  /// Response headers, one joined value per name (unmodifiable).
  ///
  /// A name the response sent more than once — `set-cookie` above all — joins
  /// its values with `, `, which is not reversible for a cookie carrying an
  /// `Expires` date. Read [headersAll] for those.
  final Map<String, String> headers;

  /// Every value for every response header, keyed by lowercased name
  /// (unmodifiable).
  final Map<String, List<String>> headersAll;

  /// Response body as a string.
  final String body;

  /// Decodes body as JSON. Throws [TestAssertionError] on invalid JSON.
  Object? get json {
    try {
      return jsonDecode(body);
    } on FormatException catch (e) {
      throw TestAssertionError(
          'response body is not valid JSON: $e\nbody: $body');
    }
  }

  /// Asserts exact [expected] status.
  void assertStatus(int expected) {
    if (statusCode != expected) {
      throw TestAssertionError(
          'expected status $expected, got $statusCode\n$body');
    }
  }

  /// Asserts 2xx.
  void assertSuccess() {
    if (statusCode < 200 || statusCode > 299) {
      throw TestAssertionError('expected 2xx, got $statusCode\n$body');
    }
  }

  /// Asserts non-2xx.
  void assertFailure() {
    if (statusCode >= 200 && statusCode <= 299) {
      throw TestAssertionError('expected non-2xx, got $statusCode\n$body');
    }
  }

  /// 200.
  void assertOk() => assertStatus(200);

  /// 201.
  void assertCreated() => assertStatus(201);

  /// 204.
  void assertNoContent() => assertStatus(204);

  /// 400.
  void assertBadRequest() => assertStatus(400);

  /// 401.
  void assertUnauthorized() => assertStatus(401);

  /// 403.
  void assertForbidden() => assertStatus(403);

  /// 404.
  void assertNotFound() => assertStatus(404);

  /// 409.
  void assertConflict() => assertStatus(409);

  /// 405.
  void assertMethodNotAllowed() => assertStatus(405);

  /// 413.
  void assertPayloadTooLarge() => assertStatus(413);

  /// 415.
  void assertUnsupportedMediaType() => assertStatus(415);

  /// 422.
  void assertUnprocessable() => assertStatus(422);

  /// 503.
  void assertServiceUnavailable() => assertStatus(503);

  /// Asserts header [name] equals [value].
  void assertHeader(String name, String value) {
    final actual = headers[name];
    if (actual != value) {
      throw TestAssertionError(
          'expected header "$name" to be "$value", got "$actual"');
    }
  }

  /// Asserts header [name] is present, regardless of value.
  void assertContainsHeader(String name) {
    if (!headers.containsKey(name)) {
      throw TestAssertionError('expected header "$name" to be present');
    }
  }

  /// Asserts empty body.
  void assertBodyEmpty() {
    if (body.isNotEmpty) {
      throw TestAssertionError('expected empty body, got "$body"');
    }
  }

  /// Asserts body equals [expected].
  void assertText(String expected) {
    if (body != expected) {
      throw TestAssertionError('expected body "$expected", got "$body"');
    }
  }

  /// Asserts body contains [expected].
  void assertTextContains(String expected) {
    if (!body.contains(expected)) {
      throw TestAssertionError(
          'expected body to contain "$expected"\nbody: $body');
    }
  }

  /// Asserts body decodes to JSON equal to [expected].
  void assertJson(Object? expected) {
    final decoded = json;
    if (!_deepEquals(decoded, expected)) {
      throw TestAssertionError('expected JSON $expected, got $decoded');
    }
  }

  /// Asserts decoded JSON contains [expected] as a subset.
  ///
  /// For maps, every key in [expected] must be present with the same value.
  void assertJsonContains(Object? expected) {
    final decoded = json;
    if (expected is Map && decoded is Map) {
      for (final entry in expected.entries) {
        if (!decoded.containsKey(entry.key) ||
            !_deepEquals(decoded[entry.key], entry.value)) {
          throw TestAssertionError(
              'expected JSON to contain {${entry.key}: ${entry.value}}, '
              'got $decoded');
        }
      }
    } else {
      assertJson(expected);
    }
  }

  @override
  String toString() => 'TestResponse($statusCode, body: $body)';
}

/// Extends [Error] so `package:test` treats it as a test failure without
/// depending on `package:test`.
class TestAssertionError extends Error {
  /// Creates with [message].
  TestAssertionError(this.message);

  /// The failure description.
  final String message;

  @override
  String toString() => message;
}

bool _deepEquals(Object? a, Object? b) {
  if (identical(a, b)) return true;
  if (a is Map && b is Map) {
    if (a.length != b.length) return false;
    for (final key in a.keys) {
      if (!b.containsKey(key) || !_deepEquals(a[key], b[key])) return false;
    }
    return true;
  }
  if (a is List && b is List) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (!_deepEquals(a[i], b[i])) return false;
    }
    return true;
  }
  return a == b;
}
