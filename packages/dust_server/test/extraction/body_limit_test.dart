import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Two places set a body limit, and neither may loosen the other.
///
/// `Router(bodyLimit:)` bounds the whole application; an extractor bounds one
/// route. It used to be "the router wins", which reached generated code
/// correctly and also quietly raised a limit an extractor had deliberately set
/// low — a route asking for 1 KB accepted whatever the application allowed.

Request _body(int bytes, {int? routerLimit}) {
  return Request(
    'POST',
    Uri.parse('http://localhost/upload'),
    headers: const {'content-type': 'application/octet-stream'},
    body: String.fromCharCodes(List.filled(bytes, 65)),
    context: {
      if (routerLimit != null) bodyLimitContextKey: routerLimit,
    },
  );
}

void main() {
  group('the effective body limit', () {
    test('is the extractor limit when no router limit is set', () {
      expect(effectiveBodyLimit(_body(0), 1024), 1024);
    });

    test('is the router limit when the router is stricter', () {
      expect(effectiveBodyLimit(_body(0, routerLimit: 512), 1024), 512);
    });

    test('is the extractor limit when the extractor is stricter', () {
      // The case that was wrong. A route asking for 1 KB must not be widened by
      // an application-wide 5 MB.
      expect(effectiveBodyLimit(_body(0, routerLimit: 5000), 1024), 1024);
    });

    test('ignores a non-integer in the context', () {
      final request = Request(
        'POST',
        Uri.parse('http://localhost/upload'),
        body: 'x',
        context: const {bodyLimitContextKey: 'not a number'},
      );

      expect(effectiveBodyLimit(request, 1024), 1024);
    });
  });

  group('reading a body', () {
    test('accepts what fits under both limits', () async {
      final outcome = await readBody(
        _body(100, routerLimit: 5000),
        limit: 1024,
      );

      expect(expectOk(outcome), hasLength(100));
    });

    test('refuses past the extractor limit even when the router is looser',
        () async {
      final outcome = await readBody(
        _body(2000, routerLimit: 5000),
        limit: 1024,
      );

      final rejection = expectStatus(outcome, 413);
      expect(rejection.message, 'body exceeds 1024 bytes');
    });

    test('refuses past the router limit even when the extractor is looser',
        () async {
      final outcome = await readBody(
        _body(2000, routerLimit: 512),
        limit: 1024,
      );

      expect(expectStatus(outcome, 413).message, 'body exceeds 512 bytes');
    });

    test('refuses a declared content-length before reading anything', () async {
      // The cheap check: an oversized upload is turned away without being
      // transferred.
      // A stream body, so the declared length is the client's claim rather than
      // something `shelf` recomputed from the bytes it already has.
      var read = false;
      final request = Request(
        'POST',
        Uri.parse('http://localhost/upload'),
        headers: const {'content-length': '999999'},
        body: Stream<List<int>>.fromIterable([
          [65],
        ]).map((chunk) {
          read = true;
          return chunk;
        }),
      );

      expectStatus(await readBody(request, limit: 1024), 413);
      expect(read, isFalse, reason: 'the body must not be read to refuse it');
    });

    test('counts a body that declared no length', () async {
      // A declared length is a claim. A chunked body has none to claim, so the
      // bytes are counted as they arrive.
      final request = Request(
        'POST',
        Uri.parse('http://localhost/upload'),
        body: Stream<List<int>>.fromIterable([
          for (var index = 0; index < 10; index++) utf8.encode('a' * 200),
        ]),
      );

      expectStatus(await readBody(request, limit: 1024), 413);
    });
  });
}
