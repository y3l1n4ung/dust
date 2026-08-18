import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Streaming multipart: the parts arrive one at a time and are never all held
/// in memory at once, which is what makes an upload larger than memory possible.

const _boundary = 'X-DUST';

/// Builds a multipart body from [parts], each `(name, filename, content)`.
List<int> _body(List<(String, String?, List<int>)> parts) {
  final out = <int>[];
  for (final (name, filename, content) in parts) {
    final disposition = filename == null
        ? 'form-data; name="$name"'
        : 'form-data; name="$name"; filename="$filename"';
    out
      ..addAll(utf8.encode('--$_boundary\r\n'))
      ..addAll(utf8.encode('content-disposition: $disposition\r\n\r\n'))
      ..addAll(content)
      ..addAll(utf8.encode('\r\n'));
  }
  out.addAll(utf8.encode('--$_boundary--\r\n'));
  return out;
}

/// A request carrying [body], optionally in several chunks.
Request _request(List<int> body, {int chunk = 1 << 20, int? contentLength}) {
  final chunks = <List<int>>[];
  for (var index = 0; index < body.length; index += chunk) {
    chunks.add(
      body.sublist(index, (index + chunk).clamp(0, body.length)),
    );
  }

  return Request(
    'POST',
    Uri.parse('http://localhost/upload'),
    headers: {
      'content-type': 'multipart/form-data; boundary=$_boundary',
      if (contentLength != null) 'content-length': '$contentLength',
    },
    body: Stream<List<int>>.fromIterable(chunks),
  );
}

Future<StreamedMultipart> _extract(Request request) async =>
    expectOk(await const StreamedMultipartExtractable().extract(request));

