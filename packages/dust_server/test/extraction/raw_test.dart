import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('RawBodyExtractable', () {
    test('returns the bytes with no media type requirement', () async {
      final outcome = await const RawBodyExtractable()
          .extract(request('POST', '/', body: 'abc'));

      expect(expectOk(outcome), [97, 98, 99]);
    });

    test('rejects past the limit with 413', () async {
      final outcome = await const RawBodyExtractable(limit: 2)
          .extract(request('POST', '/', body: 'abc'));

      expectStatus(outcome, 413);
    });

    test('returns an empty list for an empty body', () async {
      final outcome =
          await const RawBodyExtractable().extract(request('POST', '/'));

      expect(expectOk(outcome), isEmpty);
    });
  });

  group('TextBodyExtractable', () {
    test('decodes UTF-8 text', () async {
      final outcome = await const TextBodyExtractable()
          .extract(request('POST', '/', body: 'héllo'));

      expect(expectOk(outcome), 'héllo');
    });

    test('rejects invalid UTF-8 with 400', () async {
      final outcome = await const TextBodyExtractable().extract(
        request('POST', '/', body: <int>[0xC3, 0x28]),
      );

      expect(expectStatus(outcome, 400).message, 'body is not valid UTF-8');
    });

    test('rejects past the limit with 413', () async {
      final outcome = await const TextBodyExtractable(limit: 2)
          .extract(request('POST', '/', body: 'abc'));

      expectStatus(outcome, 413);
    });
  });

  group('StreamBodyExtractable', () {
    test('hands the unread stream to the handler', () async {
      final outcome = await const StreamBodyExtractable()
          .extract(request('POST', '/', body: 'streamed'));
      final collected = await expectOk(outcome)
          .fold<List<int>>(<int>[], (bytes, chunk) => bytes..addAll(chunk));

      expect(utf8.decode(collected), 'streamed');
    });
  });
}
