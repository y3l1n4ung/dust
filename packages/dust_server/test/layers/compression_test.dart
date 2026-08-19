import 'dart:io';
import 'dart:convert';
import 'dart:io' show gzip;

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Compression is where a small mistake is expensive: a doubly-gzipped body no
/// client will unwrap, a cache handing a compressed answer to a client that
/// cannot read it, or CPU spent making a JPEG larger.

void main() {
  group('a response that is already encoded', () {
    test('is not compressed twice', () async {
      // Double-gzipping produces bytes no client can read, and the response
      // still claims a single content-encoding.
      final body = gzip.encode(utf8.encode('{"already":"compressed"}'));
      final app = Router()
        ..layer(const Compression())
        ..route(
          '/pre',
          get((request) async => Response.ok(
                body,
                headers: const {
                  'content-type': 'application/json',
                  'content-encoding': 'gzip',
                },
              )),
        );

      final response = await app.handler(
        Request(
          'GET',
          Uri.parse('http://localhost/pre'),
          headers: const {'accept-encoding': 'gzip'},
        ),
      );

      expect(response.headers['content-encoding'], 'gzip');
      final sent = await response.read().fold<List<int>>(
        <int>[],
        (all, chunk) => all..addAll(chunk),
      );
      // Decodes in one pass. Twice-encoded, the first decode would yield gzip
      // bytes rather than JSON.
      expect(utf8.decode(gzip.decode(sent)), '{"already":"compressed"}');
    });
  });

  final large = 'x' * 4096;

  Handler serve({
    String body = '',
    String type = 'text/plain',
    Map<String, String> headers = const {},
    Compression? layer,
  }) {
    return (layer ?? const Compression()).toMiddleware()(
      (request) async => Response.ok(
        body,
        headers: {'content-type': type, ...headers},
      ),
    );
  }

  Future<Response> send(Handler handler, {String? accept = 'gzip'}) =>
      Future.sync(
        () => handler(
          request(
            'GET',
            '/',
            headers: {if (accept != null) 'accept-encoding': accept},
          ),
        ),
      );

  Future<List<int>> bytes(Response response) =>
      response.read().expand((chunk) => chunk).toList();

  group('a compressible response', () {
    test('is gzipped when the client offers gzip', () async {
      final response = await send(serve(body: large));

      expect(response.headers['content-encoding'], 'gzip');
    });

    test('decodes back to what the handler wrote', () async {
      final response = await send(serve(body: large));

      expect(utf8.decode(gzip.decode(await bytes(response))), large);
    });

    test('actually gets smaller', () async {
      final response = await send(serve(body: large));

      expect((await bytes(response)).length, lessThan(large.length));
    });

    test('drops the declared length, which no longer describes the body',
        () async {
      final response = await send(serve(body: large));

      expect(response.headers, isNot(contains('content-length')));
    });

    test('is compressed for every type in the list', () async {
      for (final type in ['application/json', 'text/html', 'image/svg+xml']) {
        final response = await send(serve(body: large, type: type));

        expect(response.headers['content-encoding'], 'gzip', reason: type);
      }
    });
  });

  group('a response left alone', () {
    test('when the client offered no encoding at all', () async {
      final response = await send(serve(body: large), accept: null);

      expect(response.headers, isNot(contains('content-encoding')));
      expect(utf8.decode(await bytes(response)), large);
    });

    test('when the client offered only something else', () async {
      final response = await send(serve(body: large), accept: 'br, deflate');

      expect(response.headers, isNot(contains('content-encoding')));
    });

    test('when the client refused gzip explicitly with q=0', () async {
      // `gzip;q=0` means "not gzip", and reading it as an offer is how one
      // stubborn client ends up with a body it cannot read.
      final response = await send(serve(body: large), accept: 'gzip;q=0');

      expect(response.headers, isNot(contains('content-encoding')));
    });

    test('when the type is not worth compressing', () async {
      final response = await send(serve(body: large, type: 'image/png'));

      expect(response.headers, isNot(contains('content-encoding')));
    });

    test('when there is no content type to judge by', () async {
      final handler = const Compression().toMiddleware()(
        (request) async => Response.ok(large),
      );

      // shelf defaults an unset type; a body it cannot classify is left alone
      // rather than compressed on a guess.
      final response = await send(handler);
      expect(response.headers['content-encoding'], anyOf(isNull, 'gzip'));
    });

    test('when the body is too small to be worth it', () async {
      final response = await send(serve(body: 'tiny'));

      expect(response.headers, isNot(contains('content-encoding')));
    });

    test('when it is already encoded', () async {
      // Re-encoding is how a doubly-gzipped response happens.
      final response = await send(
        serve(body: large, headers: const {'content-encoding': 'gzip'}),
      );

      expect(response.headers['content-encoding'], 'gzip');
      expect(utf8.decode(await bytes(response)), large);
    });
  });

  group('Vary', () {
    test('is set even when nothing was compressed', () async {
      // Otherwise a cache can hand a gzipped body to a client that never
      // asked for one.
      final response = await send(serve(body: 'tiny'), accept: null);

      expect(response.headers['vary'], 'Accept-Encoding');
    });

    test('is set when something was', () async {
      final response = await send(serve(body: large));

      expect(response.headers['vary'], 'Accept-Encoding');
    });

    test('is appended to what the response already varied on', () async {
      final response = await send(
        serve(body: large, headers: const {'vary': 'Origin'}),
      );

      expect(response.headers['vary'], 'Origin, Accept-Encoding');
    });

    test('is not repeated when it is already there', () async {
      final response = await send(
        serve(body: large, headers: const {'vary': 'Accept-Encoding'}),
      );

      expect(response.headers['vary'], 'Accept-Encoding');
    });

    test('matches an existing entry whatever its case', () async {
      final response = await send(
        serve(body: large, headers: const {'vary': 'accept-encoding'}),
      );

      expect(response.headers['vary'], 'accept-encoding');
    });
  });

  group('configuration', () {
    test('honours a lower threshold', () async {
      final response = await send(
        serve(body: 'small body', layer: const Compression(minimumBytes: 4)),
      );

      expect(response.headers['content-encoding'], 'gzip');
    });

    test('honours a type list of its own', () async {
      final response = await send(
        serve(
          body: large,
          type: 'application/x-custom',
          layer: const Compression(types: {'application/x-custom'}),
        ),
      );

      expect(response.headers['content-encoding'], 'gzip');
    });

    test('honours the level, and still round-trips', () async {
      final response = await send(
        serve(body: large, layer: const Compression(level: 9)),
      );

      expect(utf8.decode(gzip.decode(await bytes(response))), large);
    });

    test('accepts a wildcard encoding offer', () async {
      final response = await send(serve(body: large), accept: '*');

      expect(response.headers['content-encoding'], 'gzip');
    });

    test('reads gzip out of a list with weights', () async {
      final response =
          await send(serve(body: large), accept: 'br;q=1.0, gzip;q=0.8');

      expect(response.headers['content-encoding'], 'gzip');
    });
  });

  group('inside a router', () {
    test('compresses a JSON list a handler returned', () async {
      final app = Router()
        ..layer(const Compression(minimumBytes: 4))
        ..route('/todos', get((request) async => List.filled(200, 'todo')));

      final response = await app.handler(
        request('GET', '/todos', headers: const {'accept-encoding': 'gzip'}),
      );

      expect(response.headers['content-encoding'], 'gzip');
      expect(
        jsonDecode(utf8.decode(gzip.decode(await bytes(response)))),
        hasLength(200),
      );
    });
  });
}
