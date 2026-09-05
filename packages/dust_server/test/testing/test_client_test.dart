import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:dust_server/testing.dart';
import 'package:test/test.dart';

void main() {
  Router app() {
    final router = Router();
    router.route(
      '/hello',
      get((_) async => textResponse('world')),
    );
    router.route(
      '/echo',
      post((request) async {
        final body = await request.readAsString();
        return jsonResponse({'echo': body});
      }),
    );
    router.route(
      '/greet/{name}',
      get((request) async {
        final params = pathParametersOf(request);
        return textResponse('hello ${params['name']}');
      }),
    );
    router.route(
      '/status/{code}',
      get((request) async {
        final params = pathParametersOf(request);
        final code = int.parse(params['code']!);
        return Response(code, body: 'status $code');
      }),
    );
    router.route(
      '/headers',
      get((request) async {
        final auth = request.headers['authorization'] ?? 'none';
        return jsonResponse({'authorization': auth});
      }),
    );
    router.route(
      '/cookie-echo',
      get((request) async {
        final cookie = request.headers['cookie'] ?? 'none';
        return Response.ok(cookie);
      }),
    );
    router.route(
      '/search',
      get((request) async {
        final uri = request.requestedUri;
        return jsonResponse({
          'q': uri.queryParameters['q'] ?? '',
          'page': uri.queryParameters['page'] ?? '',
        });
      }),
    );
    router.route(
      '/empty',
      get((_) async => Response(204)),
    );
    router.route(
      '/verb',
      any((request) async => textResponse(request.method)),
    );
    router.route(
      '/list',
      get((_) async => jsonResponse([1, 2, 3])),
    );
    router.route(
      '/mixed-case-headers',
      get(
        (_) async => Response.ok(
          '{"ok":true}',
          headers: {
            'Content-Type': 'application/json',
            'X-Trace-Id': 'abc123',
          },
        ),
      ),
    );
    router.route(
      '/two-cookies',
      get(
        (_) async => Response.ok(
          'set',
          headers: {
            'set-cookie': [
              'session=abc; Path=/; HttpOnly',
              'prefs=dark; Path=/; Expires=Wed, 09 Jun 2027 10:18:14 GMT',
            ],
          },
        ),
      ),
    );
    return router;
  }

  group('TestClient handler mode', () {
    late TestClient client;

    setUp(() {
      client = TestClient(app());
    });

    tearDown(() async {
      await client.close();
    });

    test('GET returns text', () async {
      final response = await client.get('/hello').send();
      response
        ..assertOk()
        ..assertText('world');
    });

    test('POST with JSON body', () async {
      final response =
          await (client.post('/echo')..json({'key': 'value'})).send();
      response
        ..assertOk()
        ..assertJsonContains({'echo': '{"key":"value"}'});
    });

    test('path parameters work in handler mode', () async {
      final response = await client.get('/greet/alice').send();
      response
        ..assertOk()
        ..assertText('hello alice');
    });

    test('custom HTTP method', () async {
      final response = await client.method('GET', '/hello').send();
      response.assertOk();
    });

    test('404 for unknown path', () async {
      final response = await client.get('/nope').send();
      response.assertNotFound();
    });
  });

  group('TestClient.serve mode', () {
    late TestClient client;

    setUp(() async {
      client = await TestClient.serve(app());
    });

    tearDown(() async {
      await client.close();
    });

    test('origin is available', () {
      expect(client.origin, startsWith('http://'));
    });

    test('GET over real HTTP', () async {
      final response = await client.get('/hello').send();
      response
        ..assertOk()
        ..assertText('world');
    });

    test('POST with JSON over real HTTP', () async {
      final response =
          await (client.post('/echo')..json({'key': 'value'})).send();
      response.assertOk();
    });

    test('path parameters over real HTTP', () async {
      final response = await client.get('/greet/bob').send();
      response
        ..assertOk()
        ..assertText('hello bob');
    });
  });

  group('TestClient origin', () {
    test('throws in handler mode', () {
      final client = TestClient(app());
      expect(() => client.origin, throwsStateError);
    });
  });

  group('the rest of the surface', () {
    late TestClient client;

    setUp(() {
      client = TestClient(app());
    });

    tearDown(() async {
      await client.close();
    });

    test('put, patch and delete each reach the handler', () async {
      (await client.put('/verb').send())
        ..assertOk()
        ..assertText('PUT');
      (await client.patch('/verb').send())
        ..assertOk()
        ..assertText('PATCH');
      (await client.delete('/verb').send())
        ..assertOk()
        ..assertText('DELETE');
    });

    test('head answers the status without a body', () async {
      (await client.head('/verb').send()).assertOk();
    });

    test('a client-level expectFailure applies to every request', () async {
      client.expectFailure();

      (await client.get('/nope').send()).assertNotFound();
    });

    test('bytes carries an explicit content type', () async {
      final response = await (client.post('/echo')
            ..bytes(utf8.encode('raw'), contentType: 'application/x-thing'))
          .send();

      response
        ..assertOk()
        ..assertJsonContains({'echo': 'raw'});
    });

    test('contentType sets a type with no body of its own', () async {
      final response =
          await (client.post('/echo')..contentType('text/plain')).send();

      response.assertOk();
    });

    test('origin mode drives a server someone else started', () async {
      final served = await TestClient.serve(app());
      addTearDown(served.close);
      final borrowed = TestClient.origin(served.origin);
      addTearDown(borrowed.close);

      (await borrowed.get('/hello').send())
        ..assertOk()
        ..assertText('world');
    });

    test('every named status assertion checks its own code', () async {
      (await client.get('/status/409').send()).assertConflict();
      (await client.get('/status/405').send()).assertMethodNotAllowed();
      (await client.get('/status/413').send()).assertPayloadTooLarge();
      (await client.get('/status/415').send()).assertUnsupportedMediaType();
      (await client.get('/status/422').send()).assertUnprocessable();
      (await client.get('/status/503').send()).assertServiceUnavailable();
    });

    test('assertHeader reports the value it actually found', () async {
      final response = await client.get('/hello').send();

      expect(
        () => response.assertHeader('content-type', 'application/json'),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertJson compares a list element by element', () async {
      final response = await client.get('/list').send();

      response
        ..assertOk()
        ..assertJson([1, 2, 3]);
      expect(
        () => response.assertJson([1, 2, 4]),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertJsonContains falls back to equality for a non-map', () async {
      (await client.get('/list').send()).assertJsonContains([1, 2, 3]);
    });

    test('toString carries the status and the body', () async {
      final response = await client.get('/hello').send();

      expect(response.toString(), 'TestResponse(200, body: world)');
    });

    test('an assertion error prints its own message', () {
      expect(TestAssertionError('boom').toString(), 'boom');
    });
  });

  group('response headers', () {
    test('a handler that capitalizes a header name is found lowercase',
        () async {
      final client = TestClient(app());
      addTearDown(client.close);

      (await client.get('/mixed-case-headers').send())
        ..assertOk()
        ..assertContainsHeader('x-trace-id')
        ..assertHeader('x-trace-id', 'abc123')
        ..assertHeader('content-type', 'application/json');
    });

    test('real HTTP answers the same assertion the same way', () async {
      final client = await TestClient.serve(app());
      addTearDown(client.close);

      (await client.get('/mixed-case-headers').send())
        ..assertOk()
        ..assertContainsHeader('x-trace-id')
        ..assertHeader('x-trace-id', 'abc123')
        ..assertHeader('content-type', 'application/json');
    });

    test('every set-cookie reaches the jar, Expires comma and all', () async {
      final client = TestClient(app(), saveCookies: true);
      addTearDown(client.close);

      final response = await client.get('/two-cookies').send();
      expect(response.headersAll['set-cookie'], hasLength(2));

      (await client.get('/cookie-echo').send())
        ..assertOk()
        ..assertTextContains('session=abc')
        ..assertTextContains('prefs=dark');
    });

    test('real HTTP keeps both cookies too', () async {
      final client = await TestClient.serve(app(), saveCookies: true);
      addTearDown(client.close);

      await client.get('/two-cookies').send();

      (await client.get('/cookie-echo').send())
        ..assertOk()
        ..assertTextContains('session=abc')
        ..assertTextContains('prefs=dark');
    });
  });

  group('TestRequest', () {
    late TestClient client;

    setUp(() {
      client = TestClient(app());
    });

    tearDown(() async {
      await client.close();
    });

    test('bearer sets authorization header', () async {
      final response = await (client.get('/headers')..bearer('tok_abc')).send();
      response
        ..assertOk()
        ..assertJsonContains({'authorization': 'Bearer tok_abc'});
    });

    test('basic sets authorization header', () async {
      final expected = 'Basic ${base64Encode(utf8.encode('user:pass'))}';
      final response =
          await (client.get('/headers')..basic('user', 'pass')).send();
      response
        ..assertOk()
        ..assertJsonContains({'authorization': expected});
    });

    test('header sets custom header', () async {
      final response = await (client.get('/headers')
            ..header('authorization', 'Custom xyz'))
          .send();
      response
        ..assertOk()
        ..assertJsonContains({'authorization': 'Custom xyz'});
    });

    test('text body', () async {
      final response = await (client.post('/echo')..text('hello')).send();
      response
        ..assertOk()
        ..assertJsonContains({'echo': 'hello'});
    });

    test('form body', () async {
      final response = await (client.post('/echo')
            ..form({'key': 'value', 'other': 'data'}))
          .send();
      response.assertOk();
      final body = response.json as Map;
      expect(body['echo'], contains('key=value'));
    });

    test('cookie is sent', () async {
      final response = await (client.get('/cookie-echo')
            ..cookie('session', 'abc123'))
          .send();
      response
        ..assertOk()
        ..assertTextContains('session=abc123');
    });

    test('expectSuccess passes on 2xx', () async {
      final response = await (client.get('/hello')..expectSuccess()).send();
      response.assertOk();
    });

    test('expectSuccess fails on non-2xx', () async {
      expect(
        () => (client.get('/nope')..expectSuccess()).send(),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('expectFailure passes on non-2xx', () async {
      final response = await (client.get('/nope')..expectFailure()).send();
      response.assertNotFound();
    });

    test('expectFailure fails on 2xx', () async {
      expect(
        () => (client.get('/hello')..expectFailure()).send(),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('queryParam adds query string', () async {
      final response = await (client.get('/search')
            ..queryParam('q', 'dart')
            ..queryParam('page', '2'))
          .send();
      response
        ..assertOk()
        ..assertJson({'q': 'dart', 'page': '2'});
    });
  });

  group('TestClient-level expectations', () {
    test('expectSuccess applies to all requests', () async {
      final client = TestClient(app())..expectSuccess();
      addTearDown(client.close);

      final response = await client.get('/hello').send();
      response.assertOk();

      expect(
        () => client.get('/nope').send(),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('per-request overrides client default', () async {
      final client = TestClient(app())..expectSuccess();
      addTearDown(client.close);

      final response = await (client.get('/nope')..expectFailure()).send();
      response.assertNotFound();
    });
  });

  group('TestResponse assertions', () {
    late TestClient client;

    setUp(() {
      client = TestClient(app());
    });

    tearDown(() async {
      await client.close();
    });

    test('assertStatus exact match', () async {
      final response = await client.get('/status/201').send();
      response.assertStatus(201);
    });

    test('assertStatus mismatch throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.assertStatus(404),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertSuccess on 2xx', () async {
      final response = await client.get('/hello').send();
      response.assertSuccess();
    });

    test('assertSuccess on non-2xx throws', () async {
      final response = await client.get('/nope').send();
      expect(
        () => response.assertSuccess(),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertFailure on non-2xx', () async {
      final response = await client.get('/nope').send();
      response.assertFailure();
    });

    test('assertFailure on 2xx throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.assertFailure(),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('named status assertions', () async {
      for (final code in [200, 201, 400, 401, 403, 404]) {
        final response = await client.get('/status/$code').send();
        switch (code) {
          case 200:
            response.assertOk();
          case 201:
            response.assertCreated();
          case 400:
            response.assertBadRequest();
          case 401:
            response.assertUnauthorized();
          case 403:
            response.assertForbidden();
          case 404:
            response.assertNotFound();
        }
      }
    });

    test('assertText match', () async {
      final response = await client.get('/hello').send();
      response.assertText('world');
    });

    test('assertText mismatch throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.assertText('nope'),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertTextContains', () async {
      final response = await client.get('/hello').send();
      response.assertTextContains('orl');
    });

    test('assertTextContains mismatch throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.assertTextContains('xyz'),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertJson match', () async {
      final response = await client.get('/headers').send();
      response.assertJson({'authorization': 'none'});
    });

    test('assertJson mismatch throws', () async {
      final response = await client.get('/headers').send();
      expect(
        () => response.assertJson({'key': 'value'}),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertJsonContains subset', () async {
      final response = await client.get('/headers').send();
      response.assertJsonContains({'authorization': 'none'});
    });

    test('assertJsonContains missing key throws', () async {
      final response = await client.get('/headers').send();
      expect(
        () => response.assertJsonContains({'missing': 'key'}),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertHeader match', () async {
      final response = await client.get('/hello').send();
      response.assertContainsHeader('content-type');
    });

    test('assertContainsHeader missing throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.assertContainsHeader('x-custom'),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('json getter on non-JSON body throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.json,
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('assertBodyEmpty on empty body', () async {
      final response = await client.get('/empty').send();
      response
        ..assertNoContent()
        ..assertBodyEmpty();
    });

    test('assertBodyEmpty on non-empty body throws', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.assertBodyEmpty(),
        throwsA(isA<TestAssertionError>()),
      );
    });

    test('headers are unmodifiable', () async {
      final response = await client.get('/hello').send();
      expect(
        () => response.headers['x-new'] = 'value',
        throwsUnsupportedError,
      );
    });

    test('assertions cascade', () async {
      final response = await (client.get('/headers')..bearer('tok')).send();
      response
        ..assertOk()
        ..assertSuccess()
        ..assertJsonContains({'authorization': 'Bearer tok'})
        ..assertContainsHeader('content-type');
    });
  });

  group('buildRequest', () {
    test('creates shelf Request with correct method and path', () {
      final request = buildRequest('GET', '/hello');
      expect(request.method, 'GET');
      expect(request.url.path, 'hello');
    });

    test('includes headers', () {
      final request = buildRequest(
        'POST',
        '/data',
        headers: {'content-type': 'application/json'},
      );
      expect(request.headers['content-type'], 'application/json');
    });

    test('includes path parameters in context', () {
      final request = buildRequest(
        'GET',
        '/users/42',
        pathParameters: {'id': '42'},
      );
      final params = request.context[pathParametersKey] as Map<String, String>;
      expect(params['id'], '42');
    });

    test('omits pathParametersKey when empty', () {
      final request = buildRequest('GET', '/hello');
      expect(request.context.containsKey(pathParametersKey), isFalse);
    });
  });
}