void main() {
  group('the streaming multipart extractor', () {
    test('reads a field and a file in order', () async {
      final body = await _extract(
        _request(
          _body([
            ('caption', null, utf8.encode('my cat')),
            ('photo', 'cat.txt', utf8.encode('meow')),
          ]),
        ),
      );

      final seen = <String>[];
      await body.forEachPart((part) async {
        seen.add('${part.name}:${part.filename}:${await part.readText()}');
      });

      expect(seen, ['caption:null:my cat', 'photo:cat.txt:meow']);
    });

    test('isFile distinguishes a file from an ordinary field', () async {
      final body = await _extract(
        _request(
          _body([
            ('caption', null, utf8.encode('text')),
            ('photo', 'cat.png', utf8.encode('bytes')),
          ]),
        ),
      );

      final files = <String>[];
      await body.forEachPart((part) async {
        if (part.isFile) files.add(part.name);
      });

      expect(files, ['photo']);
    });

    test('writes a part out without collecting it', () async {
      final target = File(
        '${Directory.systemTemp.createTempSync('dust-mp-').path}/out.bin',
      );
      addTearDown(() => target.parent.delete(recursive: true));

      final body = await _extract(
        _request(_body([('photo', 'cat.bin', List.filled(200000, 7))])),
      );

      var written = 0;
      await body.forEachPart((part) async {
        final sink = target.openWrite();
        written = await part.writeTo(sink, limit: 1 << 20);
        await sink.close();
      });

      expect(written, 200000);
      expect(target.lengthSync(), 200000);
    });

    test('never holds the whole body at once', () async {
      // The property the whole extractor exists for. Five parts of 200 KB each,
      // and the largest chunk handed over is one socket chunk, not a megabyte.
      final body = await _extract(
        _request(
          _body([
            for (var index = 0; index < 5; index++)
              ('file$index', 'f$index.bin', List.filled(200000, 1)),
          ]),
          chunk: 8192,
        ),
      );

      var largestChunk = 0;
      var total = 0;
      await body.forEachPart((part) async {
        await for (final piece in part.content) {
          largestChunk =
              piece.length > largestChunk ? piece.length : largestChunk;
          total += piece.length;
        }
      });

      expect(total, 1000000);
      expect(largestChunk, lessThanOrEqualTo(8192));
    });

    test('forEachPart drains a part the callback ignored', () async {
      // Forgetting to drain stalls the request, so the convenience does it.
      final body = await _extract(
        _request(
          _body([
            ('skipped', 'a.bin', List.filled(50000, 2)),
            ('wanted', null, utf8.encode('here')),
          ]),
        ),
      );

      final seen = <String>[];
      await body.forEachPart((part) async {
        if (part.name == 'wanted') seen.add(await part.readText());
      });

      expect(seen, ['here']);
    });

    test('isTaken reports whether anything read the part', () async {
      final body = await _extract(
        _request(_body([('a', null, utf8.encode('one'))])),
      );

      await for (final part in body.parts) {
        expect(part.isTaken, isFalse);
        await part.readBytes();
        expect(part.isTaken, isTrue);
      }
    });

    test('skip on a part already read does nothing rather than throwing',
        () async {
      final body = await _extract(
        _request(_body([('a', null, utf8.encode('one'))])),
      );

      await for (final part in body.parts) {
        await part.readBytes();
        await part.skip();
      }
    });

    test('skip lets the next part start', () async {
      final body = await _extract(
        _request(
          _body([
            ('first', 'a.bin', List.filled(50000, 3)),
            ('second', null, utf8.encode('second')),
          ]),
        ),
      );

      final seen = <String>[];
      await for (final part in body.parts) {
        if (part.name == 'first') {
          await part.skip();
        } else {
          seen.add(await part.readText());
        }
      }

      expect(seen, ['second']);
    });
  });

  group('limits', () {
    test('a declared content-length over the limit is refused up front',
        () async {
      // Before a byte is read, when the client told us how much there is.
      final outcome = await const StreamedMultipartExtractable(limit: 1000)
          .extract(_request(
              _body([
                ('a', null, [1])
              ]),
              contentLength: 99999));

      expectStatus(outcome, 413);
    });

    test('an undeclared body over the limit fails as it flows', () async {
      // A streamed upload usually arrives with no content-length, so there is
      // nothing to check up front. Mid-stream is the earliest anything can fail.
      final body = await _extract(_request(
        _body([('big', 'big.bin', List.filled(300000, 4))]),
        chunk: 8192,
      ));

      await expectLater(
        const StreamedMultipartExtractable(limit: 1000)
            .extract(_request(
              _body([('big', 'big.bin', List.filled(300000, 4))]),
              chunk: 8192,
            ))
            .then((outcome) => expectOk(outcome).forEachPart((_) async {})),
        throwsA(
          isA<Rejection>().having((r) => r.status, 'status', 413),
        ),
      );

      // The unrestricted one still works, so the failure was the limit.
      await body.forEachPart((_) async {});
    });

    test('a part over its own limit fails, not the whole body', () async {
      final body = await _extract(
        _request(_body([('photo', 'big.bin', List.filled(5000, 5))])),
      );

      await expectLater(
        body.forEachPart((part) => part.readBytes(limit: 100)),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 413)),
      );
    });

    test('writeTo refuses past its limit rather than filling the disk',
        () async {
      final body = await _extract(
        _request(_body([('photo', 'big.bin', List.filled(5000, 6))])),
      );

      await expectLater(
        body.forEachPart(
          (part) => part.writeTo(_NullSink(), limit: 100),
        ),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 413)),
      );
    });

    test('a limit in the request context wins over the constructor', () async {
      final outcome = await const StreamedMultipartExtractable().extract(
        _request(
                _body([
                  ('a', null, [1])
                ]),
                contentLength: 5000)
            .change(
          context: {bodyLimitContextKey: 100},
        ),
      );

      expectStatus(outcome, 413);
    });
  });

  group('the shortcuts', () {
    test('multipartStream() builds the extractor', () async {
      final body = expectOk(
        await multipartStream(limit: 4096)
            .extract(_request(_body([('a', null, utf8.encode('one'))]))),
      );

      final seen = <String>[];
      await body.forEachPart((part) async => seen.add(await part.readText()));

      expect(seen, ['one']);
    });

    test('request.multipartStream() reads it off the request', () async {
      final request = _request(_body([('a', null, utf8.encode('two'))]));

      final body = await request.multipartStream();

      final seen = <String>[];
      await body.forEachPart((part) async => seen.add(await part.readText()));

      expect(seen, ['two']);
    });

    test('request.multipartStream() throws its rejection', () async {
      final request = Request(
        'POST',
        Uri.parse('http://localhost/upload'),
        headers: const {'content-type': 'application/json'},
        body: '{}',
      );

      await expectLater(
        request.multipartStream(),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 415)),
      );
    });
  });

  group('refusals', () {
    test('a non-multipart body is 415', () async {
      final outcome = await const StreamedMultipartExtractable().extract(
        Request(
          'POST',
          Uri.parse('http://localhost/upload'),
          headers: const {'content-type': 'application/json'},
          body: '{}',
        ),
      );

      expectStatus(outcome, 415);
    });

    test('a multipart body with no boundary is 400', () async {
      final outcome = await const StreamedMultipartExtractable().extract(
        Request(
          'POST',
          Uri.parse('http://localhost/upload'),
          headers: const {'content-type': 'multipart/form-data'},
          body: 'nothing',
        ),
      );

      expectStatus(outcome, 400);
    });
  });
}

/// Accepts bytes and forgets them, so writeTo can be tested without a file.
final class _NullSink implements StreamConsumer<List<int>> {
  @override
  Future<void> addStream(Stream<List<int>> stream) => stream.drain<void>();

  @override
  Future<void> close() async {}
}
