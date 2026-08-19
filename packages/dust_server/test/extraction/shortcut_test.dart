import 'package:dust_dart/derive.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Each shortcut has to build the extractor it names and nothing else. Testing
/// what comes back rather than its type is the point: a shortcut wired to the
/// wrong class, or to the right class with the wrong argument, still passes a
/// type check and fails a request.

final class _Repo {
  const _Repo(this.name);

  final String name;
}

final class _Payload implements Validatable {
  const _Payload(this.title);

  static _Payload deserialize(Map<String, Object?> json) =>
      _Payload(json['title']! as String);

  final String title;

  @override
  ValidationResult validate() => title.isEmpty
      ? const Invalid([ValidationError(field: 'title', message: 'required')])
      : const Valid();

  @override
  void validateOrThrow() {}
}

void main() {
  group('parts shortcuts', () {
    test('path reads the captured segment', () async {
      final outcome = await path<String>('id').extract(
        request('GET', '/todos/7', pathParameters: const {'id': '7'}),
      );

      expect(expectOk(outcome), '7');
    });

    test('path coerces to the type argument', () async {
      final outcome = await path<int>('id').extract(
        request('GET', '/todos/7', pathParameters: const {'id': '7'}),
      );

      expect(expectOk(outcome), 7);
    });

    test('query reads the named key', () async {
      final outcome = await query<int>('limit').extract(
        request('GET', '/todos?limit=20'),
      );

      expect(expectOk(outcome), 20);
    });

    test('queryList keeps every value of a repeated key', () async {
      final outcome = await queryList<String>('tag').extract(
        request('GET', '/todos?tag=a&tag=b'),
      );

      expect(expectOk(outcome), ['a', 'b']);
    });

    test('state reads the value attached for the type', () async {
      final outcome = await state<_Repo>().extract(
        request('GET', '/', context: {stateKeyFor<_Repo>(): const _Repo('s')}),
      );

      expect(expectOk(outcome).name, 's');
    });

    test('header reads the named header, case-insensitively', () async {
      final outcome = await header('X-Trace').extract(
        request('GET', '/', headers: const {'x-trace': 't'}),
      );

      expect(expectOk(outcome), 't');
    });

    test('header gives null when the header is absent', () async {
      final outcome = await header('x-trace').extract(request('GET', '/'));

      expect(expectOk(outcome), isNull);
    });

    test('headers reads every header at once', () async {
      final outcome = await headers().extract(
        request('GET', '/', headers: const {'X-Trace': 't', 'accept': '*/*'}),
      );

      expect(expectOk(outcome), containsPair('x-trace', 't'));
    });

    test('queries reads every query pair at once', () async {
      final outcome = await queries().extract(request('GET', '/?a=1&b=2'));

      expect(expectOk(outcome), {'a': '1', 'b': '2'});
    });

    test('rawQuery reads the query string undecoded', () async {
      final outcome = await rawQuery().extract(request('GET', '/?a=b%20c'));

      expect(expectOk(outcome), 'a=b%20c');
    });

    test('rawQuery is null when there is no query', () async {
      expect(expectOk(await rawQuery().extract(request('GET', '/'))), isNull);
    });

    test('rawRequest hands back the request it was given', () async {
      final original = request('GET', '/todos');
      final outcome = await rawRequest().extract(original);

      expect(expectOk(outcome), same(original));
    });

    test('peer rejects with 500 when there is no connection', () async {
      expectStatus(await peer().extract(request('GET', '/')), 500);
    });

    test('peer reads the connection information when it is there', () async {
      final outcome = await peer().extract(
        request(
          'GET',
          '/',
          context: {
            PeerExtractable.contextKey: const PeerInfo(
              remoteAddress: '10.0.0.1',
              remotePort: 4242,
              localPort: 8080,
            ),
          },
        ),
      );

      expect(expectOk(outcome).remoteAddress, '10.0.0.1');
    });
  });

  group('body shortcuts', () {
    test('body decodes a JSON object', () async {
      final outcome = await body<_Payload>(_Payload.deserialize).extract(
        jsonRequest('POST', '/', '{"title":"buy milk"}'),
      );

      expect(expectOk(outcome).title, 'buy milk');
    });

    test('bodyList decodes a JSON array', () async {
      final outcome = await bodyList<_Payload>(_Payload.deserialize).extract(
        jsonRequest('POST', '/', '[{"title":"a"},{"title":"b"}]'),
      );

      expect([for (final p in expectOk(outcome)) p.title], ['a', 'b']);
    });

    test('textBody reads the body as UTF-8', () async {
      final outcome = await textBody().extract(
        request('POST', '/', body: 'hello'),
      );

      expect(expectOk(outcome), 'hello');
    });

    test('rawBody reads the body as bytes', () async {
      final outcome = await rawBody().extract(request('POST', '/', body: 'hi'));

      expect(expectOk(outcome), [104, 105]);
    });

    test('bodyStream hands the body over unread', () async {
      final outcome =
          await bodyStream().extract(request('POST', '/', body: 'hi'));
      final bytes = await expectOk(outcome).expand((chunk) => chunk).toList();

      expect(bytes, [104, 105]);
    });

    test('form decodes an urlencoded body', () async {
      final outcome = await form().extract(
        request(
          'POST',
          '/',
          headers: const {
            'content-type': 'application/x-www-form-urlencoded',
          },
          body: 'title=buy+milk',
        ),
      );

      expect(expectOk(outcome).fields['title'], 'buy milk');
    });

    test('multipart decodes a multipart body', () async {
      const boundary = 'X';
      final outcome = await multipart().extract(
        request(
          'POST',
          '/',
          headers: const {
            'content-type': 'multipart/form-data; boundary=$boundary',
          },
          body: '--$boundary\r\n'
              'content-disposition: form-data; name="title"\r\n'
              '\r\n'
              'buy milk\r\n'
              '--$boundary--\r\n',
        ),
      );

      expect(expectOk(expectOk(outcome).field<String>('title')), 'buy milk');
    });
  });

  group('credential shortcuts', () {
    test('apiKey reads the header', () async {
      final outcome = await apiKey()
          .extract(request('GET', '/', headers: const {'x-api-key': 'k'}));

      expect(expectOk(outcome), 'k');
    });

    test('sessionId reads the cookie', () async {
      final outcome = await sessionId()
          .extract(request('GET', '/', headers: const {'cookie': 'session=s'}));

      expect(expectOk(outcome), 's');
    });

    test('firstOf takes whichever arrived', () async {
      final extractor = firstOf<String>([bearerToken(), sessionId()]);
      final outcome = await extractor
          .extract(request('GET', '/', headers: const {'cookie': 'session=s'}));

      expect(expectOk(outcome), 's');
    });
  });

  group('wrapping shortcuts', () {
    test('valid runs the constraints after decoding', () async {
      final outcome = await valid(body<_Payload>(_Payload.deserialize)).extract(
        jsonRequest('POST', '/', '{"title":""}'),
      );

      expect(expectStatus(outcome, 422).fields['title'], ['required']);
    });

    test('valid passes a value that satisfies them', () async {
      final outcome = await valid(body<_Payload>(_Payload.deserialize)).extract(
        jsonRequest('POST', '/', '{"title":"ok"}'),
      );

      expect(expectOk(outcome).title, 'ok');
    });

    test('optional turns a client error into None', () async {
      final outcome =
          await optional(query<int>('limit')).extract(request('GET', '/'));

      expect(expectOk(outcome).isNone, isTrue);
    });

    test('optional keeps a value that was there', () async {
      final outcome = await optional(query<int>('limit'))
          .extract(request('GET', '/?limit=5'));

      expect(expectOk(outcome), const Some(5));
    });

    test('fallible hands the rejection to the caller', () async {
      final outcome =
          await fallible(query<int>('limit')).extract(request('GET', '/'));

      expect(expectErr(expectOk(outcome)).status, 400);
    });
  });
}
