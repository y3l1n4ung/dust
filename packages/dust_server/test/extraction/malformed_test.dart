import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Input that is broken rather than merely wrong. Each of these arrives from
/// something outside the application's control — a client sending bytes that
/// are not text, a header no parser accepts, a composition mistake — and each
/// has to produce a status that says whose fault it is.

final class _Repo {
  const _Repo();
}

final class _Other {
  const _Other();
}

void main() {
  group('a form body that is not UTF-8', () {
    Request formRequest(List<int> body) => request(
          'POST',
          '/',
          headers: const {
            'content-type': 'application/x-www-form-urlencoded',
          },
          body: body,
        );

    test('belongs to the client, so 400 rather than 500', () async {
      final outcome = await const FormExtractable().extract(
        formRequest(const [0xFF, 0xFE, 0xFD]),
      );

      expectStatus(outcome, 400);
    });

    test('says the body was malformed rather than leaking the decoder',
        () async {
      final outcome = await const FormExtractable().extract(
        formRequest(const [0xFF, 0xFE, 0xFD]),
      );

      expect(expectErr(outcome).message, startsWith('malformed form body:'));
    });

    test('accepts a body that is valid UTF-8', () async {
      final outcome = await const FormExtractable().extract(
        formRequest('name=caf%C3%A9'.codeUnits),
      );

      expect(expectOk(outcome).fields['name'], 'café');
    });
  });

  group('a content-type no parser accepts', () {
    test('reads as absent rather than throwing', () {
      final parts = RequestParts.of(
        request('POST', '/', headers: const {'content-type': 'not-a-type'}),
      );

      expect(parts.contentType, isNull);
    });

    test('leaves the media type absent too', () {
      final parts = RequestParts.of(
        request('POST', '/', headers: const {'content-type': 'not-a-type'}),
      );

      expect(parts.mediaType, isNull);
    });

    test('makes a JSON extractor answer 415, not 500', () async {
      final outcome = await const JsonExtractable<Object>(_identity).extract(
        request(
          'POST',
          '/',
          headers: const {'content-type': 'not-a-type'},
          body: '{}',
        ),
      );

      expectStatus(outcome, 415);
    });
  });

  group('state attached with the wrong type', () {
    test('is a 500, since no request could have caused it', () async {
      final outcome = await const StateExtractable<_Repo>().extract(
        request('GET', '/', context: {stateKeyFor<_Repo>(): const _Other()}),
      );

      expectStatus(outcome, 500);
    });

    test('names the type that was expected', () async {
      final outcome = await const StateExtractable<_Repo>().extract(
        request('GET', '/', context: {stateKeyFor<_Repo>(): const _Other()}),
      );

      expect(expectErr(outcome).message, contains('_Repo'));
    });

    test('does not leak the detail to the client', () async {
      final outcome = await const StateExtractable<_Repo>().extract(
        request('GET', '/', context: {stateKeyFor<_Repo>(): const _Other()}),
      );

      final body = await expectErr(outcome).intoResponse().readAsString();
      expect(body, contains('wrong type'));
    });
  });

  group('an Err carrying nothing', () {
    test('is still a failure, so 500 rather than an empty 200', () {
      expect(responseFrom(const Err<int, Object?>(null)).statusCode, 500);
    });

    test('says nothing about the cause', () async {
      final body = await responseFrom(
        const Err<int, Object?>(null),
      ).readAsString();

      expect(body, contains('Internal server error'));
    });
  });
}

Object _identity(Map<String, Object?> json) => json;
