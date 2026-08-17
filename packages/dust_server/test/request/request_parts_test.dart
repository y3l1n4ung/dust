import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('RequestParts', () {
    test('reads method, url, and headers', () {
      final parts = RequestParts.of(
        request('POST', '/todos?done=true', headers: {'X-Trace': 'abc'}),
      );

      expect(parts.method, 'POST');
      expect(parts.url.path, 'todos');
      expect(parts.requestedUri.query, 'done=true');
      expect(parts.headers['x-trace'], 'abc');
    });

    test('strips parameters from the media type', () {
      final parts = RequestParts.of(
        request(
          'POST',
          '/',
          headers: {'content-type': 'application/json; charset=utf-8'},
          body: '{}',
        ),
      );

      expect(parts.mediaType, 'application/json');
    });

    test('lower-cases the media type', () {
      final parts = RequestParts.of(
        request(
          'POST',
          '/',
          headers: {'content-type': 'Application/JSON'},
          body: '{}',
        ),
      );

      expect(parts.mediaType, 'application/json');
    });

    test('reports a null media type when the header is absent', () {
      expect(RequestParts.of(request('GET', '/')).mediaType, isNull);
    });

    test('parses content-length and tolerates a malformed one', () {
      final present = RequestParts.of(
        request('POST', '/', headers: {'content-length': '12'}, body: 'x' * 12),
      );

      expect(present.contentLength, 12);
    });

    test('exposes router path parameters', () {
      final parts = RequestParts.of(
        request('GET', '/todos/7', pathParameters: {'id': '7'}),
      );

      expect(parts.pathParameters, {'id': '7'});
    });

    test('reports no path parameters outside a router', () {
      expect(
          RequestParts.of(request('GET', '/todos/7')).pathParameters, isEmpty);
    });

    test('coerces a loosely typed parameter map', () {
      final parsed = pathParametersOf(
        Request(
          'GET',
          Uri.parse('http://localhost/todos/7'),
          context: {
            pathParametersKey: <Object, Object>{'id': 7},
          },
        ),
      );

      expect(parsed, {'id': '7'});
    });

    test('exposes the shelf context for middleware handoff', () {
      final parts = RequestParts.of(
        request('GET', '/', context: {'tenant': 'acme'}),
      );

      expect(parts.context['tenant'], 'acme');
    });
  });
}
