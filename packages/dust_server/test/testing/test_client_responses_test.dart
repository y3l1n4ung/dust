import 'package:dust_server/testing.dart';
import 'package:test/test.dart';
import 'support.dart';

/// Reading a response back, and asserting on what it carries.

void main() {
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
}
