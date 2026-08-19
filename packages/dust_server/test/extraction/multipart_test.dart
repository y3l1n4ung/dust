import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

const _boundary = 'dust-boundary';
const _multipart = MultipartExtractable();

String _body(List<String> parts) {
  return '${parts.map((part) => '--$_boundary\r\n$part\r\n').join()}'
      '--$_boundary--\r\n';
}

Request _multipartRequest(String body, {String? contentType}) {
  return request(
    'POST',
    '/upload',
    headers: {
      'content-type': contentType ?? 'multipart/form-data; boundary=$_boundary',
    },
    body: body,
  );
}

void main() {
  group('MultipartExtractable', () {
    test('decodes text parts', () async {
      final outcome = await _multipart.extract(
        _multipartRequest(
          _body([
            'content-disposition: form-data; name="title"\r\n\r\nhello',
            'content-disposition: form-data; name="count"\r\n\r\n3',
          ]),
        ),
      );
      final form = expectOk(outcome);

      expect(expectOk(form.field<String>('title')), 'hello');
      expect(expectOk(form.field<int>('count')), 3);
    });

    test('decodes a file part with its metadata', () async {
      final outcome = await _multipart.extract(
        _multipartRequest(
          _body([
            'content-disposition: form-data; name="avatar"; filename="face.png"\r\ncontent-type: image/png\r\n\r\nBINARY',
          ]),
        ),
      );
      final file = expectOk(expectOk(outcome).file('avatar'));

      expect(file.name, 'avatar');
      expect(file.filename, 'face.png');
      expect(file.contentType, 'image/png');
      expect(utf8.decode(file.bytes), 'BINARY');
      expect(file.asText, 'BINARY');
    });

    test('accepts a quoted boundary', () async {
      final outcome = await _multipart.extract(
        _multipartRequest(
          _body(['content-disposition: form-data; name="a"\r\n\r\n1']),
          contentType: 'multipart/form-data; boundary="$_boundary"',
        ),
      );

      expect(expectOk(expectOk(outcome).field<String>('a')), '1');
    });

    test('rejects the wrong media type with 415', () async {
      final outcome = await _multipart.extract(
        request(
          'POST',
          '/upload',
          headers: {'content-type': 'application/json'},
          body: '{}',
        ),
      );

      expectStatus(outcome, 415);
    });

    test('rejects a missing boundary with 400', () async {
      final outcome = await _multipart.extract(
        _multipartRequest(
          _body(['content-disposition: form-data; name="a"\r\n\r\n1']),
          contentType: 'multipart/form-data',
        ),
      );

      expect(
        expectStatus(outcome, 400).message,
        'multipart body declares no boundary',
      );
    });

    test('rejects a malformed body with 400', () async {
      final outcome = await _multipart.extract(
        _multipartRequest('not really multipart at all'),
      );

      expectStatus(outcome, 400);
    });

    test('rejects an oversized body with 413', () async {
      final outcome = await const MultipartExtractable(limit: 8).extract(
        _multipartRequest(
          _body(['content-disposition: form-data; name="a"\r\n\r\n1']),
        ),
      );

      expectStatus(outcome, 413);
    });

    test('skips parts with no name', () async {
      final outcome = await _multipart.extract(
        _multipartRequest(
          _body([
            'content-disposition: form-data\r\n\r\nanonymous',
            'content-disposition: form-data; name="a"\r\n\r\n1',
          ]),
        ),
      );

      expect(expectOk(outcome).parts.keys, ['a']);
    });
  });

  group('MultipartForm', () {
    test('rejects a missing file part with 400', () async {
      final form = expectOk(
        await _multipart.extract(
          _multipartRequest(
            _body(['content-disposition: form-data; name="a"\r\n\r\n1']),
          ),
        ),
      );

      expect(
        expectStatus(form.file('avatar'), 400).message,
        'multipart part "avatar" is required',
      );
    });

    test('rejects a missing required field with 400', () async {
      final form = expectOk(
        await _multipart.extract(
          _multipartRequest(
            _body(['content-disposition: form-data; name="a"\r\n\r\n1']),
          ),
        ),
      );

      expectStatus(form.field<String>('missing'), 400);
    });

    test('returns null for a missing optional field', () async {
      final form = expectOk(
        await _multipart.extract(
          _multipartRequest(
            _body(['content-disposition: form-data; name="a"\r\n\r\n1']),
          ),
        ),
      );

      expect(expectOk(form.field<String?>('missing')), isNull);
    });
  });
}
