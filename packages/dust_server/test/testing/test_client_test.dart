import 'dart:convert';
import 'package:dust_server/testing.dart';
import 'package:test/test.dart';
import 'support.dart';

void main() {
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

    test('a binary body arrives byte for byte', () async {
      final response = await client.get('/binary').send();

      response.assertOk();
      expect(response.bodyBytes, pngHeader);
    });

    test('a binary body survives real HTTP too', () async {
      final served = await TestClient.serve(app());
      addTearDown(served.close);

      final response = await served.get('/binary').send();

      response.assertOk();
      expect(response.bodyBytes, pngHeader);
    });

    test('body decodes what it can rather than throwing', () async {
      final response = await client.get('/binary').send();

      expect(response.body, contains('�'));
      expect(identical(response.body, response.body), isTrue);
    });

    test('json decodes once and hands back the same value', () async {
      final response = await client.get('/list').send();

      expect(identical(response.json, response.json), isTrue);
    });

    test('an invalid JSON body reports itself on every read', () async {
      final response = await client.get('/hello').send();

      expect(() => response.json, throwsA(isA<TestAssertionError>()));
      expect(() => response.json, throwsA(isA<TestAssertionError>()));
    });
  });
}
