import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

const _form = FormExtractable();

Request _formRequest(String body, {String method = 'POST'}) {
  return request(
    method,
    '/login',
    headers: const {'content-type': 'application/x-www-form-urlencoded'},
    body: body,
  );
}

void main() {
  group('FormExtractable', () {
    test('decodes a urlencoded body', () async {
      final outcome = await _form.extract(_formRequest('email=a%40b.c&keep=1'));

      expect(expectOk(outcome).fields, {'email': 'a@b.c', 'keep': '1'});
    });

    test('reads the query instead of the body on GET', () async {
      final outcome = await _form.extract(request('GET', '/login?email=a@b.c'));

      expect(expectOk(outcome).fields, {'email': 'a@b.c'});
    });

    test('reads the query instead of the body on HEAD', () async {
      final outcome = await _form.extract(request('HEAD', '/login?email=x'));

      expect(expectOk(outcome).fields, {'email': 'x'});
    });

    test('rejects the wrong media type with 415', () async {
      final outcome = await _form.extract(
        request(
          'POST',
          '/login',
          headers: {'content-type': 'application/json'},
          body: '{}',
        ),
      );

      expect(
        expectStatus(outcome, 415).message,
        'expected application/x-www-form-urlencoded',
      );
    });

    test('rejects an oversized body with 413', () async {
      final outcome = await const FormExtractable(limit: 4)
          .extract(_formRequest('email=someone@example.com'));

      expectStatus(outcome, 413);
    });

    test('accepts an empty body as an empty form', () async {
      final outcome = await _form.extract(_formRequest(''));

      expect(expectOk(outcome).fields, isEmpty);
    });
  });

  group('FormMap.field', () {
    test('coerces a present field', () async {
      final form = expectOk(await _form.extract(_formRequest('age=41')));

      expect(expectOk(form.field<int>('age')), 41);
    });

    test('returns null for an absent optional field', () async {
      final form = expectOk(await _form.extract(_formRequest('')));

      expect(expectOk(form.field<String?>('nickname')), isNull);
    });

    test('rejects an absent required field with 400', () async {
      final form = expectOk(await _form.extract(_formRequest('')));

      expect(
        expectStatus(form.field<String>('email'), 400).message,
        'form field "email" is required',
      );
    });

    test('rejects an uncoercible field with 400', () async {
      final form = expectOk(await _form.extract(_formRequest('age=old')));

      expectStatus(form.field<int>('age'), 400);
    });

    test('reads every field from one body pass', () async {
      final form =
          expectOk(await _form.extract(_formRequest('email=a@b.c&age=41')));

      expect(expectOk(form.field<String>('email')), 'a@b.c');
      expect(expectOk(form.field<int>('age')), 41);
    });
  });
}
