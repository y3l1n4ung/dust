import 'dart:convert';
import 'package:dust_server/server.dart';
import 'package:dust_server/testing.dart';
import 'package:test/test.dart';
import 'support.dart';

/// Building requests, and the expectations a client carries into them.

void main() {
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
